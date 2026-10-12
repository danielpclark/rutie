//! Converting arguments from Ruby and results to Ruby.
//!
//! None of these conversions call back into Ruby code, so they can't raise:
//! a failed conversion is an `Err` that the generated method raises after
//! dropping what it holds.

use std::{
    collections::{BTreeMap, HashMap},
    ffi::CStr,
    hash::{BuildHasher, Hash as StdHash},
};

use crate::{
    rubysys::class::rb_obj_classname, types::ValueType, AnyException, AnyObject, Array, Boolean,
    Class, Encoding, Exception, Float, Hash, Integer, NilClass, Object, RString, Symbol,
    VerifiedObject,
};

/// The class name of `object`, for error messages (`rb_obj_classname`,
/// which calls no Ruby methods).
pub(crate) fn class_name(object: &AnyObject) -> String {
    match object.ty() {
        ValueType::Nil => return "nil".to_string(),
        ValueType::True => return "true".to_string(),
        ValueType::False => return "false".to_string(),
        _ => {}
    }

    unsafe {
        let name = rb_obj_classname(object.value());

        if name.is_null() {
            "object".to_string()
        } else {
            CStr::from_ptr(name).to_string_lossy().into_owned()
        }
    }
}

fn no_conversion(object: &AnyObject, into: &str) -> AnyException {
    let message = format!(
        "no implicit conversion of {} into {}",
        class_name(object),
        into
    );

    AnyException::new("TypeError", Some(&message))
}

/// A parameter type of a `#[rubymethods]`, `#[rubymodule]` or
/// `#[rubyfunction]` method.
///
/// The argument is first converted to a `Holder` that lives for the call,
/// then the method gets `&Self` (or `Self`, when `Holder` is `Self`) from it:
/// a Ruby type is its own holder, a Rust value is converted (copied) into
/// one, and a `#[rubyclass]` struct is borrowed from the Ruby object.
#[diagnostic::on_unimplemented(
    message = "`{Self}` can't be a parameter of a Ruby method",
    label = "not a Ruby type, a supported Rust type or a `#[rubyclass]` struct",
    note = "see `rutie::derive::FromArgument` for the supported types"
)]
pub trait FromArgument {
    /// What the converted argument is kept in during the call.
    type Holder;

    /// Converts `argument`, or returns the exception to raise.
    fn hold(argument: AnyObject) -> Result<Self::Holder, AnyException>;

    /// The parameter value inside `holder`.
    fn borrow(holder: &Self::Holder) -> &Self;
}

/// A parameter type that can be borrowed mutably (`&mut T`): `#[rubyclass]`
/// structs.
#[diagnostic::on_unimplemented(
    message = "`{Self}` can't be a `&mut` parameter of a Ruby method",
    label = "only `#[rubyclass]` structs can be borrowed mutably",
    note = "take `&T` or `T` instead"
)]
pub trait FromArgumentMut {
    /// What the mutably borrowed argument is kept in during the call.
    type HolderMut;

    /// Converts and borrows `argument` mutably.
    fn hold_mut(argument: AnyObject) -> Result<Self::HolderMut, AnyException>;

    /// The parameter value inside `holder`.
    fn borrow_mut(holder: &mut Self::HolderMut) -> &mut Self;
}

impl<T: Object + VerifiedObject> FromArgument for T {
    type Holder = T;

    fn hold(argument: AnyObject) -> Result<T, AnyException> {
        argument.try_convert_to::<T>()
    }

    fn borrow(holder: &T) -> &T {
        holder
    }
}

