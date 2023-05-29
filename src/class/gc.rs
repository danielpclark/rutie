use crate::{binding::gc, Object, Symbol};

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

    /// Forcibly GC object.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, GC, VM};
    /// # VM::init();
    ///
    /// let obj = RString::new_utf8("asdf");
    ///
    /// GC::force_recycle(obj);
    /// ```
    pub fn force_recycle(object: impl Object) {
        gc::force_recycle(object.value())
    }

    /// Check if object is marked
    ///
    /// CAUTION: THIS FUNCTION IS ENABLED *ONLY BEFORE* SWEEPING.
    /// This function is only for GC_END_MARK timing.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rutie::{RString, GC, VM};
    /// # VM::init();
    ///
    /// let obj = RString::new_utf8("asdf");
    ///
    /// GC::mark(&obj);
    /// assert!(unsafe {GC::is_marked(&obj) }, "Object was not marked");
    /// ```
    pub unsafe fn is_marked(object: &impl Object) -> bool {
        gc::is_marked(object.value())
    }

    /// Mark an object for Ruby to avoid garbage collecting item.
    ///
    /// If the wrapped struct in Rust references Ruby objects, then
    /// you'll have to mark those in the mark callback you are passing
    /// to wrapped struct.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rutie::{RString, GC, VM};
    /// # VM::init();
    ///
    /// let object = RString::new_utf8("1");
    ///
    /// GC::mark(&object);
    /// ```
    pub fn mark(object: &impl Object) {
        gc::mark(object.value());
    }

    /// Mark all of the object from `start` to `end` of the array for the GC.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rutie::{RString, GC, VM, AnyObject};
    /// # VM::init();
    ///
    /// let arr = [
    ///     RString::new_utf8("1"),
    ///     RString::new_utf8("2"),
    ///     RString::new_utf8("3"),
    ///     RString::new_utf8("4"),
    /// ];
    ///
    /// GC::mark_locations(&arr);
    /// ```
    pub fn mark_locations(range: &[impl Object]) {
        for object in range {
            GC::mark_maybe(object)
        }
    }

    /// Maybe mark an object for Ruby to avoid garbage collecting item.
    ///
    /// If the wrapped struct in Rust references Ruby objects, then
    /// you'll have to mark those in the mark callback you are passing
    /// to wrapped struct.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rutie::{RString, GC, VM};
    /// # VM::init();
    ///
    /// let object = RString::new_utf8("1");
    ///
    /// GC::mark_maybe(&object);
    /// ```
    pub fn mark_maybe(object: &impl Object) {
        gc::mark_maybe(object.value());
    }

    /// Registers the objects address with the GC
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, GC, VM};
    /// # VM::init();
    ///
    /// let object = RString::new_utf8("1");
    ///
    /// GC::register(&object);
    /// ```
    pub fn register(object: &impl Object) {
        gc::register(object.value())
    }

    /// Mark an object as in use for Ruby to avoid garbage collecting item.
    ///
    /// If the wrapped struct in Rust references Ruby objects, then
    /// you'll have to mark those in the mark callback you are passing
    /// to wrapped struct.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, GC, VM};
    /// # VM::init();
    ///
    /// let object = RString::new_utf8("1");
    ///
    /// GC::register_mark(&object);
    /// ```
    pub fn register_mark(object: &impl Object) {
        gc::register_mark(object.value());
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

    /// Get the GC stats for a specific key
    ///
    /// Note: Will panic if provided an invalid key.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{GC, VM};
    /// # VM::init();
    ///
    /// let result = GC::stat("heap_allocated_pages");
    /// ```
    pub fn stat(key: &str) -> usize {
        let key = Symbol::new(key);

        gc::stat(key.value())
    }

    /// Unregisters the objects address with the GC
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, GC, VM};
    /// # VM::init();
    ///
    /// let object = RString::new_utf8("1");
    ///
    /// GC::unregister(&object);
    /// ```
    pub fn unregister(object: &impl Object) {
        gc::unregister(object.value())
    }
}
