extern crate libc;

pub mod array;
pub mod class;
pub mod constant;
pub mod encoding;
pub mod exception;
pub mod fixnum;
pub mod float;
pub mod gc;
pub mod hash;
pub mod numeric;
pub mod object;
pub mod range;
pub mod regexp;
pub mod rproc;
pub mod rstruct;
pub mod string;
pub mod symbol;
pub mod thread;
pub mod time;
pub mod typed_data;
pub mod types;
pub mod value;
pub mod variable;
pub mod vm;

use crate::rubysys::types::Value;

extern "C" {
    pub static rb_cObject: Value;
}
