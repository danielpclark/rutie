use std::{
    convert::From,
    panic::{self, AssertUnwindSafe},
    ptr::{self, NonNull},
    slice,
};

use crate::{
    binding::{fixnum, io_buffer, vm},
    rubysys::io_buffer::{
        RB_IO_BUFFER_EXTERNAL, RB_IO_BUFFER_PRIVATE, RB_IO_BUFFER_READONLY,
        RUBY_IO_BUFFER_DEFAULT_SIZE, RUBY_IO_BUFFER_PAGE_SIZE,
    },
    types::Value,
    AnyException, AnyObject, Class, Exception, Fixnum, NilClass, Object, RString, VerifiedObject,
    IO,
};

fn protect<F>(func: F) -> Result<Value, AnyException>
where
    F: FnOnce() -> Value,
{
    vm::protect_value(func).map_err(AnyException::from)
}

// Runs `call` (a read or write), which returns the byte count as an Integer
// or a negative `errno`.
fn io_count<F>(operation: &str, call: F) -> Result<usize, AnyException>
where
    F: FnOnce() -> Value,
{
    let mut count = 0;

    protect(|| {
        count = fixnum::num_to_i64(call());

        NilClass::new().value()
    })?;

    if count < 0 {
        Err(AnyException::from_errno(-count as i32, operation))
    } else {
        Ok(count as usize)
    }
}

fn file_offset(from: u64) -> Result<i64, AnyException> {
    if from > i64::MAX as u64 {
        Err(AnyException::new(
            "RangeError",
            Some(&format!("file offset {} is too large", from)),
        ))
    } else {
        Ok(from as i64)
    }
}

fn usize_to_any_object(number: usize) -> AnyObject {
    AnyObject::from(fixnum::usize_to_num(number))
}

/// Ruby's `IO::Buffer` (Ruby 3.1+): memory for reading from and writing to
/// streams without going through Strings.
///
/// Ruby calls `IO::Buffer` experimental, and warns the first time some of
/// its Ruby methods are used.
///
/// Operations that can raise (out of range offsets, a locked, read-only or
/// freed buffer, a failed system call) return the exception as `Err`.
///
/// # Examples
///
/// ```
/// use rutie::{IOBuffer, Object, IO, VM};
/// # VM::init();
///
/// let pipe = VM::eval("IO.pipe").unwrap().try_convert_to::<rutie::Array>().unwrap();
/// let reader = pipe.at(0).try_convert_to::<IO>().unwrap();
/// let writer = pipe.at(1).try_convert_to::<IO>().unwrap();
///
/// let buffer = IOBuffer::from_bytes(b"ping");
/// assert_eq!(buffer.write(&writer, 4, 0).unwrap(), 4);
///
/// let received = IOBuffer::new(4);
/// assert_eq!(received.read(&reader, 4, 0).unwrap(), 4);
/// assert_eq!(received.get_string(0, 4).unwrap().to_bytes_unchecked(), b"ping");
/// # reader.close().unwrap();
/// # writer.close().unwrap();
/// ```
#[derive(Debug)]
#[repr(C)]
pub struct IOBuffer {
    value: Value,
}

impl IOBuffer {
    /// Creates a buffer of `size` zeroed bytes that Ruby allocates and owns
    /// (`rb_io_buffer_new` with `RB_IO_BUFFER_INTERNAL`).
    ///
    /// A `0` size makes a null buffer, with no memory.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IOBuffer, VM};
    /// # VM::init();
    ///
    /// let buffer = IOBuffer::new(8);
    ///
    /// assert_eq!(buffer.size(), 8);
    /// assert!(buffer.is_internal());
    /// assert_eq!(buffer.get_string(0, 8).unwrap().to_bytes_unchecked(), [0; 8]);
    ///
    /// assert!(IOBuffer::new(0).is_null());
    /// ```
    pub fn new(size: usize) -> Self {
        IOBuffer::from(io_buffer::new(size))
    }

    /// Creates a buffer that Ruby owns, holding a copy of `bytes`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IOBuffer, VM};
    /// # VM::init();
    ///
    /// let buffer = IOBuffer::from_bytes(&[1, 2, 3]);
    ///
    /// assert_eq!(buffer.size(), 3);
    /// assert_eq!(buffer.get_string(1, 2).unwrap().to_bytes_unchecked(), [2, 3]);
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let buffer = IOBuffer::new(bytes.len());
        let (base, size, _) = io_buffer::get_bytes(buffer.value());

        if size > 0 {
            // Memory that was just allocated: nothing else can refer to it.
            unsafe { ptr::copy_nonoverlapping(bytes.as_ptr(), base, size) };
        }

