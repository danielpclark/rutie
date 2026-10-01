# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html)
for the public APIs. `rubysys`, even though shared publicly, is considered a private
API and may have breaking changes during a teeny version change.


## [0.14.0] - 2026-10-01
Supports Ruby 4.0 on Linux, macOS and Windows; Ruby 3.2–3.4 stay on 0.13.x.

### Added
 - Ruby 4.0 support: `ruby_4_0`/`ruby_gte_4_0` cfg flags, and every `rubysys`
   declaration checked against Ruby 4.0's headers and `libruby` exports,
   thanks to @danielpclark
 - `Set`, over Ruby 4.0's new C API for the core `Set` class (`rb_set_new`,
   `rb_set_new_capa`, `rb_set_add`, `rb_set_lookup`, `rb_set_delete`,
   `rb_set_clear`, `rb_set_size`, `rb_set_foreach`), thanks to @danielpclark
 - `Class::set` and `Class::ruby_box` (`rb_cSet`, `rb_cBox`), thanks to
   @danielpclark
 - `Thread::has_gvl` (`ruby_thread_has_gvl_p`). `Thread::call_with_gvl` may
   also be called with the GVL held on Ruby 4.0, thanks to @danielpclark
 - `Random`: `new`, `with_seed`, `seed`, `int32`, `real`, `ulong_limited`,
   `bytes`, the default generator's `default_int32`, `default_real`,
   `default_ulong_limited` and `reset_default_seed`, and `int_pair_to_real`,
   thanks to @danielpclark
 - `Integer::bit_and`, `bit_or`, `bit_xor`, `shift_left`, `shift_right`,
   `divmod`, `abs_num_words`, `abs_is_power_of_two`, `to_long_words` and
   `Integer::from_long_words`, thanks to @danielpclark
 - `Fixnum::try_to_i16`, `try_to_u16`, `try_to_i32` and `try_to_u32`, which
   return the `RangeError` instead of raising it, thanks to @danielpclark
 - `Float::rationalize_with_precision`, and `PartialOrd` for `Float`,
   thanks to @danielpclark
 - `Complex::add`, `sub`, `mul`, `div`, `pow`, `neg` and `conjugate`,
   thanks to @danielpclark
 - `RString::new_external`, `new_external_with_encoding`, `new_locale`,
   `new_filesystem`, `new_static`, `new_static_with_encoding`, `interned`,
   `interned_bytes` and `to_interned`, thanks to @danielpclark
 - `RString::append`, `concat_codepoint`, `is_comparable`, `is_eql`,
   `reserve`, `drop_bytes`, `splice`, `byte_offset`, `char_index`, `succ`,
   `dump`, `check_ascii_compatible`, `export`, `path_basename`,
   `path_extname`, `RString::implicit_convert` and `RString::format`
   (`Kernel#format`), thanks to @danielpclark
 - `Encoding::define_dummy`, `add_alias`, `is_capable`, `locale_charmap`,
   and `Encoding::is_unicode`, `count_chars` and `search`, thanks to
   @danielpclark
 - `EncodingConverter` and `ConversionResult`, a safe wrapper of Ruby's
   `rb_econv_*` transcoding API (streaming and buffer conversion,
   replacement, decorators, `insert_output`, `putback`), thanks to
   @danielpclark
 - `Symbol::new_with_encoding`, `find_with_encoding`, `is_literal_name`,
   `all`, and `Symbol::to_setter`, `is_setter_name`,
   `is_global_variable_name`, `is_local_name` and `is_junk_name`, thanks to
   @danielpclark
 - `Regexp::new_with_encoding`, `Regexp::escape`, `Regexp::search` and
   `MatchData::last_group`, thanks to @danielpclark
 - `VM::last_line` and `VM::set_last_line` (`$_`), thanks to @danielpclark
 - `rubysys` bindings for the rest of Ruby 4.0's string, encoding,
   transcoding, sprintf, symbol, parse and regexp C API, with the `RbEconv`,
   `ReRegisters`, `OnigRegexType` and `OnigPosition` types, thanks to
   @danielpclark
 - `rubysys` bindings for the rest of Ruby 4.0's numeric C API: `bignum.h`,
   `rbignum.h`, `complex.h`, `rational.h`, `numeric.h`, the arithmetic
   headers, `intern/random.h` and `ruby/random.h` (with `rb_random_t`,
   `rb_random_interface_t` and `rb_random_data_type`), the numeric parsers of
   `util.h` and `ctype.h`, the remaining `INTEGER_PACK_*` flags, and
   `types::c_ulong`, thanks to @danielpclark

