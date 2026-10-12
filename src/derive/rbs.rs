//! RBS signatures for `#[rubyclass]`, `#[rubymodule]` and `#[rubyerror]`
//! types, for Ruby type checkers (Steep, Sorbet via RBS) and editors.

use std::{
    collections::{BTreeMap, HashMap},
    marker::PhantomData,
};

use crate::{
    AnyException, AnyObject, Array, Boolean, Class, Fixnum, Float, Hash, Integer, Module, NilClass,
    Proc, RString, Symbol,
};

use super::convert::Bytes;

/// The RBS type of a Rust type used in a method signature. Types without an
/// implementation are `untyped`.
pub trait RbsType {
    /// The type, such as `Integer` or `Array[String]?`.
    fn rbs() -> String;
}

macro_rules! rbs {
    ($($ty: ty => $name: literal),* $(,)?) => {
        $(
            impl RbsType for $ty {
                fn rbs() -> String {
                    $name.to_string()
                }
            }
        )*
    };
}

rbs!(
    () => "nil",
    bool => "bool",
    char => "String",
    i8 => "Integer", i16 => "Integer", i32 => "Integer", i64 => "Integer", i128 => "Integer",
    isize => "Integer", u8 => "Integer", u16 => "Integer", u32 => "Integer", u64 => "Integer",
    u128 => "Integer", usize => "Integer",
    f32 => "Float", f64 => "Float",
    String => "String", str => "String", [u8] => "String", Bytes => "String",
    AnyObject => "untyped", AnyException => "Exception", Array => "Array[untyped]",
    Boolean => "bool", Class => "Class", Fixnum => "Integer", Float => "Float",
    Hash => "Hash[untyped, untyped]", Integer => "Integer", Module => "Module", NilClass => "nil",
    Proc => "Proc", RString => "String", Symbol => "Symbol",
);

impl<T: RbsType + ?Sized> RbsType for &T {
    fn rbs() -> String {
        T::rbs()
    }
}

impl<T: RbsType + ?Sized> RbsType for Box<T> {
    fn rbs() -> String {
        T::rbs()
    }
}

impl<T: RbsType> RbsType for Option<T> {
    fn rbs() -> String {
        let inner = T::rbs();

        if inner == "nil" || inner == "untyped" || inner.ends_with('?') {
            inner
        } else if inner.contains(' ') {
            format!("({})?", inner)
        } else {
            format!("{}?", inner)
        }
    }
}

impl<T: RbsType, E> RbsType for Result<T, E> {
    fn rbs() -> String {
        T::rbs()
    }
}

impl<T: RbsType> RbsType for Vec<T> {
    fn rbs() -> String {
        format!("Array[{}]", T::rbs())
    }
}

impl<K: RbsType, V: RbsType, S> RbsType for HashMap<K, V, S> {
    fn rbs() -> String {
        format!("Hash[{}, {}]", K::rbs(), V::rbs())
    }
}

impl<K: RbsType, V: RbsType> RbsType for BTreeMap<K, V> {
    fn rbs() -> String {
        format!("Hash[{}, {}]", K::rbs(), V::rbs())
    }
}

macro_rules! rbs_tuple {
    ($($name: ident),+) => {
        impl<$($name: RbsType),+> RbsType for ($($name,)+) {
            fn rbs() -> String {
                format!("[{}]", [$($name::rbs()),+].join(", "))
            }
        }
    };
}

rbs_tuple!(A, B);
rbs_tuple!(A, B, C);
rbs_tuple!(A, B, C, D);

/// `T::rbs()`, or `untyped` for a type without [`RbsType`].
#[doc(hidden)]
pub struct RbsProbe<T: ?Sized>(PhantomData<T>);

#[allow(clippy::new_without_default)]
impl<T: ?Sized> RbsProbe<T> {
    pub fn new() -> Self {
        RbsProbe(PhantomData)
    }
}

#[doc(hidden)]
pub trait ProbeRbs {
    fn rbs(&self) -> String;
}

impl<T: RbsType + ?Sized> ProbeRbs for RbsProbe<T> {
    fn rbs(&self) -> String {
        T::rbs()
    }
}

#[doc(hidden)]
pub trait ProbeRbsFallback {
    fn rbs(&self) -> String;
}

impl<T: ?Sized> ProbeRbsFallback for &RbsProbe<T> {
    fn rbs(&self) -> String {
        "untyped".to_string()
    }
}

/// One declaration inside a class or module body, such as
/// `def add: (Integer amount) -> Integer`.
#[doc(hidden)]
pub fn method(
    name: &str,
    singleton: bool,
    parameters: &[String],
    block: Option<bool>,
    output: &str,
) -> String {
    let block = match block {
        None => String::new(),
        Some(true) => " { (*untyped) -> untyped }".to_string(),
        Some(false) => " ?{ (*untyped) -> untyped }".to_string(),
    };

    format!(
        "def {}{}: ({}){} -> {}",
        if singleton { "self." } else { "" },
        name,
        parameters.join(", "),
        block,
        output
    )
}

/// A whole declaration, nesting `path`'s modules: the last segment is
/// `class` (with `superclass`) or `module` (`superclass` is `None` and
/// `is_class` false).
#[doc(hidden)]
pub fn declaration(
    path: &str,
    is_class: bool,
    superclass: Option<&str>,
    members: &[String],
) -> String {
    let names: Vec<&str> = path.split("::").collect();
    let (last, modules) = names.split_last().expect("a path has a name");
    let mut out = String::new();

    for (depth, module) in modules.iter().enumerate() {
        out.push_str(&format!("{}module {}\n", "  ".repeat(depth), module));
    }

    let depth = modules.len();
    let indent = "  ".repeat(depth);
    let header = match (is_class, superclass) {
        (true, Some(superclass)) => format!("class {} < {}", last, superclass),
        (true, None) => format!("class {}", last),
        (false, _) => format!("module {}", last),
    };

    out.push_str(&format!("{}{}\n", indent, header));

    for member in members {
        out.push_str(&format!("{}  {}\n", indent, member));
    }

    out.push_str(&format!("{}end\n", indent));

    for depth in (0..depth).rev() {
        out.push_str(&format!("{}end\n", "  ".repeat(depth)));
    }

    out
}
