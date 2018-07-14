extern crate libc;

pub mod array;
pub mod constant;
pub mod class;
pub mod encoding;
pub mod fixnum;
pub mod gc;
pub mod float;
pub mod hash;
pub mod rproc;
pub mod string;
pub mod symbol;
pub mod thread;
pub mod typed_data;
pub mod types;
pub mod value;
pub mod vm;

use rubysys::types::Value;

extern "C" {
    pub static rb_cObject: Value;
}
