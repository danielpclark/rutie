use std::{
    ffi::{CStr, CString},
    ptr,
};

use crate::{
    binding::{array, fixnum, global::RubySpecialConsts, vm},
    rubysys::{exception, file, io, string},
    types::{c_char, c_int, c_long, c_void, InternalValue, Value},
    util,
};

fn nil() -> Value {
    Value::from(RubySpecialConsts::Nil as InternalValue)
}

pub fn stdin() -> Value {
    unsafe { io::rb_stdin }
}

pub fn stdout() -> Value {
    unsafe { io::rb_stdout }
}

pub fn stderr() -> Value {
    unsafe { io::rb_stderr }
}

pub fn write(io: Value, string: Value) -> Value {
    unsafe { io::rb_io_write(io, string) }
}

pub fn puts(io: Value, arguments: &[Value]) -> Value {
    let (argc, argv) = util::process_arguments(arguments);

    unsafe { io::rb_io_puts(argc, argv, io) }
}

pub fn print(io: Value, arguments: &[Value]) -> Value {
    let (argc, argv) = util::process_arguments(arguments);

    unsafe { io::rb_io_print(argc, argv, io) }
}

pub fn gets(io: Value) -> Value {
    unsafe { io::rb_io_gets(io) }
}

pub fn getbyte(io: Value) -> Value {
    unsafe { io::rb_io_getbyte(io) }
}

pub fn flush(io: Value) -> Value {
    unsafe { io::rb_io_flush(io) }
}

pub fn close(io: Value) -> Value {
    unsafe { io::rb_io_close(io) }
}

pub fn is_eof(io: Value) -> bool {
    unsafe { io::rb_io_eof(io) }.is_true()
}

pub use crate::rubysys::io::{RUBY_IO_PRIORITY, RUBY_IO_READABLE, RUBY_IO_WRITABLE};

/// `timeout` is `nil` to wait without a limit (the IO's `#timeout`, which is
/// `nil` unless set).
pub fn wait(io: Value, events: c_int, timeout: Value) -> Value {
    unsafe { io::rb_io_wait(io, fixnum::i32_to_num(events), timeout) }
}

pub fn is_closed(io: Value) -> bool {
    unsafe { io::rb_io_closed_p(io) }.is_true()
}

// Raises `IOError` for a closed stream.
pub fn descriptor(io: Value) -> c_int {
    unsafe { io::rb_io_descriptor(io) }
}

// The ready events as an Integer, or `false`.
pub fn maybe_wait(error: c_int, io: Value, events: c_int, timeout: Value) -> Value {
    unsafe { io::rb_io_maybe_wait(error, io, fixnum::i32_to_num(events), timeout) }
}

pub fn maybe_wait_readable(error: c_int, io: Value, timeout: Value) -> c_int {
    unsafe { io::rb_io_maybe_wait_readable(error, io, timeout) }
}

pub fn maybe_wait_writable(error: c_int, io: Value, timeout: Value) -> c_int {
    unsafe { io::rb_io_maybe_wait_writable(error, io, timeout) }
}

pub fn timeout(io: Value) -> Value {
    unsafe { io::rb_io_timeout(io) }
}

pub fn set_timeout(io: Value, timeout: Value) -> Value {
    unsafe { io::rb_io_set_timeout(io, timeout) }
}

// An `IO` for `fd`, with no path, timeout or encodings.
pub unsafe fn open_descriptor(fd: c_int, mode: c_int) -> Value {
    io::rb_io_open_descriptor(
        crate::rubysys::builtins::rb_cIO,
        fd,
        mode,
        nil(),
        nil(),
        std::ptr::null_mut(),
    )
}

// Raises for a closed file.
pub fn file_size(file: Value) -> i64 {
    unsafe { io::rb_file_size(file) }
}

pub fn binmode(io: Value) -> Value {
    unsafe { io::rb_io_ascii8bit_binmode(io) }
}

