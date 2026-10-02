// Native (OS) threads, mutexes and condition variables
// (`ruby/thread_native.h`), for C-level code that runs without the GVL. Ruby
// code and Rust code that only needs Ruby-aware locking should use `Mutex`
// (`rb_mutex_*`) instead; these neither release the GVL while waiting nor
// check for interrupts.
//
// A lock or condition variable must be initialized in place (it may not be
// moved afterwards) and destroyed when done with.

use crate::rubysys::{libc::c_ulong, types::c_int};

#[cfg(windows)]
use crate::rubysys::types::c_void;

// `rb_nativethread_id_t`, `rb_nativethread_lock_t` and
// `rb_nativethread_cond_t`: pthread types on Unix.
#[cfg(unix)]
pub type NativeThreadId = crate::rubysys::libc::pthread_t;
#[cfg(unix)]
pub type NativeThreadLock = crate::rubysys::libc::pthread_mutex_t;
#[cfg(unix)]
pub type NativeThreadCond = crate::rubysys::libc::pthread_cond_t;

// On Windows a `HANDLE`, a union of a `HANDLE` and a `CRITICAL_SECTION`, and
// a pair of list pointers; the storage is at least as large as those.
#[cfg(windows)]
pub type NativeThreadId = *mut c_void;
#[cfg(windows)]
#[repr(C)]
pub struct NativeThreadLock {
    _storage: [usize; 6],
}
#[cfg(windows)]
#[repr(C)]
pub struct NativeThreadCond {
    _storage: [usize; 2],
}

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // void
    // rb_native_cond_broadcast(rb_nativethread_cond_t *cond)
    pub fn rb_native_cond_broadcast(cond: *mut NativeThreadCond);
    // void
    // rb_native_cond_destroy(rb_nativethread_cond_t *cond)
    pub fn rb_native_cond_destroy(cond: *mut NativeThreadCond);
    // void
    // rb_native_cond_initialize(rb_nativethread_cond_t *cond)
    pub fn rb_native_cond_initialize(cond: *mut NativeThreadCond);
    // void
    // rb_native_cond_signal(rb_nativethread_cond_t *cond)
    pub fn rb_native_cond_signal(cond: *mut NativeThreadCond);
    // void
    // rb_native_cond_timedwait(rb_nativethread_cond_t *cond, rb_nativethread_lock_t *mutex,
    //                          unsigned long msec)
    //
    // Like `rb_native_cond_wait`, for at most `msec` milliseconds.
    pub fn rb_native_cond_timedwait(
        cond: *mut NativeThreadCond,
        mutex: *mut NativeThreadLock,
        msec: c_ulong,
    );
    // void
    // rb_native_cond_wait(rb_nativethread_cond_t *cond, rb_nativethread_lock_t *mutex)
    //
    // `mutex` must be locked; it is unlocked while waiting. Wakeups may be
    // spurious.
    pub fn rb_native_cond_wait(cond: *mut NativeThreadCond, mutex: *mut NativeThreadLock);
    // void
    // rb_native_mutex_destroy(rb_nativethread_lock_t *lock)
    pub fn rb_native_mutex_destroy(lock: *mut NativeThreadLock);
    // void
    // rb_native_mutex_initialize(rb_nativethread_lock_t *lock)
    pub fn rb_native_mutex_initialize(lock: *mut NativeThreadLock);
    // void
    // rb_native_mutex_lock(rb_nativethread_lock_t *lock)
    pub fn rb_native_mutex_lock(lock: *mut NativeThreadLock);
    // int
    // rb_native_mutex_trylock(rb_nativethread_lock_t *lock)
    //
    // 0 when locked, else non-zero (`EBUSY`).
    pub fn rb_native_mutex_trylock(lock: *mut NativeThreadLock) -> c_int;
    // void
    // rb_native_mutex_unlock(rb_nativethread_lock_t *lock)
    pub fn rb_native_mutex_unlock(lock: *mut NativeThreadLock);
    // void
    // rb_nativethread_lock_destroy(rb_nativethread_lock_t *lock)
    //
    // The same as `rb_native_mutex_destroy`, as are the other
    // `rb_nativethread_lock_*` functions for their `rb_native_mutex_*` ones.
    pub fn rb_nativethread_lock_destroy(lock: *mut NativeThreadLock);
    // void
    // rb_nativethread_lock_initialize(rb_nativethread_lock_t *lock)
    pub fn rb_nativethread_lock_initialize(lock: *mut NativeThreadLock);
    // void
    // rb_nativethread_lock_lock(rb_nativethread_lock_t *lock)
    pub fn rb_nativethread_lock_lock(lock: *mut NativeThreadLock);
    // void
    // rb_nativethread_lock_unlock(rb_nativethread_lock_t *lock)
    pub fn rb_nativethread_lock_unlock(lock: *mut NativeThreadLock);
    // rb_nativethread_id_t
    // rb_nativethread_self(void)
    pub fn rb_nativethread_self() -> NativeThreadId;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::MaybeUninit;

    #[test]
    fn test_native_locks() {
        crate::on_ruby_thread(|| unsafe {
            let mut lock = MaybeUninit::<NativeThreadLock>::uninit();
            let lock = lock.as_mut_ptr();
            rb_native_mutex_initialize(lock);

            // Held, so another thread cannot take it (the owner could again on
            // Windows, where the lock is a recursive critical section).
            rb_native_mutex_lock(lock);
            let address = lock as usize;
            let busy = std::thread::spawn(move || {
                rb_native_mutex_trylock(address as *mut NativeThreadLock)
            })
            .join()
            .unwrap();
            assert_ne!(busy, 0);
            rb_native_mutex_unlock(lock);
            assert_eq!(rb_native_mutex_trylock(lock), 0);
            rb_native_mutex_unlock(lock);

            rb_nativethread_lock_lock(lock);
            rb_nativethread_lock_unlock(lock);

            let mut cond = MaybeUninit::<NativeThreadCond>::uninit();
            let cond = cond.as_mut_ptr();
            rb_native_cond_initialize(cond);
            rb_native_cond_signal(cond);
            rb_native_cond_broadcast(cond);

            // Nobody signals: the wait times out.
            rb_native_mutex_lock(lock);
            rb_native_cond_timedwait(cond, lock, 1);
            rb_native_mutex_unlock(lock);

            rb_native_cond_destroy(cond);
            rb_native_mutex_destroy(lock);

            let mut other = MaybeUninit::<NativeThreadLock>::uninit();
            rb_nativethread_lock_initialize(other.as_mut_ptr());
            rb_nativethread_lock_destroy(other.as_mut_ptr());

            let _ = rb_nativethread_self();
        });
    }

    #[cfg(unix)]
    #[test]
    fn test_native_condition_variable_across_threads() {
        use std::sync::atomic::{AtomicBool, Ordering};

        struct Shared {
            lock: NativeThreadLock,
            cond: NativeThreadCond,
            ready: AtomicBool,
        }

        crate::on_ruby_thread(|| unsafe {
            let shared: &'static mut Shared =
                Box::leak(Box::new(MaybeUninit::<Shared>::zeroed().assume_init()));
            rb_native_mutex_initialize(&mut shared.lock);
            rb_native_cond_initialize(&mut shared.cond);
            let address = shared as *mut Shared as usize;

            let main_id = rb_nativethread_self();
            let signaller = std::thread::spawn(move || {
                let shared = &mut *(address as *mut Shared);
                assert_eq!(
                    crate::rubysys::libc::pthread_equal(rb_nativethread_self(), main_id),
                    0
                );
                rb_native_mutex_lock(&mut shared.lock);
                shared.ready.store(true, Ordering::SeqCst);
                rb_native_cond_signal(&mut shared.cond);
                rb_native_mutex_unlock(&mut shared.lock);
            });

            rb_native_mutex_lock(&mut shared.lock);
            while !shared.ready.load(Ordering::SeqCst) {
                rb_native_cond_wait(&mut shared.cond, &mut shared.lock);
            }
            rb_native_mutex_unlock(&mut shared.lock);
            signaller.join().unwrap();
        });
    }
}
