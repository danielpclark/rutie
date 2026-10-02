use std::{cell::UnsafeCell, ffi::CStr, mem::MaybeUninit, ptr, slice};

use crate::{
    binding::{memory_view, vm},
    rubysys::{
        memory_view::{rb_memory_view_t, RUBY_MEMORY_VIEW_FORMAT, RUBY_MEMORY_VIEW_INDIRECT},
        types::ssize_t,
    },
    util, AnyException, AnyObject, Exception, Object,
};

/// A read-only MemoryView (`ruby/memory_view.h`): memory that an object
/// exports to other code without copying, described as an array of items
/// with a format, a shape and strides (like Python's buffer protocol).
///
/// Ruby's core classes export none, but extensions do, such as
/// `Fiddle::Pointer` and numeric array libraries. The view keeps the
/// object alive, and is released when dropped (`rb_memory_view_release`).
/// It belongs to the Ruby thread that got it.
///
/// # Examples
///
/// ```
/// use rutie::{MemoryView, VM};
/// # VM::init();
///
/// VM::require("fiddle");
/// let pointer = VM::eval("Fiddle::Pointer[$rutie_view_bytes = 'abcd'.b.freeze]").unwrap();
///
/// let view = MemoryView::new(&pointer).unwrap();
///
/// assert_eq!(view.byte_size(), 4);
/// assert_eq!(view.format(), None); // unsigned bytes
/// assert_eq!(view.shape(), [4]);
/// assert_eq!(view.to_vec(), Some(b"abcd".to_vec()));
/// ```
pub struct MemoryView {
    // Boxed so the view stays where the exporter filled it in. Ruby writes
    // to it (`rb_memory_view_get_item` prepares `item_desc`).
    view: Box<UnsafeCell<rb_memory_view_t>>,
}

impl MemoryView {
    /// Gets a read-only view of the memory `object` exports
    /// (`rb_memory_view_get`), or `None` if it exports none.
    ///
    /// Any shape and format are accepted (`RUBY_MEMORY_VIEW_FORMAT` and
    /// `RUBY_MEMORY_VIEW_INDIRECT`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{MemoryView, RString, VM};
    /// # VM::init();
    ///
    /// VM::require("fiddle");
    /// let pointer = VM::eval("Fiddle::Pointer[$rutie_view_new = 'xyz'.b.freeze]").unwrap();
    ///
    /// assert_eq!(MemoryView::new(&pointer).unwrap().byte_size(), 3);
    /// assert!(MemoryView::new(&RString::new_utf8("exports nothing")).is_none());
    /// ```
    pub fn new<T: Object>(object: &T) -> Option<Self> {
        // All-zero is a valid (empty) `rb_memory_view_t`.
        let view = Box::new(UnsafeCell::new(unsafe {
            MaybeUninit::<rb_memory_view_t>::zeroed().assume_init()
        }));
        let flags = RUBY_MEMORY_VIEW_FORMAT | RUBY_MEMORY_VIEW_INDIRECT;

        if unsafe { memory_view::get(object.value(), view.get(), flags) } {
            Some(MemoryView { view })
        } else {
            None
        }
    }

    /// Returns `true` if `object` supports exporting a MemoryView
    /// (`rb_memory_view_available_p`), though
    /// [`MemoryView::new`](#method.new) may still fail.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, MemoryView, VM};
    /// # VM::init();
    ///
    /// VM::require("fiddle");
    /// let pointer = VM::eval("Fiddle::Pointer.malloc(8, Fiddle::RUBY_FREE)").unwrap();
    ///
    /// assert!(MemoryView::is_available(&pointer));
    /// assert!(!MemoryView::is_available(&Fixnum::new(1)));
    /// ```
    pub fn is_available<T: Object>(object: &T) -> bool {
        memory_view::is_available(object.value())
    }

