# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html)
for the public APIs. `rubysys`, even though shared publicly, is considered a private
API and may have breaking changes during a teeny version change.


## [Unreleased]

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
