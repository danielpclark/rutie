extern crate libc;

pub mod array;
pub mod builtins;
pub mod class;
pub mod constant;
pub mod cstr;
pub mod debug;
pub mod encoding;
pub mod enumerator;
pub mod exception;
pub mod file;
pub mod fixnum;
pub mod float;
pub mod gc;
pub mod hash;
pub mod interpreter;
pub mod io;
pub mod io_buffer;
pub mod memory_view;
pub mod numeric;
pub mod object;
pub mod process;
pub mod ractor;
pub mod random;
pub mod range;
pub mod regexp;
pub mod rproc;
pub mod rstruct;
pub mod scheduler;
pub mod set;
pub mod st;
pub mod string;
pub mod symbol;
pub mod thread;
pub mod thread_native;
pub mod time;
pub mod tracepoint;
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
