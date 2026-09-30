use std::convert::From;

use crate::{
    binding::{class, rstruct, vm},
    rubysys::rstruct::rb_cStruct,
    types::Value,
    AnyException, AnyObject, Array, Class, Exception, Fixnum, Integer, NilClass, Object, Symbol,
    VerifiedObject,
};

/// `Struct`, an instance of a class made with Ruby's `Struct.new`.
#[derive(Debug)]
#[repr(C)]
pub struct Struct {
    value: Value,
}

impl Struct {
    /// Defines an anonymous `Struct` class with the given members (Ruby's
    /// `Struct.new(:a, :b)`), or returns the error, such as the
    /// `ArgumentError` for a duplicate member.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, Struct, VM};
    /// # VM::init();
    ///
    /// let point = Struct::define(&["x", "y"]).unwrap();
    /// let instance = Struct::new_instance(&point, &[Fixnum::new(1).into(), Fixnum::new(2).into()]).unwrap();
    ///
    /// assert_eq!(instance.get("y").unwrap().try_convert_to::<Fixnum>(), Ok(Fixnum::new(2)));
    /// assert!(Struct::define(&["x", "x"]).is_err());
    /// ```
    pub fn define(members: &[&str]) -> Result<Class, AnyException> {
        let members: Vec<Value> = members
            .iter()
            .map(|name| Symbol::new(name).value())
            .collect();
        let struct_class = unsafe { rb_cStruct };

        vm::protect_value(|| vm::call_method(struct_class, "new", &members))
            .map(Class::from)
            .map_err(AnyException::from)
    }

    /// Defines a `Struct` class named `name` inside `outer` (a `Class` or
    /// `Module`), like `rb_struct_define_under`, or returns the error for an
    /// invalid name.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Module, Object, Struct, VM};
    /// # VM::init();
    ///
    /// let geometry = Module::new("Geometry");
    /// let point = Struct::define_under(&geometry, "Point", &["x", "y"]).unwrap();
    ///
    /// assert_eq!(point.path().to_str(), "Geometry::Point");
    /// assert!(Struct::define_under(&geometry, "lowercase", &["x"]).is_err());
    /// ```
    pub fn define_under<T: Object>(
        outer: &T,
        name: &str,
        members: &[&str],
    ) -> Result<Class, AnyException> {
        if !Symbol::new(name).is_const_name() {
            let message = format!("identifier {} needs to be constant", name);

            return Err(AnyException::new("NameError", Some(&message)));
        }

        let struct_class = Struct::define(members)?;

        class::const_set(outer.value(), name, struct_class.value());

        Ok(struct_class)
    }

    /// Creates an instance of the `Struct` class `klass` with `values` for
    /// its members, in order (`rb_struct_alloc`). Missing values are `nil`;
    /// too many values, or a class that is not a `Struct`, are an error.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, Struct, VM};
    /// # VM::init();
    ///
    /// let pair = Struct::define(&["left", "right"]).unwrap();
    ///
    /// let partial = Struct::new_instance(&pair, &[Fixnum::new(1).into()]).unwrap();
    /// assert!(partial.get("right").unwrap().is_nil());
    ///
    /// let values = [Fixnum::new(1).into(), Fixnum::new(2).into(), Fixnum::new(3).into()];
    /// assert!(Struct::new_instance(&pair, &values).is_err());
    /// ```
    pub fn new_instance(klass: &Class, values: &[AnyObject]) -> Result<Self, AnyException> {
        let klass = klass.value();
        let values: Array = values.iter().cloned().collect();

        let instance = vm::protect_value(|| rstruct::alloc(klass, values.value()))
            .map_err(AnyException::from)?;

        if class::is_kind_of(instance, unsafe { rb_cStruct }) {
            Ok(Struct::from(instance))
        } else {
            Err(AnyException::new("TypeError", Some("not a Struct class")))
        }
    }

    /// Returns the member names of the `Struct` class `klass` as `Symbol`s
    /// (`rb_struct_s_members`), or the `TypeError` if it is not a `Struct`
    /// class.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Object, Struct, Symbol, VM};
    /// # VM::init();
    ///
    /// let pair = Struct::define(&["left", "right"]).unwrap();
    /// let members = Struct::members_of(&pair).unwrap();
    ///
    /// assert_eq!(members.at(1).try_convert_to::<Symbol>(), Ok(Symbol::new("right")));
    /// assert!(Struct::members_of(&Class::from_existing("String")).is_err());
    /// ```
    pub fn members_of(klass: &Class) -> Result<Array, AnyException> {
        let klass = klass.value();

        vm::protect_value(|| rstruct::class_members(klass))
            .map(Array::from)
            .map_err(AnyException::from)
    }

    /// Returns the value of member `name`, or `None` if there is no such
    /// member (`rb_struct_getmember`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, Struct, VM};
    /// # VM::init();
    ///
    /// let point = VM::eval("Struct.new(:x).new(4)").unwrap().try_convert_to::<Struct>().unwrap();
    ///
    /// assert_eq!(point.get("x").unwrap().try_convert_to::<Fixnum>(), Ok(Fixnum::new(4)));
    /// assert!(point.get("z").is_none());
    /// ```
    pub fn get(&self, name: &str) -> Option<AnyObject> {
        let object = self.value();

        vm::protect_value(|| rstruct::get_member(object, name))
            .map(AnyObject::from)
            .ok()
    }

