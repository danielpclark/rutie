use crate::{
    binding::{gc, vm},
    AnyException, AnyObject, Hash, Object, Symbol,
};

/// Garbage collection
pub struct GC;

impl GC {
    /// Registers `finalizer` to be called with the object's id after
    /// `object` is garbage collected (Ruby's `ObjectSpace.define_finalizer`,
    /// `rb_define_finalizer`), or returns the error: an `ArgumentError` if
    /// `finalizer` is not callable, a `FrozenError` for a frozen object.
    ///
    /// The finalizer must not refer to `object`, or it is never collected.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{GC, NilClass, Object, Proc, VM};
    /// # VM::init();
    ///
    /// let object = VM::eval("Object.new").unwrap();
    /// let finalizer = Proc::new(|_| NilClass::new().into());
    ///
    /// assert!(GC::define_finalizer(&object, &finalizer).is_ok());
    /// assert!(GC::define_finalizer(&object, &NilClass::new()).is_err());
    ///
    /// GC::undefine_finalizer(&object);
    /// ```
    pub fn define_finalizer<T: Object, F: Object>(
        object: &T,
        finalizer: &F,
    ) -> Result<(), AnyException> {
        let (object, finalizer) = (object.value(), finalizer.value());

        vm::protect_value(|| gc::define_finalizer(object, finalizer))
            .map(|_| ())
            .map_err(AnyException::from)
    }

    /// Removes the finalizers of `object` (Ruby's
    /// `ObjectSpace.undefine_finalizer`, `rb_undefine_finalizer`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{GC, VM};
    /// # VM::init();
    ///
    /// GC::undefine_finalizer(&VM::eval("Object.new").unwrap());
    /// ```
    pub fn undefine_finalizer<T: Object>(object: &T) {
        gc::undefine_finalizer(object.value());
    }

    /// Returns information about the latest garbage collection, such as
    /// `:major_by` and `:gc_by` (Ruby's `GC.latest_gc_info`,
    /// `rb_gc_latest_gc_info`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{GC, Symbol, VM};
    /// # VM::init();
    ///
    /// GC::start();
    ///
    /// assert!(GC::latest_info().has_key(&Symbol::new("gc_by")));
    /// ```
    pub fn latest_info() -> Hash {
        let info = Hash::new();

        Hash::from(gc::latest_gc_info(info.value()))
    }

    /// Tells the generational GC that `parent` now references `child`
    /// (`rb_gc_writebarrier`, C's `RB_OBJ_WRITTEN`).
    ///
    /// Only needed for objects created as write-barrier protected, which
    /// Rutie's `wrappable_struct!` data objects are not.
    ///
    /// # Safety
    ///
    /// `parent` must be a heap object (not an immediate like a `Fixnum`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, GC, RString, VM};
    /// # VM::init();
    ///
    /// let parent = Array::new();
    /// let child = RString::new_utf8("child");
    ///
    /// unsafe { GC::write_barrier(&parent, &child) };
    /// ```
    pub unsafe fn write_barrier<P: Object, C: Object>(parent: &P, child: &C) {
        gc::writebarrier(parent.value(), child.value());
    }

    /// Marks `object` as not write-barrier protected, so the generational GC
    /// always rescans it (`rb_gc_writebarrier_unprotect`).
    ///
    /// # Safety
    ///
    /// `object` must be a heap object (not an immediate like a `Fixnum`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, GC, VM};
    /// # VM::init();
    ///
    /// unsafe { GC::write_barrier_unprotect(&Array::new()) };
    /// ```
    pub unsafe fn write_barrier_unprotect<T: Object>(object: &T) {
        gc::writebarrier_unprotect(object.value());
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
    /// ```
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
    /// ```
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
    /// ```
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

#[cfg(test)]
mod tests {
    use crate::{Fixnum, NilClass, Object, Proc, Symbol, GC, VM};
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    #[test]
    fn test_finalizers_run_after_collection() {
        crate::on_ruby_thread(|| {
            let finalized = Arc::new(AtomicUsize::new(0));

            for _ in 0..20 {
                let object = VM::eval("Object.new").unwrap();
                let counter = finalized.clone();
                let finalizer = Proc::new(move |_| {
                    counter.fetch_add(1, Ordering::SeqCst);
                    NilClass::new().into()
                });

                GC::define_finalizer(&object, &finalizer).unwrap();
            }

            GC::start();
            GC::start();
            VM::eval("20.times { Object.new }").unwrap();

            assert!(finalized.load(Ordering::SeqCst) > 0, "no finalizer ran");

            let frozen = VM::eval("Object.new.freeze").unwrap();
            assert!(GC::define_finalizer(&frozen, &Proc::new(|_| NilClass::new().into())).is_err());
            assert!(
                GC::define_finalizer(&VM::eval("Object.new").unwrap(), &Fixnum::new(1)).is_err()
            );
        });
    }

    #[test]
    fn test_latest_info() {
        crate::on_ruby_thread(|| {
            GC::start();

            let info = GC::latest_info();
            assert!(info.has_key(&Symbol::new("major_by")));
            assert!(info.length() > 2);
        });
    }
}
