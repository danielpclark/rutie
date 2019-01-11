use binding::gc;

use Object;

/// Garbage collection
pub struct GC;

impl GC {
    /// Mark an object for Ruby to avoid garbage collecting item.
    ///
    /// If the wrapped struct in Rust references Ruby objects, then
    /// you'll have to mark those in the mark callback you are passing
    /// to wrapped struct.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, GC, VM};
    /// # VM::init();
    ///
    /// let object = Fixnum::new(1);
    ///
    /// GC::mark(&object);
    /// ```
    pub fn mark<T: Object>(object: &T) {
        gc::mark(object.value());
    }

    /// Notify memory usage to the GC engine by extension libraries, to trigger GC
    /// This is useful when you wrap large rust objects using wrap_data,
    /// when you do so, ruby is unaware of the allocated memory and might not run GC
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{GC, VM};
    /// # VM::init();
    ///
    ///
    /// GC::adjust_memory_usage(25_000); // Tell ruby that we somehow allocated 25_000 bytes of mem
    /// GC::adjust_memory_usage(-15_000); // Tell ruby that freed 15_000 bytes of mem
    /// ```
    pub fn adjust_memory_usage(diff: isize) {
        gc::adjust_memory_usage(diff)
    }
}
