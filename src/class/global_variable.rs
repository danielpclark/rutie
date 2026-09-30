use crate::{types::Value, AnyObject, Object};

/// A Ruby global variable whose value lives in memory owned by Rust.
///
/// Returned by [`VM::define_variable`](struct.VM.html#method.define_variable)
/// and [`VM::define_readonly_variable`](struct.VM.html#method.define_readonly_variable).
/// Reading and writing through it does not look the variable up by name.
/// The storage is registered with the GC and kept for the life of the
/// process, since Ruby global variables cannot be removed.
#[derive(Debug)]
pub struct GlobalVariable {
    address: *mut Value,
}

impl GlobalVariable {
    pub(crate) fn new(address: *mut Value) -> Self {
        GlobalVariable { address }
    }

    /// Returns the variable's current value.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let counter = VM::define_variable("$rutie_get_example", Fixnum::new(1));
    ///
    /// VM::eval("$rutie_get_example += 1").unwrap();
    ///
    /// assert_eq!(counter.get().try_convert_to::<Fixnum>(), Ok(Fixnum::new(2)));
    /// ```
    pub fn get(&self) -> AnyObject {
        AnyObject::from(unsafe { *self.address })
    }

    /// Sets the variable's value, even for a read-only variable.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let limit = VM::define_readonly_variable("$rutie_set_example", Fixnum::new(1));
    ///
    /// limit.set(Fixnum::new(5));
    ///
    /// let value = VM::eval("$rutie_set_example").unwrap();
    ///
    /// assert_eq!(value.try_convert_to::<Fixnum>(), Ok(Fixnum::new(5)));
    /// ```
    pub fn set<T: Object>(&self, value: T) {
        // The address is a GC root (`rb_gc_register_address`), marked on
        // every collection, so no write barrier is needed.
        unsafe { *self.address = value.value() };
    }
}

#[cfg(test)]
mod tests {
    use crate::{Fixnum, Object, RString, GC, VM};

    #[test]
    fn test_global_variable_handle() {
        crate::on_ruby_thread(|| {
            let global = VM::define_variable("$rutie_global_variable_test", Fixnum::new(1));
            assert_eq!(global.get().try_convert_to::<Fixnum>(), Ok(Fixnum::new(1)));

            global.set(RString::new_utf8("changed"));
            GC::start();

            let from_ruby = VM::eval("$rutie_global_variable_test").unwrap();
            assert_eq!(
                from_ruby.try_convert_to::<RString>().unwrap().to_str(),
                "changed"
            );

            VM::eval("$rutie_global_variable_test = 7").unwrap();
            assert_eq!(global.get().try_convert_to::<Fixnum>(), Ok(Fixnum::new(7)));
        });
    }
}
