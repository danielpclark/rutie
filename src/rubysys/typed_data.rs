use crate::rubysys::{
    constant,
    types::{c_char, c_int, c_void, size_t, InternalValue, Value},
};

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // void *
    // rb_check_typeddata(VALUE obj, const rb_data_type_t *data_type)
    pub fn rb_check_typeddata(object: Value, data_type: *const RbDataType) -> *mut c_void;
    // int
    // rb_typeddata_inherited_p(const rb_data_type_t *child, const rb_data_type_t *parent)
    pub fn rb_typeddata_inherited_p(child: *const RbDataType, parent: *const RbDataType) -> c_int;
    // int
    // rb_typeddata_is_kind_of(VALUE obj, const rb_data_type_t *data_type)
    pub fn rb_typeddata_is_kind_of(object: Value, data_type: *const RbDataType) -> c_int;
    // VALUE
    // rb_data_typed_object_wrap(VALUE klass, void *datap, const rb_data_type_t *type)
    pub fn rb_data_typed_object_wrap(
        klass: Value,
        data: *mut c_void,
        data_type: *const RbDataType,
    ) -> Value;
}

// `RUBY_DATA_FUNC`: void (*)(void *), the mark and free functions of an
// untyped `RData` object.
pub type DataFunction = Option<extern "C" fn(*mut c_void)>;

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // VALUE
    // rb_data_object_wrap(VALUE klass, void *datap, RUBY_DATA_FUNC dmark, RUBY_DATA_FUNC dfree)
    //
    // An untyped `RData` object; prefer `rb_data_typed_object_wrap`.
    pub fn rb_data_object_wrap(
        klass: Value,
        data: *mut c_void,
        dmark: DataFunction,
        dfree: DataFunction,
    ) -> Value;
    // VALUE
    // rb_data_object_zalloc(VALUE klass, size_t size, RUBY_DATA_FUNC dmark, RUBY_DATA_FUNC dfree)
    //
    // `rb_data_object_wrap` with `size` zeroed bytes from `ruby_xcalloc`.
    pub fn rb_data_object_zalloc(
        klass: Value,
        size: size_t,
        dmark: DataFunction,
        dfree: DataFunction,
    ) -> Value;
    // VALUE
    // rb_data_typed_object_zalloc(VALUE klass, size_t size, const rb_data_type_t *type)
    //
    // `rb_data_typed_object_wrap` with `size` zeroed bytes from
    // `ruby_xcalloc`; `type`'s `dfree` must release them (`RUBY_TYPED_DEFAULT_FREE`).
    pub fn rb_data_typed_object_zalloc(
        klass: Value,
        size: size_t,
        data_type: *const RbDataType,
    ) -> Value;
}

#[repr(C)]
pub struct RbDataTypeFunction {
    pub dmark: Option<extern "C" fn(*mut c_void)>,
    pub dfree: Option<extern "C" fn(*mut c_void)>,
    pub dsize: Option<extern "C" fn(*const c_void) -> size_t>,
    // `dcompact` and `reserved[1]` in Ruby 3's headers: `reserved[0]` holds
    // the compaction callback, a `void (*)(void *)`, or null.
    pub reserved: [*mut c_void; 2],
}

unsafe impl Send for RbDataTypeFunction {}
unsafe impl Sync for RbDataTypeFunction {}

// `RbDataType::flags` (`RUBY_TYPED_*`).
//
// `dfree` runs during the sweep instead of being deferred.
pub const RUBY_TYPED_FREE_IMMEDIATELY: InternalValue = 1;
// Ruby 3.3+: the struct may be embedded in the object's slot.
pub const RUBY_TYPED_EMBEDDABLE: InternalValue = 2;
// Frozen objects of this type are Ractor-shareable.
pub const RUBY_TYPED_FROZEN_SHAREABLE: InternalValue = constant::FL_SHAREABLE as InternalValue;
// `dmark` and writes to the struct's `VALUE`s use write barriers.
pub const RUBY_TYPED_WB_PROTECTED: InternalValue = constant::FL_WB_PROTECTED as InternalValue;
// Ruby 3.3+: `dmark` is a `RUBY_REFERENCES` list of offsets, not a function.
pub const RUBY_TYPED_DECL_MARKING: InternalValue = constant::FL_USER_2 as InternalValue;

#[repr(C)]
pub struct RbDataType {
    pub wrap_struct_name: *const c_char,
    pub function: RbDataTypeFunction,
    pub parent: *const RbDataType,
    pub data: *mut c_void,
    pub flags: Value,
}

unsafe impl Send for RbDataType {}
unsafe impl Sync for RbDataType {}