    /// Returns the size of the item format `format`, in bytes
    /// (`rb_memory_view_item_size_from_format`), or `None` if it is not a
    /// valid format.
    ///
    /// A format is a sequence of `pack` template specifiers, such as `"dd"`
    /// for two doubles, with `|` at the start to align the members like a C
    /// struct.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{MemoryView, VM};
    /// # VM::init();
    ///
    /// assert_eq!(MemoryView::item_size_of("CCC"), Some(3));
    /// assert_eq!(MemoryView::item_size_of("q<d"), Some(16));
    /// assert_eq!(MemoryView::item_size_of("s2"), Some(4));
    /// assert_eq!(MemoryView::item_size_of("?"), None);
    /// assert_eq!(MemoryView::item_size_of("C\0"), None);
    /// ```
    pub fn item_size_of(format: &str) -> Option<usize> {
        if format.contains('\0') {
            return None;
        }

        memory_view::item_size_from_format(&util::str_to_cstring(format))
    }

    fn raw(&self) -> &rb_memory_view_t {
        unsafe { &*self.view.get() }
    }

    /// Returns the object that exported the memory.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{MemoryView, Object, VM};
    /// # VM::init();
    ///
    /// VM::require("fiddle");
    /// let pointer = VM::eval("Fiddle::Pointer.malloc(2, Fiddle::RUBY_FREE)").unwrap();
    ///
    /// assert!(MemoryView::new(&pointer).unwrap().object().equals(&pointer));
    /// ```
    pub fn object(&self) -> AnyObject {
        AnyObject::from(self.raw().obj)
    }

    /// Returns the number of bytes of memory.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{MemoryView, VM};
    /// # VM::init();
    ///
    /// VM::require("fiddle");
    /// let pointer = VM::eval("Fiddle::Pointer.malloc(6, Fiddle::RUBY_FREE)").unwrap();
    ///
    /// assert_eq!(MemoryView::new(&pointer).unwrap().byte_size(), 6);
    /// ```
    pub fn byte_size(&self) -> usize {
        self.raw().byte_size.max(0) as usize
    }

    /// Returns `true` if the exporter does not allow writing to the memory.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{MemoryView, Object, VM};
    /// # VM::init();
    ///
    /// VM::require("fiddle");
    /// let pointer = VM::eval("$rutie_view_pointer = Fiddle::Pointer.malloc(1, Fiddle::RUBY_FREE)").unwrap();
    /// let view = MemoryView::new(&pointer).unwrap();
    ///
    /// // What Fiddle's own MemoryView reads.
    /// let expected = VM::eval("Fiddle::MemoryView.new($rutie_view_pointer).readonly?").unwrap();
    /// assert_eq!(view.is_readonly(), expected.value().is_true());
    /// ```
    pub fn is_readonly(&self) -> bool {
        self.raw().readonly
    }

    /// Returns the format of an item as `pack` template specifiers (such as
    /// `"dd"` or `"CCC"`), or `None` for unsigned bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{MemoryView, VM};
    /// # VM::init();
    ///
    /// VM::require("fiddle");
    /// let pointer = VM::eval("Fiddle::Pointer.malloc(4, Fiddle::RUBY_FREE)").unwrap();
    /// let view = MemoryView::new(&pointer).unwrap();
    ///
    /// assert_eq!(view.format(), None);
    /// assert_eq!(view.item_size(), 1);
    /// ```
    pub fn format(&self) -> Option<&str> {
        let format = self.raw().format;

        if format.is_null() {
            None
        } else {
            unsafe { CStr::from_ptr(format) }.to_str().ok()
        }
    }

    /// Returns the size of an item in bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{MemoryView, VM};
    /// # VM::init();
    ///
    /// VM::require("fiddle");
    /// let pointer = VM::eval("Fiddle::Pointer.malloc(4, Fiddle::RUBY_FREE)").unwrap();
    ///
    /// assert_eq!(MemoryView::new(&pointer).unwrap().item_size(), 1);
    /// ```
    pub fn item_size(&self) -> usize {
        self.raw().item_size.max(0) as usize
    }