### Changed
 - `build.rs` warns about a static Ruby built with ZJIT as well as YJIT; both
   put a Rust runtime in `libruby-static.a`, thanks to @danielpclark
 - CI tests Ruby 4.0.7 on Linux, macOS and Windows, with static Rubies built
   with `--disable-yjit --disable-zjit`, and runs the gem examples
   with minitest 5.25+ (5.15 does not install on Ruby 4.0), thanks to
   @danielpclark
 - `rubysys::constant::FL_EXIVAR` is 0, as Ruby 4.0's `RUBY_FL_EXIVAR` is,
   thanks to @danielpclark
 - `Complex::from_f64` uses `rb_dbl_complex_new`, thanks to @danielpclark
 - `Encoding::is_dummy` calls `rb_enc_dummy_p`, thanks to @danielpclark

### Removed
 - Ruby 3.2, 3.3 and 3.4 support, the `ruby_3_*`/`ruby_gte_3_*` cfg flags, and
   the Ruby 3.2 `RString` layout, thanks to @danielpclark

## [0.13.0] - 2026-10-01
Supports Ruby 3.2, 3.3 and 3.4 on Linux, macOS and Windows; Ruby 3.1 stays on
0.11.x/0.12.x.

Ruby 3.4 changes some output programs may compare: `Hash#inspect` prints
`{a: 1}`, error messages quote with `'` and name the class (`undefined method
'foo' for an instance of Integer`), and an explicit `GC::start` collects even
while the GC is disabled. Ruby 3.4 also needs `ruby_init_stack` before the VM
boots, or the GC scans no machine stack; `VM::init` and `VM::try_init` call it
(since 0.11.1), so only programs that start Ruby some other way must call it
themselves.

### Added
 - Ruby 3.4 support: `ruby_3_4`/`ruby_gte_3_4` cfg flags; every `rubysys`
   declaration checked against Ruby 3.4's headers and `libruby` exports,
   thanks to @danielpclark
 - `Thread::lock_native_thread` (`rb_thread_lock_native_thread`) and
   `VM::free_at_exit` (`ruby_free_at_exit_p`), Ruby 3.4+, thanks to
   @danielpclark
 - Raw bindings for the rest of Ruby 3.4's additions:
   `rb_fiber_scheduler_blocking_operation_wait` (with
   `RbFiberSchedulerBlockingOperationState`), `ruby_malloc_add_size_overflow`
   and `rb_assert_failure_detail`, thanks to @danielpclark
 - `rubysys::scheduler::RB_NOGVL_INTR_FAIL`, `RB_NOGVL_UBF_ASYNC_SAFE` and
   `RB_NOGVL_OFFLOAD_SAFE` (Ruby 3.4+), the `flags` of
   `rb_fiber_scheduler_blocking_operation_wait`, thanks to @danielpclark

### Changed
 - `Fiber::with_storage` is available on every supported Ruby, and
   `Hash::with_capacity` always passes the capacity to Ruby,
   thanks to @danielpclark
 - CI tests Ruby 3.2.9, 3.3.12 and 3.4.11 on Linux, macOS and Windows,
   thanks to @danielpclark