// Raises (usually `Errno::*`) when the file cannot be opened, so the caller
// owns `path` and `mode`.
pub fn file_open(path: Value, mode: &std::ffi::CStr) -> Value {
    unsafe { io::rb_file_open_str(path, mode.as_ptr()) }
}

pub fn expand_path(path: Value, directory: Option<Value>) -> Value {
    unsafe { io::rb_file_expand_path(path, directory.unwrap_or_else(nil)) }
}

pub fn absolute_path(path: Value, directory: Option<Value>) -> Value {
    unsafe { io::rb_file_absolute_path(path, directory.unwrap_or_else(nil)) }
}

pub fn dirname(path: Value) -> Value {
    unsafe { io::rb_file_dirname(path) }
}

pub fn getwd() -> Value {
    unsafe { io::rb_dir_getwd() }
}

// The full path of `name` found in `$LOAD_PATH`, or `None`.
pub fn find_file(name: Value) -> Option<Value> {
    let found = unsafe { io::rb_find_file(name) };

    if found.value == 0 || found.is_nil() {
        None
    } else {
        Some(found)
    }
}

pub fn marshal_dump(object: Value) -> Value {
    unsafe { io::rb_marshal_dump(object, nil()) }
}

pub fn marshal_load(data: Value) -> Value {
    unsafe { io::rb_marshal_load(data) }
}

// `Ok(())`, or the non-zero state from `rb_load_protect`.
pub fn load_protect(path: Value, wrap: bool) -> Result<(), c_int> {
    let mut state = 0;

    unsafe { io::rb_load_protect(path, util::bool_to_c_int(wrap), &mut state) };

    if state == 0 {
        Ok(())
    } else {
        Err(state)
    }
}

pub fn require(name: Value) -> Value {
    unsafe { io::rb_f_require(nil(), name) }
}

// `rb_provide` keeps the pointer: it goes through `rb_fstring_cstr`, which
// registers a string pointing at `feature` instead of copying it (it is meant
// for C string literals). So the name is leaked on purpose; freeing it left
// `$LOADED_FEATURES` reading freed memory.
pub fn provide(feature: &str) {
    let feature = util::str_to_cstring(feature).into_raw();

    unsafe { io::rb_provide(feature) }
}

pub fn is_provided(feature: &str) -> bool {
    let feature = util::str_to_cstring(feature);

    util::c_int_to_bool(unsafe { io::rb_provided(feature.as_ptr()) })
}

pub fn add_load_path(path: &str) {
    let path = util::str_to_cstring(path);

    unsafe { io::ruby_incpush(path.as_ptr()) }
}

// `rb_sys_fail` with `errno` as a C function left it.
fn sys_fail(message: Option<&CStr>) -> ! {
    unsafe { exception::rb_sys_fail(message.map_or(ptr::null(), |message| message.as_ptr())) }
}

pub fn check_io(object: Value) -> Value {
    unsafe { io::rb_io_check_io(object) }
}

pub fn get_io(object: Value) -> Value {
    unsafe { io::rb_io_get_io(object) }
}

pub fn set_write_io(io: Value, write_io: Value) -> Value {
    unsafe { io::rb_io_set_write_io(io, write_io) }
}

pub fn ungetbyte(io: Value, byte: Value) -> Value {
    unsafe { io::rb_io_ungetbyte(io, byte) }
}

pub fn printf(io: Value, arguments: &[Value]) -> Value {
    let (argc, argv) = util::process_arguments(arguments);

    unsafe { io::rb_io_printf(argc, argv, io) }
}

// The number of bytes written; raises the `SystemCallError` for a failed
// write.
pub fn bufwrite(io: Value, bytes: &[u8]) -> usize {
    let written = unsafe { io::rb_io_bufwrite(io, bytes.as_ptr() as *const c_void, bytes.len()) };

    if written < 0 {
        sys_fail(None)
    }

    written as usize
}