macro_rules! integer_argument {
    ($($ty: ty),* $(,)?) => {
        $(
            impl FromArgument for $ty {
                type Holder = $ty;

                fn hold(argument: AnyObject) -> Result<$ty, AnyException> {
                    let integer = match argument.ty() {
                        ValueType::Fixnum | ValueType::Bignum => Integer::from(argument.value()),
                        _ => return Err(no_conversion(&argument, "Integer")),
                    };

                    integer
                        .to_i128()
                        .and_then(|value| <$ty>::try_from(value).ok())
                        .ok_or_else(|| {
                            let message = format!(
                                "integer {} out of range of {}",
                                integer.to_s_radix(10).to_string_unchecked(),
                                stringify!($ty)
                            );

                            AnyException::new("RangeError", Some(&message))
                        })
                }

                fn borrow(holder: &$ty) -> &$ty {
                    holder
                }
            }
        )*
    };
}

integer_argument!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, usize);

impl FromArgument for u128 {
    type Holder = u128;

    fn hold(argument: AnyObject) -> Result<u128, AnyException> {
        match argument.ty() {
            ValueType::Fixnum | ValueType::Bignum => {
                Integer::from(argument.value()).to_u128().ok_or_else(|| {
                    AnyException::new("RangeError", Some("integer out of range of u128"))
                })
            }
            _ => Err(no_conversion(&argument, "Integer")),
        }
    }

    fn borrow(holder: &u128) -> &u128 {
        holder
    }
}

macro_rules! float_argument {
    ($($ty: ty),*) => {
        $(
            impl FromArgument for $ty {
                type Holder = $ty;

                // Like Ruby's `Float()` for numbers: an Integer is accepted.
                fn hold(argument: AnyObject) -> Result<$ty, AnyException> {
                    match argument.ty() {
                        ValueType::Float => Ok(Float::from(argument.value()).to_f64() as $ty),
                        ValueType::Fixnum | ValueType::Bignum => {
                            Ok(Integer::from(argument.value()).to_f64() as $ty)
                        }
                        _ => Err(no_conversion(&argument, "Float")),
                    }
                }

                fn borrow(holder: &$ty) -> &$ty {
                    holder
                }
            }
        )*
    };
}

float_argument!(f32, f64);

impl FromArgument for bool {
    type Holder = bool;

    fn hold(argument: AnyObject) -> Result<bool, AnyException> {
        match argument.ty() {
            ValueType::True => Ok(true),
            ValueType::False => Ok(false),
            _ => Err(no_conversion(&argument, "true or false")),
        }
    }

    fn borrow(holder: &bool) -> &bool {
        holder
    }
}

fn string_bytes(argument: &AnyObject) -> Result<Vec<u8>, AnyException> {
    if argument.ty() == ValueType::RString {
        Ok(RString::from(argument.value())
            .to_bytes_unchecked()
            .to_vec())
    } else {
        Err(no_conversion(argument, "String"))
    }
}

fn utf8(bytes: Vec<u8>) -> Result<String, AnyException> {
    String::from_utf8(bytes)
        .map_err(|_| AnyException::new("ArgumentError", Some("invalid byte sequence in UTF-8")))
}

/// A `String` parameter is copied out of a Ruby `String`, which must hold
/// valid UTF-8.
impl FromArgument for String {
    type Holder = String;

    fn hold(argument: AnyObject) -> Result<String, AnyException> {
        utf8(string_bytes(&argument)?)
    }

    fn borrow(holder: &String) -> &String {
        holder
    }
}

/// A `&str` parameter borrows a copy of the Ruby `String`, so Ruby code the
/// method calls can't change it underneath.
impl FromArgument for str {
    type Holder = String;

    fn hold(argument: AnyObject) -> Result<String, AnyException> {
        utf8(string_bytes(&argument)?)
    }

    fn borrow(holder: &String) -> &str {
        holder
    }
}

/// A `&[u8]` parameter is a copy of a Ruby `String`'s bytes (a `Vec<u8>`
/// parameter is an `Array` of `Integer`s).
impl FromArgument for [u8] {
    type Holder = Vec<u8>;

    fn hold(argument: AnyObject) -> Result<Vec<u8>, AnyException> {
        string_bytes(&argument)
    }

    fn borrow(holder: &Vec<u8>) -> &[u8] {
        holder
    }
}

impl FromArgument for char {
    type Holder = char;

