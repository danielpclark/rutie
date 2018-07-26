use std::mem;
use std::convert::From;
use rubysys::constant;

use rubysys::types::{InternalValue, RBasic};

const SPECIAL_SHIFT: usize = 8;

#[cfg(not(target_arch = "x86_64"))]
pub enum RubySpecialConsts {
    False = 0,
    True = 0x02,
    Nil = 0x04,
    Undef = 0x06,
}

#[cfg(not(target_arch = "x86_64"))]
pub enum RubySpecialFlags {
    ImmediateMask = 0x03,
    FixnumFlag = 0x01,
    FlonumMask = 0x00,
    FlonumFlag = 0x02,
    SymbolFlag = 0x0e,
}

#[cfg(target_arch = "x86_64")]
pub enum RubySpecialConsts {
    False = 0,
    True = 0x14,
    Nil = 0x08,
    Undef = 0x34,
}

#[cfg(target_arch = "x86_64")]
pub enum RubySpecialFlags {
    ImmediateMask = 0x07,
    FixnumFlag = 0x01,
    FlonumMask = 0x03,
    FlonumFlag = 0x02,
    SymbolFlag = 0x0c,
}

#[derive(Debug, PartialEq)]
#[link_name = "ruby_value_type"]
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
        (self.value & (RubySpecialFlags::FlonumMask as InternalValue)) ==
        (RubySpecialFlags::FlonumFlag as InternalValue)
    }

    pub fn is_frozen(&self) -> bool {
        !self.is_fl_able() || self.is_obj_frozen_raw()
    }

    pub fn ty(&self) -> ValueType {
        if self.is_immediate() {
            if self.is_fixnum() {
                ValueType::Fixnum
            } else if self.is_flonum() {
                ValueType::Float
            } else if self.is_true() {
                ValueType::True
            } else if self.is_symbol() {
                ValueType::Symbol
            } else if self.is_undef() {
                ValueType::Undef
            } else {
                self.builtin_type()
            }
        } else if !self.is_test() {
            if self.is_nil() {
                ValueType::Nil
            } else if self.is_false() {
                ValueType::False
            } else {
                self.builtin_type()
            }
        } else {
            self.builtin_type()
        }
    }

    fn is_fl_able(&self) -> bool {
        !self.is_special_const() && !self.is_node()
    }

    fn is_special_const(&self) -> bool {
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
            let basic: *const RBasic = mem::transmute(self.value);
            (*basic).flags & (constant::FL_FREEZE as InternalValue) != 0
        }
    }

    fn builtin_type(&self) -> ValueType {
        unsafe {
            let basic: *const RBasic = mem::transmute(self.value);
            let masked = (*basic).flags & (ValueType::Mask as InternalValue);
            mem::transmute(masked as u32)
        }
    }
}

impl From<InternalValue> for Value {
    fn from(internal_value: InternalValue) -> Self {
        Value { value: internal_value }
    }
}
