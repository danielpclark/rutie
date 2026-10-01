use crate::rubysys::{
    io_buffer::rb_off_t,
    libc::{timeval, FILE},
    types::{c_char, c_int, c_long, c_void, size_t, ssize_t, Argc, EncodingType, Value},
};

#[cfg(unix)]
use crate::rubysys::libc::fd_set;

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
pub const FMODE_EXTERNAL: c_int = 0x0001_0000;
pub const FMODE_SETENC_BY_BOM: c_int = 0x0010_0000;

// `struct rb_io_encoding` (3.3; `struct rb_io_enc_t` before), the decomposed
// encoding settings of an IO.
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

pub use crate::rubysys::process::rb_pid_t;

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
    // The `FMODE_*` flags of `io`.
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
// VALUE (*dumper)(VALUE) and VALUE (*loader)(VALUE, VALUE) of
// `rb_marshal_define_compat`.
pub type MarshalDumper = rutie_callback!(type fn(object: Value) -> Value);
pub type MarshalLoader = rutie_callback!(type fn(object: Value, old: Value) -> Value);

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // VALUE
    // rb_marshal_dump(VALUE obj, VALUE port)
    pub fn rb_marshal_dump(object: Value, port: Value) -> Value;
    // VALUE
    // rb_marshal_load(VALUE port)
    pub fn rb_marshal_load(port: Value) -> Value;
    // void
    // rb_marshal_define_compat(VALUE newclass, VALUE oldclass,
    //                          VALUE (*dumper)(VALUE), VALUE (*loader)(VALUE, VALUE))
    //
    // Instances of `newclass` are dumped as `oldclass`: `dumper` turns one
    // into an `oldclass` object, `loader(new_object, old_object)` fills a
    // newly allocated `newclass` object back from it.
    pub fn rb_marshal_define_compat(
        new_class: Value,
        old_class: Value,
        dumper: MarshalDumper,
        loader: MarshalLoader,
    );
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

// `rb_io_t` (`struct rb_io`): the state behind an `IO` (`RFILE(io)->fptr`).
// Its fields are not part of Rutie's API, so it is only used through
// pointers, as `rb_io_make_open_file` returns them.
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct rb_io_t {
    _private: [u8; 0],
}

// `enum rb_io_mode`: a set of `FMODE_*` flags.
#[allow(non_camel_case_types)]
pub type rb_io_mode = c_int;

// `mode_t`, the permission argument of `open(2)`.
#[cfg(unix)]
#[allow(non_camel_case_types)]
pub type mode_t = crate::rubysys::libc::mode_t;
#[cfg(windows)]
#[allow(non_camel_case_types)]
pub type mode_t = c_int;

// `rb_fdset_t`: a growable `fd_set` (`maxfd` and the set, or on Windows its
// capacity and the set), filled with the `rb_fd_*` functions.
#[repr(C)]
pub struct RbFdset {
    pub maxfd: c_int,
    pub fdset: *mut c_void,
}

