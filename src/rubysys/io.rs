use crate::rubysys::{
    io_buffer::rb_off_t,
    types::{c_char, c_int, Argc, EncodingType, Value},
};

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
// The descriptor is owned outside Ruby, which does not close it
// (`IO#autoclose?` is false). Named in the headers from 3.3 (`FMODE_PREP`
// inside Ruby before that).
#[cfg(ruby_gte_3_3)]
pub const FMODE_EXTERNAL: c_int = 0x0001_0000;
pub const FMODE_SETENC_BY_BOM: c_int = 0x0010_0000;

// `struct rb_io_encoding` (3.3; `struct rb_io_enc_t` before), the decomposed
// encoding settings of an IO.
#[cfg(ruby_gte_3_3)]
#[repr(C)]
pub struct rb_io_encoding {
    // Internal encoding.
    pub enc: EncodingType,
    // External encoding.
    pub enc2: EncodingType,
    // `enum ruby_econv_flag_type` flags.
    pub ecflags: c_int,
    // The flags as a Ruby Hash.
    pub ecopts: Value,
}

// `rb_pid_t`: `pid_t` (see `scheduler::RbPid`).
#[allow(non_camel_case_types)]
pub type rb_pid_t = crate::rubysys::scheduler::RbPid;

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
    // rb_io_closed_p(VALUE io)
    pub fn rb_io_closed_p(io: Value) -> Value;
    // VALUE
    // rb_io_wait(VALUE io, VALUE events, VALUE timeout)
    //
    // Returns the ready events as an Integer, or `Qfalse` on timeout.
    pub fn rb_io_wait(io: Value, events: Value, timeout: Value) -> Value;
    // int
    // rb_io_descriptor(VALUE io)
    //
    // Raises `IOError` for a closed stream.
    pub fn rb_io_descriptor(io: Value) -> c_int;
    // int
    // rb_io_mode(VALUE io)
    //
    // Ruby 3.3+: the `FMODE_*` flags of `io`.
    #[cfg(ruby_gte_3_3)]
    pub fn rb_io_mode(io: Value) -> c_int;
    // VALUE
    // rb_io_maybe_wait(int error, VALUE io, VALUE events, VALUE timeout)
    //
    // Waits like `rb_io_wait` when `error` is `EAGAIN`/`EWOULDBLOCK`, returns
    // `events` for `EINTR` and `Qfalse` for any other error.
    pub fn rb_io_maybe_wait(error: c_int, io: Value, events: Value, timeout: Value) -> Value;
    // int
    // rb_io_maybe_wait_readable(int error, VALUE io, VALUE timeout)
    //
    // `RUBY_IO_READABLE`, or `0` (see `rb_io_maybe_wait`).
    pub fn rb_io_maybe_wait_readable(error: c_int, io: Value, timeout: Value) -> c_int;
    // int
    // rb_io_maybe_wait_writable(int error, VALUE io, VALUE timeout)
    //
    // `RUBY_IO_WRITABLE`, or `0` (see `rb_io_maybe_wait`).
    pub fn rb_io_maybe_wait_writable(error: c_int, io: Value, timeout: Value) -> c_int;
    // VALUE
    // rb_io_timeout(VALUE io)
    //
    // `Qnil`, or the timeout as it was set.
    pub fn rb_io_timeout(io: Value) -> Value;
    // VALUE
    // rb_io_set_timeout(VALUE io, VALUE timeout)
    //
    // `timeout` is `Qnil` (no timeout) or responds to `to_f`.
    pub fn rb_io_set_timeout(io: Value, timeout: Value) -> Value;
    // VALUE
    // rb_io_open_descriptor(VALUE klass, int descriptor, int mode, VALUE path, VALUE timeout, struct rb_io_encoding *encoding)
    //
    // An IO of class `klass` for the open `descriptor`, which Ruby closes
    // unless `mode` has `FMODE_EXTERNAL`; `encoding` may be null.
    #[cfg(ruby_gte_3_3)]
    pub fn rb_io_open_descriptor(
        klass: Value,
        descriptor: c_int,
        mode: c_int,
        path: Value,
        timeout: Value,
        encoding: *mut rb_io_encoding,
    ) -> Value;
    // off_t                                   (3.1)
    // rb_off_t                                (3.2+)
    // rb_file_size(VALUE file)
    //
    // Flushes buffered output first; raises for a closed file.
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

// Processes
#[cfg(ruby_gte_3_3)]
#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // VALUE
    // rb_process_status_wait(rb_pid_t pid, int flags)
    //
    // Waits for the process like `waitpid(2)` with `flags` (through the
    // Fiber scheduler, if any) and returns a `Process::Status`, or `Qnil`
    // when nothing was reaped (`WNOHANG`).
    pub fn rb_process_status_wait(pid: rb_pid_t, flags: c_int) -> Value;
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