// The two ends of a new pipe, as `IO`s that own them; raises the
// `SystemCallError` when there is no pipe.
pub fn pipe() -> (Value, Value) {
    let mut fds: [c_int; 2] = [-1, -1];

    if unsafe { io::rb_pipe(fds.as_mut_ptr()) } != 0 {
        sys_fail(None)
    }

    // `O_RDONLY` and `O_WRONLY`, which are 0 and 1 everywhere.
    let reader = unsafe { io::rb_io_fdopen(fds[0], 0, ptr::null()) };
    let writer = unsafe { io::rb_io_fdopen(fds[1], 1, ptr::null()) };

    // Unbuffered, as `IO.pipe` makes it.
    vm::call_method(writer, "sync=", &[util::bool_to_value(true)]);

    (reader, writer)
}

pub fn fdopen(fd: c_int, flags: c_int, path: Option<&CStr>) -> Value {
    unsafe { io::rb_io_fdopen(fd, flags, path.map_or(ptr::null(), |path| path.as_ptr())) }
}

pub fn modestr_fmode(mode: &CStr) -> c_int {
    unsafe { io::rb_io_modestr_fmode(mode.as_ptr()) }
}

pub fn modestr_oflags(mode: &CStr) -> c_int {
    unsafe { io::rb_io_modestr_oflags(mode.as_ptr()) }
}

pub fn oflags_fmode(oflags: c_int) -> c_int {
    unsafe { io::rb_io_oflags_fmode(oflags) }
}

pub fn is_reserved_fd(fd: c_int) -> bool {
    util::c_int_to_bool(unsafe { io::rb_reserved_fd_p(fd) })
}

pub fn kernel_gets() -> Value {
    unsafe { io::rb_gets() }
}

pub fn eof_error() -> ! {
    unsafe { io::rb_eof_error() }
}

pub fn write_error(message: &str) {
    unsafe { io::rb_write_error2(message.as_ptr() as *const c_char, message.len() as c_long) }
}

pub fn is_directory(path: Value) -> bool {
    unsafe { file::rb_file_directory_p(nil(), path) }.is_true()
}

pub fn is_absolute_path(path: &CStr) -> bool {
    util::c_int_to_bool(unsafe { file::rb_is_absolute_path(path.as_ptr()) })
}

pub fn encode_ospath(path: Value) -> Value {
    unsafe { file::rb_str_encode_ospath(path) }
}

pub fn get_path(object: Value) -> Value {
    unsafe { file::rb_get_path(object) }
}

// The full path and the index of the extension found, or `None`.
pub fn find_file_ext(name: Value, extensions: &[CString]) -> Option<(Value, usize)> {
    let mut extensions: Vec<*const c_char> = extensions.iter().map(|e| e.as_ptr()).collect();
    extensions.push(ptr::null());

    let mut feature = name;
    let found = unsafe { file::rb_find_file_ext(&mut feature, extensions.as_ptr()) };

    if found == 0 {
        None
    } else {
        Some((feature, found as usize - 1))
    }
}

// `arg` is the Array the paths are pushed to.
rutie_callback! {
    fn glob_push(path: *const c_char, arg: Value, enc: *mut c_void) {
        let path = unsafe { CStr::from_ptr(path) }.to_bytes();
        let path = unsafe {
            string::rb_enc_str_new(path.as_ptr() as *const c_char, path.len() as c_long, enc)
        };

        array::push(arg, path);
    }
}

rutie_callback! {
    fn ruby_glob_push(path: *const c_char, arg: Value, enc: *mut c_void) -> c_int {
        glob_push(path, arg, enc);

        0
    }
}

// The matching paths, as an Array.
pub fn glob(pattern: &CStr) -> Value {
    let paths = array::new();

    unsafe { file::rb_glob(pattern.as_ptr(), glob_push, paths) };

    paths
}

pub fn glob_with_flags(pattern: &CStr, flags: c_int) -> Value {
    let paths = array::new();

    unsafe { file::ruby_glob(pattern.as_ptr(), flags, ruby_glob_push, paths) };

    paths
}
