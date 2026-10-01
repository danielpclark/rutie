use std::convert::From;

use crate::{
    binding::{rproc, vm},
    types::Value,
    util, AnyException, AnyObject, Fixnum, Hash, Object, Proc, VerifiedObject,
};

/// `Method`, a method bound to its receiver, as returned by
/// [`Object::method`](trait.Object.html#method.method).
#[derive(Debug)]
#[repr(C)]
pub struct Method {
    value: Value,
}

impl Method {
    /// Calls the method with `arguments` (`rb_method_call`).
    ///
    /// Raises whatever the method raises; see
    /// [`protect_call`](#method.protect_call).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let plus = Fixnum::new(40).method("+").unwrap();
    ///
    /// assert_eq!(plus.call(&[Fixnum::new(2).into()]).try_convert_to::<Fixnum>(), Ok(Fixnum::new(42)));
    /// ```
    pub fn call(&self, arguments: &[AnyObject]) -> AnyObject {
        let arguments = util::arguments_to_values(arguments);

        AnyObject::from(rproc::method_call(self.value(), &arguments))
    }

    /// Like [`call`](#method.call), but returns the exception raised by the
    /// method instead of propagating it.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let divide = Fixnum::new(1).method("/").unwrap();
    /// let error = divide.protect_call(&[Fixnum::new(0).into()]).unwrap_err();
    ///
    /// assert!(Class::from_existing("ZeroDivisionError").case_equals(&error));
    /// ```
    pub fn protect_call(&self, arguments: &[AnyObject]) -> Result<AnyObject, AnyException> {
        let method = self.value();
        let arguments = util::arguments_to_values(arguments);

        vm::protect_value(|| rproc::method_call(method, &arguments))
            .map(AnyObject::from)
            .map_err(AnyException::from)
    }

    /// Calls the method with `arguments` and `keywords` as keyword
    /// arguments (`rb_method_call_kw`), returning the result or the
    /// exception raised.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Hash, Object, Symbol, VM};
    /// # VM::init();
    ///
    /// let calculator = VM::eval("o = Object.new; def o.power(base, exponent: 2); base ** exponent; end; o").unwrap();
    /// let power = calculator.method("power").unwrap();
    ///
    /// let mut keywords = Hash::new();
    /// keywords.store(Symbol::new("exponent"), Fixnum::new(3));
    ///
    /// let result = power.call_with_keywords(&[Fixnum::new(2).into()], keywords).unwrap();
    ///
    /// assert_eq!(result.try_convert_to::<Fixnum>(), Ok(Fixnum::new(8)));
    /// ```
    pub fn call_with_keywords(
        &self,
        arguments: &[AnyObject],
        keywords: Hash,
    ) -> Result<AnyObject, AnyException> {
        let method = self.value();
        let mut arguments = util::arguments_to_values(arguments);
        arguments.push(keywords.value());

        vm::protect_value(|| rproc::method_call_with_keywords(method, &arguments))
            .map(AnyObject::from)
            .map_err(AnyException::from)
    }

    /// Calls the method with `arguments`, passing `block` as its block
    /// (`rb_method_call_with_block_kw`), returning the result or the
    /// exception raised.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, Object, Proc, VM};
    /// # VM::init();
    ///
    /// let numbers = VM::eval("[1, 2, 3]").unwrap();
    /// let map = numbers.method("map").unwrap();
    /// let square = VM::eval("proc { |x| x * x }").unwrap().try_convert_to::<Proc>().unwrap();
    ///
    /// let squares = map.call_with_block(&[], &square).unwrap().try_convert_to::<Array>().unwrap();
    ///
    /// assert_eq!(squares.at(2).try_convert_to::<Fixnum>(), Ok(Fixnum::new(9)));
    /// ```
    pub fn call_with_block(
        &self,
        arguments: &[AnyObject],
        block: &Proc,
    ) -> Result<AnyObject, AnyException> {
        let (method, block) = (self.value(), block.value());
        let arguments = util::arguments_to_values(arguments);

        vm::protect_value(|| rproc::method_call_with_block(method, &arguments, block, false))
            .map(AnyObject::from)
            .map_err(AnyException::from)
    }