    /// Returns the number of dimensions.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{MemoryView, VM};
    /// # VM::init();
    ///
    /// VM::require("fiddle");
    /// let pointer = VM::eval("Fiddle::Pointer.malloc(4, Fiddle::RUBY_FREE)").unwrap();
    ///
    /// assert_eq!(MemoryView::new(&pointer).unwrap().ndim(), 1);
    /// ```
    pub fn ndim(&self) -> usize {
        self.raw().ndim.max(0) as usize
    }

    fn raw_shape(&self) -> Vec<ssize_t> {
        let view = self.raw();
        let ndim = self.ndim();

        if !view.shape.is_null() {
            unsafe { slice::from_raw_parts(view.shape, ndim) }.to_vec()
        } else if ndim == 1 {
            // A null shape is allowed for one dimension.
            vec![view.byte_size / view.item_size.max(1)]
        } else {
            vec![0; ndim]
        }
    }

    /// Returns the number of items in each dimension.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{MemoryView, VM};
    /// # VM::init();
    ///
    /// VM::require("fiddle");
    /// let pointer = VM::eval("Fiddle::Pointer.malloc(5, Fiddle::RUBY_FREE)").unwrap();
    ///
    /// assert_eq!(MemoryView::new(&pointer).unwrap().shape(), [5]);
    /// ```
    pub fn shape(&self) -> Vec<usize> {
        self.raw_shape()
            .into_iter()
            .map(|count| count.max(0) as usize)
            .collect()
    }

    /// Returns the number of bytes between consecutive items in each
    /// dimension (negative to go backwards). A view without strides is
    /// contiguous in row-major order, and gets those
    /// (`rb_memory_view_fill_contiguous_strides`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{MemoryView, VM};
    /// # VM::init();
    ///
    /// VM::require("fiddle");
    /// let pointer = VM::eval("Fiddle::Pointer.malloc(5, Fiddle::RUBY_FREE)").unwrap();
    ///
    /// assert_eq!(MemoryView::new(&pointer).unwrap().strides(), [1]);
    /// ```
    pub fn strides(&self) -> Vec<isize> {
        let view = self.raw();

        let strides = if view.strides.is_null() {
            memory_view::contiguous_strides(view.item_size, &self.raw_shape(), true)
        } else {
            unsafe { slice::from_raw_parts(view.strides, self.ndim()) }.to_vec()
        };

        strides.into_iter().map(|stride| stride as isize).collect()
    }

    /// Returns `true` if the items are laid out next to each other, in
    /// row-major (C) or column-major (Fortran) order
    /// (`rb_memory_view_is_row_major_contiguous`,
    /// `rb_memory_view_is_column_major_contiguous`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{MemoryView, VM};
    /// # VM::init();
    ///
    /// VM::require("fiddle");
    /// let pointer = VM::eval("Fiddle::Pointer.malloc(3, Fiddle::RUBY_FREE)").unwrap();
    ///
    /// assert!(MemoryView::new(&pointer).unwrap().is_contiguous());
    /// ```
    pub fn is_contiguous(&self) -> bool {
        let view = self.raw();

        // An indirect (nested) array is not one block of memory.
        if !view.sub_offsets.is_null() {
            false
        } else if view.strides.is_null() {
            // No strides: row-major, the way `rb_memory_view_get_item` reads
            // it (and Ruby's checks would read a null pointer).
            true
        } else if view.shape.is_null() {
            // Only allowed for one dimension.
            self.strides() == [view.item_size as isize]
        } else {
            unsafe { memory_view::is_contiguous(self.view.get()) }
        }
    }

    // The memory, when it is one block.
    fn contiguous_memory(&self) -> Option<(*const u8, usize)> {
        if !self.is_contiguous() {
            return None;
        }

        let (data, size) = (self.raw().data as *const u8, self.byte_size());

        if data.is_null() || size == 0 {
            Some((ptr::NonNull::dangling().as_ptr(), 0))
        } else {
            Some((data, size))
        }
    }

