use crate::{
    binding::{fixnum, global::RubySpecialConsts},
    rubysys::io,
    types::{c_int, InternalValue, Value},
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

/// `timeout` is `nil` to wait without a limit (Ruby 3.2+: the IO's
/// `#timeout`, which is `nil` unless set).
pub fn wait(io: Value, events: c_int, timeout: Value) -> Value {
    unsafe { io::rb_io_wait(io, fixnum::i32_to_num(events), timeout) }
}

// `rb_io_closed_p` is Ruby 3.3+; earlier Rubies call `closed?`.
#[cfg(ruby_gte_3_3)]
pub fn is_closed(io: Value) -> bool {
    unsafe { io::rb_io_closed_p(io) }.is_true()
}

#[cfg(not(ruby_gte_3_3))]
pub fn is_closed(io: Value) -> bool {
    crate::binding::vm::call_method(io, "closed?", &[]).is_true()
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

#[cfg(ruby_gte_3_2)]
pub fn timeout(io: Value) -> Value {
    unsafe { io::rb_io_timeout(io) }
}

#[cfg(ruby_gte_3_2)]
pub fn set_timeout(io: Value, timeout: Value) -> Value {
    unsafe { io::rb_io_set_timeout(io, timeout) }
}

// An `IO` for `fd`, with no path, timeout or encodings.
#[cfg(ruby_gte_3_3)]
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
