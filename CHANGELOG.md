# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html)
for the public APIs. `rubysys`, even though shared publicly, is considered a private
API and may have breaking changes during a teeny version change.


## [0.10.2] - 2026-09-30
### Added
 - Windows support: Rutie builds, links and passes its tests on 64-bit
   Windows with RubyInstaller's Ruby 2.5, 2.6 and 2.7, with both the MSVC and
   the GNU Rust toolchains, for programs embedding Ruby and for Ruby
   extensions, and Windows is tested in CI, thanks to @danielpclark
 - `Thread::wait_fd` and `Thread::wait_fd_writable` on Windows, where they take
   a descriptor of Ruby's C runtime (`IO#fileno`), thanks to @danielpclark
 - `rutie_callback!` defines a function for Ruby to call (a method body, an
   allocator, ...) with the right ABI, or names its type: `extern "C"`, or
   `extern "C-unwind"` on Windows, thanks to @danielpclark
### Changed
 - Windows: functions Ruby calls that a Ruby exception can pass through are
   `extern "C-unwind"`: `methods!`/`unsafe_methods!` methods, the
   `types::Callback` and `Class::define_alloc_func` types, and the `rubysys`
   callback types. Ruby 2.5 for Windows raises with a `longjmp` that unwinds,
   and Rust 1.81 and later abort when that leaves an `extern "C"` function
   (`STATUS_STACK_BUFFER_OVERRUN`). Windows needs Rust 1.71 or later; write
   hand-written callbacks with `rutie_callback!`. Other platforms are
   unchanged, thanks to @danielpclark
### Fixed
 - Windows: `build.rs` makes the MSVC import library for the Ruby DLL from the
   DLL's export table in `OUT_DIR`, instead of running `dumpbin` and batch
   files and writing into the current directory and `target/`, which failed
   whenever Rutie was a dependency, thanks to @danielpclark
 - Windows: Ruby's global variables (`rb_cObject`, `rb_eRuntimeError`, ...) were
   read from the linker's jump thunks instead of the Ruby DLL, which crashed
   nearly everything; `rubysys` now imports them from the DLL, thanks to @danielpclark
 - Windows: `VM::init` calls `ruby_sysinit` before `ruby_init`, as `ruby.exe`
   does, thanks to @danielpclark
 - Windows: `Integer`/`Fixnum` `to_u32` (Ruby has no `rb_num2uint` where `int`
   and `long` are the same size), and `to_isize`/`to_usize` (Windows' `long` is
   32 bits), thanks to @danielpclark
 - Windows: `VM::sys_fail` and `AnyException::from_io_error` map Win32 error
   codes to `errno` like Ruby does (`ERROR_PATH_NOT_FOUND` is `Errno::ENOENT`),
   thanks to @danielpclark
 - Windows: `Thread::sleep` builds Winsock's `timeval`, thanks to @danielpclark
 - Windows: `cargo run` and `cargo test` find the DLLs the Ruby DLL needs
   (`bin\ruby_builtin_dlls`), thanks to @danielpclark
 - Static linking on Linux: `build.rs` linked `-lruby` (the shared library)
   as a static library, and never looked for the archive Ruby installs. It now
   links the whole `libruby-static.a` (`LIBRUBY_A`) with `LIBS`/`MAINLIBS`,
   exports Ruby's functions from Rutie's tests and examples so a static
   Ruby's extensions load, and explains a missing archive instead of failing
   in the linker, thanks to @danielpclark
 - CI: the static-Ruby jobs never passed `--disable-shared` to RVM (the check
   read `RUBY_STATIC` in the step that sets it), so they built a shared Ruby
   and then linked it statically, thanks to @danielpclark
 - macOS arm64 with Ruby 2.5 and 2.6: `Enumerator` iteration (`next`,
   `peek`, `iter`, ...) and `Fiber` crashed with a segfault since 0.10.0.
   These Rubies have no native fibers there; theirs copy the machine stack
   and start a new fiber by jumping to a frame that an embedded VM only has
   inside `rb_protect`, while Rutie switched fibers under `rb_rescue2`. On
   such Rubies (`rutie_copy_stack_fibers`, set by `build.rs`) fibers are now
   switched under `rb_protect`: `Enumerator::next` from one place (a loop)
   works, and what these Rubies cannot do from Rust, such as resuming a
   `Fiber::new` fiber or one enumerator from different stack depths, returns
   a `FiberError` instead of crashing. `VM::init` and `VM::try_init` also call
   `ruby_init_stack` first on macOS, as `ruby` does. Other platforms and Ruby
   2.7 are unchanged, thanks to @danielpclark

