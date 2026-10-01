// The fiber scheduler interface (`ruby/fiber/scheduler.h`): getting and
// setting a thread's scheduler, and the hooks C code calls to hand a
// blocking operation over to the current scheduler.

use crate::rubysys::{
    libc::timeval,
    types::{c_int, c_void, size_t, Argc, CallbackPtr, Value},
};

// `rb_pid_t`: `pid_t`, which is `int` on Unix and with Visual C++, and a
// 64-bit `__int64` with MinGW-w64 on 64-bit Windows.
#[cfg(unix)]
pub type RbPid = crate::rubysys::libc::pid_t;
#[cfg(all(windows, target_env = "gnu"))]
pub type RbPid = isize;
#[cfg(all(windows, not(target_env = "gnu")))]
pub type RbPid = c_int;

// `rb_off_t` (Ruby 3.2+) and `off_t` (Ruby 3.1): Ruby is built with a 64-bit
// file offset everywhere (large file support on Unix, `__int64` on Windows).
pub type RbOff = i64;

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // The scheduler of the current thread (`Fiber.scheduler`), or `Qnil`.
    //
    // VALUE
    // rb_fiber_scheduler_get(void)
    pub fn rb_fiber_scheduler_get() -> Value;
    // Sets the scheduler of the current thread (`Fiber.set_scheduler`),
    // closing the previous one. Raises an `ArgumentError` when `scheduler`
    // lacks one of the required hooks (`block`, `unblock`, `kernel_sleep`
    // and `io_wait`).
    //
    // VALUE
    // rb_fiber_scheduler_set(VALUE scheduler)
    pub fn rb_fiber_scheduler_set(scheduler: Value) -> Value;
    // The scheduler of the current thread if its current fiber is
    // non-blocking, `Qnil` otherwise.
    //
    // VALUE
    // rb_fiber_scheduler_current(void)
    pub fn rb_fiber_scheduler_current() -> Value;
    // Like `rb_fiber_scheduler_current`, for `thread`.
    //
    // VALUE
    // rb_fiber_scheduler_current_for_thread(VALUE thread)
    pub fn rb_fiber_scheduler_current_for_thread(thread: Value) -> Value;
    // Converts `timeout` (NULL for none) to what the hooks take: `Qnil` or
    // a `Float` of seconds.
    //
    // VALUE
    // rb_fiber_scheduler_make_timeout(struct timeval *timeout)
    pub fn rb_fiber_scheduler_make_timeout(timeout: *mut timeval) -> Value;
    // VALUE
    // rb_fiber_scheduler_close(VALUE scheduler)
    pub fn rb_fiber_scheduler_close(scheduler: Value) -> Value;
    // VALUE
    // rb_fiber_scheduler_kernel_sleep(VALUE scheduler, VALUE duration)
    pub fn rb_fiber_scheduler_kernel_sleep(scheduler: Value, duration: Value) -> Value;
    // VALUE
    // rb_fiber_scheduler_kernel_sleepv(VALUE scheduler, int argc, VALUE * argv)
    pub fn rb_fiber_scheduler_kernel_sleepv(
        scheduler: Value,
        argc: Argc,
        argv: *mut Value,
    ) -> Value;
    // VALUE
    // rb_fiber_scheduler_process_wait(VALUE scheduler, rb_pid_t pid, int flags)
    pub fn rb_fiber_scheduler_process_wait(scheduler: Value, pid: RbPid, flags: c_int) -> Value;
    // VALUE
    // rb_fiber_scheduler_block(VALUE scheduler, VALUE blocker, VALUE timeout)
    pub fn rb_fiber_scheduler_block(scheduler: Value, blocker: Value, timeout: Value) -> Value;
    // VALUE
    // rb_fiber_scheduler_unblock(VALUE scheduler, VALUE blocker, VALUE fiber)
    pub fn rb_fiber_scheduler_unblock(scheduler: Value, blocker: Value, fiber: Value) -> Value;
    // VALUE
    // rb_fiber_scheduler_io_wait(VALUE scheduler, VALUE io, VALUE events, VALUE timeout)
    pub fn rb_fiber_scheduler_io_wait(
        scheduler: Value,
        io: Value,
        events: Value,
        timeout: Value,
    ) -> Value;
    // VALUE
    // rb_fiber_scheduler_io_wait_readable(VALUE scheduler, VALUE io)
    pub fn rb_fiber_scheduler_io_wait_readable(scheduler: Value, io: Value) -> Value;
    // VALUE
    // rb_fiber_scheduler_io_wait_writable(VALUE scheduler, VALUE io)
    pub fn rb_fiber_scheduler_io_wait_writable(scheduler: Value, io: Value) -> Value;
    // VALUE
    // rb_fiber_scheduler_io_select(VALUE scheduler, VALUE readables, VALUE writables,
    //                              VALUE exceptables, VALUE timeout)
    pub fn rb_fiber_scheduler_io_select(
        scheduler: Value,
        readables: Value,
        writables: Value,
        exceptables: Value,
        timeout: Value,
    ) -> Value;
    // VALUE
    // rb_fiber_scheduler_io_selectv(VALUE scheduler, int argc, VALUE *argv)
    pub fn rb_fiber_scheduler_io_selectv(scheduler: Value, argc: Argc, argv: *mut Value) -> Value;
    // VALUE
    // rb_fiber_scheduler_io_read(VALUE scheduler, VALUE io, VALUE buffer, size_t length)
    // VALUE
    // rb_fiber_scheduler_io_read(VALUE scheduler, VALUE io, VALUE buffer, size_t length,
    //                            size_t offset)
    pub fn rb_fiber_scheduler_io_read(
        scheduler: Value,
        io: Value,
        buffer: Value,
        length: size_t,
        offset: size_t,
    ) -> Value;
    // VALUE
    // rb_fiber_scheduler_io_write(VALUE scheduler, VALUE io, VALUE buffer, size_t length)
    // VALUE
    // rb_fiber_scheduler_io_write(VALUE scheduler, VALUE io, VALUE buffer, size_t length,
    //                             size_t offset)
    pub fn rb_fiber_scheduler_io_write(
        scheduler: Value,
        io: Value,
        buffer: Value,
        length: size_t,
        offset: size_t,
    ) -> Value;
    // VALUE
    // rb_fiber_scheduler_io_pread(VALUE scheduler, VALUE io, VALUE buffer, size_t length,
    //                             off_t offset)
    // VALUE
    // rb_fiber_scheduler_io_pread(VALUE scheduler, VALUE io, rb_off_t from, VALUE buffer,
    //                             size_t length, size_t offset)
    pub fn rb_fiber_scheduler_io_pread(
        scheduler: Value,
        io: Value,
        from: RbOff,
        buffer: Value,
        length: size_t,
        offset: size_t,
    ) -> Value;
    // VALUE
    // rb_fiber_scheduler_io_pwrite(VALUE scheduler, VALUE io, VALUE buffer, size_t length,
    //                              off_t offset)
    // VALUE
    // rb_fiber_scheduler_io_pwrite(VALUE scheduler, VALUE io, rb_off_t from, VALUE buffer,
    //                              size_t length, size_t offset)
    pub fn rb_fiber_scheduler_io_pwrite(
        scheduler: Value,
        io: Value,
        from: RbOff,
        buffer: Value,
        length: size_t,
        offset: size_t,
    ) -> Value;
    // VALUE
    // rb_fiber_scheduler_io_read_memory(VALUE scheduler, VALUE io, void *base, size_t size,
    //                                   size_t length)
    pub fn rb_fiber_scheduler_io_read_memory(
        scheduler: Value,
        io: Value,
        base: *mut c_void,
        size: size_t,
        length: size_t,
    ) -> Value;
    // VALUE
    // rb_fiber_scheduler_io_write_memory(VALUE scheduler, VALUE io, const void *base,
    //                                    size_t size, size_t length)
    pub fn rb_fiber_scheduler_io_write_memory(
        scheduler: Value,
        io: Value,
        base: *const c_void,
        size: size_t,
        length: size_t,
    ) -> Value;
    // VALUE
    // rb_fiber_scheduler_io_pread_memory(VALUE scheduler, VALUE io, rb_off_t from, void *base,
    //                                    size_t size, size_t length)
    #[cfg(ruby_gte_3_3)]
    pub fn rb_fiber_scheduler_io_pread_memory(
        scheduler: Value,
        io: Value,
        from: RbOff,
        base: *mut c_void,
        size: size_t,
        length: size_t,
    ) -> Value;
    // VALUE
    // rb_fiber_scheduler_io_pwrite_memory(VALUE scheduler, VALUE io, rb_off_t from,
    //                                     const void *base, size_t size, size_t length)
    #[cfg(ruby_gte_3_3)]
    pub fn rb_fiber_scheduler_io_pwrite_memory(
        scheduler: Value,
        io: Value,
        from: RbOff,
        base: *const c_void,
        size: size_t,
        length: size_t,
    ) -> Value;
    // VALUE
    // rb_fiber_scheduler_io_close(VALUE scheduler, VALUE io)
    pub fn rb_fiber_scheduler_io_close(scheduler: Value, io: Value) -> Value;
    // VALUE
    // rb_fiber_scheduler_address_resolve(VALUE scheduler, VALUE hostname)
    pub fn rb_fiber_scheduler_address_resolve(scheduler: Value, hostname: Value) -> Value;
    // Creates a non-blocking fiber through the scheduler's `fiber` hook
    // (`Fiber.schedule`); `argv` are the arguments of `Fiber.new`.
    //
    // VALUE
    // rb_fiber_scheduler_fiber(VALUE scheduler, int argc, VALUE *argv, int kw_splat)
    pub fn rb_fiber_scheduler_fiber(
        scheduler: Value,
        argc: Argc,
        argv: *mut Value,
        kw_splat: c_int,
    ) -> Value;
}

// What `rb_fiber_scheduler_blocking_operation_wait` reports back (Ruby 3.4).
#[cfg(ruby_gte_3_4)]
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct RbFiberSchedulerBlockingOperationState {
    pub result: *mut c_void,
    pub saved_errno: c_int,
}

#[cfg(ruby_gte_3_4)]
#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // Runs `function(data)` (a blocking operation that does not need the GVL)
    // through the scheduler's `blocking_operation_wait` hook, which may run it
    // on another thread; `flags` are `rb_nogvl` flags.
    //
    // VALUE
    // rb_fiber_scheduler_blocking_operation_wait(VALUE scheduler, void* (*function)(void *),
    //     void *data, rb_unblock_function_t *unblock_function, void *data2, int flags,
    //     struct rb_fiber_scheduler_blocking_operation_state *state)
    pub fn rb_fiber_scheduler_blocking_operation_wait(
        scheduler: Value,
        function: CallbackPtr,
        data: *mut c_void,
        unblock_function: CallbackPtr,
        data2: *mut c_void,
        flags: c_int,
        state: *mut RbFiberSchedulerBlockingOperationState,
    ) -> Value;
}
