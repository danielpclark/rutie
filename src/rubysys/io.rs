use crate::rubysys::types::{c_char, c_int, Argc, Value};

extern "C" {
    pub static rb_cFile: Value;
    pub static rb_cIO: Value;
    // The values behind `$stdin`, `$stdout` and `$stderr`.
    pub static rb_stderr: Value;
    pub static rb_stdin: Value;
    pub static rb_stdout: Value;
}

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
extern "C" {
    // VALUE
    // rb_marshal_dump(VALUE obj, VALUE port)
    pub fn rb_marshal_dump(object: Value, port: Value) -> Value;
    // VALUE
    // rb_marshal_load(VALUE port)
    pub fn rb_marshal_load(port: Value) -> Value;
}

// Loading code
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
