use std::{convert::From, marker::PhantomData, time::Duration};

use crate::{
    binding::{thread, vm},
    types::Value,
    AnyException, AnyObject, Class, Float, NilClass, Object, VerifiedObject,
};

/// Ruby's `Mutex` (`Thread::Mutex`), for coordinating Ruby threads.
#[derive(Debug)]
#[repr(C)]
pub struct Mutex {
    value: Value,
}

impl Mutex {
    /// Creates an unlocked mutex (`rb_mutex_new`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Mutex, VM};
    /// # VM::init();
    ///
    /// assert!(!Mutex::new().is_locked());
    /// ```
    pub fn new() -> Self {
        Mutex::from(thread::mutex_new())
    }

    /// Locks the mutex, waiting for other threads to release it
    /// (`rb_mutex_lock`), and returns a guard that unlocks it when dropped.
    /// Returns the `ThreadError` when the current thread already holds it
    /// or waiting would deadlock.
    ///
    /// A Ruby exception unwinding through Rust frames does not run Rust
    /// destructors, so when the code holding the lock calls Ruby methods
    /// that may raise, prefer [`synchronize`](#method.synchronize).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Mutex, VM};
    /// # VM::init();
    ///
    /// let mutex = Mutex::new();
    ///
    /// {
    ///     let _guard = mutex.lock().unwrap();
    ///     assert!(mutex.is_locked());
    ///
    ///     // Locking again from the same thread is an error, not a deadlock.
    ///     assert!(mutex.lock().is_err());
    /// }
    ///
    /// assert!(!mutex.is_locked());
    /// ```
    pub fn lock(&self) -> Result<MutexGuard<'_>, AnyException> {
        let mutex = self.value();

        vm::protect_value(|| thread::mutex_lock(mutex))
            .map(|_| MutexGuard::new(self))
            .map_err(AnyException::from)
    }

    /// Locks the mutex if it is free, returning `None` otherwise
    /// (`rb_mutex_trylock`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Mutex, VM};
    /// # VM::init();
    ///
    /// let mutex = Mutex::new();
    /// let guard = mutex.try_lock().unwrap();
    ///
    /// assert!(mutex.try_lock().is_none());
    ///
    /// drop(guard);
    /// assert!(mutex.try_lock().is_some());
    /// ```
    pub fn try_lock(&self) -> Option<MutexGuard<'_>> {
        if thread::mutex_try_lock(self.value()) {
            Some(MutexGuard::new(self))
        } else {
            None
        }
    }

    /// Returns `true` if some thread holds the mutex (`rb_mutex_locked_p`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Mutex, VM};
    /// # VM::init();
    ///
    /// let mutex = Mutex::new();
    /// let _guard = mutex.lock().unwrap();
    ///
    /// assert!(mutex.is_locked());
    /// ```
    pub fn is_locked(&self) -> bool {
        thread::mutex_is_locked(self.value())
    }

    /// Runs `func` holding the mutex and returns its result
    /// (`rb_mutex_synchronize`). The mutex is released even when `func`
    /// raises a Ruby exception or panics; either is returned as `Err`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Exception, Mutex, VM};
    /// # VM::init();
    ///
    /// let mutex = Mutex::new();
    ///
    /// let answer = mutex.synchronize(|| {
    ///     assert!(mutex.is_locked());
    ///     42
    /// });
    /// assert_eq!(answer.unwrap(), 42);
    ///
    /// let failed = mutex.synchronize(|| unsafe { VM::eval_str("raise 'inside'") });
    /// assert_eq!(failed.unwrap_err().message(), "inside");
    /// assert!(!mutex.is_locked());
    /// ```
    pub fn synchronize<F, R>(&self, func: F) -> Result<R, AnyException>
    where
        F: FnOnce() -> R,
    {
        let mutex = self.value();
        let mut result = None;

        vm::protect_value(|| {
            thread::mutex_synchronize(mutex, || {
                result = Some(func());

                NilClass::new().value()
            })
        })
        .map_err(AnyException::from)?;

        Ok(result.expect("synchronized block did not run"))
    }
}

