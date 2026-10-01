use crate::rubysys::types::{c_char, c_int, Argc, EncodingType, Value};

#[cfg(ruby_gte_3_1)]
use crate::rubysys::io_buffer::rb_off_t;

// `enum rb_io_event_t` (`RB_WAITFD_IN` / `RB_WAITFD_PRI` / `RB_WAITFD_OUT`).
pub const RUBY_IO_READABLE: c_int = 0x001;
pub const RUBY_IO_PRIORITY: c_int = 0x002;
pub const RUBY_IO_WRITABLE: c_int = 0x004;

// `FMODE_*`: the `mode` of an IO (`rb_io_t::mode`, `rb_io_open_descriptor`).
pub const FMODE_READABLE: c_int = 0x0000_0001;
pub const FMODE_WRITABLE: c_int = 0x0000_0002;
pub const FMODE_READWRITE: c_int = FMODE_READABLE | FMODE_WRITABLE;
pub const FMODE_BINMODE: c_int = 0x0000_0004;
pub const FMODE_SYNC: c_int = 0x0000_0008;
pub const FMODE_TTY: c_int = 0x0000_0010;
pub const FMODE_DUPLEX: c_int = 0x0000_0020;
pub const FMODE_APPEND: c_int = 0x0000_0040;
pub const FMODE_CREATE: c_int = 0x0000_0080;
pub const FMODE_EXCL: c_int = 0x0000_0400;
pub const FMODE_TRUNC: c_int = 0x0000_0800;
pub const FMODE_TEXTMODE: c_int = 0x0000_1000;
pub const FMODE_SETENC_BY_BOM: c_int = 0x0010_0000;