// `ruby/io.h`: the `rb_io_t` (`fptr`) API, for C-level IO code. The
// `rb_io_check_*` functions raise an `IOError` when the IO is not in the
// expected state.
#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // void
    // rb_eof_error(void)
    //
    // Raises an `EOFError`.
    pub fn rb_eof_error() -> !;
    // FILE *
    // rb_fdopen(int fd, const char *modestr)
    //
    // `fdopen(3)`, raising a `SystemCallError` on failure.
    pub fn rb_fdopen(fd: c_int, modestr: *const c_char) -> *mut FILE;
    // ssize_t
    // rb_io_bufwrite(VALUE io, const void *buf, size_t size)
    //
    // Writes `size` bytes through the IO's write buffer. Returns the bytes
    // written, or -1 with `errno` set; raises when `io` is closed or not open
    // for writing.
    pub fn rb_io_bufwrite(io: Value, buf: *const c_void, size: size_t) -> ssize_t;
    // void
    // rb_io_check_byte_readable(rb_io_t *fptr)
    pub fn rb_io_check_byte_readable(fptr: *mut rb_io_t);
    // void
    // rb_io_check_char_readable(rb_io_t *fptr)
    pub fn rb_io_check_char_readable(fptr: *mut rb_io_t);
    // void
    // rb_io_check_closed(rb_io_t *fptr)
    pub fn rb_io_check_closed(fptr: *mut rb_io_t);
    // void
    // rb_io_check_initialized(rb_io_t *fptr)
    pub fn rb_io_check_initialized(fptr: *mut rb_io_t);
    // VALUE
    // rb_io_check_io(VALUE io)
    //
    // `io.to_io`, or `Qnil` when `io` has no `to_io`; raises a `TypeError`
    // when `to_io` returns something that is not an `IO`.
    pub fn rb_io_check_io(io: Value) -> Value;
    // void
    // rb_io_check_readable(rb_io_t *fptr)
    pub fn rb_io_check_readable(fptr: *mut rb_io_t);
    // void
    // rb_io_check_writable(rb_io_t *fptr)
    pub fn rb_io_check_writable(fptr: *mut rb_io_t);
    // int
    // rb_io_extract_encoding_option(VALUE opt, rb_encoding **enc_p, rb_encoding **enc2_p,
    //                               enum rb_io_mode *fmode_p)
    //
    // Reads the `encoding:`, `external_encoding:` and `internal_encoding:`
    // options of `opt` (a Hash or `Qnil`). Returns 1 when any was given.
    pub fn rb_io_extract_encoding_option(
        opt: Value,
        enc_p: *mut EncodingType,
        enc2_p: *mut EncodingType,
        fmode_p: *mut rb_io_mode,
    ) -> c_int;
    // void
    // rb_io_extract_modeenc(VALUE *vmode_p, VALUE *vperm_p, VALUE opthash, int *oflags_p,
    //                       enum rb_io_mode *fmode_p, rb_io_enc_t *convconfig_p)
    //
    // What `IO.new` and `File.open` do with their mode, permission and
    // option arguments.
    pub fn rb_io_extract_modeenc(
        vmode_p: *mut Value,
        vperm_p: *mut Value,
        opthash: Value,
        oflags_p: *mut c_int,
        fmode_p: *mut rb_io_mode,
        convconfig_p: *mut rb_io_encoding,
    );
    // int
    // rb_io_fptr_finalize(rb_io_t *fptr)
    //
    // Closes and frees `fptr`; for an IO's own free function only.
    pub fn rb_io_fptr_finalize(fptr: *mut rb_io_t) -> c_int;
    // VALUE
    // rb_io_get_io(VALUE io)
    //
    // Like `rb_io_check_io`, but raises a `TypeError` when there is no
    // conversion.
    pub fn rb_io_get_io(io: Value) -> Value;
    // rb_io_t *
    // rb_io_make_open_file(VALUE obj)
    //
    // Allocates the `rb_io_t` of a new IO object (`MakeOpenFile`), freeing
    // any previous one.
    pub fn rb_io_make_open_file(obj: Value) -> *mut rb_io_t;
    // enum rb_io_mode
    // rb_io_modestr_fmode(const char *modestr)
    //
    // The `FMODE_*` flags of a mode string such as `"r+b"`; raises an
    // `ArgumentError` for an invalid one.
    pub fn rb_io_modestr_fmode(modestr: *const c_char) -> rb_io_mode;
    // int
    // rb_io_modestr_oflags(const char *modestr)
    //
    // The `O_*` flags of a mode string; raises an `ArgumentError` for an
    // invalid one.
    pub fn rb_io_modestr_oflags(modestr: *const c_char) -> c_int;
    // int
    // rb_io_oflags_fmode(int oflags)
    //
    // The `FMODE_*` flags of a set of `O_*` flags.
    pub fn rb_io_oflags_fmode(oflags: c_int) -> rb_io_mode;
    // void
    // rb_io_read_check(rb_io_t *fptr)
    //
    // Waits until the IO is readable, unless it has buffered input.
    pub fn rb_io_read_check(fptr: *mut rb_io_t);
    // int
    // rb_io_read_pending(rb_io_t *fptr)
    //
    // Non-zero when there is buffered input (the number of bytes, or 1 for
    // buffered characters).
    pub fn rb_io_read_pending(fptr: *mut rb_io_t) -> c_int;
    // void
    // rb_io_set_nonblock(rb_io_t *fptr)
    pub fn rb_io_set_nonblock(fptr: *mut rb_io_t);
    // VALUE
    // rb_io_set_write_io(VALUE io, VALUE w)
    //
    // Makes `w` (or `Qnil`) the IO that `io` writes to; returns the previous
    // one, or `Qnil`.
    pub fn rb_io_set_write_io(io: Value, w: Value) -> Value;
    // FILE *
    // rb_io_stdio_file(rb_io_t *fptr)
    pub fn rb_io_stdio_file(fptr: *mut rb_io_t) -> *mut FILE;
    // void
    // rb_io_synchronized(rb_io_t *fptr)
    pub fn rb_io_synchronized(fptr: *mut rb_io_t);
    // int
    // rb_io_wait_readable(int fd)
    //
    // After a failed read: waits for `fd` and returns 1 when `errno` says it
    // would block (or was interrupted), else returns 0.
    pub fn rb_io_wait_readable(fd: c_int) -> c_int;
    // int
    // rb_io_wait_writable(int fd)
    pub fn rb_io_wait_writable(fd: c_int) -> c_int;
    // VALUE
    // rb_stat_new(const struct stat *st)
    //
    // A `File::Stat` made from a `struct stat`.
    pub fn rb_stat_new(st: *const c_void) -> Value;
    // VALUE
    // rb_statx_new(const rb_io_stat_data *st)
    //
    // A `File::Stat` made from a `struct statx`; only where Ruby was built
    // with `statx(2)` (otherwise it is a macro for `rb_stat_new`).
    #[cfg(target_os = "linux")]
    pub fn rb_statx_new(st: *const c_void) -> Value;
    // int
    // rb_wait_for_single_fd(int fd, int events, struct timeval *tv)
    //
    // Waits for `events` (`RUBY_IO_READABLE`, ...) on `fd`; `tv` NULL waits
    // forever. Returns the ready events, 0 on timeout or -1 with `errno` set.
    pub fn rb_wait_for_single_fd(fd: c_int, events: c_int, tv: *mut timeval) -> c_int;
}

