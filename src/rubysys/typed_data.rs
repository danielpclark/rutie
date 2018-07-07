use rubysys::types::{c_char, c_void, c_int, size_t, Value};

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
    pub fn rb_data_typed_object_wrap(klass: Value,
                                     data: *mut c_void,
                                     data_type: *const RbDataType)
                                     -> Value;
}

#[repr(C)]
pub struct RbDataTypeFunction {
    pub dmark: Option<extern "C" fn(*mut c_void)>,
    pub dfree: Option<extern "C" fn(*mut c_void)>,
    pub dsize: Option<extern "C" fn(*const c_void) -> size_t>,
    pub reserved: [*mut c_void; 2],
}

#[repr(C)]
pub struct RbDataType {
    pub wrap_struct_name: *const c_char,
    pub function: RbDataTypeFunction,
    pub parent: *const RbDataType,
    pub data: *mut c_void,
    pub flags: Value,
}