    /// Copies the memory of a contiguous view (see
    /// [`is_contiguous`](#method.is_contiguous)), or returns `None` for
    /// another layout.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{MemoryView, VM};
    /// # VM::init();
    ///
    /// VM::require("fiddle");
    /// let pointer = VM::eval("Fiddle::Pointer[$rutie_view_copy = \"\\x01\\x02\".b.freeze]").unwrap();
    ///
    /// assert_eq!(MemoryView::new(&pointer).unwrap().to_vec(), Some(vec![1, 2]));
    /// ```
    pub fn to_vec(&self) -> Option<Vec<u8>> {
        self.contiguous_memory().map(|(data, size)| {
            let mut bytes = Vec::with_capacity(size);

            unsafe {
                ptr::copy_nonoverlapping(data, bytes.as_mut_ptr(), size);
                bytes.set_len(size);
            }

            bytes
        })
    }

    /// Returns the memory of a contiguous view (see
    /// [`is_contiguous`](#method.is_contiguous)) without copying it, or
    /// `None` for another layout.
    ///
    /// # Safety
    ///
    /// The view keeps the memory allocated, but not unchanged: the exporter
    /// or Ruby code can still write to it. Nothing may write to it while the
    /// slice is used, including Ruby code called meanwhile and other Ruby
    /// threads.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{MemoryView, VM};
    /// # VM::init();
    ///
    /// VM::require("fiddle");
    /// let pointer = VM::eval("Fiddle::Pointer[$rutie_view_slice = 'slice'.b.freeze]").unwrap();
    /// let view = MemoryView::new(&pointer).unwrap();
    ///
    /// assert_eq!(unsafe { view.as_bytes() }, Some(&b"slice"[..]));
    /// ```
    pub unsafe fn as_bytes(&self) -> Option<&[u8]> {
        self.contiguous_memory()
            .map(|(data, size)| slice::from_raw_parts(data, size))
    }

    /// Returns the item at `indices` (one index per dimension) as a Ruby
    /// value, an Array for an item with several members
    /// (`rb_memory_view_get_item`). Returns `IndexError` for indices
    /// outside the shape, or the error for a format Ruby can't read.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, MemoryView, Object, VM};
    /// # VM::init();
    ///
    /// VM::require("fiddle");
    /// let pointer = VM::eval("Fiddle::Pointer[$rutie_view_item = \"\\x05\\xff\".b.freeze]").unwrap();
    /// let view = MemoryView::new(&pointer).unwrap();
    ///
    /// assert_eq!(view.get_item(&[1]).unwrap().try_convert_to::<Fixnum>().unwrap().to_i64(), 255);
    /// assert!(view.get_item(&[2]).is_err());
    /// assert!(view.get_item(&[0, 0]).is_err());
    /// ```
    pub fn get_item(&self, indices: &[usize]) -> Result<AnyObject, AnyException> {
        let shape = self.shape();

        if indices.len() != shape.len() || indices.iter().zip(&shape).any(|(i, n)| i >= n) {
            return Err(AnyException::new(
                "IndexError",
                Some(&format!(
                    "indices {:?} are outside the shape {:?}",
                    indices, shape
                )),
            ));
        }

        let indices: Vec<ssize_t> = indices.iter().map(|&index| index as ssize_t).collect();
        let view = self.view.get();

        vm::protect_value(|| unsafe { memory_view::get_item(view, &indices) })
            .map(AnyObject::from)
            .map_err(AnyException::from)
    }
}

impl Drop for MemoryView {
    fn drop(&mut self) {
        unsafe { memory_view::release(self.view.get()) };
    }
}

impl ::std::fmt::Debug for MemoryView {
    fn fmt(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        formatter
            .debug_struct("MemoryView")
            .field("byte_size", &self.byte_size())
            .field("readonly", &self.is_readonly())
            .field("format", &self.format())
            .field("shape", &self.shape())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        mem, ptr,
        sync::atomic::{AtomicUsize, Ordering},
    };

    use crate::{
        rubysys::{
            memory_view::*,
            types::{c_char, c_int, c_long, c_void, ssize_t},
        },
        types::Value,
        Class, Exception, Fixnum, MemoryView, Object, RString, VM,
    };

