use std::{ffi::CStr, ptr};

use crate::types::c_char;
use crate::{binding::symbol, rubysys::rstruct, types::Value};

pub fn alloc(klass: Value, values: Value) -> Value {
    unsafe { rstruct::rb_struct_alloc(klass, values) }
}

pub fn aref(object: Value, index: Value) -> Value {
    unsafe { rstruct::rb_struct_aref(object, index) }
}

pub fn aset(object: Value, index: Value, value: Value) -> Value {
    unsafe { rstruct::rb_struct_aset(object, index, value) }
}

pub fn get_member(object: Value, name: &str) -> Value {
    unsafe { rstruct::rb_struct_getmember(object, symbol::internal_id(name)) }
}

pub fn members(object: Value) -> Value {
    unsafe { rstruct::rb_struct_members(object) }
}

pub fn class_members(klass: Value) -> Value {
    unsafe { rstruct::rb_struct_s_members(klass) }
}

pub fn size(object: Value) -> Value {
    unsafe { rstruct::rb_struct_size(object) }
}

// The most members one `rb_data_define` call takes here: it is variadic, so
// the call always passes this many name pointers and a terminator, with null
// pointers after the last member (C ignores arguments after the terminator).
pub const DATA_DEFINE_MAX_MEMBERS: usize = 32;

// Raises `ArgumentError` for a duplicate member, so the caller owns the
// names. Panics with more than `DATA_DEFINE_MAX_MEMBERS` members.
pub fn data_define(superclass: Value, members: &[&CStr]) -> Value {
    assert!(members.len() <= DATA_DEFINE_MAX_MEMBERS);

    let mut names = [ptr::null::<c_char>(); DATA_DEFINE_MAX_MEMBERS + 1];

    for (slot, member) in names.iter_mut().zip(members) {
        *slot = member.as_ptr();
    }

    let n = &names;

    unsafe {
        rstruct::rb_data_define(
            superclass, n[0], n[1], n[2], n[3], n[4], n[5], n[6], n[7], n[8], n[9], n[10], n[11],
            n[12], n[13], n[14], n[15], n[16], n[17], n[18], n[19], n[20], n[21], n[22], n[23],
            n[24], n[25], n[26], n[27], n[28], n[29], n[30], n[31], n[32],
        )
    }
}