### Removed
 - Ruby 3.1 support, and the `ruby_3_1`/`ruby_gte_3_1` cfg flags,
   thanks to @danielpclark
 - `GC::force_recycle` and `GC::is_marked` (deprecated in 0.12): Ruby 3.4
   removes `rb_gc_force_recycle` and no longer exports
   `rb_objspace_marked_object_p`, thanks to @danielpclark

### Fixed
 - `rubysys::constant::ELTS_SHARED` and `FL_SINGLETON` follow Ruby 3.4, which
   moved them to `FL_USER_0` and `FL_USER_1`, thanks to @danielpclark

## [0.12.0] - 2026-10-01
Supports Ruby 3.1, 3.2 and 3.3 on Linux, macOS and Windows; Ruby 3.0 stays
on 0.11.x.

### Added
 - Ruby 3.3 support: `ruby_3_3`/`ruby_gte_3_3` cfg flags, and `RString`
   reads for 3.3's layout (the length moved out of `as.heap` to the top of
   the struct, for embedded strings too), thanks to @danielpclark
 - `IOBuffer`, a wrapper for `IO::Buffer` (`rb_io_buffer_*`): `new`,
   `from_bytes`, `size`, `lock`, `free`, `transfer`, `resize`, `clear`,
   `get_string`, `set_string`, `read`/`pread`/`write`/`pwrite` with buffer
   offsets, flag predicates, `page_size`/`default_size`, and unsafe
   `from_raw_parts`, `map`, `unlock`, `try_unlock`, `free_locked` (Ruby
   3.3+) and `with_bytes`/`with_bytes_mut`, plus `Class::io_buffer`, thanks
   to @danielpclark
 - `IO::descriptor`, `IO::maybe_wait`, `IO::maybe_wait_readable`,
   `IO::maybe_wait_writable` and the `IO::READABLE`/`IO::PRIORITY`/
   `IO::WRITABLE` constants; `IO::timeout`, `IO::set_timeout` and
   `Class::io_timeout_error` (Ruby 3.2+); unsafe `IO::from_raw_fd`
   (`rb_io_open_descriptor`, Ruby 3.3+); `File::size`, thanks to @danielpclark
 - `MemoryView`, a read-only wrapper for Ruby's MemoryView protocol
   (`rb_memory_view_get`/`release`), with shape, strides, format,
   `get_item`, `to_vec`, unsafe `as_bytes`, and `MemoryView::item_size_of`,
   thanks to @danielpclark
 - `Fiber::transfer`, `transfer_with_keywords`, `raise`,
   `resume_with_keywords` and `yield_with_keywords`; the `Fiber` type check
   uses `rb_obj_is_fiber`, thanks to @danielpclark
 - Fiber scheduler support: `Fiber::scheduler`, `set_scheduler`,
   `current_scheduler`, `current_scheduler_for_thread` and
   `make_scheduler_timeout`, and every `rb_fiber_scheduler_*` function of
   Ruby 3.1–3.3 in `rubysys::scheduler`, thanks to @danielpclark
 - `Thread::add_internal_event_hook` with `InternalThreadEvent` and
   `InternalThreadEventHook` (Ruby 3.2+), and `InternalThreadSpecificKey`,
   `Thread::internal_specific` and `Thread::set_internal_specific` (Ruby
   3.3+), thanks to @danielpclark
 - `VM::profile_frames`, `Thread::profile_frames` (Ruby 3.3+) and
   `ProfileFrame`; `DebugInspector` (including `frame_depth` and
   `current_depth` on Ruby 3.2+); `PostponedJob` (Ruby 3.3+), thanks to
   @danielpclark
 - `Class::subclasses`, `Class::attached_object` (Ruby 3.2+),
   `Module::new_refinement`, and `class_variable_find` and
   `deprecate_constant` on `Class` and `Module`, thanks to @danielpclark
 - `Class::data_define` for anonymous `Data` classes (`rb_data_define`,
   Ruby 3.3+), thanks to @danielpclark
 - `Ractor` (per-Ractor standard streams, `is_shareable`, `make_shareable`,
   `make_shareable_copy`) and `RactorLocalKey` for Ractor-local storage,
   thanks to @danielpclark
 - `VM::errno` and `VM::set_errno` (Ruby 3.3+), `VM::ext_resolve_symbol`
   (Ruby 3.3+) and `VM::clear_constant_cache_for` (Ruby 3.2+), thanks to
   @danielpclark
 - `Integer::positive_pow`, thanks to @danielpclark
 - `Class::refinement`, `Class::ractor`, `Class::no_matching_pattern_error`
   and `Class::no_matching_pattern_key_error`, thanks to @danielpclark
 - Raw bindings in `rubysys::io_buffer`, `rubysys::memory_view`,
   `rubysys::scheduler`, `rubysys::debug`, `rubysys::ractor` and
   `rubysys::st`, plus `rb_process_status_wait` (Ruby 3.3+), `struct
   rb_io_encoding`, the `FMODE_*` constants, `rb_ary_hidden_new`,
   `rb_obj_freeze_inline`, `ruby_scan_digits`, `ruby_hexdigits` and
   `rb_cNameErrorMesg`, thanks to @danielpclark
 - Raw bindings for Ruby's allocator (`ruby_xmalloc`, `ruby_xmalloc2`,
   `ruby_xcalloc`, `ruby_xrealloc`, `ruby_xrealloc2` and `ruby_xfree`),
   `rb_reg_onig_match` (Ruby 3.3+, with `ReRegisters` and
   `OnigMatchFunction`), `rb_st_init_existing_table_with_size` and
   `rb_st_replace` (Ruby 3.3 only, with `StHashType`), `rb_io_mode` (Ruby
   3.3+) and `rb_debug_rstring_null_ptr`, thanks to @danielpclark
 - `ValueType::Moved` (`T_MOVED`), and the `RbDataType` flags
   `RUBY_TYPED_FREE_IMMEDIATELY`, `RUBY_TYPED_FROZEN_SHAREABLE`,
   `RUBY_TYPED_WB_PROTECTED`, `RUBY_TYPED_EMBEDDABLE` (Ruby 3.3+) and
   `RUBY_TYPED_DECL_MARKING` (Ruby 3.3+), thanks to @danielpclark

