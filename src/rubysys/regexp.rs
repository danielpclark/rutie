use crate::rubysys::types::{c_char, c_int, c_long, c_void, EncodingType, Value};

// `OnigPosition`, Onigmo's `ptrdiff_t` offsets.
pub type OnigPosition = isize;

// `regex_t` (`OnigRegexType`), a compiled Onigmo pattern (opaque).
#[repr(C)]
pub struct OnigRegexType {
    _private: [u8; 0],
}

// `struct re_registers`, the match offsets. Ruby builds Onigmo without
// `USE_CAPTURE_HISTORY`, so there is no `history_root`.
#[repr(C)]
pub struct ReRegisters {
    pub allocated: c_int,
    pub num_regs: c_int,
    pub beg: *mut OnigPosition,
    pub end: *mut OnigPosition,
}

// The `match` callback of `rb_reg_onig_match`.
pub type OnigMatchFunction = rutie_callback!(type fn(
    reg: *mut OnigRegexType,
    str: Value,
    regs: *mut ReRegisters,
    args: *mut c_void,
) -> OnigPosition);

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    pub static rb_cMatch: Value;
    pub static rb_cRegexp: Value;
}

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // VALUE
    // rb_backref_get(void)
    pub fn rb_backref_get() -> Value;
    // void
    // rb_backref_set(VALUE val)
    pub fn rb_backref_set(value: Value);
    // int
    // rb_reg_backref_number(VALUE match, VALUE backref)
    pub fn rb_reg_backref_number(match_data: Value, backref: Value) -> c_int;
    // VALUE
    // rb_reg_last_match(VALUE match)
    pub fn rb_reg_last_match(match_data: Value) -> Value;
    // VALUE
    // rb_reg_match(VALUE re, VALUE str)
    //
    // Sets `$~`; returns the character offset or `Qnil`.
    pub fn rb_reg_match(regexp: Value, string: Value) -> Value;
    // VALUE
    // rb_reg_match2(VALUE re)
    //
    // Matches against `$_`.
    pub fn rb_reg_match2(regexp: Value) -> Value;
    // VALUE
    // rb_reg_match_post(VALUE match)
    pub fn rb_reg_match_post(match_data: Value) -> Value;
    // VALUE
    // rb_reg_match_pre(VALUE match)
    pub fn rb_reg_match_pre(match_data: Value) -> Value;
    // VALUE
    // rb_reg_new(const char *s, long len, int options)
    pub fn rb_reg_new(pattern: *const c_char, len: c_long, options: c_int) -> Value;
    // VALUE
    // rb_reg_new_str(VALUE s, int options)
    pub fn rb_reg_new_str(pattern: Value, options: c_int) -> Value;
    // VALUE
    // rb_reg_nth_defined(int nth, VALUE match)
    pub fn rb_reg_nth_defined(nth: c_int, match_data: Value) -> Value;
    // VALUE
    // rb_reg_nth_match(int nth, VALUE match)
    pub fn rb_reg_nth_match(nth: c_int, match_data: Value) -> Value;
    // int
    // rb_reg_options(VALUE re)
    pub fn rb_reg_options(regexp: Value) -> c_int;
    // VALUE
    // rb_reg_regcomp(VALUE str)
    //
    // Cached compilation of a pattern string.
    pub fn rb_reg_regcomp(pattern: Value) -> Value;
    // VALUE
    // rb_enc_reg_new(const char *ptr, long len, rb_encoding *enc, int opts)
    pub fn rb_enc_reg_new(ptr: *const c_char, len: c_long, enc: EncodingType, opts: c_int)
        -> Value;
    // void
    // rb_match_busy(VALUE md)
    pub fn rb_match_busy(md: Value);
    // int
    // rb_memcicmp(const void *s1,const void *s2, long n)
    pub fn rb_memcicmp(s1: *const c_void, s2: *const c_void, n: c_long) -> c_int;
    // VALUE
    // rb_reg_alloc(void)
    pub fn rb_reg_alloc() -> Value;
    // VALUE
    // rb_reg_init_str(VALUE re, VALUE s, int options)
    pub fn rb_reg_init_str(re: Value, s: Value, options: c_int) -> Value;
    // VALUE
    // rb_reg_match_last(VALUE md)
    //
    // The last matched group (`$+`).
    pub fn rb_reg_match_last(md: Value) -> Value;
    // long
    // rb_reg_search(VALUE re, VALUE str, long pos, int dir)
    //
    // Byte offset of the match, or -1; sets `$~`. Searches backwards from
    // `pos` when `dir` is non-zero.
    pub fn rb_reg_search(re: Value, str: Value, pos: c_long, dir: c_int) -> c_long;
    // VALUE
    // rb_reg_regsub(VALUE repl, VALUE src, struct re_registers *regs, VALUE rexp)
    pub fn rb_reg_regsub(repl: Value, src: Value, regs: *mut ReRegisters, rexp: Value) -> Value;
    // long
    // rb_reg_adjust_startpos(VALUE re, VALUE str, long pos, int dir)
    pub fn rb_reg_adjust_startpos(re: Value, str: Value, pos: c_long, dir: c_int) -> c_long;
    // VALUE
    // rb_reg_quote(VALUE str)
    pub fn rb_reg_quote(str: Value) -> Value;
    // regex_t *
    // rb_reg_prepare_re(VALUE re, VALUE str)
    pub fn rb_reg_prepare_re(re: Value, str: Value) -> *mut OnigRegexType;
    // OnigPosition
    // rb_reg_onig_match(VALUE re, VALUE str,
    //                   OnigPosition (*match)(regex_t *reg, VALUE str, struct re_registers *regs, void *args),
    //                   void *args, struct re_registers *regs)
    pub fn rb_reg_onig_match(
        re: Value,
        str: Value,
        match_: OnigMatchFunction,
        args: *mut c_void,
        regs: *mut ReRegisters,
    ) -> OnigPosition;
    // int
    // rb_reg_region_copy(struct re_registers *dst, const struct re_registers *src)
    pub fn rb_reg_region_copy(dst: *mut ReRegisters, src: *const ReRegisters) -> c_int;
}

