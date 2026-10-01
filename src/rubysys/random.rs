use crate::rubysys::{
    libc::{c_long, c_ulong},
    types::{c_double, c_int, c_uint, c_void, size_t, RbDataType, Value},
};

// ruby/random.h: the interface for defining a PRNG that Ruby's `Random`
// methods can use. That header is not included by ruby/ruby.h.

pub const RUBY_RANDOM_INTERFACE_VERSION_MAJOR: u8 = 1;
pub const RUBY_RANDOM_INTERFACE_VERSION_MINOR: u8 = 0;
pub const RUBY_RANDOM_INTERFACE_VERSION_MAJOR_MAX: u8 = 0xff;
pub const RUBY_RANDOM_INTERFACE_VERSION_MINOR_MAX: u8 = 0xff;

// struct rb_random_struct { VALUE seed; } (`rb_random_t`): the base part a
// PRNG's own struct starts with.
#[repr(C)]
pub struct RbRandom {
    pub seed: Value,
}

// typedef void rb_random_init_func(rb_random_t *rng, const uint32_t *buf, size_t len);
pub type RbRandomInitFunc =
    rutie_callback!(type fn(rng: *mut RbRandom, buf: *const u32, len: size_t));
// typedef void rb_random_init_int32_func(rb_random_t *rng, uint32_t data);
pub type RbRandomInitInt32Func = rutie_callback!(type fn(rng: *mut RbRandom, data: u32));
// typedef unsigned int rb_random_get_int32_func(rb_random_t *rng);
pub type RbRandomGetInt32Func = rutie_callback!(type fn(rng: *mut RbRandom) -> c_uint);
// typedef void rb_random_get_bytes_func(rb_random_t *rng, void *buf, size_t len);
pub type RbRandomGetBytesFunc =
    rutie_callback!(type fn(rng: *mut RbRandom, buf: *mut c_void, len: size_t));
// typedef double rb_random_get_real_func(rb_random_t *rng, int excl);
pub type RbRandomGetRealFunc =
    rutie_callback!(type fn(rng: *mut RbRandom, excl: c_int) -> c_double);

// struct { uint8_t major, minor; } version; in `rb_random_interface_t`.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RbRandomInterfaceVersion {
    pub major: u8,
    pub minor: u8,
}

// `rb_random_interface_t`: the PRNG's algorithms, pointed to by the `data`
// field of its `rb_data_type_t`.
#[repr(C)]
pub struct RbRandomInterface {
    pub default_seed_bits: size_t,
    pub version: RbRandomInterfaceVersion,
    pub flags: u16,
    pub init: Option<RbRandomInitFunc>,
    pub init_int32: Option<RbRandomInitInt32Func>,
    pub get_int32: Option<RbRandomGetInt32Func>,
    pub get_bytes: Option<RbRandomGetBytesFunc>,
    pub get_real: Option<RbRandomGetRealFunc>,
}

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // ruby/internal/intern/random.h

    // unsigned int
    // rb_genrand_int32(void)
    pub fn rb_genrand_int32() -> c_uint;
    // double
    // rb_genrand_real(void)
    pub fn rb_genrand_real() -> c_double;
    // unsigned long
    // rb_genrand_ulong_limited(unsigned long i)
    pub fn rb_genrand_ulong_limited(limit: c_ulong) -> c_ulong;
    // VALUE
    // rb_random_bytes(VALUE rnd, long n)
    pub fn rb_random_bytes(random: Value, n: c_long) -> Value;
    // unsigned int
    // rb_random_int32(VALUE rnd)
    pub fn rb_random_int32(random: Value) -> c_uint;
    // double
    // rb_random_real(VALUE rnd)
    pub fn rb_random_real(random: Value) -> c_double;
    // unsigned long
    // rb_random_ulong_limited(VALUE rnd, unsigned long limit)
    pub fn rb_random_ulong_limited(random: Value, limit: c_ulong) -> c_ulong;
    // void
    // rb_reset_random_seed(void)
    pub fn rb_reset_random_seed();

    // ruby/random.h

    // double
    // rb_int_pair_to_real(uint32_t a, uint32_t b, int excl)
    pub fn rb_int_pair_to_real(a: u32, b: u32, excl: c_int) -> c_double;
    // void
    // rb_rand_bytes_int32(rb_random_get_int32_func *func, rb_random_t *prng,
    //                     void *buff, size_t size)
    pub fn rb_rand_bytes_int32(
        func: RbRandomGetInt32Func,
        prng: *mut RbRandom,
        buff: *mut c_void,
        size: size_t,
    );
    // void
    // rb_random_base_init(rb_random_t *rnd)
    pub fn rb_random_base_init(random: *mut RbRandom);
    // void
    // rb_random_mark(void *ptr)
    pub fn rb_random_mark(ptr: *mut c_void);

    // const rb_data_type_t rb_random_data_type
    //
    // NOT IN PUBLIC HEADERS under this name: `rb_random_data_type` is a macro
    // for the versioned symbol. The parent type of a PRNG's `rb_data_type_t`.
    pub static rb_random_data_type_1_0: RbDataType;
}
