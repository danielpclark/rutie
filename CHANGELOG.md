# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html)
for the public APIs. `rubysys`, even though shared publicly, is considered a private
API and may have breaking changes during a teeny version change.

## [Unreleased]

## [0.4.2]
### Fixed
- Wrapping struct changed from Ruru to Rutie & some of the same changes in documentation

## [0.4.1]
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
  match equivelant Ruby C macros for use in `CodepointIterator`

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
- Example CLI eval program in examples directory
- `RString::count_chars`

### Changed
- Refactor of `VM::protect`
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
- Refactor away from using heap to stack memory

### Fixed
- A few Ruby `ValueType` flags were incorrect in `rubysys`

## [0.2.2] - 2018-07-07
### Added
- `String#concat`
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