#[cfg(test)]
mod tests {
    use super::{rb_reg_onig_match, OnigPosition, OnigRegexType, ReRegisters};
    use crate::{rubysys::types::c_void, Object, RString, Regexp};

    rutie_callback! {
        fn report_match(
            reg: *mut OnigRegexType,
            _str: crate::types::Value,
            _regs: *mut ReRegisters,
            args: *mut c_void,
        ) -> OnigPosition {
            let calls = unsafe { &mut *(args as *mut (usize, OnigPosition)) };
            assert!(!reg.is_null());
            calls.0 += 1;
            calls.1
        }
    }

    // Ruby passes the prepared pattern and our argument to the callback and
    // returns what it returns.
    #[test]
    fn test_onig_match_callback() {
        crate::on_ruby_thread(|| {
            let regexp = Regexp::new("b", 0).unwrap();
            let string = RString::new_utf8("abc");
            let mut regs = ReRegisters {
                allocated: 0,
                num_regs: 0,
                beg: std::ptr::null_mut(),
                end: std::ptr::null_mut(),
            };
            let mut calls: (usize, OnigPosition) = (0, 1);

            let found = unsafe {
                rb_reg_onig_match(
                    regexp.value(),
                    string.value(),
                    report_match,
                    &mut calls as *mut _ as *mut c_void,
                    &mut regs,
                )
            };
            assert_eq!((found, calls.0), (1, 1));

            // `ONIG_MISMATCH`: Ruby frees the (empty) registers.
            calls.1 = -1;
            let found = unsafe {
                rb_reg_onig_match(
                    regexp.value(),
                    string.value(),
                    report_match,
                    &mut calls as *mut _ as *mut c_void,
                    &mut regs,
                )
            };
            assert_eq!((found, calls.0), (-1, 2));
        });
    }
}
