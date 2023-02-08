use std::{convert::From, mem};

use super::{
    constant,
    types::{InternalValue, RBasic},
};

const SPECIAL_SHIFT: usize = 8;

use rb_sys::{ruby_special_consts, ruby_value_type};

// I assume pointer width is handled by rb_sys;
pub enum RubySpecialConsts {
    False = ruby_special_consts::RUBY_Qfalse as isize,
    True = ruby_special_consts::RUBY_Qtrue as isize,
    Nil = ruby_special_consts::RUBY_Qnil as isize,
    Undef = ruby_special_consts::RUBY_Qundef as isize,
}

pub enum RubySpecialFlags {
    ImmediateMask = ruby_special_consts::RUBY_IMMEDIATE_MASK as isize,
    FixnumFlag = ruby_special_consts::RUBY_FIXNUM_FLAG as isize,
    FlonumMask = ruby_special_consts::RUBY_FLONUM_MASK as isize,
    FlonumFlag = ruby_special_consts::RUBY_FLONUM_FLAG as isize,
    SymbolFlag = ruby_special_consts::RUBY_SYMBOL_FLAG as isize,
}

// #[link_name = "ruby_value_type"]
#[derive(Debug, PartialEq)]
#[repr(C)]
pub enum ValueType {
    None = ruby_value_type::RUBY_T_NONE as isize,

    Object = ruby_value_type::RUBY_T_OBJECT as isize,
    Class = ruby_value_type::RUBY_T_CLASS as isize,
    Module = ruby_value_type::RUBY_T_MODULE as isize,
    Float = ruby_value_type::RUBY_T_FLOAT as isize,
    RString = ruby_value_type::RUBY_T_STRING as isize,
    Regexp = ruby_value_type::RUBY_T_REGEXP as isize,
    Array = ruby_value_type::RUBY_T_ARRAY as isize,
    Hash = ruby_value_type::RUBY_T_HASH as isize,
    Struct = ruby_value_type::RUBY_T_STRUCT as isize,
    Bignum = ruby_value_type::RUBY_T_BIGNUM as isize,
    File = ruby_value_type::RUBY_T_FILE as isize,
    Data = ruby_value_type::RUBY_T_DATA as isize,
    Match = ruby_value_type::RUBY_T_MATCH as isize,
    Complex = ruby_value_type::RUBY_T_COMPLEX as isize,
    Rational = ruby_value_type::RUBY_T_RATIONAL as isize,

    Nil = ruby_value_type::RUBY_T_NIL as isize,
    True = ruby_value_type::RUBY_T_TRUE as isize,
    False = ruby_value_type::RUBY_T_FALSE as isize,
    Symbol = ruby_value_type::RUBY_T_SYMBOL as isize,
    Fixnum = ruby_value_type::RUBY_T_FIXNUM as isize,
    Undef = ruby_value_type::RUBY_T_UNDEF as isize,

    IMemo = ruby_value_type::RUBY_T_IMEMO as isize,
    Node = ruby_value_type::RUBY_T_NODE as isize,
    IClass = ruby_value_type::RUBY_T_ICLASS as isize,
    Zombie = ruby_value_type::RUBY_T_ZOMBIE as isize,

    Mask = ruby_value_type::RUBY_T_MASK as isize,
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
        Value {
            value: internal_value,
        }
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value {
            value: value as InternalValue,
        }
    }
}

impl From<Value> for u64 {
    fn from(value: Value) -> Self {
        value.value
    }
}
