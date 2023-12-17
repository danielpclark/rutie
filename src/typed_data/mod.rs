mod data_type_wrapper;

use crate::types::c_void;

pub use self::data_type_wrapper::DataTypeWrapper;

pub extern "C" fn free<T: Sized>(data: *mut c_void) {
    // Memory is freed when the box goes out of the scope
    unsafe {
        let _ = Box::from_raw(data as *mut T);
    };
}