        buffer
    }

    /// Creates a buffer over `size` bytes at `base` that the caller owns
    /// (`rb_io_buffer_new` with `RB_IO_BUFFER_EXTERNAL`, and
    /// `RB_IO_BUFFER_READONLY` when `readonly`). Ruby never frees or resizes
    /// the memory.
    ///
    /// # Safety
    ///
    /// `base` must be valid for reads of `size` bytes (and writes, unless
    /// `readonly`) until the buffer is [freed](#method.free) or no longer
    /// reachable from Ruby, and also while any slice of the buffer
    /// (`IO::Buffer#slice`) is used. With a null `base`, `size` must be `0`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IOBuffer, RString, VM};
    /// # VM::init();
    ///
    /// let mut memory = *b"rust";
    /// let buffer = unsafe { IOBuffer::from_raw_parts(memory.as_mut_ptr(), memory.len(), false) };
    ///
    /// assert!(buffer.is_external());
    /// assert_eq!(buffer.set_string(&RString::new_utf8("R"), 0).unwrap(), 1);
    ///
    /// // Detach the buffer before using the memory again.
    /// buffer.free().unwrap();
    ///
    /// assert_eq!(&memory, b"Rust");
    /// ```
    pub unsafe fn from_raw_parts(base: *mut u8, size: usize, readonly: bool) -> Self {
        let mut flags = RB_IO_BUFFER_EXTERNAL;

        if readonly {
            flags |= RB_IO_BUFFER_READONLY;
        }

        IOBuffer::from(io_buffer::new_external(base as *mut _, size, flags))
    }

    /// Maps `size` bytes of the file open in `io`, from `offset`, into a
    /// buffer (`rb_io_buffer_map`, Ruby's `IO::Buffer.map`), or returns the
    /// error.
    ///
    /// A `readonly` mapping can't be written to. Writes to a `private`
    /// mapping are not written to the file or seen by other processes;
    /// writes to a shared one are. Ruby 3.1 and 3.2 can fail to make
    /// private mappings (`Errno::EINVAL` on Linux). [`free`](#method.free)
    /// unmaps the memory.
    ///
    /// # Safety
    ///
    /// Other processes, and other mappings and descriptors of the same
    /// file, can change the mapped memory at any time (unless `private`),
    /// and accessing a part of the mapping beyond the end of the file (for
    /// instance after the file is truncated) crashes the process
    /// (`SIGBUS`). The caller must make sure neither happens while the
    /// buffer is used. `offset` must be a multiple of the page size
    /// ([`IOBuffer::page_size`](#method.page_size)).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{File, IOBuffer, VM};
    /// # VM::init();
    ///
    /// let path = std::env::temp_dir().join(format!("rutie_io_buffer_map_{}.txt", std::process::id()));
    /// std::fs::write(&path, "mapped").unwrap();
    ///
    /// let file = File::open(path.to_str().unwrap(), "r").unwrap();
    /// let buffer = unsafe { IOBuffer::map(&file, 6, 0, true, false) }.unwrap();
    ///
    /// assert!(buffer.is_mapped());
    /// assert!(buffer.is_readonly());
    /// assert_eq!(buffer.get_string(0, 6).unwrap().to_str(), "mapped");
    ///
    /// buffer.free().unwrap();
    /// file.close().unwrap();
    /// # std::fs::remove_file(path).unwrap();
    /// ```
    pub unsafe fn map(
        io: &IO,
        size: usize,
        offset: u64,
        readonly: bool,
        private: bool,
    ) -> Result<Self, AnyException> {
        let offset = file_offset(offset)?;
        let io_value = io.value();
        let mut flags = 0;

        if readonly {
            flags |= RB_IO_BUFFER_READONLY;
        }

        if private {
            flags |= RB_IO_BUFFER_PRIVATE;
        }

        protect(|| io_buffer::map(io_value, size, offset, flags)).map(IOBuffer::from)
    }

    /// Returns the operating system's page size (`RUBY_IO_BUFFER_PAGE_SIZE`,
    /// Ruby's `IO::Buffer::PAGE_SIZE`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, IOBuffer, Object, VM};
    /// # VM::init();
    ///
    /// let page_size = VM::eval("IO::Buffer::PAGE_SIZE").unwrap().try_convert_to::<Fixnum>().unwrap();
    ///
    /// assert_eq!(IOBuffer::page_size(), page_size.to_i64() as usize);
    /// assert!(IOBuffer::page_size().is_power_of_two());
    /// ```
    pub fn page_size() -> usize {
        unsafe { RUBY_IO_BUFFER_PAGE_SIZE }
    }

    /// Returns the default buffer size (`RUBY_IO_BUFFER_DEFAULT_SIZE`, Ruby's
    /// `IO::Buffer::DEFAULT_SIZE`), a multiple of the page size that the
    /// `RUBY_IO_BUFFER_DEFAULT_SIZE` environment variable can change.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, IOBuffer, Object, VM};
    /// # VM::init();
    ///
    /// let default_size = VM::eval("IO::Buffer::DEFAULT_SIZE").unwrap().try_convert_to::<Fixnum>().unwrap();
    ///
    /// assert_eq!(IOBuffer::default_size(), default_size.to_i64() as usize);
    /// ```
    pub fn default_size() -> usize {
        unsafe { RUBY_IO_BUFFER_DEFAULT_SIZE }
    }

    /// Returns the size in bytes, `0` for a null buffer
    /// (`rb_io_buffer_get_bytes`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IOBuffer, VM};
    /// # VM::init();
    ///
    /// let buffer = IOBuffer::new(16);
    /// assert_eq!(buffer.size(), 16);
    ///
    /// buffer.free().unwrap();
    /// assert_eq!(buffer.size(), 0);
    /// ```
    pub fn size(&self) -> usize {
        io_buffer::get_bytes(self.value()).1
    }

    fn predicate(&self, method: &str) -> bool {
        vm::call_public_method(self.value(), method, &[]).is_true()
    }

    /// Returns `true` if the buffer has no memory (Ruby's `null?`): it has a
    /// `0` size, was freed, or its memory was transferred.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IOBuffer, VM};
    /// # VM::init();
    ///
    /// let buffer = IOBuffer::new(1);
    /// assert!(!buffer.is_null());
    ///
    /// let moved = buffer.transfer().unwrap();
    /// assert!(buffer.is_null());
    /// assert!(!moved.is_null());
    /// ```
    pub fn is_null(&self) -> bool {
        self.predicate("null?")
    }

    /// Returns `true` if the memory belongs to someone else (Ruby's
    /// `external?`), such as a String (`IO::Buffer.for`) or the caller of
    /// [`IOBuffer::from_raw_parts`](#method.from_raw_parts).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IOBuffer, Object, VM};
    /// # VM::init();
    ///
    /// let for_string = VM::eval("IO::Buffer.for('text')").unwrap().try_convert_to::<IOBuffer>().unwrap();
    ///
    /// assert!(for_string.is_external());
    /// assert!(!IOBuffer::new(1).is_external());
    /// ```
    pub fn is_external(&self) -> bool {
        self.predicate("external?")
    }

    /// Returns `true` if Ruby allocated the memory (Ruby's `internal?`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IOBuffer, VM};
    /// # VM::init();
    ///
    /// assert!(IOBuffer::new(1).is_internal());
    /// assert!(!IOBuffer::new(0).is_internal());
    /// ```
    pub fn is_internal(&self) -> bool {
        self.predicate("internal?")
    }

    /// Returns `true` if the memory is mapped (Ruby's `mapped?`), from a
    /// file ([`IOBuffer::map`](#method.map)) or, for large buffers created
    /// from Ruby, anonymously.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IOBuffer, Object, VM};
    /// # VM::init();
    ///
    /// let large = VM::eval("IO::Buffer.new(IO::Buffer::PAGE_SIZE * 4)").unwrap().try_convert_to::<IOBuffer>().unwrap();
    ///
    /// assert!(large.is_mapped());
    /// assert!(!IOBuffer::new(1).is_mapped());
    /// ```
    pub fn is_mapped(&self) -> bool {
        self.predicate("mapped?")
    }

    /// Returns `true` if the buffer is locked (Ruby's `locked?`): it can't
    /// be resized, freed or transferred.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IOBuffer, VM};
    /// # VM::init();
    ///
    /// let buffer = IOBuffer::new(1);
    /// assert!(!buffer.is_locked());
    ///
    /// buffer.lock().unwrap();
    /// assert!(buffer.is_locked());
    /// # unsafe { buffer.unlock() }.unwrap();
    /// ```
    pub fn is_locked(&self) -> bool {
        self.predicate("locked?")
    }

    /// Returns `true` if the buffer can't be written to (Ruby's
    /// `readonly?`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IOBuffer, Object, VM};
    /// # VM::init();
    ///
    /// let for_string = VM::eval("IO::Buffer.for('text')").unwrap().try_convert_to::<IOBuffer>().unwrap();
    ///
    /// assert!(for_string.is_readonly());
    /// assert!(!IOBuffer::new(1).is_readonly());
    /// ```
    pub fn is_readonly(&self) -> bool {
        self.predicate("readonly?")
    }

    /// Locks the buffer (`rb_io_buffer_lock`), so it can't be resized,
    /// freed or transferred until it is unlocked, or returns
    /// `IO::Buffer::LockedError` if it is locked already.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IOBuffer, VM};
    /// # VM::init();
    ///
    /// let buffer = IOBuffer::new(4);
    /// buffer.lock().unwrap();
    ///
    /// assert!(buffer.lock().is_err());
    /// assert!(buffer.resize(8).is_err());
    /// assert!(buffer.free().is_err());
    ///
    /// unsafe { buffer.unlock() }.unwrap();
    /// assert!(buffer.resize(8).is_ok());
    /// ```
    pub fn lock(&self) -> Result<(), AnyException> {
        let value = self.value();

        protect(|| io_buffer::lock(value)).map(|_| ())
    }

    /// Unlocks the buffer (`rb_io_buffer_unlock`), or returns
    /// `IO::Buffer::LockedError` if it is not locked.
    ///
    /// # Safety
    ///
    /// The lock must be one the caller took with [`lock`](#method.lock).
    /// Locks are not counted, so releasing a lock held by someone else
    /// (Ruby's `IO::Buffer#locked`, a system call running in another
    /// thread, [`with_bytes`](#method.with_bytes)) lets the memory be freed
    /// or moved while they use it.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IOBuffer, VM};
    /// # VM::init();
    ///
    /// let buffer = IOBuffer::new(4);
    ///
    /// assert!(unsafe { buffer.unlock() }.is_err());
    ///
    /// buffer.lock().unwrap();
    /// assert!(unsafe { buffer.unlock() }.is_ok());
    /// assert!(!buffer.is_locked());
    /// ```
    pub unsafe fn unlock(&self) -> Result<(), AnyException> {
        let value = self.value();

        protect(|| io_buffer::unlock(value)).map(|_| ())
    }

    /// Unlocks the buffer if it is locked, and returns whether it was
    /// (`rb_io_buffer_try_unlock`).
    ///
    /// # Safety
    ///
    /// See [`unlock`](#method.unlock).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IOBuffer, VM};
    /// # VM::init();
    ///
    /// let buffer = IOBuffer::new(4);
    /// buffer.lock().unwrap();
    ///
    /// assert!(unsafe { buffer.try_unlock() });
    /// assert!(!unsafe { buffer.try_unlock() });
    /// ```
    pub unsafe fn try_unlock(&self) -> bool {
        io_buffer::try_unlock(self.value())
    }

    /// Releases the memory, unmapping or freeing it if the buffer owns it
    /// (`rb_io_buffer_free`), and leaves a null buffer. Returns
    /// `IO::Buffer::LockedError` for a locked buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IOBuffer, VM};
    /// # VM::init();
    ///
    /// let buffer = IOBuffer::new(4);
    ///
    /// buffer.free().unwrap();
    /// assert!(buffer.is_null());
    /// assert!(buffer.free().is_ok());
    /// ```
    pub fn free(&self) -> Result<(), AnyException> {
        let value = self.value();

        protect(|| io_buffer::free(value)).map(|_| ())
    }

    /// Like [`free`](#method.free), but also frees a locked buffer
    /// (`rb_io_buffer_free_locked`). Ruby 3.3+.
    ///
    /// # Safety
    ///
    /// Whoever holds the lock may still be using the memory (see
    /// [`unlock`](#method.unlock)); the caller must make sure nobody is.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IOBuffer, VM};
    /// # VM::init();
    ///
    /// let buffer = IOBuffer::new(4);
    /// buffer.lock().unwrap();
    ///
    /// assert!(buffer.free().is_err());
    ///
    /// unsafe { buffer.free_locked() };
    /// assert!(buffer.is_null());
    /// ```
    pub unsafe fn free_locked(&self) {
        io_buffer::free_locked(self.value());
    }

    /// Moves the memory to a new buffer, leaving this one null
    /// (`rb_io_buffer_transfer`), or returns `IO::Buffer::LockedError` for
    /// a locked buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IOBuffer, VM};
    /// # VM::init();
    ///
    /// let buffer = IOBuffer::from_bytes(b"data");
    /// let moved = buffer.transfer().unwrap();
    ///
    /// assert_eq!(buffer.size(), 0);
    /// assert_eq!(moved.get_string(0, 4).unwrap().to_str(), "data");
    ///
    /// moved.lock().unwrap();
    /// assert!(moved.transfer().is_err());
    /// # unsafe { moved.unlock() }.unwrap();
    /// ```
    pub fn transfer(&self) -> Result<IOBuffer, AnyException> {
        let value = self.value();

        protect(|| io_buffer::transfer(value)).map(IOBuffer::from)
    }

    /// Changes the size, keeping the contents that fit and zeroing new
    /// bytes (`rb_io_buffer_resize`). Returns the error for a locked
    /// buffer, or one whose memory belongs to someone else.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IOBuffer, Object, VM};
    /// # VM::init();
    ///
    /// let buffer = IOBuffer::from_bytes(b"ab");
    /// buffer.resize(4).unwrap();
    ///
    /// assert_eq!(buffer.get_string(0, 4).unwrap().to_bytes_unchecked(), b"ab\0\0");
    ///
    /// let for_string = VM::eval("IO::Buffer.for('text')").unwrap().try_convert_to::<IOBuffer>().unwrap();
    /// assert!(for_string.resize(8).is_err());
    /// ```
    pub fn resize(&self, size: usize) -> Result<(), AnyException> {
        let value = self.value();

        protect(|| {
            io_buffer::resize(value, size);

            NilClass::new().value()
        })
        .map(|_| ())
    }

    fn check_range(&self, offset: usize, length: usize) -> Result<(), AnyException> {
        let size = self.size();

        if offset > size || length > size - offset {
            Err(AnyException::new(
                "ArgumentError",
                Some(&format!(
                    "offset {} and length {} exceed the buffer size {}",
                    offset, length, size
                )),
            ))
        } else {
            Ok(())
        }
    }

    /// Sets `length` bytes from `offset` to `value` (`rb_io_buffer_clear`),
    /// or returns the error for a read-only buffer, or `ArgumentError` for
    /// a range beyond the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IOBuffer, VM};
    /// # VM::init();
    ///
    /// let buffer = IOBuffer::new(4);
    /// buffer.clear(0xff, 1, 2).unwrap();
    ///
    /// assert_eq!(buffer.get_string(0, 4).unwrap().to_bytes_unchecked(), [0, 0xff, 0xff, 0]);
    /// assert!(buffer.clear(0, 3, 2).is_err());
    /// assert!(buffer.clear(0, 1, usize::MAX).is_err());
    /// ```
    pub fn clear(&self, value: u8, offset: usize, length: usize) -> Result<(), AnyException> {
        // Ruby adds `offset` and `length` without checking for overflow.
        self.check_range(offset, length)?;

        let buffer = self.value();

        protect(|| {
            io_buffer::clear(buffer, value, offset, length);

            NilClass::new().value()
        })
        .map(|_| ())
    }

    /// Copies `length` bytes from `offset` into a new binary (ASCII-8BIT)
    /// String (Ruby's `get_string`), or returns the error.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{EncodingSupport, IOBuffer, VM};
    /// # VM::init();
    ///
    /// let buffer = IOBuffer::from_bytes(b"hello");
    /// let string = buffer.get_string(1, 3).unwrap();
    ///
    /// assert_eq!(string.to_str(), "ell");
    /// assert_eq!(string.encoding().name(), "ASCII-8BIT");
    /// assert!(buffer.get_string(4, 2).is_err());
    /// ```
    pub fn get_string(&self, offset: usize, length: usize) -> Result<RString, AnyException> {
        self.check_range(offset, length)?;

        let arguments = [usize_to_any_object(offset), usize_to_any_object(length)];

        self.protect_public_send("get_string", &arguments)
            .map(|string| RString::from(string.value()))
    }

    /// Copies the bytes of `string` into the buffer at `offset` and returns
    /// how many were copied (Ruby's `set_string`), or the error, such as
    /// `IO::Buffer::AccessError` for a read-only buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IOBuffer, Object, RString, VM};
    /// # VM::init();
    ///
    /// let buffer = IOBuffer::new(4);
    ///
    /// assert_eq!(buffer.set_string(&RString::new_utf8("hi"), 1).unwrap(), 2);
    /// assert_eq!(buffer.get_string(0, 4).unwrap().to_bytes_unchecked(), b"\0hi\0");
    /// assert!(buffer.set_string(&RString::new_utf8("long"), 1).is_err());
    ///
    /// let for_string = VM::eval("IO::Buffer.for('text')").unwrap().try_convert_to::<IOBuffer>().unwrap();
    /// assert!(for_string.set_string(&RString::new_utf8("T"), 0).is_err());
    /// ```
    pub fn set_string(&self, string: &RString, offset: usize) -> Result<usize, AnyException> {
        self.check_range(offset, string.to_bytes_unchecked().len())?;

        let arguments = [string.to_any_object(), usize_to_any_object(offset)];

        self.protect_public_send("set_string", &arguments)
            .map(|copied| fixnum::num_to_u64(copied.value()) as usize)
    }

    // Locks the buffer and runs `body` with its memory, unlocking it when
    // `body` returns, panics or raises.
    unsafe fn with_locked_memory<F, R>(&self, writable: bool, body: F) -> Result<R, AnyException>
    where
        F: FnOnce(*mut u8, usize) -> R,
    {
        let value = self.value();

        // No memory to keep in place (a null buffer).
        if self.size() == 0 {
            return Ok(body(NonNull::dangling().as_ptr(), 0));
        }

        self.lock()?;

        let mut memory = (ptr::null_mut(), 0);
        let got = protect(|| {
            memory = if writable {
                io_buffer::get_bytes_for_writing(value)
            } else {
                let (base, size) = io_buffer::get_bytes_for_reading(value);

                (base as *mut u8, size)
            };

            NilClass::new().value()
        });

        if let Err(error) = got {
            io_buffer::try_unlock(value);

            return Err(error);
        }

        let (base, size) = match memory {
            (base, _) if base.is_null() => (NonNull::dangling().as_ptr(), 0),
            memory => memory,
        };

        let mut outcome = None;

        vm::ensure(
            || {
                outcome = Some(panic::catch_unwind(AssertUnwindSafe(|| body(base, size))));

                NilClass::new().value()
            },
            || {
                io_buffer::try_unlock(value);
            },
        );

        match outcome.expect("ensure body did not run") {
            Ok(result) => Ok(result),
            Err(payload) => panic::resume_unwind(payload),
        }
    }

    /// Runs `f` with the buffer's memory, keeping the buffer locked
    /// (`rb_io_buffer_lock`, `rb_io_buffer_get_bytes_for_reading`) so it
    /// can't be resized, freed or transferred meanwhile.
    ///
    /// Returns `IO::Buffer::LockedError` if the buffer is already locked. A
    /// null buffer gives an empty slice. The lock is released when `f`
    /// returns, panics, or a Ruby exception propagates out of it.
    ///
    /// # Safety
    ///
    /// The lock does not stop writes, so nothing may write to the memory
    /// while `f` runs: not Ruby code that `f` calls, another Ruby thread or
    /// a system call running in one, a slice of the buffer
    /// (`IO::Buffer#slice`, which shares its memory but is locked on its
    /// own), or another buffer over the same memory.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IOBuffer, VM};
    /// # VM::init();
    ///
    /// let buffer = IOBuffer::from_bytes(b"bytes");
    ///
    /// let (copy, locked) = unsafe { buffer.with_bytes(|bytes| (bytes.to_vec(), buffer.is_locked())) }.unwrap();
    ///
    /// assert_eq!(copy, b"bytes");
    /// assert!(locked);
    /// assert!(!buffer.is_locked());
    ///
    /// // The buffer can't be freed while its memory is borrowed.
    /// assert!(unsafe { buffer.with_bytes(|_| buffer.free().is_err()) }.unwrap());
    /// ```
    pub unsafe fn with_bytes<F, R>(&self, f: F) -> Result<R, AnyException>
    where
        F: FnOnce(&[u8]) -> R,
    {
        self.with_locked_memory(false, |base, size| {
            f(slice::from_raw_parts(base as *const u8, size))
        })
    }

    /// Like [`with_bytes`](#method.with_bytes), with the memory writable
    /// (`rb_io_buffer_get_bytes_for_writing`). Returns
    /// `IO::Buffer::AccessError` for a read-only buffer.
    ///
    /// # Safety
    ///
    /// Nothing may read or write the memory while `f` runs, other than
    /// through the slice (see [`with_bytes`](#method.with_bytes)).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IOBuffer, Object, VM};
    /// # VM::init();
    ///
    /// let buffer = IOBuffer::new(3);
    ///
    /// unsafe { buffer.with_bytes_mut(|bytes| bytes.copy_from_slice(b"abc")) }.unwrap();
    /// assert_eq!(buffer.get_string(0, 3).unwrap().to_str(), "abc");
    ///
    /// let for_string = VM::eval("IO::Buffer.for('text')").unwrap().try_convert_to::<IOBuffer>().unwrap();
    /// assert!(unsafe { for_string.with_bytes_mut(|_| ()) }.is_err());
    /// ```
    pub unsafe fn with_bytes_mut<F, R>(&self, f: F) -> Result<R, AnyException>
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        self.with_locked_memory(true, |base, size| f(slice::from_raw_parts_mut(base, size)))
    }

    /// Reads from `io` into the buffer at `offset` and returns the number of
    /// bytes read, `0` at the end of the stream (`rb_io_buffer_read`).
    /// Returns the error for a failed read (`Errno::*`), a closed stream, or
    /// a range beyond the buffer.
    ///
    /// Ruby 3.2+ reads until it has at least `length` bytes or reaches the
    /// end of the stream; Ruby 3.1 makes one read of up to the rest of the
    /// buffer. Either may read more than `length` bytes, up to the end of
    /// the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IOBuffer, Object, RString, IO, VM};
    /// # VM::init();
    ///
    /// let pipe = VM::eval("IO.pipe").unwrap().try_convert_to::<rutie::Array>().unwrap();
    /// let reader = pipe.at(0).try_convert_to::<IO>().unwrap();
    /// let writer = pipe.at(1).try_convert_to::<IO>().unwrap();
    ///
    /// writer.write(&RString::new_utf8("abc")).unwrap();
    ///
    /// let buffer = IOBuffer::new(5);
    /// assert_eq!(buffer.read(&reader, 3, 2).unwrap(), 3);
    /// assert_eq!(buffer.get_string(0, 5).unwrap().to_bytes_unchecked(), b"\0\0abc");
    ///
    /// // Beyond the buffer, and a closed stream.
    /// assert!(buffer.read(&reader, 4, 2).is_err());
    /// reader.close().unwrap();
    /// assert!(buffer.read(&reader, 1, 0).is_err());
    /// # writer.close().unwrap();
    /// ```
    pub fn read(&self, io: &IO, length: usize, offset: usize) -> Result<usize, AnyException> {
        self.check_range(offset, length)?;

        let (buffer, io) = (self.value(), io.value());

        io_count("read", || io_buffer::read(buffer, io, length, offset))
    }

    /// Like [`read`](#method.read), but reads from the file offset `from`
    /// without moving the stream's position (`rb_io_buffer_pread`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{File, IOBuffer, VM};
    /// # VM::init();
    ///
    /// let path = std::env::temp_dir().join(format!("rutie_io_buffer_pread_{}.txt", std::process::id()));
    /// std::fs::write(&path, "0123456789").unwrap();
    ///
    /// let file = File::open(path.to_str().unwrap(), "rb").unwrap();
    /// let buffer = IOBuffer::new(3);
    ///
    /// assert_eq!(buffer.pread(&file, 6, 3, 0).unwrap(), 3);
    /// assert_eq!(buffer.get_string(0, 3).unwrap().to_str(), "678");
    ///
    /// // The position did not move.
    /// assert_eq!(file.getbyte().unwrap(), Some(b'0'));
    /// # file.close().unwrap();
    /// # std::fs::remove_file(path).unwrap();
    /// ```
    pub fn pread(
        &self,
        io: &IO,
        from: u64,
        length: usize,
        offset: usize,
    ) -> Result<usize, AnyException> {
        self.check_range(offset, length)?;

        let from = file_offset(from)?;
        let (buffer, io) = (self.value(), io.value());

        io_count("pread", || {
            io_buffer::pread(buffer, io, from, length, offset)
        })
    }

    /// Writes `length` bytes from `offset` in the buffer to `io` and returns
    /// the number of bytes written (`rb_io_buffer_write`), or the error.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IOBuffer, Object, IO, VM};
    /// # VM::init();
    ///
    /// let pipe = VM::eval("IO.pipe").unwrap().try_convert_to::<rutie::Array>().unwrap();
    /// let reader = pipe.at(0).try_convert_to::<IO>().unwrap();
    /// let writer = pipe.at(1).try_convert_to::<IO>().unwrap();
    ///
    /// let buffer = IOBuffer::from_bytes(b"skip:sent\n");
    /// assert_eq!(buffer.write(&writer, 5, 5).unwrap(), 5);
    ///
    /// assert_eq!(reader.gets().unwrap().unwrap().to_str(), "sent\n");
    /// assert!(buffer.write(&writer, 6, 5).is_err());
    /// # reader.close().unwrap();
    /// # writer.close().unwrap();
    /// ```
    pub fn write(&self, io: &IO, length: usize, offset: usize) -> Result<usize, AnyException> {
        self.check_range(offset, length)?;

        let (buffer, io) = (self.value(), io.value());

        io_count("write", || io_buffer::write(buffer, io, length, offset))
    }

    /// Like [`write`](#method.write), but writes at the file offset `from`
    /// without moving the stream's position (`rb_io_buffer_pwrite`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{File, IOBuffer, VM};
    /// # VM::init();
    ///
    /// let path = std::env::temp_dir().join(format!("rutie_io_buffer_pwrite_{}.txt", std::process::id()));
    /// std::fs::write(&path, "0123456789").unwrap();
    ///
    /// let file = File::open(path.to_str().unwrap(), "r+b").unwrap();
    /// let buffer = IOBuffer::from_bytes(b"ab");
    ///
    /// assert_eq!(buffer.pwrite(&file, 4, 2, 0).unwrap(), 2);
    /// file.close().unwrap();
    ///
    /// assert_eq!(std::fs::read_to_string(&path).unwrap(), "0123ab6789");
    /// # std::fs::remove_file(path).unwrap();
    /// ```
    pub fn pwrite(
        &self,
        io: &IO,
        from: u64,
        length: usize,
        offset: usize,
    ) -> Result<usize, AnyException> {
        self.check_range(offset, length)?;

        let from = file_offset(from)?;
        let (buffer, io) = (self.value(), io.value());

        io_count("pwrite", || {
            io_buffer::pwrite(buffer, io, from, length, offset)
        })
    }
}