### Changed
 - `IO::is_closed` calls `rb_io_closed_p` on Ruby 3.3, thanks to @danielpclark
 - CI tests Ruby 3.1.7, 3.2.9 and 3.3.12 on Linux (`ubuntu-latest`), macOS
   and Windows, thanks to @danielpclark

### Deprecated
 - `GC::force_recycle`: a no-op on Ruby 3.1+, and Ruby 3.4 removes
   `rb_gc_force_recycle`, thanks to @danielpclark
 - `GC::is_marked`: Ruby 3.4 no longer exports `rb_objspace_marked_object_p`,
   thanks to @danielpclark
 - `Thread::wait_fd` and `Thread::wait_fd_writable`, in favour of
   `Thread::wait_readable` and `Thread::wait_writable`, thanks to @danielpclark

### Removed
 - Ruby 3.0 support, and the `ruby_3_0`/`ruby_gte_3_0` cfg flags,
   thanks to @danielpclark

### Fixed
 - `rubysys::constant::FL_PROMOTED` is bit 5 alone on Ruby 3.3, which
   dropped `FL_PROMOTED0`/`FL_PROMOTED1` and left bit 6 unused; those two
   constants now exist only before 3.3, thanks to @danielpclark
 - `rubysys::io::rb_pid_t` is 64-bit with 64-bit MinGW, where `pid_t` is,
   thanks to @danielpclark
 - Linking a static Ruby failed when a library it needs is outside the
   linker's default search path: ruby-build's macOS Ruby 3.3 uses Homebrew's
   GMP (`ld: library 'gmp' not found`). `build.rs` now adds the absolute `-L`
   directories in the Ruby's `LDFLAGS` to the search path,
   thanks to @danielpclark
 - A stack walk inside a fiber made by `Fiber::new` or `Fiber::with_storage`
   crashed on Ruby 3.3 on arm64 macOS: a Rust panic in the fiber with
   `RUST_BACKTRACE` set segfaulted, and Ruby's crash report then hung.
   Ruby 3.3 leaves a return address at the bottom of fiber stacks where 3.2
   and x86_64 have 0; the fiber body now clears it before running,
   thanks to @danielpclark

