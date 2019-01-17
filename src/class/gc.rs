use binding::gc;

use Object;

/// Garbage collection
pub struct GC;

impl GC {
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

    /// The number of times GC occurred.
    ///
    /// It returns the number of times GC occurred since the process started.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{GC, VM};
    /// # VM::init();
    ///
    /// GC::count();
    /// ```
    pub fn count() -> usize {
        gc::count()
    }

    /// Disable the garbage collector
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{GC, VM};
    /// # VM::init();
    ///
    /// let _ = GC::disable();
    /// ```
    pub fn disable() -> bool {
        gc::disable().is_true()
    }

    /// Enable the garbage collector
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{GC, VM};
    /// # VM::init();
    ///
    /// let _ = GC::enable();
    /// ```
    pub fn enable() -> bool {
        gc::enable().is_true()
    }

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

    /// Start the garbage collector
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{GC, VM};
    /// # VM::init();
    ///
    /// GC::start();
    /// ```
    pub fn start() {
        gc::start()
    }
}