    // A 2x3 row-major array of shorts.
    static DATA: [i16; 6] = [1, 2, 3, 4, 5, 6];
    static SHAPE: [ssize_t; 2] = [2, 3];
    static ROW_MAJOR: [ssize_t; 2] = [6, 2];
    // The first two columns only.
    static SHAPE_2X2: [ssize_t; 2] = [2, 2];
    static FORMAT: &[u8] = b"s\0";

    static RELEASED: AtomicUsize = AtomicUsize::new(0);

    unsafe fn fill(obj: Value, view: *mut rb_memory_view_t, shape: &'static [ssize_t; 2]) -> bool {
        let filled = rb_memory_view_init_as_byte_array(
            view,
            obj,
            DATA.as_ptr() as *mut c_void,
            mem::size_of_val(&DATA) as ssize_t,
            true,
        );

        (*view).format = FORMAT.as_ptr() as *const c_char;
        (*view).item_size = 2;
        (*view).ndim = 2;
        (*view).shape = shape.as_ptr();
        (*view).strides = ROW_MAJOR.as_ptr();

        filled
    }

    rutie_callback! {
        fn get_full(obj: Value, view: *mut rb_memory_view_t, _flags: c_int) -> bool {
            unsafe { fill(obj, view, &SHAPE) }
        }
    }

    rutie_callback! {
        fn get_columns(obj: Value, view: *mut rb_memory_view_t, _flags: c_int) -> bool {
            unsafe { fill(obj, view, &SHAPE_2X2) }
        }
    }

    rutie_callback! {
        fn release(_obj: Value, _view: *mut rb_memory_view_t) -> bool {
            RELEASED.fetch_add(1, Ordering::SeqCst);

            true
        }
    }

    rutie_callback! {
        fn available(_obj: Value) -> bool {
            true
        }
    }

    static FULL: rb_memory_view_entry_t = rb_memory_view_entry_t {
        get_func: Some(get_full),
        release_func: Some(release),
        available_p_func: Some(available),
    };

    static COLUMNS: rb_memory_view_entry_t = rb_memory_view_entry_t {
        get_func: Some(get_columns),
        release_func: Some(release),
        available_p_func: Some(available),
    };

    #[test]
    fn test_memory_view_layouts() {
        #[cfg(target_pointer_width = "64")]
        {
            // Measured against the C headers of 3.1, 3.2 and 3.3.
            assert_eq!(mem::size_of::<rb_memory_view_t>(), 112);
            assert_eq!(mem::size_of::<rb_memory_view_item_component_t>(), 32);
        }

        assert_eq!(
            mem::size_of::<rb_memory_view_entry_t>(),
            3 * mem::size_of::<usize>()
        );
    }