## [0.11.2] - 2026-10-01
Adds the C API Ruby 3.1 and 3.2 introduced, which 0.11 did not bind. What
needs Ruby 3.1 is marked "Ruby 3.1+" and is not built on Ruby 3.0.

### Added
 - `IOBuffer`, a wrapper for `IO::Buffer` (`rb_io_buffer_*`, Ruby 3.1+):
   `new`, `from_bytes`, `size`, `lock`, `free`, `transfer`, `resize`,
   `clear`, `get_string`, `set_string`, `read`/`pread`/`write`/`pwrite` with
   buffer offsets, flag predicates, `page_size`/`default_size`, and unsafe
   `from_raw_parts`, `map`, `unlock`, `try_unlock` and
   `with_bytes`/`with_bytes_mut`, plus `Class::io_buffer`, thanks to
   @danielpclark
 - `IO::descriptor`, `IO::maybe_wait`, `IO::maybe_wait_readable`,
   `IO::maybe_wait_writable` and `File::size` (Ruby 3.1+); the
   `IO::READABLE`/`IO::PRIORITY`/`IO::WRITABLE` constants; `IO::timeout`,
   `IO::set_timeout` and `Class::io_timeout_error` (Ruby 3.2+), thanks to
   @danielpclark
 - `MemoryView`, a read-only wrapper for Ruby's MemoryView protocol
   (`rb_memory_view_get`/`release`), with shape, strides, format,
   `get_item`, `to_vec`, unsafe `as_bytes`, and `MemoryView::item_size_of`,
   thanks to @danielpclark
 - `Fiber::transfer`, `transfer_with_keywords` and `raise` (Ruby 3.1+), and
   `Fiber::resume_with_keywords` and `yield_with_keywords`; the `Fiber` type
   check uses `rb_obj_is_fiber` on Ruby 3.1+, thanks to @danielpclark
 - Fiber scheduler support (Ruby 3.1+): `Fiber::scheduler`,
   `set_scheduler`, `current_scheduler`, `current_scheduler_for_thread` and
   `make_scheduler_timeout`, and every `rb_fiber_scheduler_*` function of
   Ruby 3.1 and 3.2 in `rubysys::scheduler`, thanks to @danielpclark
 - `Thread::add_internal_event_hook` with `InternalThreadEvent` and
   `InternalThreadEventHook` (Ruby 3.2+), thanks to @danielpclark
 - `VM::profile_frames` and `ProfileFrame`; `DebugInspector` (including
   `frame_depth` and `current_depth` on Ruby 3.2+), thanks to @danielpclark
 - `Class::subclasses`, `Module::new_refinement`, and `class_variable_find`
   and `deprecate_constant` on `Class` and `Module` (Ruby 3.1+);
   `Class::attached_object` (Ruby 3.2+), thanks to @danielpclark
 - `Ractor` (per-Ractor standard streams, `is_shareable`, `make_shareable`,
   `make_shareable_copy`) and `RactorLocalKey` for Ractor-local storage,
   thanks to @danielpclark
 - `VM::clear_constant_cache_for` (Ruby 3.2+), thanks to @danielpclark
 - `Integer::positive_pow` (Ruby 3.1+), thanks to @danielpclark
 - `Class::ractor` and `Class::no_matching_pattern_error`; `Class::refinement`
   and `Class::no_matching_pattern_key_error` (Ruby 3.1+), thanks to
   @danielpclark
 - Raw bindings in `rubysys::io_buffer` and `rubysys::scheduler` (Ruby
   3.1+), `rubysys::memory_view`, `rubysys::debug`, `rubysys::ractor` and
   `rubysys::st`, plus the `FMODE_*` constants, `rb_ary_hidden_new` and
   `rb_obj_freeze_inline` (Ruby 3.2+), `ruby_scan_digits`, `ruby_hexdigits`
   and `rb_debug_rstring_null_ptr` (Ruby 3.1+), and `rb_cNameErrorMesg`,
   thanks to @danielpclark
 - Raw bindings for Ruby's allocator: `ruby_xmalloc`, `ruby_xmalloc2`,
   `ruby_xcalloc`, `ruby_xrealloc`, `ruby_xrealloc2` and `ruby_xfree`,
   thanks to @danielpclark
 - `ValueType::Moved` (`T_MOVED`), and the `RbDataType` flags
   `RUBY_TYPED_FREE_IMMEDIATELY`, `RUBY_TYPED_FROZEN_SHAREABLE` and
   `RUBY_TYPED_WB_PROTECTED`, thanks to @danielpclark

