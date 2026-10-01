use std::{convert::From, mem};

use crate::rubysys::{
    constant,
    types::{InternalValue, RBasic},
};

const SPECIAL_SHIFT: usize = 8;

// `Qnil`, `Qtrue` and `Qundef` changed in Ruby 3.2 (`special_consts.h`).
#[cfg(all(target_pointer_width = "32", not(ruby_gte_3_2)))]
pub enum RubySpecialConsts {
    False = 0,
    True = 0x02,
    Nil = 0x04,
    Undef = 0x06,
}

#[cfg(all(target_pointer_width = "32", ruby_gte_3_2))]
pub enum RubySpecialConsts {
    False = 0,
    True = 0x06,
    Nil = 0x02,
    Undef = 0x0a,
}

#[cfg(target_pointer_width = "32")]
pub enum RubySpecialFlags {
    ImmediateMask = 0x03,
    FixnumFlag = 0x01,
    FlonumMask = 0x00,
    FlonumFlag = 0x02,
    SymbolFlag = 0x0e,
}

#[cfg(all(target_pointer_width = "64", not(ruby_gte_3_2)))]
pub enum RubySpecialConsts {
    False = 0,
    True = 0x14,
    Nil = 0x08,
    Undef = 0x34,
}

#[cfg(all(target_pointer_width = "64", ruby_gte_3_2))]
pub enum RubySpecialConsts {
    False = 0,
    True = 0x14,
    Nil = 0x04,
    Undef = 0x24,
}

#[cfg(target_pointer_width = "64")]
pub enum RubySpecialFlags {
    ImmediateMask = 0x07,
    FixnumFlag = 0x01,
    FlonumMask = 0x03,
    FlonumFlag = 0x02,
    SymbolFlag = 0x0c,
}

// #[link_name = "ruby_value_type"]
#[derive(Debug, PartialEq)]
#[repr(C)]
pub enum ValueType {
    None = 0x00,

    Object = 0x01,
    Class = 0x02,
    Module = 0x03,
    Float = 0x04,
    RString = 0x05,
    Regexp = 0x06,
    Array = 0x07,
    Hash = 0x08,
    Struct = 0x09,
    Bignum = 0x0a,
    File = 0x0b,
    Data = 0x0c,
    Match = 0x0d,
    Complex = 0x0e,
    Rational = 0x0f,

    Nil = 0x11,
    True = 0x12,
    False = 0x13,
    Symbol = 0x14,
    Fixnum = 0x15,
    Undef = 0x16,

    IMemo = 0x1a,
    Node = 0x1b,
    IClass = 0x1c,
    Zombie = 0x1d,
    // `T_MOVED`: a slot GC compaction moved the object out of.
    Moved = 0x1e,

    Mask = 0x1f,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Value {
    pub value: InternalValue,
}

impl Value {
    pub fn is_true(&self) -> bool {
        self.value == (RubySpecialConsts::True as InternalValue)
    }

    pub fn is_false(&self) -> bool {
        self.value == (RubySpecialConsts::False as InternalValue)
    }

    pub fn is_nil(&self) -> bool {
        self.value == (RubySpecialConsts::Nil as InternalValue)
    }

    pub fn is_node(&self) -> bool {
        self.builtin_type() == ValueType::Node
    }

    pub fn is_undef(&self) -> bool {
        self.value == (RubySpecialConsts::Undef as InternalValue)
    }

    pub fn is_symbol(&self) -> bool {
        (self.value & !((!0) << SPECIAL_SHIFT)) == (RubySpecialFlags::SymbolFlag as InternalValue)
    }

    pub fn is_fixnum(&self) -> bool {
        (self.value & (RubySpecialFlags::FixnumFlag as InternalValue)) != 0
    }

    pub fn is_flonum(&self) -> bool {
        (self.value & (RubySpecialFlags::FlonumMask as InternalValue))
            == (RubySpecialFlags::FlonumFlag as InternalValue)
    }

    pub fn is_frozen(&self) -> bool {
        !self.is_fl_able() || self.is_obj_frozen_raw()
    }