    fn hold(argument: AnyObject) -> Result<char, AnyException> {
        let string = utf8(string_bytes(&argument)?)?;
        let mut chars = string.chars();

        match (chars.next(), chars.next()) {
            (Some(character), None) => Ok(character),
            _ => Err(AnyException::new(
                "ArgumentError",
                Some("expected a String of one character"),
            )),
        }
    }

    fn borrow(holder: &char) -> &char {
        holder
    }
}

/// `nil` is `None`; anything else converts to `T`. (An `Option` parameter
/// at the top level is an optional argument instead.)
impl<T: FromArgument<Holder = T>> FromArgument for Option<T> {
    type Holder = Option<T>;

    fn hold(argument: AnyObject) -> Result<Option<T>, AnyException> {
        if argument.is_nil() {
            Ok(None)
        } else {
            T::hold(argument).map(Some)
        }
    }

    fn borrow(holder: &Option<T>) -> &Option<T> {
        holder
    }
}

fn array_elements(argument: &AnyObject) -> Result<Vec<AnyObject>, AnyException> {
    if argument.ty() != ValueType::Array {
        return Err(no_conversion(argument, "Array"));
    }

    let array = Array::from(argument.value());

    Ok((0..array.length())
        .map(|index| array.at(index as i64))
        .collect())
}

/// A `Vec<T>` parameter is converted from an `Array`, element by element.
impl<T: FromArgument<Holder = T>> FromArgument for Vec<T> {
    type Holder = Vec<T>;

    fn hold(argument: AnyObject) -> Result<Vec<T>, AnyException> {
        array_elements(&argument)?
            .into_iter()
            .map(T::hold)
            .collect()
    }

    fn borrow(holder: &Vec<T>) -> &Vec<T> {
        holder
    }
}

fn hash_pairs(argument: &AnyObject) -> Result<Vec<(AnyObject, AnyObject)>, AnyException> {
    if argument.ty() != ValueType::Hash {
        return Err(no_conversion(argument, "Hash"));
    }

    let mut pairs = Vec::new();
    Hash::from(argument.value()).each(|key, value| pairs.push((key, value)));

    Ok(pairs)
}

/// A `HashMap<K, V>` parameter is converted from a `Hash`.
impl<K, V, S> FromArgument for HashMap<K, V, S>
where
    K: FromArgument<Holder = K> + Eq + StdHash,
    V: FromArgument<Holder = V>,
    S: BuildHasher + Default,
{
    type Holder = HashMap<K, V, S>;

    fn hold(argument: AnyObject) -> Result<Self, AnyException> {
        hash_pairs(&argument)?
            .into_iter()
            .map(|(key, value)| Ok((K::hold(key)?, V::hold(value)?)))
            .collect()
    }

    fn borrow(holder: &Self) -> &Self {
        holder
    }
}

impl<K: FromArgument<Holder = K> + Ord, V: FromArgument<Holder = V>> FromArgument
    for BTreeMap<K, V>
{
    type Holder = BTreeMap<K, V>;

    fn hold(argument: AnyObject) -> Result<Self, AnyException> {
        hash_pairs(&argument)?
            .into_iter()
            .map(|(key, value)| Ok((K::hold(key)?, V::hold(value)?)))
            .collect()
    }

    fn borrow(holder: &Self) -> &Self {
        holder
    }
}

/// The trailing arguments of a `#[ruby(splat)]` parameter (Ruby's `*rest`).
pub trait FromSplat: Sized {
    /// Converts the arguments after the positional ones.
    fn from_splat(arguments: &[AnyObject]) -> Result<Self, AnyException>;
}

impl FromSplat for Array {
    fn from_splat(arguments: &[AnyObject]) -> Result<Array, AnyException> {
        Ok(arguments.iter().cloned().collect())
    }
}

impl<T: FromArgument<Holder = T>> FromSplat for Vec<T> {
    fn from_splat(arguments: &[AnyObject]) -> Result<Vec<T>, AnyException> {
        arguments.iter().cloned().map(T::hold).collect()
    }
}