### Fixed
 - `Value::ty` read a slot that GC compaction had moved (`T_MOVED`) as a
   `ValueType` that does not exist; it is now `ValueType::Moved`, thanks to
   @danielpclark
 - Linking a static Ruby failed when a library it needs is outside the
   linker's default search path (a ruby-build macOS Ruby built against
   Homebrew's GMP: `ld: library 'gmp' not found`). `build.rs` now adds the
   absolute `-L` directories in the Ruby's `LDFLAGS` to the search path,
   thanks to @danielpclark

## [0.11.1] - 2026-09-30
Adds macOS and Windows: supports Ruby 3.0, 3.1 and 3.2 on Linux, macOS and
Windows. 0.11.0 was Linux only.

### Added
 - Windows support on Ruby 3: Rutie builds, links and passes its tests on
   64-bit Windows with RubyInstaller's Ruby 3.0, 3.1 and 3.2, with both the
   MSVC and the GNU Rust toolchains, for programs embedding Ruby and for Ruby
   extensions. This is 0.10.2's Windows support (see its entry) on Ruby 3;
   `VM::init` and `VM::try_init` call `ruby_sysinit` before booting, which
   Ruby 3 needs, thanks to @danielpclark
 - macOS support on Ruby 3: Rutie builds and passes its tests on macOS with
   Ruby 3.0, 3.1 and 3.2, thanks to @danielpclark
 - Static Ruby linking on Linux and macOS (`libruby-static.a`, `RUBY_STATIC`,
   `RUBY_STATIC_PATH`), as in 0.10.2. Programs that embed a static Ruby must
   export its functions (see the README); Rutie publishes the flag as
   `DEP_RUBY_LINK_ARG` (and `DEP_RUBY_STATIC`) for their build scripts, as
   `examples/rutie_rust_example/build.rs` uses, thanks to @danielpclark
 - `rutie_callback!` defines a function for Ruby to call (a method body, an
   allocator, ...) with the right ABI, or names its type: `extern "C"`, or
   `extern "C-unwind"` on Windows, thanks to @danielpclark
 - `Thread::wait_fd` and `Thread::wait_fd_writable` on Windows, where they take
   a descriptor of Ruby's C runtime (`IO#fileno`), thanks to @danielpclark

