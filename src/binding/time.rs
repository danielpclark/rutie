use libc::{time_t, timespec};

use crate::{
    rubysys::time,
    types::{c_long, Value},
};

pub fn from_unix(seconds: i64, nanoseconds: u32) -> Value {
    unsafe { time::rb_time_nano_new(seconds as time_t, nanoseconds as c_long) }
}

// `(seconds, nanoseconds)` since the Unix epoch; nanoseconds in 0..1e9.
pub fn to_unix(time: Value) -> (i64, u32) {
    let timespec: timespec = unsafe { time::rb_time_timespec(time) };

    (timespec.tv_sec as i64, timespec.tv_nsec as u32)
}

pub fn utc_offset(time: Value) -> Value {
    unsafe { time::rb_time_utc_offset(time) }
}

// Raises `ArgumentError` for a negative interval and `TypeError` for a
// non-numeric one.
pub fn interval(number: Value) -> (i64, u32) {
    let timeval = unsafe { time::rb_time_interval(number) };

    (timeval.tv_sec as i64, timeval.tv_usec as u32)
}

// `number` must already be an exact `Integer` or `Rational`: unlike
// `Time.at`, `rb_time_num_new` does not check it.
pub fn from_numeric(number: Value, offset: Value) -> Value {
    unsafe { time::rb_time_num_new(number, offset) }
}