// `rb_pid_t`: `pid_t`, or `int` on Windows.
#[allow(non_camel_case_types)]
pub type rb_pid_t = c_int;

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    pub static rb_cFile: Value;
    pub static rb_cIO: Value;
    // The values behind `$stdin`, `$stdout` and `$stderr`.
    pub static rb_stderr: Value;
    pub static rb_stdin: Value;
    pub static rb_stdout: Value;
}

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // VALUE
    // rb_dir_getwd(void)
    pub fn rb_dir_getwd() -> Value;
    // VALUE
    // rb_file_absolute_path(VALUE fname, VALUE dname)
    pub fn rb_file_absolute_path(path: Value, directory: Value) -> Value;
    // VALUE
    // rb_file_dirname(VALUE fname)
    pub fn rb_file_dirname(path: Value) -> Value;
    // VALUE
    // rb_file_expand_path(VALUE fname, VALUE dname)
    pub fn rb_file_expand_path(path: Value, directory: Value) -> Value;
    // VALUE
    // rb_file_open(const char *fname, const char *modestr)
    pub fn rb_file_open(path: *const c_char, mode: *const c_char) -> Value;
    // VALUE
    // rb_file_open_str(VALUE fname, const char *modestr)
    pub fn rb_file_open_str(path: Value, mode: *const c_char) -> Value;
    // VALUE
    // rb_find_file(VALUE path)
    //
    // Searches `$LOAD_PATH`; `0` when not found.
    pub fn rb_find_file(path: Value) -> Value;
    // VALUE
    // rb_io_addstr(VALUE io, VALUE str)
    pub fn rb_io_addstr(io: Value, string: Value) -> Value;
    // VALUE
    // rb_io_binmode(VALUE io)
    pub fn rb_io_binmode(io: Value) -> Value;
    // VALUE
    // rb_io_ascii8bit_binmode(VALUE io)
    //
    // What `IO#binmode` calls: binary mode plus ASCII-8BIT external encoding.
    pub fn rb_io_ascii8bit_binmode(io: Value) -> Value;
    // VALUE
    // rb_io_wait(VALUE io, VALUE events, VALUE timeout)
    //
    // Returns the ready events as an Integer, or `Qfalse` on timeout.
    pub fn rb_io_wait(io: Value, events: Value, timeout: Value) -> Value;
    // int
    // rb_io_descriptor(VALUE io)
    //
    // Raises `IOError` for a closed stream.
    #[cfg(ruby_gte_3_1)]
    pub fn rb_io_descriptor(io: Value) -> c_int;
    // VALUE
    // rb_io_maybe_wait(int error, VALUE io, VALUE events, VALUE timeout)
    //
    // Waits like `rb_io_wait` when `error` is `EAGAIN`/`EWOULDBLOCK`, returns
    // `events` for `EINTR` and `Qfalse` for any other error.
    #[cfg(ruby_gte_3_1)]
    pub fn rb_io_maybe_wait(error: c_int, io: Value, events: Value, timeout: Value) -> Value;
    // int
    // rb_io_maybe_wait_readable(int error, VALUE io, VALUE timeout)
    //
    // `RUBY_IO_READABLE`, or `0` (see `rb_io_maybe_wait`).
    #[cfg(ruby_gte_3_1)]
    pub fn rb_io_maybe_wait_readable(error: c_int, io: Value, timeout: Value) -> c_int;
    // int
    // rb_io_maybe_wait_writable(int error, VALUE io, VALUE timeout)
    //
    // `RUBY_IO_WRITABLE`, or `0` (see `rb_io_maybe_wait`).
    #[cfg(ruby_gte_3_1)]
    pub fn rb_io_maybe_wait_writable(error: c_int, io: Value, timeout: Value) -> c_int;
    // VALUE
    // rb_io_timeout(VALUE io)
    //
    // `Qnil`, or the timeout as it was set.
    #[cfg(ruby_gte_3_2)]
    pub fn rb_io_timeout(io: Value) -> Value;
    // VALUE
    // rb_io_set_timeout(VALUE io, VALUE timeout)
    //
    // `timeout` is `Qnil` (no timeout) or responds to `to_f`.
    #[cfg(ruby_gte_3_2)]
    pub fn rb_io_set_timeout(io: Value, timeout: Value) -> Value;
    // off_t                                   (3.1)
    // rb_off_t                                (3.2+)
    // rb_file_size(VALUE file)
    //
    // Flushes buffered output first; raises for a closed file.
    #[cfg(ruby_gte_3_1)]
    pub fn rb_file_size(file: Value) -> rb_off_t;
    // VALUE
    // rb_io_close(VALUE io)
    pub fn rb_io_close(io: Value) -> Value;
    // VALUE
    // rb_io_eof(VALUE io)
    pub fn rb_io_eof(io: Value) -> Value;
    // VALUE
    // rb_io_flush(VALUE io)
    pub fn rb_io_flush(io: Value) -> Value;
    // VALUE
    // rb_io_getbyte(VALUE io)
    pub fn rb_io_getbyte(io: Value) -> Value;
    // VALUE
    // rb_io_gets(VALUE io)
    pub fn rb_io_gets(io: Value) -> Value;
    // VALUE
    // rb_io_print(int argc, const VALUE *argv, VALUE out)
    pub fn rb_io_print(argc: Argc, argv: *const Value, io: Value) -> Value;
    // VALUE
    // rb_io_puts(int argc, const VALUE *argv, VALUE out)
    pub fn rb_io_puts(argc: Argc, argv: *const Value, io: Value) -> Value;
    // VALUE
    // rb_io_ungetc(VALUE io, VALUE c)
    pub fn rb_io_ungetc(io: Value, character: Value) -> Value;
    // VALUE
    // rb_io_write(VALUE io, VALUE str)
    pub fn rb_io_write(io: Value, string: Value) -> Value;
}

// `Marshal`
#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // VALUE
    // rb_marshal_dump(VALUE obj, VALUE port)
    pub fn rb_marshal_dump(object: Value, port: Value) -> Value;
    // VALUE
    // rb_marshal_load(VALUE port)
    pub fn rb_marshal_load(port: Value) -> Value;
}

// Loading code
#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // int
    // rb_feature_provided(const char *feature, const char **loading)
    pub fn rb_feature_provided(feature: *const c_char, loading: *mut *const c_char) -> c_int;
    // VALUE
    // rb_f_require(VALUE obj, VALUE fname)
    pub fn rb_f_require(object: Value, name: Value) -> Value;
    // void
    // rb_load(VALUE fname, int wrap)
    pub fn rb_load(path: Value, wrap: c_int);
    // void
    // rb_load_protect(VALUE fname, int wrap, int *pstate)
    pub fn rb_load_protect(path: Value, wrap: c_int, state: *mut c_int);
    // void
    // rb_provide(const char *feature)
    pub fn rb_provide(feature: *const c_char);
    // int
    // rb_provided(const char *feature)
    pub fn rb_provided(feature: *const c_char) -> c_int;
    // VALUE
    // rb_require_string(VALUE fname)
    pub fn rb_require_string(name: Value) -> Value;
    // void
    // ruby_incpush(const char *path)
    //
    // Adds each `PATH_SEP`-separated directory to `$LOAD_PATH`.
    pub fn ruby_incpush(path: *const c_char);
}