    #[test]
    fn test_memory_view_from_registered_exporter() {
        crate::on_ruby_thread(|| {
            let full_class = Class::new("RutieMemoryViewFull", None);
            let columns_class = Class::new("RutieMemoryViewColumns", None);

            unsafe {
                assert!(rb_memory_view_register(full_class.value(), &FULL));
                assert!(rb_memory_view_register(columns_class.value(), &COLUMNS));
            }

            let full = full_class.new_instance(&[]);
            assert!(MemoryView::is_available(&full));

            let released = RELEASED.load(Ordering::SeqCst);
            let view = MemoryView::new(&full).unwrap();

            assert!(view.object().equals(&full));
            assert_eq!(view.byte_size(), 12);
            assert!(view.is_readonly());
            assert_eq!(view.format(), Some("s"));
            assert_eq!(view.item_size(), 2);
            assert_eq!(view.ndim(), 2);
            assert_eq!(view.shape(), [2, 3]);
            assert_eq!(view.strides(), [6, 2]);
            assert!(view.is_contiguous());

            let expected: Vec<u8> = DATA.iter().flat_map(|n| n.to_ne_bytes()).collect();
            assert_eq!(view.to_vec(), Some(expected.clone()));
            assert_eq!(unsafe { view.as_bytes() }, Some(&expected[..]));

            let item = |view: &MemoryView, indices: &[usize]| {
                view.get_item(indices)
                    .unwrap()
                    .try_convert_to::<Fixnum>()
                    .unwrap()
                    .to_i64()
            };
            assert_eq!(item(&view, &[0, 0]), 1);
            assert_eq!(item(&view, &[1, 2]), 6);
            assert!(view.get_item(&[2, 0]).is_err());
            assert!(view.get_item(&[0, 3]).is_err());
            let error = view.get_item(&[0]).unwrap_err();
            assert_eq!(error.class().path().to_str(), "IndexError");

            drop(view);
            assert_eq!(RELEASED.load(Ordering::SeqCst), released + 1);

            // Strides that skip the last column: not one block of memory.
            let columns = MemoryView::new(&columns_class.new_instance(&[])).unwrap();
            assert_eq!(columns.shape(), [2, 2]);
            assert!(!columns.is_contiguous());
            assert_eq!(columns.to_vec(), None);
            assert_eq!(unsafe { columns.as_bytes() }, None);
            assert_eq!(item(&columns, &[1, 1]), 5);
            drop(columns);
            assert_eq!(RELEASED.load(Ordering::SeqCst), released + 2);

            assert!(!MemoryView::is_available(&RString::new_utf8("no")));
            assert!(MemoryView::new(&RString::new_utf8("no")).is_none());
            let _ = VM::eval("1");
        });
    }

    #[test]
    fn test_memory_view_formats() {
        crate::on_ruby_thread(|| {
            assert_eq!(MemoryView::item_size_of(""), Some(0));
            assert_eq!(MemoryView::item_size_of("dd"), Some(16));
            assert_eq!(
                MemoryView::item_size_of("l!"),
                Some(mem::size_of::<c_long>())
            );
            assert_eq!(MemoryView::item_size_of("Z"), None);

            let mut members = ptr::null_mut();
            let mut count = 0;
            let mut error = ptr::null();
            let size = unsafe {
                rb_memory_view_parse_item_format(
                    b"s<l!\0".as_ptr() as *const c_char,
                    &mut members,
                    &mut count,
                    &mut error,
                )
            };

            assert_eq!(size as usize, 2 + mem::size_of::<c_long>());
            assert_eq!(count, 2);

            // Allocated with `ruby_xmalloc`; freed below.
            let members = unsafe { std::slice::from_raw_parts(members, count) };

            assert_eq!(members[0].format as u8, b's');
            assert_eq!(members[0].size, 2);
            assert_eq!(members[0].offset, 0);
            assert!(members[0].little_endian_p());
            assert!(!members[0].native_size_p());

            assert_eq!(members[1].format as u8, b'l');
            assert_eq!(members[1].offset, 2);
            assert_eq!(members[1].size, mem::size_of::<c_long>());
            assert!(members[1].native_size_p());
            assert_eq!(members[1].little_endian_p(), cfg!(target_endian = "little"));
            assert_eq!(members[1].repeat, 1);

            // Unpacking one item with those members.
            let mut item = [0u8; 16];
            item[..2].copy_from_slice(&(-2i16).to_le_bytes());
            item[2..2 + mem::size_of::<c_long>()].copy_from_slice(&(7 as c_long).to_ne_bytes());

            let values = unsafe {
                rb_memory_view_extract_item_members(
                    item.as_ptr() as *const c_void,
                    members.as_ptr(),
                    members.len(),
                )
            };
            let values = crate::AnyObject::from(values)
                .try_convert_to::<crate::Array>()
                .unwrap();

            assert_eq!(values.length(), 2);
            assert_eq!(values.at(0).try_convert_to::<Fixnum>(), Ok(Fixnum::new(-2)));
            assert_eq!(values.at(1).try_convert_to::<Fixnum>(), Ok(Fixnum::new(7)));

            unsafe { crate::rubysys::gc::ruby_xfree(members.as_ptr() as *mut c_void) };
        });
    }
}
