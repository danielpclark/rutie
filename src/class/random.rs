use std::convert::From;

use crate::{
    binding::{class, random, vm},
    rubysys::builtins::rb_cRandom,
    types::{c_long, c_ulong, Value},
    AnyObject, Integer, Object, RString, VerifiedObject,
};

/// `Random`, Ruby's pseudo-random number generator (Mersenne Twister
/// MT19937 by default).
///
/// The associated functions without a receiver, such as
/// [`default_int32`](#method.default_int32), use the generator behind
/// `Kernel#rand` (one per Ractor).
#[derive(Debug)]
#[repr(C)]
pub struct Random {
    value: Value,
}

impl Random {
    /// Creates a generator with a random seed (Ruby's `Random.new`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Random, VM};
    /// # VM::init();
    ///
    /// let random = Random::new();
    ///
    /// assert!(random.real() < 1.0);
    /// ```
    pub fn new() -> Self {
        Random::from(class::new_instance(unsafe { rb_cRandom }, &[]))
    }

    /// Creates a generator from `seed` (Ruby's `Random.new(seed)`); the same
    /// seed gives the same sequence.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, Random, VM};
    /// # VM::init();
    ///
    /// let first = Random::with_seed(&Integer::new(1234));
    /// let second = Random::with_seed(&Integer::new(1234));
    ///
    /// assert_eq!(first.int32(), second.int32());
    /// assert_eq!(first.seed().to_i64(), 1234);
    /// ```
    pub fn with_seed(seed: &Integer) -> Self {
        Random::from(class::new_instance(unsafe { rb_cRandom }, &[seed.value()]))
    }

    /// Returns the seed the generator was created with (Ruby's `seed`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, Random, VM};
    /// # VM::init();
    ///
    /// let big = Integer::from(u128::MAX);
    ///
    /// assert_eq!(Random::with_seed(&big).seed(), big);
    /// ```
    pub fn seed(&self) -> Integer {
        Integer::from(vm::call_method(self.value(), "seed", &[]))
    }

    /// Returns a random 32-bit integer (`rb_random_int32`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, Object, Random, VM};
    /// # VM::init();
    ///
    /// // Ruby's `bytes` packs the same 32-bit numbers, little-endian.
    /// let ruby = VM::eval("Random.new(5489).bytes(4).unpack1('V')").unwrap();
    /// let number = Random::with_seed(&Integer::new(5489)).int32();
    ///
    /// assert_eq!(ruby.try_convert_to::<Integer>().unwrap().to_u64(), number as u64);
    /// ```
    pub fn int32(&self) -> u32 {
        random::int32(self.value())
    }

    /// Returns a random float in `[0, 1)` (Ruby's `rand`,
    /// `rb_random_real`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Random, VM};
    /// # VM::init();
    ///
    /// let random = Random::new();
    ///
    /// for _ in 0..100 {
    ///     let number = random.real();
    ///
    ///     assert!(number >= 0.0 && number < 1.0);
    /// }
    /// ```
    pub fn real(&self) -> f64 {
        random::real(self.value())
    }

    /// Returns a random integer in `[0, limit]`, which includes `limit`
    /// (`rb_random_ulong_limited`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Random, VM};
    /// # VM::init();
    ///
    /// let random = Random::new();
    ///
    /// assert_eq!(random.ulong_limited(0), 0);
    /// assert!((0..100).all(|_| random.ulong_limited(6) <= 6));
    /// ```
    pub fn ulong_limited(&self, limit: c_ulong) -> c_ulong {
        random::ulong_limited(self.value(), limit)
    }

    /// Returns `len` random bytes as a binary string (Ruby's `bytes`,
    /// `rb_random_bytes`).
    ///
    /// # Panics
    ///
    /// If `len` does not fit a C `long`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, Random, VM};
    /// # VM::init();
    ///
    /// let bytes = Random::with_seed(&Integer::new(1)).bytes(16);
    ///
    /// assert_eq!(bytes.to_bytes_unchecked().len(), 16);
    /// assert_eq!(bytes.to_bytes_unchecked(), Random::with_seed(&Integer::new(1)).bytes(16).to_bytes_unchecked());
    /// ```
    pub fn bytes(&self, len: usize) -> RString {
        let len = c_long::try_from(len).expect("length out of range for a C long");

        RString::from(random::bytes(self.value(), len))
    }

    /// Returns a random 32-bit integer from the default generator
    /// (`rb_genrand_int32`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Random, VM};
    /// # VM::init();
    ///
    /// let numbers: Vec<u32> = (0..10).map(|_| Random::default_int32()).collect();
    ///
    /// assert!(numbers.iter().any(|&number| number != numbers[0]));
    /// ```
    pub fn default_int32() -> u32 {
        random::genrand_int32()
    }

