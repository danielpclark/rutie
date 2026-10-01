// C string helpers: `ruby/util.h` (`ruby_strdup`, `ruby_each_words`) and
// `ruby/ruby.h` (`ruby_snprintf`, class names as C strings).

use crate::rubysys::types::{c_char, c_int, c_void, size_t, Value};

// The callback of `ruby_each_words`: a word (not NUL-terminated), its length
// and the `argv` given.
pub type EachWordsFunction =
    rutie_callback!(type fn(word: *const c_char, len: c_int, argv: *mut c_void));

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // const char *
    // rb_class2name(VALUE klass)
    //
    // The name of `klass` (skipping singleton classes), valid while the
    // name String lives; anonymous classes get a `#<Class:0x...>` name.
    pub fn rb_class2name(klass: Value) -> *const c_char;
    // const char *
    // rb_obj_classname(VALUE obj)
    //
    // `rb_class2name(CLASS_OF(obj))`.
    pub fn rb_obj_classname(obj: Value) -> *const c_char;
    // void
    // ruby_each_words(const char *str, void (*func)(const char *word, int len, void *argv),
    //                 void *argv)
    //
    // Calls `func` for each word of `str`, split at white space and commas.
    pub fn ruby_each_words(str: *const c_char, func: EachWordsFunction, argv: *mut c_void);
    // int
    // ruby_snprintf(char *str, size_t n, char const *fmt, ...)
    //
    // `snprintf(3)` with Ruby's own implementation (the same on every
    // platform; no `PRIsVALUE`).
    pub fn ruby_snprintf(str: *mut c_char, n: size_t, fmt: *const c_char, ...) -> c_int;
    // char *
    // ruby_strdup(const char *str)
    //
    // A copy allocated with `ruby_xmalloc` (free it with `ruby_xfree`).
    pub fn ruby_strdup(str: *const c_char) -> *mut c_char;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding::vm;
    use std::ffi::CStr;

    rutie_callback! {
        fn collect_word(word: *const c_char, len: c_int, argv: *mut c_void) {
            let word = unsafe { std::slice::from_raw_parts(word as *const u8, len as usize) };
            let words = unsafe { &mut *(argv as *mut Vec<String>) };
            words.push(String::from_utf8(word.to_vec()).unwrap());
        }
    }

    #[test]
    fn test_c_strings() {
        crate::on_ruby_thread(|| unsafe {
            let klass = vm::eval_string("Comparable");
            assert_eq!(
                CStr::from_ptr(rb_class2name(klass)).to_str().unwrap(),
                "Comparable"
            );
            let object = vm::eval_string("1.0");
            assert_eq!(
                CStr::from_ptr(rb_obj_classname(object)).to_str().unwrap(),
                "Float"
            );

            let mut words: Vec<String> = Vec::new();
            ruby_each_words(
                b"one two,three  four\0".as_ptr() as *const c_char,
                collect_word,
                &mut words as *mut Vec<String> as *mut c_void,
            );
            assert_eq!(words, ["one", "two", "three", "four"]);

            let mut buffer = [0 as c_char; 32];
            let written = ruby_snprintf(
                buffer.as_mut_ptr(),
                buffer.len(),
                b"%s-%d-%.2f\0".as_ptr() as *const c_char,
                b"x\0".as_ptr() as *const c_char,
                42 as c_int,
                1.5f64,
            );
            assert_eq!(written, 9);
            assert_eq!(
                CStr::from_ptr(buffer.as_ptr()).to_str().unwrap(),
                "x-42-1.50"
            );
            // Truncated to the buffer, returning the full length.
            let written = ruby_snprintf(
                buffer.as_mut_ptr(),
                3,
                b"%s\0".as_ptr() as *const c_char,
                b"abcdef\0".as_ptr() as *const c_char,
            );
            assert_eq!(written, 6);
            assert_eq!(CStr::from_ptr(buffer.as_ptr()).to_str().unwrap(), "ab");

            let copy = ruby_strdup(b"rutie\0".as_ptr() as *const c_char);
            assert_eq!(CStr::from_ptr(copy).to_str().unwrap(), "rutie");
            crate::rubysys::gc::ruby_xfree(copy as *mut c_void);
        });
    }
}
