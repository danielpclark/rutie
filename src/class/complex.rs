use std::convert::From;

use crate::{
    binding::{float, numeric, vm},
    types::{Value, ValueType},
    AnyException, AnyObject, Float, Object, VerifiedObject,
};

/// `Complex`, a complex number.
#[derive(Debug)]
#[repr(C)]
pub struct Complex {
    value: Value,
}

impl Complex {
    /// Creates `real + imaginary·i` from numeric parts, or returns the
    /// exception when a part is not numeric (Ruby's `Complex(real,
    /// imaginary)`, `rb_Complex`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Complex, Fixnum, Object, RString, VM};
    /// # VM::init();
    ///
    /// let z = Complex::new(&Fixnum::new(3), &Fixnum::new(4)).unwrap();
    ///
    /// assert_eq!(z.real().try_convert_to::<Fixnum>(), Ok(Fixnum::new(3)));
    /// assert_eq!(z.abs(), 5.0);
    ///
    /// assert!(Complex::new(&Fixnum::new(1), &VM::eval("Object.new").unwrap()).is_err());
    /// ```
    pub fn new<R: Object, I: Object>(real: &R, imaginary: &I) -> Result<Self, AnyException> {
        let (real, imaginary) = (real.value(), imaginary.value());

        vm::protect_value(|| numeric::complex_from_parts(real, imaginary))
            .map(Complex::from)
            .map_err(AnyException::from)
    }

    /// Creates `real + imaginary·i` from two `f64`s (`rb_dbl_complex_new`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Complex, VM};
    /// # VM::init();
    ///
    /// let z = Complex::from_f64(1.5, -2.0);
    ///
    /// assert_eq!(z.arg(), (-2.0f64).atan2(1.5));
    /// ```
    pub fn from_f64(real: f64, imaginary: f64) -> Self {
        Complex::from(numeric::complex_from_f64(real, imaginary))
    }

    /// Creates the complex number with magnitude `abs` and angle `arg`
    /// radians (Ruby's `Complex.polar`, `rb_complex_new_polar`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Complex, VM};
    /// # VM::init();
    ///
    /// let z = Complex::polar(2.0, 0.0);
    ///
    /// assert_eq!(z.abs(), 2.0);
    /// ```
    pub fn polar(abs: f64, arg: f64) -> Self {
        let abs = Float::new(abs);
        let arg = Float::new(arg);

        Complex::from(numeric::complex_polar(abs.value(), arg.value()))
    }

    /// Converts `object` to a `Complex` the way Ruby's `Complex(object)`
    /// (`rb_Complex`) does, parsing strings such as `"1+2i"`. Returns the
    /// exception when it cannot be converted.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Complex, Fixnum, Object, RString, VM};
    /// # VM::init();
    ///
    /// let z = Complex::convert(&RString::new_utf8("1+2i")).unwrap();
    ///
    /// assert_eq!(z.imaginary().try_convert_to::<Fixnum>(), Ok(Fixnum::new(2)));
    /// assert!(Complex::convert(&RString::new_utf8("not a number")).is_err());
    /// ```
    pub fn convert<T: Object>(object: &T) -> Result<Self, AnyException> {
        let object = object.value();

        vm::protect_value(|| numeric::to_complex(object))
            .map(Complex::from)
            .map_err(AnyException::from)
    }

    /// Returns the real part (`rb_complex_real`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Complex, Float, Object, VM};
    /// # VM::init();
    ///
    /// let real = Complex::from_f64(1.5, 2.0).real();
    ///
    /// assert_eq!(real.try_convert_to::<Float>().unwrap().to_f64(), 1.5);
    /// ```
    pub fn real(&self) -> AnyObject {
        AnyObject::from(numeric::complex_real(self.value()))
    }

    /// Returns the imaginary part (`rb_complex_imag`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Complex, Float, Object, VM};
    /// # VM::init();
    ///
    /// let imaginary = Complex::from_f64(1.5, 2.0).imaginary();
    ///
    /// assert_eq!(imaginary.try_convert_to::<Float>().unwrap().to_f64(), 2.0);
    /// ```
    pub fn imaginary(&self) -> AnyObject {
        AnyObject::from(numeric::complex_imaginary(self.value()))
    }

    /// Returns the magnitude (`rb_complex_abs`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Complex, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Complex::from_f64(3.0, 4.0).abs(), 5.0);
    /// ```
    pub fn abs(&self) -> f64 {
        float::num_to_float(numeric::complex_abs(self.value()))
    }

    /// Returns the angle in radians (`rb_complex_arg`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Complex, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Complex::from_f64(0.0, 1.0).arg(), std::f64::consts::FRAC_PI_2);
    /// ```
    pub fn arg(&self) -> f64 {
        float::num_to_float(numeric::complex_arg(self.value()))
    }