    pub fn ty(&self) -> ValueType {
        // The exact special constants first: from Ruby 3.2 `Qnil` has a bit
        // inside the immediate mask, so it must not reach `builtin_type`.
        if self.is_nil() {
            ValueType::Nil
        } else if self.is_false() {
            ValueType::False
        } else if self.is_true() {
            ValueType::True
        } else if self.is_undef() {
            ValueType::Undef
        } else if self.is_fixnum() {
            ValueType::Fixnum
        } else if self.is_flonum() {
            ValueType::Float
        } else if self.is_symbol() {
            ValueType::Symbol
        } else if self.is_special_const() {
            // No other special constants exist; never dereference one.
            ValueType::None
        } else {
            self.builtin_type()
        }
    }

    fn is_fl_able(&self) -> bool {
        !self.is_special_const() && !self.is_node()
    }

    pub(crate) fn is_special_const(&self) -> bool {
        self.is_immediate() || !self.is_test()
    }

    fn is_immediate(&self) -> bool {
        (self.value & (RubySpecialFlags::ImmediateMask as InternalValue)) != 0
    }

    fn is_test(&self) -> bool {
        (self.value & !(RubySpecialConsts::Nil as InternalValue)) != 0
    }

    fn is_obj_frozen_raw(&self) -> bool {
        unsafe {
            let basic: *const RBasic = self.value as *const RBasic;
            (*basic).flags & (constant::FL_FREEZE as InternalValue) != 0
        }
    }

    fn builtin_type(&self) -> ValueType {
        unsafe {
            let basic: *const RBasic = self.value as *const RBasic;
            let masked = (*basic).flags & (ValueType::Mask as InternalValue);
            mem::transmute(masked as u32)
        }
    }
}

impl From<InternalValue> for Value {
    fn from(internal_value: InternalValue) -> Self {
        Value {
            value: internal_value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ValueType;
    use crate::{AnyObject, Boolean, NilClass, Object, VM};

    fn eval(code: &str) -> AnyObject {
        VM::eval(code).unwrap()
    }

    // The special constants are per version (`Qnil` moved in 3.2): values
    // made by Ruby must be recognised, and values made by Rutie must be
    // what Ruby expects.
    #[test]
    fn test_special_constants_match_ruby() {
        crate::on_ruby_thread(|| {
            assert!(eval("nil").value().is_nil());
            assert!(eval("true").value().is_true());
            assert!(eval("false").value().is_false());
            assert!(!eval("false").value().is_nil());
            assert!(!eval("nil").value().is_false());

            let is_nil = unsafe { NilClass::new().send("nil?", &[]) };
            assert!(is_nil.value().is_true());

            let classes = eval("->(*values) { values.map { |v| v.class.name } }");
            let names = unsafe {
                classes.send(
                    "call",
                    &[
                        NilClass::new().to_any_object(),
                        Boolean::new(true).to_any_object(),
                        Boolean::new(false).to_any_object(),
                    ],
                )
            };
            let names = names.try_convert_to::<crate::Array>().unwrap();
            let names: Vec<String> = names
                .into_iter()
                .map(|name| name.try_convert_to::<crate::RString>().unwrap().to_string())
                .collect();
            assert_eq!(names, ["NilClass", "TrueClass", "FalseClass"]);
        });
    }

    #[test]
    fn test_value_types() {
        crate::on_ruby_thread(|| {
            let cases = [
                ("nil", ValueType::Nil),
                ("true", ValueType::True),
                ("false", ValueType::False),
                ("1", ValueType::Fixnum),
                // A Fixnum on every platform: 64-bit Windows has a 32-bit `long`.
                ("-(2 ** 20)", ValueType::Fixnum),
                ("1.5", ValueType::Float),
                ("1e300", ValueType::Float),
                (":sym", ValueType::Symbol),
                ("'dynamic'.to_sym", ValueType::Symbol),
                ("'str'", ValueType::RString),
                ("[]", ValueType::Array),
                ("{}", ValueType::Hash),
                ("2 ** 100", ValueType::Bignum),
                ("Object.new", ValueType::Object),
                ("String", ValueType::Class),
                ("Kernel", ValueType::Module),
            ];

            for (code, expected) in cases.iter() {
                assert_eq!(eval(code).value().ty(), *expected, "{}", code);
            }
        });
    }
}