/// A Rust value a method can return to Ruby.
#[diagnostic::on_unimplemented(
    message = "`{Self}` can't be returned to Ruby",
    label = "not a Ruby type, a supported Rust type or a `#[rubyclass]` struct",
    note = "see `rutie::derive::IntoRubyObject` for the supported types"
)]
pub trait IntoRubyObject {
    /// The Ruby object for `self`. `receiver` is the method's `self` (a class
    /// for a singleton method), so a `#[rubyclass]` value returned by a
    /// subclass's singleton method becomes an object of that subclass.
    fn into_ruby_object(self, receiver: &AnyObject) -> Result<AnyObject, AnyException>;
}

impl<T: Object> IntoRubyObject for T {
    fn into_ruby_object(self, _receiver: &AnyObject) -> Result<AnyObject, AnyException> {
        Ok(self.to_any_object())
    }
}

macro_rules! into_ruby_object {
    ($($ty: ty => |$value: ident| $object: expr;)*) => {
        $(
            impl IntoRubyObject for $ty {
                fn into_ruby_object(self, _receiver: &AnyObject) -> Result<AnyObject, AnyException> {
                    let $value = self;

                    Ok($object.to_any_object())
                }
            }
        )*
    };
}

into_ruby_object! {
    () => |_value| NilClass::new();
    bool => |value| Boolean::new(value);
    i8 => |value| Integer::new(i64::from(value));
    i16 => |value| Integer::new(i64::from(value));
    i32 => |value| Integer::from(value);
    i64 => |value| Integer::new(value);
    i128 => |value| Integer::from(value);
    isize => |value| Integer::new(value as i64);
    u8 => |value| Integer::new(i64::from(value));
    u16 => |value| Integer::new(i64::from(value));
    u32 => |value| Integer::from(value);
    u64 => |value| Integer::from(value);
    u128 => |value| Integer::from(value);
    usize => |value| Integer::from(value as u64);
    f32 => |value| Float::new(f64::from(value));
    f64 => |value| Float::new(value);
    char => |value| RString::new_utf8(value.encode_utf8(&mut [0; 4]));
    String => |value| RString::new_utf8(&value);
    &str => |value| RString::new_utf8(value);
    Bytes => |value| RString::from_bytes(&value.0, &Encoding::ascii_8bit());
}

/// Binary data, converted to and from a Ruby `String` (ASCII-8BIT when
/// returned). A `Vec<u8>` is an `Array` of `Integer`s instead.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Bytes(pub Vec<u8>);

impl FromArgument for Bytes {
    type Holder = Bytes;

    fn hold(argument: AnyObject) -> Result<Bytes, AnyException> {
        string_bytes(&argument).map(Bytes)
    }

    fn borrow(holder: &Bytes) -> &Bytes {
        holder
    }
}

impl<T: IntoRubyObject> IntoRubyObject for Option<T> {
    fn into_ruby_object(self, receiver: &AnyObject) -> Result<AnyObject, AnyException> {
        match self {
            Some(value) => value.into_ruby_object(receiver),
            None => Ok(NilClass::new().to_any_object()),
        }
    }
}

fn collect_array<T: IntoRubyObject>(
    values: impl IntoIterator<Item = T>,
    receiver: &AnyObject,
) -> Result<AnyObject, AnyException> {
    let mut array = Array::new();

    for value in values {
        array.push(value.into_ruby_object(receiver)?);
    }

    Ok(array.to_any_object())
}

/// A `Vec` becomes an `Array` (use [`Bytes`] for a binary `String`).
impl<T: IntoRubyObject> IntoRubyObject for Vec<T> {
    fn into_ruby_object(self, receiver: &AnyObject) -> Result<AnyObject, AnyException> {
        collect_array(self, receiver)
    }
}