impl Default for Mutex {
    fn default() -> Self {
        Mutex::new()
    }
}

/// Holds a [`Mutex`](struct.Mutex.html) locked; unlocks it when dropped.
///
/// It cannot be sent to another thread, since only the locking Ruby thread
/// may unlock.
#[derive(Debug)]
pub struct MutexGuard<'a> {
    mutex: &'a Mutex,
    _not_send: PhantomData<*const ()>,
}

impl<'a> MutexGuard<'a> {
    fn new(mutex: &'a Mutex) -> Self {
        MutexGuard {
            mutex,
            _not_send: PhantomData,
        }
    }

    /// Releases the lock and sleeps until woken or until `timeout` passes,
    /// then locks it again (Ruby's `Mutex#sleep`, `rb_mutex_sleep`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Mutex, VM};
    /// use std::time::Duration;
    /// # VM::init();
    ///
    /// let mutex = Mutex::new();
    /// let guard = mutex.lock().unwrap();
    ///
    /// guard.sleep(Some(Duration::from_millis(10))).unwrap();
    ///
    /// assert!(mutex.is_locked());
    /// ```
    pub fn sleep(&self, timeout: Option<Duration>) -> Result<(), AnyException> {
        let mutex = self.mutex.value();
        let timeout = timeout
            .map(|timeout| Float::new(timeout.as_secs_f64()).value())
            .unwrap_or_else(|| NilClass::new().value());

        vm::protect_value(|| thread::mutex_sleep(mutex, timeout))
            .map(|_| ())
            .map_err(AnyException::from)
    }
}

impl<'a> Drop for MutexGuard<'a> {
    fn drop(&mut self) {
        let mutex = self.mutex.value();

        // Only fails if the lock was already released some other way.
        let _ = vm::protect_value(|| thread::mutex_unlock(mutex));
    }
}

impl From<Value> for Mutex {
    fn from(value: Value) -> Self {
        Mutex { value }
    }
}

impl Into<Value> for Mutex {
    fn into(self) -> Value {
        self.value
    }
}

impl Into<AnyObject> for Mutex {
    fn into(self) -> AnyObject {
        AnyObject::from(self.value)
    }
}

impl Object for Mutex {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for Mutex {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        Class::from_existing("Mutex").case_equals(object)
    }

    fn error_message() -> &'static str {
        "Error converting to Mutex"
    }
}

impl PartialEq for Mutex {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Exception, Mutex, Object, VM};
    use std::time::Duration;

    #[test]
    fn test_mutex() {
        crate::on_ruby_thread(|| {
            let mutex = Mutex::new();

            {
                let guard = mutex.lock().unwrap();
                assert!(mutex.try_lock().is_none());
                guard.sleep(Some(Duration::from_millis(1))).unwrap();
                assert!(mutex.is_locked());
            }
            assert!(!mutex.is_locked());

            // Shared with Ruby code, which sees the same lock.
            VM::global_set("$rutie_test_mutex", Mutex::from(mutex.value()));
            let held = mutex
                .synchronize(|| {
                    VM::eval("$rutie_test_mutex.owned?")
                        .unwrap()
                        .value()
                        .is_true()
                })
                .unwrap();
            assert!(held);

            let panicked = mutex.synchronize(|| -> i32 { panic!("in synchronize") });
            assert_eq!(
                panicked.unwrap_err().message(),
                "Rust panic: in synchronize"
            );
            assert!(!mutex.is_locked());

            assert!(VM::eval("Mutex.new")
                .unwrap()
                .try_convert_to::<Mutex>()
                .is_ok());
            VM::global_set("$rutie_test_mutex", crate::NilClass::new());
        });
    }
}