    /// Returns the method's arity, like [`Proc::arity`](struct.Proc.html#method.arity).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, RString, VM};
    /// # VM::init();
    ///
    /// assert_eq!(RString::new_utf8("").method("upcase").unwrap().arity(), -1);
    /// assert_eq!(RString::new_utf8("").method("start_with?").unwrap().arity(), -1);
    /// assert_eq!(RString::new_utf8("").method("length").unwrap().arity(), 0);
    /// ```
    pub fn arity(&self) -> i32 {
        Fixnum::from(vm::call_method(self.value(), "arity", &[])).to_i64() as i32
    }

    /// Returns the method as a `Proc` (Ruby's `to_proc`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let to_s = Fixnum::new(42).method("to_s").unwrap().to_proc();
    ///
    /// assert!(to_s.is_lambda());
    /// ```
    pub fn to_proc(&self) -> Proc {
        Proc::from(vm::call_method(self.value(), "to_proc", &[]))
    }
}

impl From<Value> for Method {
    fn from(value: Value) -> Self {
        Method { value }
    }
}

impl Into<Value> for Method {
    fn into(self) -> Value {
        self.value
    }
}

impl Into<AnyObject> for Method {
    fn into(self) -> AnyObject {
        AnyObject::from(self.value)
    }
}

impl Object for Method {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for Method {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        util::is_method(object.value())
    }

    fn error_message() -> &'static str {
        "Error converting to Method"
    }
}

impl PartialEq for Method {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Array, Class, Fixnum, Hash, Method, Object, Proc, RString, VM};

    #[test]
    fn test_method() {
        crate::on_ruby_thread(|| {
            let array = Array::new().push(Fixnum::new(1));
            let push = array.method("push").unwrap();

            push.call(&[Fixnum::new(2).into()]);
            assert_eq!(array.length(), 2);
            assert_eq!(push.arity(), -1);
            assert_eq!(array.method_arity("push"), -1);
            assert_eq!(array.method_arity("no_such_method"), 0);

            let frozen = Array::new().freeze();
            let error = frozen
                .method("push")
                .unwrap()
                .protect_call(&[Fixnum::new(1).into()])
                .unwrap_err();
            assert!(Class::from_existing("FrozenError").case_equals(&error));

            let upcase = RString::new_utf8("abc").method("upcase").unwrap().to_proc();
            let result = upcase.call(&[]);
            assert_eq!(result.try_convert_to::<RString>().unwrap().to_str(), "ABC");

            assert!(VM::eval("1.method(:+)")
                .unwrap()
                .try_convert_to::<Method>()
                .is_ok());
            assert!(VM::eval("proc {}")
                .unwrap()
                .try_convert_to::<Method>()
                .is_err());
            assert_eq!(
                Class::from_existing("Array").instance_method_arity("push"),
                -1
            );
        });
    }

    #[test]
    fn test_method_calls_with_keywords_and_blocks() {
        crate::on_ruby_thread(|| {
            let object = VM::eval(
                "o = Object.new
                 def o.collect(*args, **kw, &block); [args, kw, block && block.call]; end
                 o",
            )
            .unwrap();
            let collect = object.method("collect").unwrap();

            let keywords = VM::eval("{ b: 2 }")
                .unwrap()
                .try_convert_to::<Hash>()
                .unwrap();
            let result = collect
                .call_with_keywords(&[Fixnum::new(1).into()], keywords)
                .unwrap();
            assert_eq!(result.inspect_object().to_str(), "[[1], {b: 2}, nil]");

            let block = VM::eval("proc { :blocked }")
                .unwrap()
                .try_convert_to::<Proc>()
                .unwrap();
            let result = collect.call_with_block(&[], &block).unwrap();
            assert_eq!(result.inspect_object().to_str(), "[[], {}, :blocked]");

            let strict = VM::eval("o = Object.new; def o.strict(a); a; end; o")
                .unwrap()
                .method("strict")
                .unwrap();
            assert!(strict.call_with_keywords(&[], Hash::new()).is_err());
            assert!(strict.call_with_block(&[], &block).is_err());
        });
    }
}
