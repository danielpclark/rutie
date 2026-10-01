use crate::{
    binding::{gc, vm},
    AnyException, AnyObject, Hash, Module, Object, Symbol,
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
    /// use rutie::{GC, NilClass, Object, Proc, VM};
    /// use std::sync::atomic::{AtomicBool, Ordering};
    ///
    /// static FINALIZED: AtomicBool = AtomicBool::new(false);
    ///
    /// # VM::init();
    /// let object = VM::eval("Object.new").unwrap();
    /// let finalizer = Proc::new(|_| {
    ///     FINALIZED.store(true, Ordering::SeqCst);
    ///     NilClass::new().into()
    /// });
    /// GC::define_finalizer(&object, &finalizer).unwrap();
    /// GC::undefine_finalizer(&object);
    ///
    /// // Shutting the VM down runs the finalizers still defined; this one is gone.
    /// unsafe { VM::cleanup() };
    /// assert!(!FINALIZED.load(Ordering::SeqCst));
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
    ///
    /// GC::start();
    /// assert_eq!(child.to_str(), "child");
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
    /// let array = Array::new();
    /// unsafe { GC::write_barrier_unprotect(&array) };
    ///
    /// GC::start();
    /// assert_eq!(array.length(), 0);
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
    /// let collections = GC::count();
    /// let before = GC::stat("malloc_increase_bytes");
    ///
    /// GC::adjust_memory_usage(25_000); // Tell ruby that we somehow allocated 25_000 bytes of mem
    ///
    /// // Ruby counts it towards its next collection (unless one just ran).
    /// if GC::count() == collections {
    ///     assert!(GC::stat("malloc_increase_bytes") >= before + 25_000);
    /// }
    ///
    /// GC::adjust_memory_usage(-25_000); // Tell ruby that freed 25_000 bytes of mem
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
    /// let before = GC::count();
    /// GC::start();
    ///
    /// assert_eq!(GC::count(), before + 1);
    /// ```
    pub fn count() -> usize {
        gc::count()
    }

    /// Disable the garbage collector: allocation no longer triggers a
    /// collection. An explicit [`GC::start`](#method.start) still collects,
    /// as Ruby's `GC.start` does.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, GC, VM};
    /// # VM::init();
    ///
    /// // Returns whether it was already disabled.
    /// assert!(!GC::disable());
    /// assert!(GC::disable());
    ///
    /// // Allocating doesn't collect while disabled.
    /// let before = GC::count();
    /// for i in 0..100_000 {
    ///     RString::new_utf8(&i.to_string());
    /// }
    /// assert_eq!(GC::count(), before);
    ///
    /// GC::enable();
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
    /// // Returns whether it was disabled.
    /// assert!(!GC::enable());
    ///
    /// GC::disable();
    /// assert!(GC::enable());
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
    /// #[macro_use] extern crate rutie;
    /// #[macro_use] extern crate lazy_static;
    ///
    /// use rutie::{AnyObject, Class, Object, RString, GC, VM};
    ///
    /// pub struct Names {
    ///     names: Vec<RString>,
    /// }
    ///
    /// wrappable_struct! {
    ///     Names,
    ///     NamesWrapper,
    ///     NAMES_WRAPPER,
    ///
    ///     // Called by the GC; the Rust heap is not scanned, so these objects
    ///     // are only kept alive by being marked here.
    ///     mark(data) {
    ///         for name in &data.names { GC::mark(name); }
    ///     }
    /// }
    ///
    /// fn main() {
    ///     # VM::init();
    ///     let names = (0..4).map(|i| RString::new_utf8(&i.to_string())).collect();
    ///     let object: AnyObject = Class::new("Names", None)
    ///         .wrap_data(Names { names }, &*NAMES_WRAPPER);
    ///
    ///     GC::start();
    ///
    ///     let names = &object.get_data(&*NAMES_WRAPPER).names;
    ///     assert_eq!(names[3].to_str(), "3");
    /// }
    /// ```
    pub fn mark(object: &impl Object) {
        gc::mark(object.value());
    }

    /// Marks an object like [`GC::mark`](#method.mark), but lets
    /// `GC.compact` move it (`rb_gc_mark_movable`).
    ///
    /// Use it only in the `mark` clause of a `wrappable_struct!` that also has
    /// a `compact` clause updating the same object with
    /// [`GC::location`](#method.location). An object marked movable and not
    /// updated is left pointing at the object's old, freed slot.
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    /// #[macro_use] extern crate lazy_static;
    ///
    /// use rutie::{AnyObject, Class, Object, RString, GC, VM};
    ///
    /// pub struct Label {
    ///     text: RString,
    /// }
    ///
    /// wrappable_struct! {
    ///     Label,
    ///     LabelWrapper,
    ///     LABEL_WRAPPER,
    ///
    ///     mark(data) {
    ///         GC::mark_movable(&data.text);
    ///     },
    ///
    ///     // Runs after compaction: fetch each object's new address.
    ///     compact(data) {
    ///         data.text = GC::location(&data.text);
    ///     },
    /// }
    ///
    /// fn main() {
    ///     # VM::init();
    ///     let label: AnyObject = Class::new("Label", None)
    ///         .wrap_data(Label { text: RString::new_utf8("still here") }, &*LABEL_WRAPPER);
    ///
    ///     // `NotImplementedError` where the platform can't compact.
    ///     let _ = GC::compact();
    ///
    ///     assert_eq!(label.get_data(&*LABEL_WRAPPER).text.to_str(), "still here");
    /// }
    /// ```
    pub fn mark_movable(object: &impl Object) {
        gc::mark_movable(object.value());
    }

    /// Returns where `object` is after `GC.compact` moved it
    /// (`rb_gc_location`), or `object` itself if it did not move.
    ///
    /// Only meaningful in a `compact` clause of `wrappable_struct!`; see
    /// [`GC::mark_movable`](#method.mark_movable).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, RString, GC, VM};
    /// # VM::init();
    ///
    /// // Outside compaction nothing is moving, so this is the same object.
    /// let string = RString::new_utf8("here");
    /// assert!(GC::location(&string).equals(&string));
    /// ```
    pub fn location<T: Object>(object: &T) -> T {
        T::from(gc::location(object.value()))
    }

    /// Compacts the heap (`GC.compact`), moving objects that are not pinned.
    ///
    /// Returns a `NotImplementedError` where the platform does not support
    /// compaction.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Exception, Object, RString, GC, VM};
    /// # VM::init();
    ///
    /// let string = RString::new_utf8("survives");
    ///
    /// match GC::compact() {
    ///     Ok(()) => {}
    ///     Err(error) => assert_eq!(error.class().name().unwrap().to_str(), "NotImplementedError"),
    /// }
    ///
    /// assert_eq!(string.to_str(), "survives");
    /// ```
    pub fn compact() -> Result<(), AnyException> {
        Module::from_existing("GC")
            .protect_send("compact", &[])
            .map(|_| ())
    }

    /// Mark all of the object from `start` to `end` of the array for the GC.
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    /// #[macro_use] extern crate lazy_static;
    ///
    /// use rutie::{AnyObject, Class, Object, RString, GC, VM};
    ///
    /// pub struct Names {
    ///     names: Vec<RString>,
    /// }
    ///
    /// wrappable_struct! {
    ///     Names,
    ///     NamesWrapper,
    ///     NAMES_WRAPPER,
    ///
    ///     // Called by the GC; the Rust heap is not scanned, so these objects
    ///     // are only kept alive by being marked here.
    ///     mark(data) {
    ///         GC::mark_locations(&data.names);
    ///     }
    /// }
    ///
    /// fn main() {
    ///     # VM::init();
    ///     let names = (0..4).map(|i| RString::new_utf8(&i.to_string())).collect();
    ///     let object: AnyObject = Class::new("Names", None)
    ///         .wrap_data(Names { names }, &*NAMES_WRAPPER);
    ///
    ///     GC::start();
    ///
    ///     let names = &object.get_data(&*NAMES_WRAPPER).names;
    ///     assert_eq!(names[3].to_str(), "3");
    /// }
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
    /// #[macro_use] extern crate rutie;
    /// #[macro_use] extern crate lazy_static;
    ///
    /// use rutie::{AnyObject, Class, Object, RString, GC, VM};
    ///
    /// pub struct Names {
    ///     names: Vec<RString>,
    /// }
    ///
    /// wrappable_struct! {
    ///     Names,
    ///     NamesWrapper,
    ///     NAMES_WRAPPER,
    ///
    ///     // Called by the GC; the Rust heap is not scanned, so these objects
    ///     // are only kept alive by being marked here.
    ///     mark(data) {
    ///         for name in &data.names { GC::mark_maybe(name); }
    ///     }
    /// }
    ///
    /// fn main() {
    ///     # VM::init();
    ///     let names = (0..4).map(|i| RString::new_utf8(&i.to_string())).collect();
    ///     let object: AnyObject = Class::new("Names", None)
    ///         .wrap_data(Names { names }, &*NAMES_WRAPPER);
    ///
    ///     GC::start();
    ///
    ///     let names = &object.get_data(&*NAMES_WRAPPER).names;
    ///     assert_eq!(names[3].to_str(), "3");
    /// }
    /// ```
    pub fn mark_maybe(object: &impl Object) {
        gc::mark_maybe(object.value());
    }

    /// Keeps `object` alive until a matching [`GC::unregister`](#method.unregister),
    /// even when nothing else references it (for example when only a raw
    /// `Value` is kept in Rust heap memory, which the GC does not scan).
    ///
    /// Registrations are counted: an object registered twice needs two
    /// `unregister` calls. (Before 0.10 this registered the address of a
    /// temporary copy with `rb_gc_register_address`, which did not protect
    /// the object and left the GC reading a stale stack slot.)
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, RString, GC, VM};
    /// # VM::init();
    ///
    /// let id = {
    ///     let object = RString::new_utf8("kept");
    ///     GC::register(&object);
    ///
    ///     unsafe { object.send("object_id", &[]) }
    /// };
    ///
    /// GC::start();
    ///
    /// let object_space = rutie::Module::from_existing("ObjectSpace");
    /// let found = object_space.protect_send("_id2ref", &[id]).unwrap();
    /// assert_eq!(found.try_convert_to::<RString>().unwrap().to_str(), "kept");
    ///
    /// GC::unregister(&found);
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
    /// Unlike [`GC::register`](#method.register) this is permanent: the
    /// object lives until the process exits.
    ///
    /// ```
    /// use rutie::{Module, Object, RString, GC, VM};
    /// # VM::init();
    ///
    /// let id = {
    ///     let object = RString::new_utf8("permanent");
    ///     GC::register_mark(&object);
    ///
    ///     unsafe { object.send("object_id", &[]) }
    /// };
    ///
    /// GC::start();
    ///
    /// let found = Module::from_existing("ObjectSpace").protect_send("_id2ref", &[id]).unwrap();
    /// assert_eq!(found.try_convert_to::<RString>().unwrap().to_str(), "permanent");
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
    /// let before = GC::count();
    ///
    /// GC::start();
    ///
    /// assert_eq!(GC::count(), before + 1);
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
    /// assert!(GC::stat("heap_allocated_pages") > 0);
    /// assert_eq!(GC::stat("count"), GC::count());
    /// ```
    pub fn stat(key: &str) -> usize {
        let key = Symbol::new(key);

        gc::stat(key.value())
    }

    /// Undoes one [`GC::register`](#method.register) of `object`; once every
    /// registration is undone, the GC may collect it again. Unregistering an
    /// object that is not registered does nothing.
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
    /// GC::register(&object);
    /// GC::unregister(&object);
    /// GC::unregister(&object);
    ///
    /// // Extra calls are harmless.
    /// GC::unregister(&object);
    ///
    /// // Unregistering only lets the GC collect it once nothing references
    /// // it; `object` is still in use here.
    /// GC::start();
    /// assert_eq!(object.to_str(), "1");
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

    pub struct RutieGcHolder {
        objects: Vec<crate::AnyObject>,
    }

    crate::wrappable_struct! {
        RutieGcHolder,
        RutieGcHolderWrapper,
        RUTIE_GC_HOLDER,

        mark(data) {
            // `mark_locations` and `mark_maybe` are for mark functions.
            GC::mark_locations(&data.objects);
            for object in &data.objects {
                GC::mark_maybe(object);
            }
            // Records that the GC called this mark function.
            HOLDER_CONTENTS_MARKED.store(true, Ordering::SeqCst);
        }
    }

    static HOLDER_CONTENTS_MARKED: std::sync::atomic::AtomicBool =
        std::sync::atomic::AtomicBool::new(false);

    fn id2ref(id: &crate::AnyObject) -> Result<crate::AnyObject, crate::AnyException> {
        crate::Module::from_existing("ObjectSpace").protect_send("_id2ref", &[id.clone()])
    }

    #[test]
    fn test_gc_register_keeps_objects_alive() {
        crate::on_ruby_thread(|| {
            // Only a raw value in Rust heap memory, which the GC does not scan.
            let raw: Box<crate::types::Value> =
                Box::new(crate::RString::new_utf8("registered").value());
            let object = crate::RString::from(*raw);
            GC::register(&object);
            GC::register(&object);
            assert_eq!(crate::binding::gc::registered_count(*raw), 2);

            let id = unsafe { object.send("object_id", &[]) };
            drop(object);

            for _ in 0..3 {
                GC::start();
            }

            let found = id2ref(&id).unwrap();
            assert_eq!(
                found.try_convert_to::<crate::RString>().unwrap().to_str(),
                "registered"
            );

            GC::unregister(&found);
            assert_eq!(crate::binding::gc::registered_count(*raw), 1);
            GC::unregister(&found);
            assert_eq!(crate::binding::gc::registered_count(*raw), 0);
            GC::unregister(&found);
            assert_eq!(crate::binding::gc::registered_count(*raw), 0);

            let permanent = crate::RString::new_utf8("permanent");
            GC::register_mark(&permanent);
            GC::start();
            assert_eq!(permanent.to_str(), "permanent");
        });
    }

    #[test]
    fn test_gc_controls_and_marking() {
        crate::on_ruby_thread(|| {
            let was_disabled = GC::disable();
            assert!(
                GC::disable(),
                "second disable reports it was already disabled"
            );
            assert!(GC::enable(), "enable reports it was disabled");
            assert!(!GC::enable());
            if was_disabled {
                GC::disable();
            }

            let before = GC::count();
            GC::start();
            assert!(GC::count() > before);
            assert_eq!(GC::stat("count"), GC::count());

            GC::adjust_memory_usage(4096);
            GC::adjust_memory_usage(-4096);

            // The string is only in the `Vec` (on the Rust heap, which the GC
            // does not scan) until `wrap_data` returns.
            GC::disable();
            let holder = crate::Class::new("RutieGcHolderClass", None).wrap_data(
                RutieGcHolder {
                    objects: vec![crate::RString::new_utf8("held").to_any_object()],
                },
                &*RUTIE_GC_HOLDER,
            );
            GC::enable();
            let holder: crate::AnyObject = holder;
            GC::start();
            let held = holder.get_data(&*RUTIE_GC_HOLDER).objects[0].clone();
            assert_eq!(
                held.try_convert_to::<crate::RString>().unwrap().to_str(),
                "held"
            );
            assert!(HOLDER_CONTENTS_MARKED.load(Ordering::SeqCst));

            let object = crate::RString::new_utf8("finalizable");
            let finalizer = Proc::new(|_| NilClass::new().into());
            GC::define_finalizer(&object, &finalizer).unwrap();
            GC::undefine_finalizer(&object);
        });
    }

    #[test]
    fn test_write_barriers() {
        crate::on_ruby_thread(|| {
            let parent = crate::Array::new();
            let child = crate::RString::new_utf8("child");

            // Required when a Rust-managed parent starts referencing a child.
            unsafe { GC::write_barrier(&parent, &child) };
            // Opting an object out of generational GC is always allowed.
            unsafe { GC::write_barrier_unprotect(&parent) };

            GC::start();
            assert_eq!(child.to_str(), "child");
        });
    }
}
