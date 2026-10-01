use crate::rubysys::types::{c_char, c_int, c_long, c_void, Value};

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

// The `match` callback of `rb_reg_onig_match` (Ruby 3.3+).
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
}

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // OnigPosition
    // rb_reg_onig_match(VALUE re, VALUE str,
    //                   OnigPosition (*match)(regex_t *reg, VALUE str, struct re_registers *regs, void *args),
    //                   void *args, struct re_registers *regs)
    //
    // Ruby 3.3+: prepares `re` for `str` (recompiling it for another
    // encoding), calls `match` with it and returns what `match` returns,
    // freeing `regs` on `ONIG_MISMATCH` (-1). Raises for a negative error.
    #[cfg(ruby_gte_3_3)]
    pub fn rb_reg_onig_match(
        re: Value,
        str: Value,
        match_: OnigMatchFunction,
        args: *mut c_void,
        regs: *mut ReRegisters,
    ) -> OnigPosition;
}

#[cfg(all(test, ruby_gte_3_3))]
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
            let calls = unsafe { &mut *(args as *mut (usize, OnigPosition, bool)) };
            calls.0 += 1;
            calls.2 &= !reg.is_null();
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
            let mut calls: (usize, OnigPosition, bool) = (0, 1, true);

            let found = unsafe {
                rb_reg_onig_match(
                    regexp.value(),
                    string.value(),
                    report_match,
                    &mut calls as *mut _ as *mut c_void,
                    &mut regs,
                )
            };
            assert_eq!((found, calls.0, calls.2), (1, 1, true));

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
            assert_eq!((found, calls.0, calls.2), (-1, 2, true));
        });
    }
}
