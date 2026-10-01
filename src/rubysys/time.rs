use crate::rubysys::{
    libc::{time_t, timespec, timeval},
    types::{c_int, c_long, Value},
};

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    pub static rb_cTime: Value;
}

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // struct timeval
    // rb_time_interval(VALUE num)
    pub fn rb_time_interval(number: Value) -> timeval;
    // VALUE
    // rb_time_nano_new(time_t sec, long nsec)
    pub fn rb_time_nano_new(seconds: time_t, nanoseconds: c_long) -> Value;
    // VALUE
    // rb_time_new(time_t sec, long usec)
    pub fn rb_time_new(seconds: time_t, microseconds: c_long) -> Value;
    // VALUE
    // rb_time_num_new(VALUE timev, VALUE off)
    //
    // `timev` is not checked: it must be an exact Integer or Rational.
    pub fn rb_time_num_new(time: Value, offset: Value) -> Value;
    // struct timespec
    // rb_time_timespec(VALUE time)
    pub fn rb_time_timespec(time: Value) -> timespec;
    // VALUE
    // rb_time_timespec_new(const struct timespec *ts, int offset)
    pub fn rb_time_timespec_new(timespec: *const timespec, offset: c_int) -> Value;
    // struct timeval
    // rb_time_timeval(VALUE time)
    pub fn rb_time_timeval(time: Value) -> timeval;
    // VALUE
    // rb_time_utc_offset(VALUE time)
    pub fn rb_time_utc_offset(time: Value) -> Value;
}

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // struct timespec
    // rb_time_timespec_interval(VALUE num)
    //
    // Raises `ArgumentError` for a negative interval, `TypeError` for a
    // non-numeric one.
    pub fn rb_time_timespec_interval(number: Value) -> timespec;
    // void
    // rb_timespec_now(struct timespec *ts)
    //
    // The realtime clock (`CLOCK_REALTIME`).
    pub fn rb_timespec_now(timespec: *mut timespec);
}
