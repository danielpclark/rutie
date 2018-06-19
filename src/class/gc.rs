use binding::gc;

use Object;

/// Garbage collection
pub struct GC;

impl GC {
    /// Mark an object for garbage collection.
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
}
