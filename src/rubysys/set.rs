use crate::rubysys::types::{c_int, size_t, Value};

// Ruby 4.0's C API for `Set` (`ruby/internal/intern/set.h`).
#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // void
    // rb_set_foreach(VALUE set, int (*func)(VALUE element, VALUE arg), VALUE arg)
    //
    // `func` returns an `st_retval` (`ST_CONTINUE`, `ST_STOP`, ...).
    pub fn rb_set_foreach(
        set: Value,
        func: crate::rutie_callback!(type fn(Value, Value) -> c_int),
        arg: Value,
    );
    // VALUE
    // rb_set_new(void)
    pub fn rb_set_new() -> Value;
    // VALUE
    // rb_set_new_capa(size_t capa)
    pub fn rb_set_new_capa(capa: size_t) -> Value;
    // bool
    // rb_set_lookup(VALUE set, VALUE element)
    pub fn rb_set_lookup(set: Value, element: Value) -> bool;
    // bool
    // rb_set_add(VALUE set, VALUE element)
    pub fn rb_set_add(set: Value, element: Value) -> bool;
    // VALUE
    // rb_set_clear(VALUE set)
    pub fn rb_set_clear(set: Value) -> Value;
    // bool
    // rb_set_delete(VALUE set, VALUE element)
    pub fn rb_set_delete(set: Value, element: Value) -> bool;
    // size_t
    // rb_set_size(VALUE set)
    pub fn rb_set_size(set: Value) -> size_t;
}
