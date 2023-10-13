use super::types::{c_char, c_int, c_void, Value};

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

#[repr(C)]
pub struct RbDataTypeFunction {
    pub dmark: Option<unsafe extern "C" fn(*mut c_void)>,
    pub dfree: Option<unsafe extern "C" fn(*mut c_void)>,
    pub dsize: Option<unsafe extern "C" fn(*const c_void) -> u64>,
    pub compact: Option<unsafe extern "C" fn(*mut c_void)>,
    pub reserved: [*mut c_void; 1],
}

impl Copy for RbDataTypeFunction {}
impl Clone for RbDataTypeFunction {
    fn clone(&self) -> Self {
        *self
    }
}
unsafe impl Send for RbDataTypeFunction {}
unsafe impl Sync for RbDataTypeFunction {}

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

impl From<RbDataTypeFunction> for rb_sys::bindings::rb_data_type_struct__bindgen_ty_1 {
    fn from(rb_data_type_fn: RbDataTypeFunction) -> Self {
        rb_sys::bindings::rb_data_type_struct__bindgen_ty_1 {
            dmark: rb_data_type_fn.dmark,
            dfree: rb_data_type_fn.dfree,
            dsize: rb_data_type_fn.dsize,
            dcompact: rb_data_type_fn.compact,
            reserved: rb_data_type_fn.reserved,
        }
    }
}

impl From<&RbDataType> for rb_sys::rb_data_type_struct {
    fn from(rb_data_type: &RbDataType) -> Self {
        rb_sys::rb_data_type_struct {
            wrap_struct_name: rb_data_type.wrap_struct_name,
            function: rb_data_type.function.into(),
            parent: rb_data_type.parent as *const _,
            data: rb_data_type.data,
            flags: rb_data_type.flags.into(),
        }
    }
}