// `ruby/internal/intern/io.h`: file descriptors. Unless noted these do not
// raise: they return -1 with `errno` set.
#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // int
    // rb_cloexec_dup(int oldfd)
    pub fn rb_cloexec_dup(oldfd: c_int) -> c_int;
    // int
    // rb_cloexec_dup2(int oldfd, int newfd)
    pub fn rb_cloexec_dup2(oldfd: c_int, newfd: c_int) -> c_int;
    // int
    // rb_cloexec_fcntl_dupfd(int fd, int minfd)
    pub fn rb_cloexec_fcntl_dupfd(fd: c_int, minfd: c_int) -> c_int;
    // int
    // rb_cloexec_open(const char *pathname, int flags, mode_t mode)
    pub fn rb_cloexec_open(pathname: *const c_char, flags: c_int, mode: mode_t) -> c_int;
    // int
    // rb_cloexec_pipe(int fildes[2])
    pub fn rb_cloexec_pipe(fildes: *mut c_int) -> c_int;
    // void
    // rb_close_before_exec(int lowfd, int maxhint, VALUE noclose_fds)
    //
    // Makes every descriptor from `lowfd` up close-on-exec, except the keys
    // of the Hash `noclose_fds` (or `Qnil`); meant for a forked child about
    // to exec.
    pub fn rb_close_before_exec(lowfd: c_int, maxhint: c_int, noclose_fds: Value);
    // void
    // rb_fd_fix_cloexec(int fd)
    //
    // Sets close-on-exec on `fd` unless it is 0, 1 or 2.
    pub fn rb_fd_fix_cloexec(fd: c_int);
    // VALUE
    // rb_gets(void)
    //
    // `Kernel#gets`: the next line of `ARGF`, or `Qnil` at its end; sets `$_`.
    pub fn rb_gets() -> Value;
    // VALUE
    // rb_io_fdopen(int fd, int flags, const char *path)
    //
    // A new `IO` that owns `fd` (it closes `fd` when closed or collected).
    // `flags` are `O_*` flags; `path` may be NULL.
    pub fn rb_io_fdopen(fd: c_int, flags: c_int, path: *const c_char) -> Value;
    // VALUE
    // rb_io_printf(int argc, const VALUE *argv, VALUE io)
    //
    // `io.printf(*argv)`.
    pub fn rb_io_printf(argc: Argc, argv: *const Value, io: Value) -> Value;
    // VALUE
    // rb_io_ungetbyte(VALUE io, VALUE b)
    //
    // `io.ungetbyte(b)`: `b` is an Integer, a String or `Qnil`.
    pub fn rb_io_ungetbyte(io: Value, byte: Value) -> Value;
    // int
    // rb_pipe(int *pipes)
    //
    // `rb_cloexec_pipe` plus `rb_update_max_fd`; `pipes` holds 2 elements.
    pub fn rb_pipe(pipes: *mut c_int) -> c_int;
    // int
    // rb_reserved_fd_p(int fd)
    //
    // 1 when the VM uses `fd` itself (such as for its timer thread).
    pub fn rb_reserved_fd_p(fd: c_int) -> c_int;
    // void
    // rb_update_max_fd(int fd)
    pub fn rb_update_max_fd(fd: c_int);
    // void
    // rb_write_error(const char *str)
    //
    // Writes `str` to `$stderr`.
    pub fn rb_write_error(str: *const c_char);
    // void
    // rb_write_error2(const char *str, long len)
    pub fn rb_write_error2(str: *const c_char, len: c_long);
}

