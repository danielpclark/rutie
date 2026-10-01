extern crate libc;

pub mod array;
pub mod builtins;
pub mod class;
pub mod constant;
pub mod debug;
pub mod encoding;
pub mod enumerator;
pub mod exception;
pub mod fixnum;
pub mod float;
pub mod gc;
pub mod hash;
pub mod io;
#[cfg(ruby_gte_3_1)]
pub mod io_buffer;
pub mod memory_view;
pub mod numeric;
pub mod object;
pub mod ractor;
pub mod range;
pub mod regexp;
pub mod rproc;
pub mod rstruct;
#[cfg(ruby_gte_3_1)]
pub mod scheduler;
pub mod st;
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

// Every `extern` block in `rubysys` names the Ruby DLL on Windows
// (`rutie_dllimport`, set by `build.rs`). Rust only imports variables such as
// `rb_cObject` from a DLL when their block names it; without it they would
// resolve to jump thunks and read as garbage.
#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    pub static rb_cObject: Value;
}
