use crate::{
    rubysys::random,
    types::{c_long, c_ulong, Value},
};

pub fn genrand_int32() -> u32 {
    unsafe { random::rb_genrand_int32() as u32 }
}

pub fn genrand_real() -> f64 {
    unsafe { random::rb_genrand_real() }
}

pub fn genrand_ulong_limited(limit: c_ulong) -> c_ulong {
    unsafe { random::rb_genrand_ulong_limited(limit) }
}

pub fn reset_seed() {
    unsafe { random::rb_reset_random_seed() }
}

pub fn int32(rng: Value) -> u32 {
    unsafe { random::rb_random_int32(rng) as u32 }
}

pub fn real(rng: Value) -> f64 {
    unsafe { random::rb_random_real(rng) }
}

pub fn ulong_limited(rng: Value, limit: c_ulong) -> c_ulong {
    unsafe { random::rb_random_ulong_limited(rng, limit) }
}

pub fn bytes(rng: Value, len: c_long) -> Value {
    unsafe { random::rb_random_bytes(rng, len) }
}

pub fn int_pair_to_real(a: u32, b: u32, exclude_one: bool) -> f64 {
    unsafe { random::rb_int_pair_to_real(a, b, exclude_one as _) }
}