## [0.10.1] - 2026-09-30
### Fixed
 - Build warnings on current Rust: `Object::get_data` and `Object::get_data_mut`
   name the `'a` lifetime on their return types (`mismatched_lifetime_syntaxes`),
   and `rubysys` casts `VALUE`s to `RBasic`/`RArray`/`RString` pointers with
   `as` instead of `mem::transmute` (`integer_to_ptr_transmutes`),
   thanks to @danielpclark

## [0.10.0] - 2026-09-30
### Added
 - `build.rs` sets `ruby_2_5`/`ruby_2_6`/`ruby_2_7` and cumulative
   `ruby_gte_2_5`/`ruby_gte_2_6`/`ruby_gte_2_7` cfg flags for the Ruby it builds
   against, exports the version to dependent crates as `DEP_RUBY_VERSION_MAJOR`
   and `DEP_RUBY_VERSION_MINOR`, and warns when that Ruby is not 2.5–2.7,
   thanks to @danielpclark
 - Exception control flow: `VM::ensure`, `VM::rescue`, `VM::rescue_from`,
   `VM::catch`, `VM::throw`, `VM::iter_break`, `VM::iter_break_value` and
   `VM::jump_tag`. Rust panics inside these closures, blocks and `at_exit`
   handlers are raised as a Ruby `RuntimeError` instead of unwinding into
   Ruby, thanks to @danielpclark
 - `Object::send_with_block` and `Object::protect_send_with_block` call a
   method with a Rust closure as its block, thanks to @danielpclark
 - Argument and state checks: `VM::check_arity`, `VM::raise_arity_error`,
   `VM::raise_zero_division`, `VM::not_implemented`, `VM::sys_fail`,
   `VM::warn`, `VM::warning`, `Object::check_frozen` and `Object::check_type`,
   thanks to @danielpclark
 - Variable-arity methods: `VM::scan_args` (`rb_scan_args` formats such as
   `"21*1:&"`, returning `ScannedArgs`), `VM::get_kwargs` (returning
   `KeywordArgs`) and, on Ruby 2.7, `VM::is_keyword_given`, thanks to @danielpclark
 - `Class::define_alias`, `Class::undef_method`, `Class::define_alloc_func`,
   `Class::undef_alloc_func`, `Module::define_alias`, `Module::undef_method`
   and `Object::call_init`, thanks to @danielpclark
 - `VM::cleanup` shuts the VM down (`ruby_cleanup`), running `at_exit`
   handlers, thanks to @danielpclark
 - `VM::call_protected`, the previous (immediate) behaviour of `VM::at_exit`,
   thanks to @danielpclark
 - `Object` methods: `dup`, `clone_object`, `object_id`, `inspect_object`,
   `as_string`, `is_kind_of`, `is_instance_of`, `method`, `check_send`,
   `send_with_proc`, `send_with_keywords` (Ruby 2.7), `instance_variables`,
   `is_instance_variable_defined`, `remove_instance_variable`, `hash_value`
   and `instance_eval`, thanks to @danielpclark
 - `Class` and `Module` methods: `name`, `path`, `from_path` (nested paths,
   returns an error instead of raising), `is_method_defined`, `inherits`,
   `includes_module`, `module_eval`, `instance_methods`, `class_variable_get`,
   `class_variable_set`, `is_class_variable_defined`, `is_const_defined`,
   `is_const_defined_at` and `const_remove`, thanks to @danielpclark
 - Global variables: `VM::global_get`, `VM::global_set`,
   `VM::protect_global_set`, `VM::define_variable` and
   `VM::define_readonly_variable` (returning a `GlobalVariable` handle) and
   `VM::define_virtual_variable` (Rust closures as getter and setter), plus
   `VM::define_global_const`, thanks to @danielpclark
 - `Symbol::find` (looks a symbol up without creating it), `Symbol::from_rstring`,
   `Symbol::to_rstring`, `Symbol::is_const_name`,
   `Symbol::is_instance_variable_name` and `Symbol::is_class_variable_name`,
   thanks to @danielpclark
 - Kernel conversions returning `Result`: `RString::convert`, `Array::convert`,
   `Integer::convert`, `Float::convert` and `Hash::convert`; `VM::format`
   (Ruby's `format`) and `VM::p`, thanks to @danielpclark
 - `ScannedArgs`, `KeywordArgs` and `GlobalVariable` are exported from the
   crate root, thanks to @danielpclark
 - `RString` methods: `with_capacity`, `capacity`, `compare` (and
   `PartialOrd`), `ellipsize`, `plus`, `replace`, `truncate`, `scrub`, `split`,
   `byte_slice` (bounds-checked), `substr`, `times`, `to_i`, `parse_integer`,
   `to_f`, `parse_float`, `coderange` (with the new `CodeRange` enum) and
   `with_locked_bytes` (borrows the bytes while Ruby can't modify the string),
   thanks to @danielpclark
 - `Array` methods: `delete`, `delete_at`, `includes`, `clear`, `slice`,
   `plus`, `compare`, `replace`, `resize`, `rotate_bang`, `assoc`, `rassoc`,
   and `TryConvert` (`Array.try_convert`), thanks to @danielpclark
 - `Hash` methods: `lookup` (ignores the default), `has_key`, `fetch`, `keys`,
   `values`, `update`, `set_default`, `iter` (with `HashIterator`) and
   `TryConvert` (`Hash.try_convert`), thanks to @danielpclark
 - `Integer`: `from_str_radix`, `to_s_radix`, `is_bignum`, `to_i128`,
   `to_u128`, `to_f64`, `add`, `sub`, `mul`, `div`, `modulo`, `pow`,
   `compare` (and `PartialOrd`); `From<i128>`, `From<u128>`,
   `TryFrom<f64>`, and `TryFrom<Integer>` for `i128` and `u128`,
   thanks to @danielpclark
 - New `Rational` and `Complex` types, and `Float::rationalize`,
   thanks to @danielpclark
 - New `Range` type (`new`, `begin`, `end`, `excludes_end`,
   `offset_and_length`, and on Ruby 2.6+ `arithmetic_sequence`),
   thanks to @danielpclark
 - New `Regexp` (`new`, `source`, `options`, `find`, `match_data`) and
   `MatchData` (`nth`, `named`, `matched`, `pre_match`, `post_match`, `last`,
   `set_last`) types, thanks to @danielpclark
 - New `Time` type (`now`, `from_unix`, `at`, `to_unix`, `to_system_time`,
   `utc_offset`, `interval`, `From<SystemTime>`), thanks to @danielpclark
 - New `Struct` type (`define`, `define_under`, `new_instance`, `members_of`,
   `get`, `at`, `set`, `members`, `size`), thanks to @danielpclark
 - `Proc::new` (a Ruby `Proc` backed by a Rust closure, freed with the proc),
   `Proc::arity` and `Proc::protect_call`; new `Method` type (`call`,
   `protect_call`, `arity`, `to_proc`) returned by `Object::method`;
   `Object::method_arity` and `Class`/`Module::instance_method_arity`,
   thanks to @danielpclark
 - `Enumerator::new` (`to_enum`) and Rust iteration over enumerators
   (`Enumerator::iter`, `IntoIterator`, `EnumeratorIterator`);
   `Object::try_compare` (`<=>` with Ruby's comparison error),
   thanks to @danielpclark
 - `VM::yield_values` and `VM::need_block`, thanks to @danielpclark
 - `Binding` methods: `local_variable_get`, `local_variable_set`,
   `is_local_variable_defined`, `local_variables`, `receiver` and `eval`,
   thanks to @danielpclark
 - `Encoding` methods: `ascii_8bit`, `locale`, `filesystem`, `of`, `index`,
   `chr`, `is_ascii_compatible` and `is_dummy`; `RString::concat_bytes`,
   thanks to @danielpclark
 - Typed accessors for Ruby's built-in classes (`Class::array()`,
   `Class::string()`, `Class::struct_class()`, ...), exception classes
   (`Class::standard_error()`, `Class::argument_error()`, ...) and modules
   (`Module::kernel()`, `Module::enumerable()`, ...), read from Ruby's
   `rb_c*`/`rb_e*`/`rb_m*` globals, thanks to @danielpclark
 - `AnyException::from_class`, `AnyException::from_errno`,
   `AnyException::from_io_error` and `VM::raise_interrupt`,
   thanks to @danielpclark
 - New `IO` type (`stdin`, `stdout`, `stderr`, `write`, `puts`, `print`,
   `gets`, `getbyte`, `flush`, `close`, `is_closed`, `is_eof`, `binmode`) and
   `File` type (`open`, `expand_path`, `absolute_path`, `dirname`,
   `current_directory`, and the `IO` methods through `Deref`); operations
   that can raise return `Result`, thanks to @danielpclark
 - `Marshal::dump` and `Marshal::load`, thanks to @danielpclark
 - `GC::define_finalizer`, `GC::undefine_finalizer`, `GC::latest_info`,
   `GC::write_barrier` and `GC::write_barrier_unprotect`,
   thanks to @danielpclark
 - `VM::load`, `VM::protect_require`, `VM::provide`, `VM::is_provided`,
   `VM::add_load_path` and `VM::find_file`, thanks to @danielpclark
 - `Thread` methods: `current`, `main`, `is_alone`, `pass`, `sleep`,
   `check_interrupts`, `wait_fd_writable`, `join`, `join_value`, `is_alive`,
   `kill`, `wakeup`, `local_get`, `local_set`, and `call_without_gvl_io`
   (Ruby's `RUBY_UBF_IO` unblocking function), thanks to @danielpclark
 - New `Mutex` type (`new`, `lock`, `try_lock`, `is_locked`, `synchronize`)
   with a `MutexGuard` that unlocks on drop and can `sleep`,
   thanks to @danielpclark
 - New `Fiber` type (`new` from a Rust closure, `resume`, `yield_values`,
   `current`, `is_alive`), thanks to @danielpclark
 - `VM::raise_message` raises with a plain-text message; `VM::raise` passes
   its message to `rb_raise` as a printf format (unchanged, now documented),
   so `%` must be written `%%` there, thanks to @danielpclark
 - `methods!` accepts a trailing splat parameter (`fn log(level: RString, *parts)`)
   that receives the remaining arguments as an `Array`, thanks to @danielpclark
 - `wrappable_struct!` accepts an optional `size(data) { .. }` clause (Ruby's
   `dsize`, reported by `ObjectSpace.memsize_of`), thanks to @danielpclark
 - VM lifecycle: `VM::try_init` (returns the error instead of exiting),
   `VM::init_with_args`, `VM::set_argv`, `VM::set_script_name`,
   `VM::run_file` (runs a script as the main program, once per process),
   `VM::is_initialized`, `VM::is_ruby_thread`, `VM::is_stack_near_limit`,
   `VM::stack_length` and `VM::at_vm_exit` (`ruby_vm_at_exit`),
   thanks to @danielpclark

### Changed
 - Every public item's documentation example now runs and asserts its
   result (no more `ignore`/`no_run` examples), thanks to @danielpclark
 - **Breaking:** `VM::at_exit` now registers a real end proc
   (`rb_set_end_proc`) that runs when the VM shuts down, instead of calling
   the closure immediately. The closure must be `FnOnce(VmPointer) + 'static`,
   thanks to @danielpclark
 - Unit tests run on a single dedicated Ruby thread (`on_ruby_thread`), since
   Ruby 2 must be used from the thread that started it, thanks to @danielpclark
 - Reverted the `rb-sys` integration (PR #172) and returned to Rutie's own
   hand-maintained FFI bindings (`rubysys`). Rutie targets Ruby 2 (2.5, 2.6
   and 2.7) again; Ruby 3 support will be revisited once Ruby 2 support is
   complete, thanks to @danielpclark
 - Static Ruby and Windows CI jobs are best-effort and no longer fail the
   workflow, thanks to @danielpclark

### Removed
 - The `rubysys` declaration of `rb_str_valid_encoding_p`, a `static` function
   Ruby never exports (unused; `rubysys` is a private API), thanks to @danielpclark
 - The unpublished `rb-sys`-based tree that lived on `master` from February
   2025 (self-labelled 0.10.0, tested only against Ruby 2.7 and 3.0-3.4). It
   was never released to crates.io and is not supported; users of it via a git
   dependency should pin their commit or move to 0.10.x, thanks to @danielpclark

### Fixed
 - `RString::encode` with options aborted Ruby (`[BUG] rb_econv_open_opts
   called with invalid opthash`): the prepared options Ruby writes back were
   discarded and the raw hash passed on instead, thanks to @danielpclark
 - `GC::register` registered the address of a temporary copy of the object
   with `rb_gc_register_address`, so the object was not protected and the GC
   kept reading a stale stack slot; `GC::unregister` never removed it.
   Registered objects are now kept alive in a GC-rooted identity table
   (counted, so each `register` needs one `unregister`), thanks to @danielpclark
 - `RString::new_usascii_unchecked` is documented as creating an
   `ASCII-8BIT` string, which is what it has always done, thanks to @danielpclark
 - `wrappable_struct!` and `methods!` can be called by path
   (`rutie::wrappable_struct!`) without importing the macro,
   thanks to @danielpclark
 - `Enumerator::next`, `next_values`, `peek`, `peek_values` and `feed` could
   fail with `FiberError: fiber called across stack rewinding barrier` when
   called from different stack depths (for example the first and later
   `next` calls made by `Iterator::collect`); they now rescue exceptions
   without `rb_protect`, thanks to @danielpclark
 - `try_convert_to::<Encoding>()` always failed, because `Encoding` objects
   were expected to be classes, thanks to @danielpclark
 - `VM::at_exit` called its closure through the wrong argument, which crashed
   on aarch64 macOS and with capturing closures on every platform; that
   immediate call (now `VM::call_protected`, see Changed) uses a proper
   single-argument `rb_protect` callback and has a regression test,
   thanks to @danielpclark
 - `build.rs` now works with current Cargo, which no longer puts
   `target/<profile>/deps` on the library path when running test binaries and
   doctests: `libruby` is linked into `target/<profile>` as well and exposed
   as a native search path, thanks to @danielpclark
 - macOS CI builds Ruby 2 against a source-built OpenSSL 1.1.1 (Homebrew no
   longer ships `openssl@1.1`), keeps `openssl@3` out of the `openssl`
   extension via `PKG_CONFIG_PATH`, and relaxes clang's implicit-declaration
   error for Ruby 2's C sources, thanks to @danielpclark

## [0.9.0] - 2023-12-17
### Added
 - Support for trailing comma in method macros, thanks to @andrewtbiehl
 - Github actions testing support, thanks to @danielpclark & @striezel
 - Rust 2021 Edition, thanks to @goyox86
 - Support for compiling on OpenBSD, thanks to @marvinthepa

### Fixed
 - removed warnings for allow(unused_mut) and allow(unused_variables) on methods! rtself arg, thanks to @danlarkin
 - internal ruby string length check, thanks to @mpalmer

## [0.8.4] - 2022-03-29
### Added
 - Implement `Eq` and `Hash` for `Symbol`, thanks to @ahogappa0613

### Fixed
 - not FFI-safe warnings when Rutie structs are used as return types, thanks to @ankane

## [0.8.3] - 2021-09-06
### Added
 - Implement `Integer::to_u32` and `Fixnum::to_u32`, thanks to @Hywan

### Fixed
 - Docstring description for `to_enum` on `Array`, thanks to @jhwiig
 - `methods!` macro now uses `:ty` for return type instead of `:ident`, thanks to @n8ta

## [0.8.2] - 2021-02-09
### Added
 - Implement `VM::call_super` -> `rb_call_super`, thanks to @askreet

### Changed
 - Allow commas after methods macros, thanks to @gemmaro

### Fixed
 - Build for FreeBSD, thanks to @Stazer
 - Issue when Rutie dependency included with relative path

## [0.8.1] - 2020-09-28
### Added
- cargo feature `no-link` disables linking to `libruby`, thanks to @danlarkin
- initial changes for Android support, thanks to @Riey

### Changed
- Optimized equality methods, thanks to @asppsa

### Fixed
- `Hash::each` (via `binding::hash::each`) now calls `rubysys::rb_hash_foreach` with a
  callback that properly returns a `st_retval` instead of `()`, thanks to @danlarkin
- fixed build warnings due to trait objects, thanks to @danlarkin
- With dropping support for Ruby 2.4 we can now perfectly model the `force_encoding` method
  which checks if the object is frozen and raises the appropriate error.

## [0.7.0] - 2019-08-19
### Added
- `VM::error_pop` to get the Ruby Exception and remove it from interfering
  with the current thread
- `VM::exit` to exit the Ruby VM with status code given
- `VM::exit_bang` to exit skipping exit handlers
- `NilClass` has had `Copy` and `Clone` derived on it
- Readme section for Ruby's Future and SemVer
- `VM::abort` exit the Ruby VM via abort
- `VM::trap` for signal handling
- `VM::at_exit` for executing Rust code after the Ruby VM stops
- `Float::implicit_to_f`

### Changed
- `VM::protect` takes a function that now returns an `AnyObject` instead of a `Value`.
  `VM::protect` will become more frequently used and encouraged which is why this change
  is necessary as `Value` is meant to be internal.
- Avoid showing `Value` or `.value()` in any documentation.  Prefer `.into()` when necessary.
  `Value` should always be treated as a private API.

## [0.6.1] - 2019-06-18
### Added
- `Encoding::is_compatible` which is the same as Ruby's `Encoding.compatible?`

## [0.6.0] - 2019-06-16
### Changed
- Updated `libc` and `lazy_static` dependency versions.

## [0.6.0-rc.2] - 2019-05-16
### Fixed
- Restored use of `Path` for Windows `build.rs` which had been removed in 0.5.5

## [0.5.6] - 2019-05-16
### Fixed
- Restored use of `Path` for Windows `build.rs` which had been removed in 0.5.5

## [0.6.0-rc.1] - 2019-05-16
### Changed
- Methods that took type `Option<&[]>` now take only type `&[]`, thanks to @dsander

## [0.5.5] - 2019-05-13
### Added
- Safety policy in README
- `Fixnum.to_u64`, thanks to @irxground
- `Integer.to_u64`, thanks to @irxground
- `impl From<u64> for Integer`, thanks to @irxground
- `impl Into<u64> for Integer`, thanks to @irxground
- `impl From<i32> for Integer`, thanks to @irxground
- `impl From<u32> for Integer`, thanks to @irxground
- `impl Into<u32> for Integer`, thanks to @irxground
- `rubysys::fixnum::rb_uint2inum`, thanks to @irxground
- `rubysys::fixnum::rb_ll2inum`, thanks to @irxground
- `rubysys::fixnum::rb_ull2inum`, thanks to @irxground
- `rubysys::fixnum::rb_num2short`, thanks to @irxground
- `rubysys::fixnum::rb_num2ushort`, thanks to @irxground
- `rubysys::fixnum::rb_num2uint`, thanks to @irxground
- `rubysys::fixnum::rb_num2ulong`, thanks to @irxground
- `rubysys::fixnum::rb_num2ll`, thanks to @irxground
- `rubysys::fixnum::rb_num2ull`, thanks to @irxground

### Changed
- Integer `is_correct_type` to permit Bignum, thanks to @irxground
- `rubysys::fixnum::rb_num2int` returns `libc::c_long` rather than `c_int`, thanks to @irxground

### Fixed
- symlink check in `build.rs` which had rare systems in which `exists` didn't work on symlink, thanks to @ekump

## [0.5.4] - 2019-04-15
### Added
- `GC::adjust_memory_usage`, thanks to @Antti
- `examples/rutie_ruby_gvl_example`, thanks to @dsander
- `GC::count`
- `GC::disable`
- `GC::enable`
- `GC::force_recycle`
- `GC::mark_locations`
- `GC::mark_maybe`
- `GC::register`
- `GC::start`
- `GC::stat`
- `GC::unregister`
- `util::inmost_rb_object` which is a string recurse tool to get nested ruby objects

### Fixed
- `GC::mark` documentation notes.
- `util::closure_to_ptr` from `'static + FnOnce` to `FnMut`, thanks to @dsander
- `Thread::new` from `'static + FnOnce` to `FnMut`, thanks to @dsander
- `Thread::call_without_gvl` from `'static + FnOnce` to `FnMut`, thanks to @dsander
- `Thread::call_without_gvl2` from `'static + FnOnce` to `FnMut`, thanks to @dsander
- `Thread::call_with_gvl` from `'static + FnOnce` to `FnMut`, thanks to @dsander
- `AnyException::new` to work with nested exception classes

## [0.5.3] - 2019-01-10
### Added
- `util::is_proc` & `util::is_method`
- `rb_enc_compatible` useful for internal string encoding compatibility checks from
  which we now have `binding::is_compatible_encoding` and `binding::compatible_encoding`
- `RString.compatible_with` as the public API for `rb_enc_compatible` with trait `EncodingSupport`
- `RString::compatible_encoding` as the public API for `rb_enc_compatible` with trait `EncodingSupport`
- `impl Deref for AnyException`
- `impl Deref for AnyObject`
- `impl Borrow<Value> for AnyObject`
- `impl Borrow<Value> for AnyException`
- `impl AsRef<Value> for AnyObject`
- `impl AsRef<Value> for AnyException`
- `impl AsRef<AnyObject> for AnyObject`
- `impl AsRef<AnyException> for AnyException`
- `impl<T: Object> From<&T> for AnyObject`

### Changed
- Removed Ruby 2.3 support & added 2.6
- `VM::raise_ex` now accepts `Into<AnyException>` rather than just `AnyException`
- Refactor internal encoding types
- Refactor `build.rs` script to use Ruby provided cflags

### Removed
- pkg-config-rs removed from Rutie and from the build process

## [0.5.2] - 2018-12-18
### Added
- `impl Into<i32> for Integer` thanks to @Antti
- `Integer.to_i32`, thanks to @Antti
- `Fixname.to_i32`, thanks to @Antti

### Fixed
- `Integer.to_i64` to use `rb_num2long` for genuine `i64` result, thanks to @Antti
- `impl Into<i64> for Integer` to use `rb_num2long` for genuine `i64` result, thanks to @Antti
- `Fixname.to_i64` to use `rb_num2long` for genuine `i64` result, thanks to @Antti

## [0.5.1] - 2018-12-11
### Added
- Windows build support (partially working)
- Mac static build support, thanks to @felix-d
- Rutie pronunciation guide

## [0.5.0] - 2018-10-23
### Changed
- `CodepointIterator` now borrows RString parameter instead of consuming ownership

## [0.4.3] - 2018-10-23
### Fixed
- `RString.codepoints` uses a new internal implementation as `rb_str_codepoints` isn't exported/available on some OSes

## [0.4.2] - 2018-10-16
### Fixed
- Wrapping struct changed from Ruru to Rutie & some of the same changes in documentation, thanks to @turboladen

## [0.4.1] - 2018-10-04
### Added
- Static build support

## [0.4.0] - 2018-08-20
### Added
- Methods `VM::yield_object` and `VM::yield_splat`
- `Enumerator` object
- `Array.to_enum`
- `TryConvert` for `AnyException`
- `VM::error_info` and `VM::clear_error_info`
- Documentation for `VM::protect`
- `Binding`
- `Into<Value>` for all types which `impl Object`
- `Into<AnyObject>` for all types which `impl Object`
- `From<i64>` and `Into<i64>` for `Integer`
- `From<&'static str>` for `RString`
- `eval!()` macro with `binding, filename, linenum` for *optional* arguments
- `rubysys::rproc::check_arity` for simple numeric bounds checking
- `Symbol.to_proc`
- `Proc.is_lambda`

### Changed
- `Object.protect_send` and `Object.protect_public_send` have changed the
  first parameter from requiring `String` to `&str`
- `VM::protect` returns `Result<AnyObject, i32>` rather than `Result<Value, i32>`
- `PartialEq` is now implemented for Ruby objects via the `==` method

## [0.3.4] - 2018-08-08
### Added
- This `CHANGELOG.md` file
- Method `RString.codepoints`
- `CodepointIterator` which uses direct ruby calls to get character value
  from bytes as determeined by the strings own `Encoding` and produces
  them one at a time
- `binding::new_frozen` for internal use with `CodepointIterator`
- `rubysys::string::{rstring_embed_len, rstring_ptr, rstring_end}` to
  match equivalent Ruby C macros for use in `CodepointIterator`

### Changed
- `rubysys::rb_str_len` renamed to `rubysys::rstring_len` to match the name
  of the Ruby C macro which it is a copy of

### Fixed
- `rubysys::string::{RStringAs, RStringHeap, RStringAux}` to match Ruby's
  C code implementation perfectly

### Removed
- CI testing for Rust 1.25 as pointer addition wasn't stable until 1.26

## [0.3.3] - 2018-08-07
### Added
- Full encoding support with `VM::init_loadpath`, `RString.encode`,
  `RString.is_valid_encoding`
- `RString::from_bytes` which takes both a byte sequence for characters
  and an `Encoding` object to interpret how to get those characters from bytes
- Documentation about what to try if binary installs of Ruby panic on CI
  servers
- `rubysys::encoding::{coderange_set, coderange_clear}` and encoding flags
- `EncodingIndex` type for internal use in the `binding` layer

### Changed
- Updated code examples to remove deprecated `RString::new` from them
- TravisCI Linux builds now compile all Rubies

## [0.3.2] - 2018-08-03
### Added
- CI server logging for the Rust build process
- Ruby gem `rutie` version 0.0.3
- Documentation for Ruby gem `rutie`
- Build documentation with `build.md`
- Customization options for using `pkg-config`
- Example CLI eval program in examples directory, thanks to @irxground
- `RString::count_chars`, thanks to @irxground

### Changed
- Refactor of `VM::protect`, thanks to @irxground
- Internally use `RString::new_utf8`
- `TryConvert` moved to `src/class/traits/try_convert.rs` but still shared in root of crate
- Refactor internal method names for `Value` in `src/rubysys/value.rs` to match Ruby source code

### Deprecated
- `RString::new` — use either `RString::new_utf8` or `RString::new_usascii_unchecked`

### Removed
- Use of `fiddle` from examples and documentation

## [0.3.1] - 2018-07-17
### Added
- CI testing for Rust 1.25 for purpose of older match ref syntax

### Changed
- `cargo test` and `cargo build` require the `-vv` flag afterwards in older Rust versions
- refactor `option_to_slice` for Rust 1.25 compatible syntax

## [0.3.0] - 2018-07-17
### Added
- `TryConvert` implicit conversion or `NilClass` result
- `Encoding` and `EncodingSupport`
- `TryConvert` for `RString`
- Majority of Ruby main constants in `src/rubysys/constant.rs`
- `rubysys::class::{rb_define_singleton_method, rb_scan_args}`
- `rubysys::string::{rb_check_string_type, rb_str_locktmp, rb_str_unlocktmp, is_lockedtmp}`
- `is_frozen` check for `Value` and several Ruby macros for `Value`
- `util::option_to_slice`

### Changed
- Refactor Pathname example in README
- Refactor away `util.rs` files from `binding` and `rubysys`
- Refactor away from using heap to stack memory, thanks to @irxground

### Fixed
- A few Ruby `ValueType` flags were incorrect in `rubysys`

## [0.2.2] - 2018-07-07
### Added
- `String#concat`, thanks to @irxground
- Method signatures for all of `rubysys` direct method mappings documented

### Fixed
- `Array.store` does not return anything
- Misnamed `rubysys::string` method `rb_str_ascii_only_p` to `rb_enc_str_asciionly_p`

## [0.2.1] - 2018-06-30
### Added
- OSX testing on Travis CI
- `Cargo.toml` badges for Travis CI and maintenance status
- Full README details
- Ruby & Rust examples

## [0.2.0] - 2018-06-26
### Changed
- Migrated `parse_arguments` from `VM` to `util`

## [0.1.4] - 2018-05-25
### Changed
- Refactor build script

## [0.1.3] - 2018-05-25
### Added
- Verbose CI output for Rust
- Set default `pkg-config` path for Ruby

## [0.1.2] - 2018-05-25
### Added
- `pkg-config` support
- Basic migrating from Ruru to Rutie notes

### Changed
- TravisCI testing to not use feature flag

## [0.1.0] - 2018-05-20
### Added
- `Display` and `Debug` traits for `AnyException`

### Changed
- Migrated from `Error` to `AnyException`
- `Object.protect_send`, `Object.protect_public_send`, `Object.try_convert_to`,
  `VM::eval` method signatures changed to return `AnyException` on `Err`
- Macro DSL to have the error types as `AnyException`

### Removed
- Duplicate thread methods from `VM`
- `result::Error`

## [0.0.3] - 2018-05-20
### Added
- Officially forked [Ruru](https://github.com/d-unseductable/ruru) and renamed
  to `Rutie` with the following Pull Requests merged
  * 79 [eval functions with panic safe implementation](https://github.com/d-unseductable/ruru/pull/79)
  * 80 [Module support](https://github.com/d-unseductable/ruru/pull/80)
  * 82 [Private method and module function](https://github.com/d-unseductable/ruru/pull/82)
  * 87 [Object equality methods](https://github.com/d-unseductable/ruru/pull/87)
  * 88 [Protect send: a panic safe feature for Ruby interaction](https://github.com/d-unseductable/ruru/pull/88)
  * 89 [allocate -> Class](https://github.com/d-unseductable/ruru/pull/89)
  * 93 [Working `Exception` and `AnyException`](https://github.com/d-unseductable/ruru/pull/93)
  * 98 [String methods to convert to `&[u8]` and `Vec<u8>`](https://github.com/d-unseductable/ruru/pull/98)
- Merged [ruby-sys](https://github.com/steveklabnik/ruby-sys) into `src/rubysys` with
  the following Pull Requests merged
  * 26 [private method and module method added](https://github.com/steveklabnik/ruby-sys/pull/26)
  * 27 [Two additional type data check methods](https://github.com/steveklabnik/ruby-sys/pull/27)
  * 28 [Thread specific error setter and getter](https://github.com/steveklabnik/ruby-sys/pull/28)
  * 29 [public send](https://github.com/steveklabnik/ruby-sys/pull/29)
  * 30 [variadic function support](https://github.com/steveklabnik/ruby-sys/pull/30)
  * 33 [Encoding support](https://github.com/steveklabnik/ruby-sys/pull/33)