impl From<Value> for IOBuffer {
    fn from(value: Value) -> Self {
        IOBuffer { value }
    }
}

impl Into<Value> for IOBuffer {
    fn into(self) -> Value {
        self.value
    }
}

impl Into<AnyObject> for IOBuffer {
    fn into(self) -> AnyObject {
        AnyObject::from(self.value)
    }
}

impl Object for IOBuffer {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for IOBuffer {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        Class::io_buffer().case_equals(object)
    }

    fn error_message() -> &'static str {
        "Error converting to IO::Buffer"
    }
}

impl PartialEq for IOBuffer {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        binding::vm, Array, Class, Exception, File, IOBuffer, NilClass, Object, RString, IO, VM,
    };

    fn pipe() -> (IO, IO) {
        let pipe = VM::eval("IO.pipe")
            .unwrap()
            .try_convert_to::<Array>()
            .unwrap();

        (
            pipe.at(0).try_convert_to::<IO>().unwrap(),
            pipe.at(1).try_convert_to::<IO>().unwrap(),
        )
    }

    #[test]
    fn test_io_buffer_memory_and_locking() {
        crate::on_ruby_thread(|| {
            assert!(IOBuffer::page_size() > 0);
            assert_eq!(IOBuffer::default_size() % IOBuffer::page_size(), 0);

            // A null buffer lends an empty slice.
            let null = IOBuffer::new(0);
            assert!(null.is_null());
            assert_eq!(unsafe { null.with_bytes(|bytes| bytes.len()) }, Ok(0));
            assert_eq!(unsafe { null.with_bytes_mut(|bytes| bytes.len()) }, Ok(0));

            let buffer = IOBuffer::from_bytes(b"abcdef");
            assert!(buffer.is_internal() && !buffer.is_external() && !buffer.is_readonly());

            unsafe { buffer.with_bytes_mut(|bytes| bytes[0] = b'A') }.unwrap();
            assert_eq!(buffer.get_string(0, 6).unwrap().to_str(), "Abcdef");

            // Locked while borrowed: nothing can move or free the memory.
            let attempts = unsafe {
                buffer.with_bytes(|_| {
                    (
                        buffer.resize(1).is_err(),
                        buffer.free().is_err(),
                        buffer.transfer().is_err(),
                        buffer.with_bytes(|_| ()).is_err(),
                    )
                })
            };
            assert_eq!(attempts, Ok((true, true, true, true)));
            assert!(!buffer.is_locked());

            // A buffer locked by someone else is not lent.
            buffer.lock().unwrap();
            assert!(unsafe { buffer.with_bytes(|_| ()) }.is_err());
            assert!(unsafe { buffer.unlock() }.is_ok());

            // Unlocked after a panic, and after a Ruby exception.
            let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
                buffer.with_bytes(|_| panic!("inside with_bytes"))
            }));
            assert!(panicked.is_err());
            assert!(!buffer.is_locked());

            let raised = vm::protect_value(|| {
                let _ =
                    unsafe { buffer.with_bytes(|_| VM::raise(Class::runtime_error(), "inside")) };

                NilClass::new().value()
            });
            assert!(raised.is_err());
            assert!(!buffer.is_locked());

            // Ranges are checked before Ruby sees them.
            assert!(buffer.clear(0, usize::MAX, 2).is_err());
            assert!(buffer.get_string(2, usize::MAX).is_err());
            assert!(buffer
                .set_string(&RString::new_utf8("x"), usize::MAX)
                .is_err());
            buffer.clear(b'-', 4, 2).unwrap();
            assert_eq!(buffer.get_string(0, 6).unwrap().to_str(), "Abcd--");

            let moved = buffer.transfer().unwrap();
            assert!(buffer.is_null());
            assert_eq!(moved.size(), 6);
            moved.free().unwrap();
            assert!(moved.is_null());

            // External memory that is read-only can't be written through.
            let memory = *b"fixed";
            let readonly = unsafe { IOBuffer::from_raw_parts(memory.as_ptr() as *mut u8, 5, true) };
            assert!(readonly.is_readonly() && readonly.is_external());
            assert!(readonly.set_string(&RString::new_utf8("F"), 0).is_err());
            assert!(readonly.clear(0, 0, 1).is_err());
            assert!(readonly.resize(10).is_err());
            assert_eq!(
                unsafe { readonly.with_bytes(|bytes| bytes.to_vec()) },
                Ok(b"fixed".to_vec())
            );
            let error = unsafe { readonly.with_bytes_mut(|_| ()) }.unwrap_err();
            assert_eq!(error.class().path().to_str(), "IO::Buffer::AccessError");
            readonly.free().unwrap();

            assert!(RString::new_utf8("x")
                .to_any_object()
                .try_convert_to::<IOBuffer>()
                .is_err());
            assert!(VM::eval("IO::Buffer.new(1)")
                .unwrap()
                .try_convert_to::<IOBuffer>()
                .is_ok());
        });
    }

    #[test]
    fn test_io_buffer_free_locked() {
        crate::on_ruby_thread(|| {
            let buffer = IOBuffer::new(4);
            buffer.lock().unwrap();

            unsafe { buffer.free_locked() };

            assert!(buffer.is_null());
        });
    }

    #[test]
    fn test_io_buffer_reads_and_writes() {
        crate::on_ruby_thread(|| {
            let (reader, writer) = pipe();

            // Offsets into the buffer (slices on Ruby 3.1).
            let buffer = IOBuffer::from_bytes(b"..xyz");
            assert_eq!(buffer.write(&writer, 3, 2).unwrap(), 3);

            let received = IOBuffer::new(6);
            assert_eq!(received.read(&reader, 3, 3).unwrap(), 3);
            assert_eq!(
                received.get_string(0, 6).unwrap().to_bytes_unchecked(),
                b"\0\0\0xyz"
            );

            writer.close().unwrap();
            assert_eq!(received.read(&reader, 1, 0).unwrap(), 0); // end of stream
            reader.close().unwrap();
            assert!(received.read(&reader, 1, 0).is_err());

            let path = std::env::temp_dir()
                .join(format!("rutie_io_buffer_unit_{}.bin", std::process::id()));
            std::fs::write(&path, "0123456789").unwrap();

            // A failed system call is an `Errno::*` error, not a count. Files,
            // not the wrong ends of a pipe: Windows pipes are non-blocking, and
            // Ruby would wait forever for the wrong end to become ready.
            let write_only = File::open(path.to_str().unwrap(), "ab").unwrap();
            let error = received.read(&write_only, 1, 0).unwrap_err();
            assert!(Class::system_call_error().case_equals(&error));
            write_only.close().unwrap();
            let read_only = File::open(path.to_str().unwrap(), "rb").unwrap();
            let error = buffer.write(&read_only, 1, 0).unwrap_err();
            assert!(Class::system_call_error().case_equals(&error));
            read_only.close().unwrap();
            let file = File::open(path.to_str().unwrap(), "r+b").unwrap();

            let buffer = IOBuffer::new(4);
            assert_eq!(buffer.pread(&file, 2, 2, 2).unwrap(), 2);
            assert_eq!(
                buffer.get_string(0, 4).unwrap().to_bytes_unchecked(),
                b"\0\023"
            );
            assert_eq!(buffer.pwrite(&file, 8, 2, 2).unwrap(), 2);
            assert!(buffer.pread(&file, u64::MAX, 1, 0).is_err());
            assert!(buffer.pread(&file, 0, 5, 0).is_err());

            let shared = unsafe { IOBuffer::map(&file, 10, 0, true, false) }.unwrap();
            assert!(shared.is_mapped() && shared.is_readonly());
            assert_eq!(shared.get_string(0, 10).unwrap().to_str(), "0123456723");
            shared.free().unwrap();

            // A private mapping does not write to the file.
            let private = unsafe { IOBuffer::map(&file, 10, 0, false, true) };
            {
                let private = private.unwrap();
                assert!(private.is_mapped() && !private.is_readonly());
                private.set_string(&RString::new_utf8("AB"), 0).unwrap();
                assert_eq!(private.get_string(0, 3).unwrap().to_str(), "AB2");
                private.free().unwrap();
            }

            file.close().unwrap();
            assert_eq!(std::fs::read_to_string(&path).unwrap(), "0123456723");
            std::fs::remove_file(path).unwrap();
        });
    }
}