    /// Returns `self + other` (Ruby's `+`, `rb_complex_plus`), or the
    /// exception when `other` cannot be added, such as a `TypeError` for
    /// a non-numeric object.
    ///
    /// The result is a `Complex` for any `Numeric` `other`; an object whose
    /// `coerce` method takes over may return something else.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Complex, Fixnum, Object, RString, VM};
    /// # VM::init();
    ///
    /// let z = Complex::from_f64(1.0, 2.0);
    /// let sum = z.add(&Fixnum::new(2)).unwrap().try_convert_to::<Complex>().unwrap();
    ///
    /// assert_eq!(sum, Complex::from_f64(3.0, 2.0));
    /// assert!(z.add(&RString::new_utf8("1")).is_err());
    /// ```
    pub fn add<T: Object>(&self, other: &T) -> Result<AnyObject, AnyException> {
        self.protect_binary(other, numeric::complex_plus)
    }

    /// Returns `self - other` (Ruby's `-`, `rb_complex_minus`), or the
    /// exception when `other` cannot be subtracted.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Complex, Object, VM};
    /// # VM::init();
    ///
    /// let difference = Complex::from_f64(1.0, 2.0).sub(&Complex::from_f64(0.5, 3.0)).unwrap();
    ///
    /// assert_eq!(difference.try_convert_to::<Complex>(), Ok(Complex::from_f64(0.5, -1.0)));
    /// ```
    pub fn sub<T: Object>(&self, other: &T) -> Result<AnyObject, AnyException> {
        self.protect_binary(other, numeric::complex_minus)
    }

    /// Returns `self * other` (Ruby's `*`, `rb_complex_mul`), or the
    /// exception when `other` cannot be multiplied.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Complex, Object, VM};
    /// # VM::init();
    ///
    /// let i = Complex::from_f64(0.0, 1.0);
    /// let product = i.mul(&i).unwrap().try_convert_to::<Complex>().unwrap();
    ///
    /// assert_eq!(product, Complex::from_f64(-1.0, 0.0));
    /// ```
    pub fn mul<T: Object>(&self, other: &T) -> Result<AnyObject, AnyException> {
        self.protect_binary(other, numeric::complex_mul)
    }

    /// Returns `self / other` (Ruby's `/`, `rb_complex_div`), or the
    /// exception, such as the `ZeroDivisionError` for an exact zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Complex, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let z = Complex::new(&Fixnum::new(4), &Fixnum::new(2)).unwrap();
    /// let half = z.div(&Fixnum::new(2)).unwrap().try_convert_to::<Complex>().unwrap();
    ///
    /// assert_eq!(half, Complex::new(&Fixnum::new(2), &Fixnum::new(1)).unwrap());
    /// assert!(z.div(&Fixnum::new(0)).is_err());
    /// ```
    pub fn div<T: Object>(&self, other: &T) -> Result<AnyObject, AnyException> {
        self.protect_binary(other, numeric::complex_div)
    }

    /// Returns `self` raised to `exponent` (Ruby's `**`, `rb_complex_pow`),
    /// or the exception, such as the `ZeroDivisionError` for an exact zero
    /// raised to a negative power.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Complex, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let z = Complex::new(&Fixnum::new(1), &Fixnum::new(1)).unwrap();
    /// let square = z.pow(&Fixnum::new(2)).unwrap().try_convert_to::<Complex>().unwrap();
    ///
    /// assert_eq!(square, Complex::new(&Fixnum::new(0), &Fixnum::new(2)).unwrap());
    ///
    /// let zero = Complex::new(&Fixnum::new(0), &Fixnum::new(0)).unwrap();
    /// assert!(zero.pow(&Fixnum::new(-1)).is_err());
    /// ```
    pub fn pow<T: Object>(&self, exponent: &T) -> Result<AnyObject, AnyException> {
        self.protect_binary(exponent, numeric::complex_pow)
    }

    /// Returns `-self` (Ruby's unary `-`, `rb_complex_uminus`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Complex, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Complex::from_f64(1.0, -2.0).neg(), Complex::from_f64(-1.0, 2.0));
    /// ```
    pub fn neg(&self) -> Complex {
        Complex::from(numeric::complex_uminus(self.value()))
    }

    /// Returns the complex conjugate, `real - imaginary·i` (Ruby's
    /// `conjugate`, `rb_complex_conjugate`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Complex, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Complex::from_f64(1.0, 2.0).conjugate(), Complex::from_f64(1.0, -2.0));
    /// ```
    pub fn conjugate(&self) -> Complex {
        Complex::from(numeric::complex_conjugate(self.value()))
    }