    /// Returns a random float in `[0, 1)` from the default generator
    /// (`rb_genrand_real`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Random, VM};
    /// # VM::init();
    ///
    /// let number = Random::default_real();
    ///
    /// assert!(number >= 0.0 && number < 1.0);
    /// ```
    pub fn default_real() -> f64 {
        random::genrand_real()
    }

    /// Returns a random integer in `[0, limit]`, which includes `limit`,
    /// from the default generator (`rb_genrand_ulong_limited`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Random, VM};
    /// # VM::init();
    ///
    /// assert!((0..100).all(|_| Random::default_ulong_limited(9) <= 9));
    /// ```
    pub fn default_ulong_limited(limit: c_ulong) -> c_ulong {
        random::genrand_ulong_limited(limit)
    }

    /// Discards the state of the default generator, which seeds itself
    /// again on next use (`rb_reset_random_seed`). Useful in a child
    /// process after `fork`, so it does not repeat its parent's numbers.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Random, VM};
    /// # VM::init();
    ///
    /// Random::reset_default_seed();
    ///
    /// assert!(Random::default_real() < 1.0);
    /// ```
    pub fn reset_default_seed() {
        random::reset_seed()
    }

    /// Makes a float in `[0, 1)`, or `[0, 1]` unless `exclude_one`, from
    /// the 64 bits of two random 32-bit integers, `a` the most significant
    /// (`rb_int_pair_to_real`). Generators written against `ruby/random.h`
    /// use it to implement `get_real`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Random, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Random::int_pair_to_real(0, 0, true), 0.0);
    /// assert!(Random::int_pair_to_real(u32::MAX, u32::MAX, true) < 1.0);
    /// assert_eq!(Random::int_pair_to_real(u32::MAX, u32::MAX, false), 1.0);
    /// ```
    pub fn int_pair_to_real(a: u32, b: u32, exclude_one: bool) -> f64 {
        random::int_pair_to_real(a, b, exclude_one)
    }
}

impl From<Value> for Random {
    fn from(value: Value) -> Self {
        Random { value }
    }
}

impl Into<Value> for Random {
    fn into(self) -> Value {
        self.value
    }
}

impl Into<AnyObject> for Random {
    fn into(self) -> AnyObject {
        AnyObject::from(self.value)
    }
}

impl Object for Random {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for Random {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        class::is_kind_of(object.value(), unsafe { rb_cRandom })
    }

    fn error_message() -> &'static str {
        "Error converting to Random"
    }
}

impl PartialEq for Random {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

#[cfg(test)]
mod tests {
    use std::{ffi::CString, ptr};

    use crate::{
        rubysys::types::c_uint,
        rubysys::{class::rb_define_alloc_func, random::*, typed_data::rb_data_typed_object_wrap},
        types::{c_int, c_void, size_t, DataType, DataTypeFunction, Value},
        AnyObject, Class, Integer, Object, Random, VerifiedObject, VM,
    };

    #[test]
    fn test_random() {
        crate::on_ruby_thread(|| {
            let first = Random::with_seed(&Integer::new(42));
            let second = Random::with_seed(&Integer::new(42));
            for _ in 0..5 {
                assert_eq!(first.int32(), second.int32());
                assert_eq!(first.real(), second.real());
                assert_eq!(first.ulong_limited(1000), second.ulong_limited(1000));
            }
            assert_eq!(
                first.bytes(5).to_bytes_unchecked(),
                second.bytes(5).to_bytes_unchecked()
            );
            assert_eq!(first.bytes(0).to_bytes_unchecked().len(), 0);
            assert_eq!(first.seed().to_i64(), 42);

            // The C functions draw from the same stream as Ruby's methods.
            let ruby = VM::eval("r = Random.new(7); [r.bytes(4), r.rand]").unwrap();
            let ours = Random::with_seed(&Integer::new(7));
            let expected = crate::Array::from(ruby.value());
            assert_eq!(
                ours.bytes(4).to_bytes_unchecked(),
                crate::RString::from(expected.at(0).value()).to_bytes_unchecked()
            );
            assert_eq!(
                ours.real(),
                crate::Float::from(expected.at(1).value()).to_f64()
            );

            let random = Random::new();
            assert!((0..1000).all(|_| random.ulong_limited(3) <= 3));
            assert!((0..1000).any(|_| random.ulong_limited(3) == 3));
            assert!((0..1000).all(|_| Random::default_ulong_limited(3) <= 3));

            Random::reset_default_seed();
            let real = Random::default_real();
            assert!((0.0..1.0).contains(&real));
            let _ = Random::default_int32();

            let any: AnyObject = random.into();
            assert!(Random::is_correct_type(&any));
            assert!(any.try_convert_to::<Random>().is_ok());
            assert!(Integer::new(1).try_convert_to::<Random>().is_err());

            assert_eq!(Random::int_pair_to_real(1 << 31, 0, true), 0.5);
        });
    }