impl<K: IntoRubyObject, V: IntoRubyObject, S> IntoRubyObject for HashMap<K, V, S> {
    fn into_ruby_object(self, receiver: &AnyObject) -> Result<AnyObject, AnyException> {
        let mut hash = Hash::new();

        for (key, value) in self {
            hash.store(
                key.into_ruby_object(receiver)?,
                value.into_ruby_object(receiver)?,
            );
        }

        Ok(hash.to_any_object())
    }
}

impl<K: IntoRubyObject, V: IntoRubyObject> IntoRubyObject for BTreeMap<K, V> {
    fn into_ruby_object(self, receiver: &AnyObject) -> Result<AnyObject, AnyException> {
        let mut hash = Hash::new();

        for (key, value) in self {
            hash.store(
                key.into_ruby_object(receiver)?,
                value.into_ruby_object(receiver)?,
            );
        }

        Ok(hash.to_any_object())
    }
}

macro_rules! into_ruby_tuple {
    ($($name: ident . $index: tt),+) => {
        /// A tuple becomes an `Array` (Ruby's multiple return values).
        impl<$($name: IntoRubyObject),+> IntoRubyObject for ($($name,)+) {
            fn into_ruby_object(self, receiver: &AnyObject) -> Result<AnyObject, AnyException> {
                let mut array = Array::new();
                $(array.push(self.$index.into_ruby_object(receiver)?);)+

                Ok(array.to_any_object())
            }
        }
    };
}

into_ruby_tuple!(A.0, B.1);
into_ruby_tuple!(A.0, B.1, C.2);
into_ruby_tuple!(A.0, B.1, C.2, D.3);

/// An error a method can return in `Err`, raised as a Ruby exception.
///
/// `#[rubyerror]` implements it for your error types, mapping them (or each
/// enum variant) to a Ruby exception class.
#[diagnostic::on_unimplemented(
    message = "`{Self}` can't be raised as a Ruby exception",
    label = "returned in `Err` from a Ruby method",
    note = "use `AnyException`, `String`, `std::io::Error`, or mark the type `#[rubyerror]`"
)]
pub trait IntoException {
    /// The exception to raise.
    fn into_exception(self) -> AnyException;
}

impl IntoException for AnyException {
    fn into_exception(self) -> AnyException {
        self
    }
}

/// A message is raised as a `RuntimeError`.
impl IntoException for String {
    fn into_exception(self) -> AnyException {
        AnyException::new("RuntimeError", Some(&self))
    }
}

impl IntoException for &str {
    fn into_exception(self) -> AnyException {
        AnyException::new("RuntimeError", Some(self))
    }
}

/// An I/O error is raised as the matching `Errno::*` (`IOError` without an
/// OS error code).
impl IntoException for std::io::Error {
    fn into_exception(self) -> AnyException {
        AnyException::from_io_error(&self, "")
    }
}

/// Any other error is raised as a `RuntimeError` with its `Display` message.
impl IntoException for Box<dyn std::error::Error + Send + Sync> {
    fn into_exception(self) -> AnyException {
        AnyException::new("RuntimeError", Some(&self.to_string()))
    }
}

impl IntoException for Box<dyn std::error::Error> {
    fn into_exception(self) -> AnyException {
        AnyException::new("RuntimeError", Some(&self.to_string()))
    }
}

macro_rules! display_exception {
    ($($ty: ty => $class: literal),* $(,)?) => {
        $(
            impl IntoException for $ty {
                fn into_exception(self) -> AnyException {
                    AnyException::new($class, Some(&self.to_string()))
                }
            }
        )*
    };
}

display_exception!(
    std::num::ParseIntError => "ArgumentError",
    std::num::ParseFloatError => "ArgumentError",
    std::str::Utf8Error => "ArgumentError",
    std::string::FromUtf8Error => "ArgumentError",
    std::fmt::Error => "RuntimeError",
);