// `ruby/internal/intern/select.h`: `select(2)` over `rb_fdset_t`. The
// `rb_fd_*` functions are for the `rb_fdset_t` above; on Windows they are
// macros for `rb_w32_fd_*` functions instead.
#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // int
    // rb_thread_fd_select(int nfds, rb_fdset_t *rfds, rb_fdset_t *wfds, rb_fdset_t *efds,
    //                     struct timeval *timeout)
    //
    // `select(2)` that releases the GVL and lets other threads run. Each set
    // may be NULL; `timeout` NULL waits forever.
    pub fn rb_thread_fd_select(
        nfds: c_int,
        rfds: *mut RbFdset,
        wfds: *mut RbFdset,
        efds: *mut RbFdset,
        timeout: *mut timeval,
    ) -> c_int;
    // void
    // rb_fd_clr(int fd, rb_fdset_t *f)
    #[cfg(unix)]
    pub fn rb_fd_clr(fd: c_int, set: *mut RbFdset);
    // void
    // rb_fd_copy(rb_fdset_t *dst, const fd_set *src, int max)
    #[cfg(unix)]
    pub fn rb_fd_copy(dst: *mut RbFdset, src: *const fd_set, max: c_int);
    // void
    // rb_fd_dup(rb_fdset_t *dst, const rb_fdset_t *src)
    #[cfg(unix)]
    pub fn rb_fd_dup(dst: *mut RbFdset, src: *const RbFdset);
    // void
    // rb_fd_init(rb_fdset_t *f)
    //
    // Must be called before the other `rb_fd_*` functions.
    #[cfg(unix)]
    pub fn rb_fd_init(set: *mut RbFdset);
    // int
    // rb_fd_isset(int fd, const rb_fdset_t *f)
    #[cfg(unix)]
    pub fn rb_fd_isset(fd: c_int, set: *const RbFdset) -> c_int;
    // int
    // rb_fd_select(int nfds, rb_fdset_t *rfds, rb_fdset_t *wfds, rb_fdset_t *efds,
    //              struct timeval *timeout)
    //
    // `select(2)`, holding the GVL.
    #[cfg(unix)]
    pub fn rb_fd_select(
        nfds: c_int,
        rfds: *mut RbFdset,
        wfds: *mut RbFdset,
        efds: *mut RbFdset,
        timeout: *mut timeval,
    ) -> c_int;
    // void
    // rb_fd_set(int fd, rb_fdset_t *f)
    #[cfg(unix)]
    pub fn rb_fd_set(fd: c_int, set: *mut RbFdset);
    // void
    // rb_fd_term(rb_fdset_t *f)
    //
    // Frees the set; `rb_fd_init` makes it usable again.
    #[cfg(unix)]
    pub fn rb_fd_term(set: *mut RbFdset);
    // void
    // rb_fd_zero(rb_fdset_t *f)
    #[cfg(unix)]
    pub fn rb_fd_zero(set: *mut RbFdset);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding::{fixnum, global::RubySpecialConsts, vm};
    use std::ptr;

    fn nil() -> Value {
        Value::from(RubySpecialConsts::Nil as crate::types::InternalValue)
    }

    // `RFILE(io)->fptr`: `struct RFile` is an `RBasic` and the pointer.
    unsafe fn fptr(io: Value) -> *mut rb_io_t {
        *(io.value as *const *mut rb_io_t).add(2)
    }

    fn fileno(io: Value) -> c_int {
        fixnum::num_to_i32(vm::call_method(io, "fileno", &[]))
    }

    #[cfg(unix)]
    #[test]
    fn test_descriptors_and_fptr() {
        crate::on_ruby_thread(|| unsafe {
            let mut fds = [-1; 2];
            assert_eq!(rb_pipe(fds.as_mut_ptr()), 0);
            assert_eq!(rb_reserved_fd_p(fds[0]), 0);
            rb_update_max_fd(fds[1]);
            rb_fd_fix_cloexec(fds[1]);

            let reader = rb_io_fdopen(fds[0], 0, ptr::null());
            let writer = rb_io_fdopen(fds[1], 1, b"pipe\0".as_ptr() as *const c_char);
            assert_eq!(rb_io_check_io(reader).value, reader.value);
            assert!(rb_io_check_io(fixnum::i32_to_num(1)).is_nil());
            assert_eq!(rb_io_get_io(writer).value, writer.value);
            assert!(vm::protect_value(|| rb_io_get_io(nil())).is_err());

            let (read_fptr, write_fptr) = (fptr(reader), fptr(writer));
            rb_io_check_initialized(read_fptr);
            rb_io_check_closed(read_fptr);
            rb_io_check_readable(read_fptr);
            rb_io_check_char_readable(read_fptr);
            rb_io_check_byte_readable(read_fptr);
            rb_io_check_writable(write_fptr);
            assert!(vm::protect_value(|| {
                rb_io_check_writable(read_fptr);
                nil()
            })
            .is_err());
            rb_io_synchronized(write_fptr);
            rb_io_set_nonblock(read_fptr);

            let bytes = b"abc\n";
            assert_eq!(
                rb_io_bufwrite(writer, bytes.as_ptr() as *const c_void, 4),
                4
            );
            assert_eq!(rb_io_read_pending(read_fptr), 0);
            rb_io_read_check(read_fptr);
            assert_eq!(
                rb_wait_for_single_fd(fds[0], RUBY_IO_READABLE, ptr::null_mut()),
                RUBY_IO_READABLE
            );
            assert_eq!(
                fixnum::num_to_i32(vm::call_method(reader, "getbyte", &[])),
                b'a' as i32
            );
            // The rest of the line is now in the read buffer.
            assert_eq!(rb_io_read_pending(read_fptr), 3);

            rb_io_ungetbyte(reader, fixnum::i32_to_num(b'y' as i32));
            assert_eq!(
                fixnum::num_to_i32(vm::call_method(reader, "getbyte", &[])),
                b'y' as i32
            );

            let format = vm::eval_string("'%s!'");
            let arguments = [format, vm::eval_string("'ok'")];
            rb_io_printf(2, arguments.as_ptr(), writer);

            let mut timeout = timeval {
                tv_sec: 0,
                tv_usec: 0,
            };
            assert_eq!(
                rb_wait_for_single_fd(fds[1], RUBY_IO_WRITABLE, &mut timeout),
                RUBY_IO_WRITABLE
            );

            assert!(rb_io_set_write_io(reader, writer).is_nil());
            assert_eq!(rb_io_set_write_io(reader, nil()).value, writer.value);

            let stdio = rb_io_stdio_file(write_fptr);
            assert!(!stdio.is_null());

            let copy = rb_cloexec_dup(fileno(reader));
            assert!(copy > 2);
            let higher = rb_cloexec_fcntl_dupfd(copy, 100);
            assert!(higher >= 100);
            assert_eq!(rb_cloexec_dup2(copy, higher), higher);
            let file = rb_fdopen(higher, b"r\0".as_ptr() as *const c_char);
            assert!(!file.is_null());
            crate::rubysys::libc::fclose(file);
            crate::rubysys::libc::close(copy);

            let mut more = [-1; 2];
            assert_eq!(rb_cloexec_pipe(more.as_mut_ptr()), 0);
            crate::rubysys::libc::close(more[0]);
            crate::rubysys::libc::close(more[1]);

            let path =
                std::env::temp_dir().join(format!("rutie_cloexec_open_{}", std::process::id()));
            let cpath = std::ffi::CString::new(path.to_str().unwrap()).unwrap();
            let fd = rb_cloexec_open(
                cpath.as_ptr(),
                crate::rubysys::libc::O_WRONLY | crate::rubysys::libc::O_CREAT,
                0o600,
            );
            assert!(fd > 2);
            crate::rubysys::libc::close(fd);
            assert_eq!(
                rb_cloexec_open(b"/no/such/rutie\0".as_ptr() as *const c_char, 0, 0),
                -1
            );
            std::fs::remove_file(path).unwrap();

            vm::call_method(reader, "close", &[]);
            vm::call_method(writer, "close", &[]);

            assert_eq!(
                rb_io_modestr_fmode(b"w\0".as_ptr() as *const c_char),
                0x2 | 0x80 | 0x800
            );
            let oflags = rb_io_modestr_oflags(b"r+\0".as_ptr() as *const c_char);
            assert_eq!(rb_io_oflags_fmode(oflags), 0x3);

            // `encoding: "UTF-8:EUC-JP"` sets both encodings.
            let options = vm::eval_string("{ encoding: 'UTF-8:EUC-JP' }");
            let (mut enc, mut enc2, mut fmode) = (ptr::null(), ptr::null(), 0);
            assert_eq!(
                rb_io_extract_encoding_option(options, &mut enc, &mut enc2, &mut fmode),
                1
            );
            assert!(!enc.is_null() && !enc2.is_null());
            assert_eq!(
                rb_io_extract_encoding_option(nil(), &mut enc, &mut enc2, &mut fmode),
                0
            );

            let mut vmode = vm::eval_string("'rb'");
            let mut vperm = nil();
            let (mut oflags, mut fmode) = (0, 0);
            let mut convconfig = rb_io_encoding {
                enc: ptr::null(),
                enc2: ptr::null(),
                ecflags: 0,
                ecopts: nil(),
            };
            rb_io_extract_modeenc(
                &mut vmode,
                &mut vperm,
                nil(),
                &mut oflags,
                &mut fmode,
                &mut convconfig,
            );
            assert_eq!(fmode & 0x5, 0x5);

            // A new, unopened IO.
            let blank = vm::call_method(rb_cIO, "allocate", &[]);
            let blank_fptr = rb_io_make_open_file(blank);
            assert!(!blank_fptr.is_null());
            assert!(vm::protect_value(|| {
                rb_io_check_closed(blank_fptr);
                nil()
            })
            .is_err());

            assert!(vm::protect_value(|| rb_eof_error()).is_err());

            // Into `$stderr`.
            vm::eval_string("require 'stringio'; $rutie_err = $stderr; $stderr = StringIO.new");
            rb_write_error(b"one \0".as_ptr() as *const c_char);
            rb_write_error2(b"two".as_ptr() as *const c_char, 3);
            let written = vm::eval_string("s = $stderr.string; $stderr = $rutie_err; s");
            assert_eq!(crate::binding::string::value_to_string(written), "one two");
        });
    }

    #[cfg(unix)]
    #[test]
    fn test_fdsets_select_and_stat() {
        use crate::rubysys::libc;

        crate::on_ruby_thread(|| unsafe {
            let mut fds = [-1; 2];
            assert_eq!(rb_pipe(fds.as_mut_ptr()), 0);

            let mut set = RbFdset {
                maxfd: 0,
                fdset: ptr::null_mut(),
            };
            rb_fd_init(&mut set);
            rb_fd_set(fds[0], &mut set);
            assert_eq!(rb_fd_isset(fds[0], &set), 1);
            rb_fd_clr(fds[0], &mut set);
            assert_eq!(rb_fd_isset(fds[0], &set), 0);
            rb_fd_set(fds[0], &mut set);

            let mut copy = RbFdset {
                maxfd: 0,
                fdset: ptr::null_mut(),
            };
            rb_fd_init(&mut copy);
            rb_fd_dup(&mut copy, &set);
            assert_eq!(rb_fd_isset(fds[0], &copy), 1);

            // Nothing to read yet.
            let mut zero = timeval {
                tv_sec: 0,
                tv_usec: 0,
            };
            assert_eq!(
                rb_thread_fd_select(
                    fds[0] + 1,
                    &mut copy,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    &mut zero
                ),
                0
            );

            assert_eq!(libc::write(fds[1], b"x".as_ptr() as *const c_void, 1), 1);
            rb_fd_dup(&mut copy, &set);
            let mut zero = timeval {
                tv_sec: 0,
                tv_usec: 0,
            };
            assert_eq!(
                rb_thread_fd_select(
                    fds[0] + 1,
                    &mut copy,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    &mut zero
                ),
                1
            );
            assert_eq!(rb_fd_isset(fds[0], &copy), 1);
            rb_fd_dup(&mut copy, &set);
            let mut zero = timeval {
                tv_sec: 0,
                tv_usec: 0,
            };
            assert_eq!(
                rb_fd_select(
                    fds[0] + 1,
                    &mut copy,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    &mut zero
                ),
                1
            );

            let mut raw: libc::fd_set = std::mem::zeroed();
            libc::FD_SET(fds[1], &mut raw);
            rb_fd_copy(&mut copy, &raw, fds[1] + 1);
            assert_eq!(rb_fd_isset(fds[1], &copy), 1);
            assert_eq!(rb_fd_isset(fds[0], &copy), 0);
            rb_fd_zero(&mut copy);
            assert_eq!(rb_fd_isset(fds[1], &copy), 0);

            rb_fd_term(&mut copy);
            rb_fd_term(&mut set);

            // Close-on-exec for a descriptor that did not have it.
            let plain = libc::dup(fds[0]);
            assert_eq!(libc::fcntl(plain, libc::F_GETFD) & libc::FD_CLOEXEC, 0);
            rb_close_before_exec(
                plain,
                plain + 1,
                Value::from(RubySpecialConsts::Nil as crate::types::InternalValue),
            );
            assert_ne!(libc::fcntl(plain, libc::F_GETFD) & libc::FD_CLOEXEC, 0);

            let mut stat: libc::stat = std::mem::zeroed();
            assert_eq!(libc::fstat(plain, &mut stat), 0);
            let stat = rb_stat_new(&stat as *const libc::stat as *const c_void);
            assert!(vm::call_method(stat, "pipe?", &[]).is_true());

            for fd in [plain, fds[0], fds[1]] {
                libc::close(fd);
            }
        });
    }
}