    // A generator defined through ruby/random.h: a counter that returns
    // the seed, then the seed plus one, and so on.
    #[repr(C)]
    struct Counter {
        base: RbRandom,
        next: u32,
    }

    rutie_callback! {
        fn counter_init(rng: *mut RbRandom, buf: *const u32, len: size_t) {
            let counter = unsafe { &mut *(rng as *mut Counter) };

            counter.next = if len == 0 { 0 } else { unsafe { *buf } };
        }
    }

    rutie_callback! {
        fn counter_init_int32(rng: *mut RbRandom, data: u32) {
            unsafe { (*(rng as *mut Counter)).next = data };
        }
    }

    rutie_callback! {
        fn counter_get_int32(rng: *mut RbRandom) -> c_uint {
            let counter = unsafe { &mut *(rng as *mut Counter) };
            let number = counter.next;

            counter.next = number.wrapping_add(1);

            number
        }
    }

    rutie_callback! {
        fn counter_get_bytes(rng: *mut RbRandom, buf: *mut c_void, len: size_t) {
            unsafe { rb_rand_bytes_int32(counter_get_int32, rng, buf, len) }
        }
    }

    rutie_callback! {
        fn counter_get_real(rng: *mut RbRandom, excl: c_int) -> f64 {
            let a = counter_get_int32(rng);
            let b = counter_get_int32(rng);

            unsafe { rb_int_pair_to_real(a, b, excl) }
        }
    }

    extern "C" fn counter_mark(ptr: *mut c_void) {
        unsafe { rb_random_mark(ptr) }
    }

    extern "C" fn counter_free(ptr: *mut c_void) {
        drop(unsafe { Box::from_raw(ptr as *mut Counter) });
    }

    static mut COUNTER_TYPE: *const DataType = ptr::null();

    rutie_callback! {
        fn counter_alloc(klass: Value) -> Value {
            let mut counter = Box::new(Counter {
                base: RbRandom { seed: Value::from(0) },
                next: 0,
            });

            unsafe {
                rb_random_base_init(&mut counter.base);

                rb_data_typed_object_wrap(
                    klass,
                    Box::into_raw(counter) as *mut c_void,
                    COUNTER_TYPE,
                )
            }
        }
    }

    #[test]
    fn test_random_interface() {
        crate::on_ruby_thread(|| {
            let interface = Box::leak(Box::new(RbRandomInterface {
                default_seed_bits: 32,
                version: RbRandomInterfaceVersion {
                    major: RUBY_RANDOM_INTERFACE_VERSION_MAJOR,
                    minor: RUBY_RANDOM_INTERFACE_VERSION_MINOR,
                },
                flags: 0,
                init: Some(counter_init),
                init_int32: Some(counter_init_int32),
                get_int32: Some(counter_get_int32),
                get_bytes: Some(counter_get_bytes),
                get_real: Some(counter_get_real),
            }));

            // Filled in at run time, which `RB_RANDOM_DATA_INIT_PARENT` does
            // for C on Windows.
            let data_type = Box::leak(Box::new(DataType {
                wrap_struct_name: CString::new("Rutie/Counter").unwrap().into_raw(),
                function: DataTypeFunction {
                    dmark: Some(counter_mark),
                    dfree: Some(counter_free),
                    dsize: None,
                    reserved: [ptr::null_mut(); 2],
                },
                parent: ptr::addr_of!(rb_random_data_type_1_0),
                data: interface as *mut RbRandomInterface as *mut c_void,
                flags: Value::from(0),
            }));

            let class = Class::new("RutieCounterRandom", Some(&Class::random()));
            unsafe {
                COUNTER_TYPE = data_type;
                rb_define_alloc_func(class.value(), counter_alloc);
            }

            let counter = VM::eval("RutieCounterRandom.new(7)")
                .unwrap()
                .try_convert_to::<Random>()
                .unwrap();

            assert_eq!(counter.seed().to_i64(), 7);
            assert_eq!(counter.int32(), 7);
            assert_eq!(counter.int32(), 8);
            assert_eq!(counter.bytes(6).to_bytes_unchecked(), &[9, 0, 0, 0, 10, 0]);
            assert_eq!(counter.real(), Random::int_pair_to_real(12, 13, true));

            // Ruby's own methods use the interface too.
            let big = VM::eval("RutieCounterRandom.new(2**40).bytes(4).unpack1('V')").unwrap();
            assert!(big.try_convert_to::<Integer>().is_ok());
            assert_eq!(
                VM::eval("RutieCounterRandom.new(1).rand(1.0)")
                    .unwrap()
                    .try_convert_to::<crate::Float>()
                    .unwrap()
                    .to_f64(),
                Random::int_pair_to_real(1, 2, true)
            );

            crate::GC::start();
        });
    }
}