    /// Returns the value of the member at `index` (from the end when
    /// negative), or `None` when out of range (`rb_struct_aref`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, Struct, VM};
    /// # VM::init();
    ///
    /// let pair = VM::eval("Struct.new(:a, :b).new(1, 2)").unwrap().try_convert_to::<Struct>().unwrap();
    ///
    /// assert_eq!(pair.at(-1).unwrap().try_convert_to::<Fixnum>(), Ok(Fixnum::new(2)));
    /// assert!(pair.at(2).is_none());
    /// ```
    pub fn at(&self, index: i64) -> Option<AnyObject> {
        let (object, index) = (self.value(), Integer::new(index).value());

        vm::protect_value(|| rstruct::aref(object, index))
            .map(AnyObject::from)
            .ok()
    }

    /// Sets member `name` to `value` (`rb_struct_aset`), or returns the
    /// error: a `NameError` for an unknown member, a `FrozenError` for a
    /// frozen struct.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, Struct, VM};
    /// # VM::init();
    ///
    /// let mut point = VM::eval("Struct.new(:x).new(1)").unwrap().try_convert_to::<Struct>().unwrap();
    ///
    /// point.set("x", Fixnum::new(9)).unwrap();
    ///
    /// assert_eq!(point.get("x").unwrap().try_convert_to::<Fixnum>(), Ok(Fixnum::new(9)));
    /// assert!(point.set("y", Fixnum::new(1)).is_err());
    /// ```
    pub fn set<T: Object>(&mut self, name: &str, value: T) -> Result<(), AnyException> {
        let (object, name, value) = (self.value(), Symbol::new(name).value(), value.value());

        vm::protect_value(|| rstruct::aset(object, name, value))
            .map(|_| ())
            .map_err(AnyException::from)
    }

    /// Returns the member names as `Symbol`s (`rb_struct_members`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, Struct, Symbol, VM};
    /// # VM::init();
    ///
    /// let pair = VM::eval("Struct.new(:a, :b).new").unwrap().try_convert_to::<Struct>().unwrap();
    ///
    /// assert_eq!(pair.members().at(0).try_convert_to::<Symbol>(), Ok(Symbol::new("a")));
    /// ```
    pub fn members(&self) -> Array {
        Array::from(rstruct::members(self.value()))
    }

    /// Returns the number of members (`rb_struct_size`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, Struct, VM};
    /// # VM::init();
    ///
    /// let pair = VM::eval("Struct.new(:a, :b).new").unwrap().try_convert_to::<Struct>().unwrap();
    ///
    /// assert_eq!(pair.size(), 2);
    /// ```
    pub fn size(&self) -> usize {
        Fixnum::from(rstruct::size(self.value())).to_i64() as usize
    }
}

impl From<Value> for Struct {
    fn from(value: Value) -> Self {
        Struct { value }
    }
}

impl Into<Value> for Struct {
    fn into(self) -> Value {
        self.value
    }
}

impl Into<AnyObject> for Struct {
    fn into(self) -> AnyObject {
        AnyObject::from(self.value)
    }
}

impl Object for Struct {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for Struct {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        class::is_kind_of(object.value(), unsafe { rb_cStruct })
    }

    fn error_message() -> &'static str {
        "Error converting to Struct"
    }
}

impl PartialEq for Struct {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Class, Fixnum, Module, Object, Struct, Symbol, GC, VM};

    #[test]
    fn test_struct() {
        crate::on_ruby_thread(|| {
            let point = Struct::define(&["x", "y"]).unwrap();
            let mut instance =
                Struct::new_instance(&point, &[Fixnum::new(1).into(), Fixnum::new(2).into()])
                    .unwrap();
            GC::start();

            assert_eq!(instance.size(), 2);
            assert_eq!(
                instance.at(0).unwrap().try_convert_to::<Fixnum>(),
                Ok(Fixnum::new(1))
            );
            assert!(instance.at(-3).is_none());
            assert_eq!(
                instance.members().at(1).try_convert_to::<Symbol>(),
                Ok(Symbol::new("y"))
            );

            instance.set("x", Fixnum::new(10)).unwrap();
            let sum = unsafe { instance.send("x", &[]) };
            assert_eq!(sum.try_convert_to::<Fixnum>(), Ok(Fixnum::new(10)));

            let mut frozen = instance.dup().freeze();
            assert!(frozen.set("x", Fixnum::new(0)).is_err());

            let outer = Module::new("RutieStructTest");
            let named = Struct::define_under(&outer, "Pair", &["a"]).unwrap();
            assert_eq!(named.name().unwrap().to_str(), "RutieStructTest::Pair");
            let from_ruby = VM::eval("RutieStructTest::Pair.new(5)")
                .unwrap()
                .try_convert_to::<Struct>()
                .unwrap();
            assert_eq!(
                from_ruby.get("a").unwrap().try_convert_to::<Fixnum>(),
                Ok(Fixnum::new(5))
            );

            assert!(Struct::new_instance(&Class::from_existing("Object"), &[]).is_err());
            assert!(Struct::define(&["x", "x"]).is_err());
            assert!(VM::eval("1..2")
                .unwrap()
                .try_convert_to::<Struct>()
                .is_err());
        });
    }

    #[test]
    fn test_struct_members_of() {
        crate::on_ruby_thread(|| {
            let point = Struct::define(&["x", "y"]).unwrap();
            GC::start();

            let members = Struct::members_of(&point).unwrap();
            let names: Vec<String> = members
                .into_iter()
                .map(|member| member.try_convert_to::<Symbol>().unwrap().to_string())
                .collect();
            assert_eq!(names, vec!["x", "y"]);

            // A class that is not a Struct has no members.
            assert!(Struct::members_of(&Class::object()).is_err());
        });
    }
}