### Changed
 - Windows: functions Ruby calls that a Ruby exception can pass through are
   `extern "C-unwind"`: `methods!`/`unsafe_methods!` methods, the
   `types::Callback` and `Class::define_alloc_func` types, and the `rubysys`
   callback types. Windows needs Rust 1.71 or later; write hand-written
   callbacks with `rutie_callback!`. Other platforms are unchanged,
   thanks to @danielpclark
 - `VM::init` and `VM::try_init` call `ruby_init_stack` on every platform, as
   `ruby` does. Fibers always use native coroutines on Ruby 3, so 0.10.2's
   Ruby 2.5/2.6 arm64 macOS workaround (`rutie_copy_stack_fibers`) is not
   needed, thanks to @danielpclark
 - `VM::run_file` sets `$0` to the path `load` resolves (`rb_find_file`), so
   `__FILE__ == $0` also holds on Windows, where that path uses `/`,
   thanks to @danielpclark
 - A static Ruby's archive is not linked whole: with YJIT (Ruby 3.2) it
   contains YJIT's Rust runtime, whose allocator symbols clash with the
   program's (`duplicate symbol: __rust_alloc`). A static Ruby 3.2 built with
   YJIT can still fail to link with newer Rust toolchains (`duplicate symbol:
   rust_eh_personality`, from YJIT's copy of the standard library):
   `build.rs` warns about it, and the README says to build a static Ruby with
   `--disable-yjit`, thanks to @danielpclark
 - CI tests Ruby 3.0, 3.1 and 3.2 from `ruby/setup-ruby` on Linux, macOS
   and Windows (MSVC and GNU toolchains), with stable and beta Rust, plus
   static Rubies built with `ruby-build` (without YJIT) on Linux and macOS.
   It prints the end of `ruby-build`'s log when that fails, builds macOS's
   static Ruby 3.0 without `bigdecimal`, which doesn't compile with the
   current Apple clang, and builds the gem examples on macOS with
   `NO_LINK_RUTIE`, so an extension uses the `ruby` process's libruby,
   thanks to @danielpclark

### Removed
 - The README's "Migrating from Ruru to Rutie" section, thanks to @danielpclark

## [0.11.0] - 2026-09-30
Supports Ruby 3.0, 3.1 and 3.2 on Linux (macOS and Windows came in 0.11.1);
Ruby 2 stays on 0.10.x.

### Added
 - `build.rs` sets `ruby_3_0`/`ruby_3_1`/`ruby_3_2` and cumulative
   `ruby_gte_3_0`/`ruby_gte_3_1`/`ruby_gte_3_2` cfg flags, and fails on any
   other Ruby with a message naming the Rutie line that supports it,
   thanks to @danielpclark
 - `VM::scan_args_with_keywords`: `VM::scan_args` that takes a trailing
   `Hash` as the keywords, as Ruby 2 did (`rb_scan_args_kw` with
   `RB_SCAN_ARGS_LAST_HASH_KEYWORDS`), thanks to @danielpclark
 - `Thread::wait_readable` and `Thread::wait_writable` (`rb_io_wait`), with a
   timeout and a `Result`; they replace `Thread::wait_fd` and
   `Thread::wait_fd_writable`, thanks to @danielpclark
 - GC compaction: `GC::mark_movable`, `GC::location`, `GC::compact`, and a
   `compact(data) { .. }` clause for `wrappable_struct!` (the `dcompact`
   function). Without it wrapped objects stay pinned, as before,
   thanks to @danielpclark
 - `VM::ext_ractor_safe` (unsafe) and `VM::ext_ractor_unsafe` choose whether
   methods defined afterwards may run outside the main Ractor
   (`rb_ext_ractor_safe`), thanks to @danielpclark
 - `Fiber::with_storage` (Ruby 3.2+, `rb_fiber_new_storage`),
   thanks to @danielpclark
 - `Hash::with_capacity` (`rb_hash_new_capa` on Ruby 3.2; a plain empty hash
   on 3.0 and 3.1), thanks to @danielpclark
 - `ci/check_rubysys.py` checks every `rubysys` declaration against the
   Ruby's headers and `libruby` exports; CI runs it, thanks to @danielpclark

### Changed
 - `rubysys` follows Ruby 3's ABI: the special constants `Qnil`, `Qtrue` and
   `Qundef` per version (they moved in 3.2), 3.2's embedded `RString` length
   and `RArray` length mask, and `FL_SHAREABLE` replacing `FL_TAINT`,
   `FL_UNTRUSTED` and `FL_DUPPED`, thanks to @danielpclark
 - `VM::init` and `VM::try_init` process Ruby's command line once (an empty
   `-e` script, with RubyGems and `RUBYOPT` off), because Ruby 3.2 loads
   Ruby-defined core methods such as `Marshal.load` and `Time.at` there;
   `$0` can now be assigned. A `ruby` process that already did this (an
   extension) is left alone, thanks to @danielpclark
 - `VM::scan_args` follows Ruby 3's keyword separation: the keywords are
   only filled when the method was called with keywords,
   thanks to @danielpclark
 - `VM::run_file` sets `$0` and `ARGV` and loads the script like `VM::load`:
   it can run more than once and returns errors, including `SystemExit`,
   thanks to @danielpclark
 - `IO::binmode` calls `rb_io_ascii8bit_binmode`, so the external encoding
   becomes ASCII-8BIT as with Ruby's `IO#binmode`, thanks to @danielpclark
 - Methods an embedded VM defines through Rutie are no longer Ractor-safe by
   default (Ruby's main thread starts with C methods marked safe);
   extensions loaded by `require` were already unsafe by default,
   thanks to @danielpclark
 - `methods!` and `unsafe_methods!` generate `extern "C"` functions,
   thanks to @danielpclark
 - `build.rs` reruns when `RUBY`, `PATH`, `RBENV_VERSION`,
   `ASDF_RUBY_VERSION` or the static-linking variables change, so switching
   Ruby no longer keeps another Ruby's cfgs and link flags,
   thanks to @danielpclark
 - CI tests Ruby 3.0, 3.1 and 3.2 from `ruby/setup-ruby` on Linux (macOS and
   Windows best-effort), with stable and beta Rust; static-Ruby rows are
   dropped, thanks to @danielpclark
 - `GC::force_recycle` is documented as a no-op from Ruby 3.1,
   thanks to @danielpclark

### Removed
 - Ruby 2.5, 2.6 and 2.7 support, and the `ruby_2_*` cfg flags,
   thanks to @danielpclark
 - `Class::data`: Ruby 3 does not export `rb_cData`, thanks to @danielpclark

### Fixed
 - `Object::is_eql` returned `false` for equal objects on Ruby 3, where
   `rb_eql` returns `1` instead of `Qtrue`, thanks to @danielpclark
 - `rb_enc_codepoint_len` (used by `CodepointIterator`) is declared
   as returning `unsigned int`; reading it as `size_t` took garbage in the
   upper half on 64-bit targets. `rb_enc_str_asciionly_p` is declared as
   returning `int`, thanks to @danielpclark
 - `Complex::polar` uses `rb_complex_new_polar`; `rb_complex_polar` is
   deprecated, thanks to @danielpclark
 - The `eval` example flushes `$stdout` by calling `VM::cleanup`,
   thanks to @danielpclark
 - Rutie's unit tests aborted or crashed on Ruby 3.2 built with YJIT, whose
   `libruby` exports YJIT's copy of the Rust runtime: libruby came before
   the Rust standard library on the test binary's link line, so panics were
   started and caught by YJIT's `__rust_start_panic`/`__rust_panic_cleanup`.
   On Unix, libruby is now linked through a `#[link]` attribute (after std,
   for crates that depend on Rutie) and as a trailing linker argument for
   Rutie's own tests, thanks to @danielpclark

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
 - Static linking on macOS: Rutie's tests failed to link a static Ruby 2.6 or
   2.7 (`SecRandomCopyBytes` undefined), and with 2.5 crashed at boot loading
   `enc/encdb.bundle`. `build.rs` now also links the frameworks Ruby lists in
   `LIBRUBYARG_STATIC` (`Security`, `Foundation`), and exports Ruby's
   functions from Rutie's tests and examples with `-Wl,-export_dynamic`, as
   it does with `--export-dynamic` on Linux, thanks to @danielpclark
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