    fn protect_binary<T: Object>(
        &self,
        other: &T,
        operation: fn(Value, Value) -> Value,
    ) -> Result<AnyObject, AnyException> {
        let (complex, other) = (self.value(), other.value());

        vm::protect_value(|| operation(complex, other))
            .map(AnyObject::from)
            .map_err(AnyException::from)
    }
}

impl From<Value> for Complex {
    fn from(value: Value) -> Self {
        Complex { value }
    }
}

impl Into<Value> for Complex {
    fn into(self) -> Value {
        self.value
    }
}

impl Into<AnyObject> for Complex {
    fn into(self) -> AnyObject {
        AnyObject::from(self.value)
    }
}

impl Object for Complex {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for Complex {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        object.value().ty() == ValueType::Complex
    }

    fn error_message() -> &'static str {
        "Error converting to Complex"
    }
}

impl PartialEq for Complex {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Complex, Fixnum, Float, Object, RString, Rational, VM};

    #[test]
    fn test_complex() {
        crate::on_ruby_thread(|| {
            let z = Complex::new(&Fixnum::new(1), &Rational::new(1, 2).unwrap()).unwrap();
            assert_eq!(z.real().try_convert_to::<Fixnum>(), Ok(Fixnum::new(1)));
            assert_eq!(
                z.imaginary().try_convert_to::<Rational>(),
                Ok(Rational::new(1, 2).unwrap())
            );

            let parsed = Complex::convert(&RString::new_utf8("3-4i")).unwrap();
            assert_eq!(parsed.abs(), 5.0);

            let from_ruby = VM::eval("Complex(0, 2)")
                .unwrap()
                .try_convert_to::<Complex>()
                .unwrap();
            assert_eq!(from_ruby.arg(), std::f64::consts::FRAC_PI_2);

            let polar = Complex::polar(1.0, std::f64::consts::PI);
            let real = polar.real().try_convert_to::<Float>().unwrap().to_f64();
            assert!((real + 1.0).abs() < 1e-12);

            assert!(Fixnum::new(1)
                .to_any_object()
                .try_convert_to::<Complex>()
                .is_err());
            assert!(Complex::new(&RString::new_utf8("x"), &Fixnum::new(1)).is_err());
        });
    }

    #[test]
    fn test_complex_from_f64() {
        crate::on_ruby_thread(|| {
            let complex = Complex::from_f64(1.5, -2.0);
            assert_eq!(
                complex.real().try_convert_to::<Float>().unwrap().to_f64(),
                1.5
            );
            assert_eq!(
                complex
                    .imaginary()
                    .try_convert_to::<Float>()
                    .unwrap()
                    .to_f64(),
                -2.0
            );
            assert_eq!(complex.inspect_object().to_str(), "(1.5-2.0i)");
        });
    }

    #[test]
    fn test_complex_arithmetic() {
        crate::on_ruby_thread(|| {
            let ruby = |code: &str| VM::eval(code).unwrap();
            let z = ruby("Complex(3, 4)").try_convert_to::<Complex>().unwrap();
            let w = ruby("Complex(1, -2)");

            assert!(z.add(&w).unwrap().equals(&ruby("Complex(4, 2)")));
            assert!(z.sub(&w).unwrap().equals(&ruby("Complex(2, 6)")));
            assert!(z.mul(&w).unwrap().equals(&ruby("Complex(11, -2)")));
            assert!(z
                .div(&w)
                .unwrap()
                .equals(&ruby("Complex(3, 4) / Complex(1, -2)")));
            assert!(z
                .pow(&Fixnum::new(2))
                .unwrap()
                .equals(&ruby("Complex(-7, 24)")));
            assert!(z
                .add(&Rational::new(1, 2).unwrap())
                .unwrap()
                .equals(&ruby("Complex(3.5r, 4)")));
            assert!(z
                .mul(&Float::new(0.5))
                .unwrap()
                .equals(&ruby("Complex(1.5, 2.0)")));
            assert!(z.neg().equals(&ruby("Complex(-3, -4)")));
            assert!(z.conjugate().equals(&ruby("Complex(3, -4)")));

            assert!(z.div(&Fixnum::new(0)).is_err());
            assert!(z.add(&crate::NilClass::new()).is_err());
            let error = z.mul(&RString::new_utf8("2")).unwrap_err();
            assert_eq!(error.class().name().unwrap().to_str(), "TypeError");

            // A class whose `coerce` takes over decides the result.
            ruby("class RutieCoerced; def coerce(other) = [1, 2]; end");
            let result = z.add(&ruby("RutieCoerced.new")).unwrap();
            assert_eq!(result.try_convert_to::<Fixnum>(), Ok(Fixnum::new(3)));

            let from_f64 = Complex::from_f64(-0.5, 2.5);
            assert!(from_f64.equals(&ruby("Complex(-0.5, 2.5)")));
        });
    }
}
