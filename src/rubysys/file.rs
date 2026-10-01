// Paths and the file system: `ruby/internal/intern/file.h`,
// `ruby/internal/glob.h` and the path helpers of `ruby/ruby.h` and
// `ruby/util.h` (the rest of `file.h` is in `io.rs`).

use crate::rubysys::types::{c_char, c_int, c_void, Argc, Value};

// typedef int ruby_glob_func(const char *path, VALUE arg, void *enc)
//
// Called for each path `ruby_glob` finds; `enc` is the path's
// `rb_encoding *`. Returning non-zero stops the glob, which returns that
// value.
pub type RubyGlobFunction =
    rutie_callback!(type fn(path: *const c_char, arg: Value, enc: *mut c_void) -> c_int);

// The callback of `rb_glob`, which may raise.
pub type GlobFunction = rutie_callback!(type fn(path: *const c_char, arg: Value, enc: *mut c_void));

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // VALUE
    // rb_file_directory_p(VALUE _, VALUE path)
    //
    // `File.directory?(path)`; `path` is a String or an IO.
    pub fn rb_file_directory_p(_unused: Value, path: Value) -> Value;
    // VALUE
    // rb_file_s_absolute_path(int argc, const VALUE *argv)
    //
    // `File.absolute_path(*argv)`.
    pub fn rb_file_s_absolute_path(argc: Argc, argv: *const Value) -> Value;
    // VALUE
    // rb_file_s_expand_path(int argc, const VALUE *argv)
    //
    // `File.expand_path(*argv)`.
    pub fn rb_file_s_expand_path(argc: Argc, argv: *const Value) -> Value;
    // int
    // rb_find_file_ext(VALUE *feature, const char *const *exts)
    //
    // Looks for `feature` with each of the NULL-terminated `exts` in
    // `$LOAD_PATH`. Returns 0 when nothing is found, else the index of the
    // extension found plus one, with `*feature` replaced by the full path.
    pub fn rb_find_file_ext(feature: *mut Value, exts: *const *const c_char) -> c_int;
    // int
    // rb_is_absolute_path(const char *path)
    pub fn rb_is_absolute_path(path: *const c_char) -> c_int;
    // VALUE
    // rb_str_encode_ospath(VALUE path)
    //
    // `path` in the encoding of the OS's file names where there is one
    // (UTF-8 on Windows and macOS); elsewhere `path` itself.
    pub fn rb_str_encode_ospath(path: Value) -> Value;

    // VALUE
    // rb_get_path(VALUE obj)
    //
    // `obj` as a path String: `obj.to_path` when it has one, then
    // `String(obj)`, checked for NUL bytes and a path-compatible encoding.
    pub fn rb_get_path(obj: Value) -> Value;
    // VALUE
    // rb_get_path_no_checksafe(VALUE)
    //
    // The same as `rb_get_path` (it is from when Ruby had `$SAFE`).
    pub fn rb_get_path_no_checksafe(obj: Value) -> Value;

    // void
    // rb_glob(const char *pattern, void (*func)(const char *path, VALUE arg, void *enc),
    //         VALUE arg)
    //
    // Calls `func` for each path matching `pattern` (like `Dir.glob`).
    // Raises what `func` raises.
    pub fn rb_glob(pattern: *const c_char, func: GlobFunction, arg: Value);
    // int
    // ruby_glob(const char *pattern, int flags, ruby_glob_func *func, VALUE arg)
    //
    // Like `rb_glob`, without raising; `flags` are `File::FNM_*` flags.
    // Returns 0, or what `func` returned to stop.
    pub fn ruby_glob(
        pattern: *const c_char,
        flags: c_int,
        func: RubyGlobFunction,
        arg: Value,
    ) -> c_int;
    // int
    // ruby_brace_glob(const char *pattern, int flags, ruby_glob_func *func, VALUE arg)
    //
    // Like `ruby_glob`, also expanding `{a,b}`.
    pub fn ruby_brace_glob(
        pattern: *const c_char,
        flags: c_int,
        func: RubyGlobFunction,
        arg: Value,
    ) -> c_int;

    // char *
    // ruby_getcwd(void)
    //
    // The working directory, allocated with `ruby_xmalloc` (free it with
    // `ruby_xfree`). Raises a `SystemCallError` on failure.
    pub fn ruby_getcwd() -> *mut c_char;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding::{array, string, vm};
    use std::ffi::CStr;

    rutie_callback! {
        fn count_path(_path: *const c_char, arg: Value, _enc: *mut c_void) -> c_int {
            unsafe { *(arg.value as *mut usize) += 1 };
            0
        }
    }

    rutie_callback! {
        fn stop_at_first(_path: *const c_char, _arg: Value, _enc: *mut c_void) -> c_int {
            7
        }
    }

    rutie_callback! {
        fn push_path(path: *const c_char, arg: Value, _enc: *mut c_void) {
            let path = unsafe { CStr::from_ptr(path) }.to_str().unwrap();
            array::push(arg, string::new_utf8(path));
        }
    }

    #[test]
    fn test_paths_and_globs() {
        crate::on_ruby_thread(|| unsafe {
            let dir = std::env::temp_dir().join(format!("rutie_sys_glob_{}", std::process::id()));
            std::fs::create_dir_all(&dir).unwrap();
            for name in ["a.c", "b.h"] {
                std::fs::write(dir.join(name), "").unwrap();
            }
            let base = dir.to_str().unwrap().replace('\\', "/");

            let pattern = std::ffi::CString::new(format!("{}/*.{{c,h}}", base)).unwrap();
            let mut count = 0usize;
            let arg = Value::from(&mut count as *mut usize as crate::types::InternalValue);
            assert_eq!(ruby_brace_glob(pattern.as_ptr(), 0, count_path, arg), 0);
            assert_eq!(count, 2);
            assert_eq!(ruby_glob(pattern.as_ptr(), 0, stop_at_first, arg), 7);

            let paths = array::new();
            rb_glob(pattern.as_ptr(), push_path, paths);
            assert_eq!(array::len(paths), 2);

            let directory = string::new_utf8(&base);
            assert!(rb_file_directory_p(Value::from(0), directory).is_true());
            let arguments = [string::new_utf8("x"), directory];
            let expanded = rb_file_s_expand_path(2, arguments.as_ptr());
            assert_eq!(string::value_to_string(expanded), format!("{}/x", base));
            let absolute = rb_file_s_absolute_path(1, arguments.as_ptr());
            assert!(string::value_to_string(absolute).ends_with("/x"));

            let pathname = vm::eval_string("require 'pathname'; Pathname.new('p/q')");
            assert_eq!(
                string::value_to_string(rb_get_path_no_checksafe(pathname)),
                "p/q"
            );
            assert_eq!(string::value_to_string(rb_get_path(pathname)), "p/q");
            assert_eq!(
                string::value_to_string(rb_str_encode_ospath(string::new_utf8("r"))),
                "r"
            );
            assert_eq!(rb_is_absolute_path(b"rel\0".as_ptr() as *const c_char), 0);

            let mut feature = string::new_utf8("rutie_no_such_feature");
            let extensions = [b".rb\0".as_ptr() as *const c_char, std::ptr::null()];
            assert_eq!(rb_find_file_ext(&mut feature, extensions.as_ptr()), 0);

            let cwd = ruby_getcwd();
            let expected = std::env::current_dir()
                .unwrap()
                .to_str()
                .unwrap()
                .replace('\\', "/");
            assert_eq!(
                CStr::from_ptr(cwd).to_str().unwrap().replace('\\', "/"),
                expected
            );
            crate::rubysys::gc::ruby_xfree(cwd as *mut c_void);

            std::fs::remove_dir_all(dir).unwrap();
        });
    }
}