/// A method's return value: an [`IntoRubyObject`], or a `Result` of one
/// whose `Err` is raised.
#[diagnostic::on_unimplemented(
    message = "`{Self}` can't be returned from a Ruby method",
    note = "return an `IntoRubyObject` type, or a `Result` of one whose error is `IntoException`"
)]
pub trait MethodOutput {
    #[doc(hidden)]
    fn into_output(self, receiver: &AnyObject) -> Result<AnyObject, AnyException>;
}

impl<T: IntoRubyObject> MethodOutput for T {
    fn into_output(self, receiver: &AnyObject) -> Result<AnyObject, AnyException> {
        self.into_ruby_object(receiver)
    }
}

impl<T: IntoRubyObject, E: IntoException> MethodOutput for Result<T, E> {
    fn into_output(self, receiver: &AnyObject) -> Result<AnyObject, AnyException> {
        self.map_err(IntoException::into_exception)?
            .into_ruby_object(receiver)
    }
}

/// What a constructor (`new`, or `#[ruby(constructor)]`) returns: the
/// struct, or a `Result` of it.
pub trait ConstructorOutput<T> {
    #[doc(hidden)]
    fn into_value(self) -> Result<T, AnyException>;
}

impl<T> ConstructorOutput<T> for T {
    fn into_value(self) -> Result<T, AnyException> {
        Ok(self)
    }
}

impl<T, E: IntoException> ConstructorOutput<T> for Result<T, E> {
    fn into_value(self) -> Result<T, AnyException> {
        self.map_err(IntoException::into_exception)
    }
}

/// Values a `#[ruby(without_gvl)]` method may take and return: plain Rust
/// data with no Ruby objects in it, since Ruby can't be used while the GVL
/// is released.
///
/// # Safety
///
/// Implement it only for types that hold no Ruby objects.
#[diagnostic::on_unimplemented(
    message = "`{Self}` can't be used while the GVL is released",
    label = "`#[ruby(without_gvl)]` methods take and return plain Rust data",
    note = "Ruby objects can't be touched without the GVL; convert to a Rust type (`&str`, `i64`, `Vec<T>`, …)"
)]
pub unsafe trait WithoutGvl {}

macro_rules! without_gvl {
    ($($ty: ty),* $(,)?) => {
        $(unsafe impl WithoutGvl for $ty {})*
    };
}

without_gvl!(
    (),
    bool,
    char,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    f32,
    f64,
    String,
    str,
    std::path::PathBuf,
    std::time::Duration,
);

unsafe impl<T: WithoutGvl> WithoutGvl for [T] {}
unsafe impl<T: WithoutGvl, const N: usize> WithoutGvl for [T; N] {}
unsafe impl<T: WithoutGvl> WithoutGvl for Vec<T> {}
unsafe impl<T: WithoutGvl> WithoutGvl for Option<T> {}
unsafe impl<T: WithoutGvl + ?Sized> WithoutGvl for Box<T> {}
unsafe impl<T: WithoutGvl, E: WithoutGvl> WithoutGvl for Result<T, E> {}
unsafe impl<K: WithoutGvl, V: WithoutGvl, S> WithoutGvl for HashMap<K, V, S> {}
unsafe impl<K: WithoutGvl, V: WithoutGvl> WithoutGvl for BTreeMap<K, V> {}
unsafe impl<A: WithoutGvl, B: WithoutGvl> WithoutGvl for (A, B) {}
unsafe impl<A: WithoutGvl, B: WithoutGvl, C: WithoutGvl> WithoutGvl for (A, B, C) {}
unsafe impl WithoutGvl for Bytes {}
unsafe impl WithoutGvl for std::io::Error {}
unsafe impl WithoutGvl for std::num::ParseIntError {}
unsafe impl WithoutGvl for std::num::ParseFloatError {}

#[doc(hidden)]
pub fn assert_without_gvl<T: WithoutGvl + ?Sized>() {}

#[doc(hidden)]
pub fn assert_sync<T: Sync + ?Sized>() {}

#[doc(hidden)]
pub fn class_of(object: &AnyObject) -> Class {
    object.class()
}
