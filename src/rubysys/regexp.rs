use crate::rubysys::types::{c_char, c_int, c_long, Value};

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
