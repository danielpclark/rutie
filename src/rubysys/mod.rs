extern crate libc;

pub mod array;
pub mod class;
pub mod constant;
pub mod encoding;
pub mod fixnum;
pub mod float;
pub mod gc;
pub mod hash;
pub mod marshal;
pub mod rproc;
pub mod string;
pub mod symbol;
pub mod thread;
pub mod typed_data;
pub mod types;
pub mod value;
pub mod vm;

use crate::rubysys::types::Value;

extern "C" {
    pub static rb_cObject: Value;
}
