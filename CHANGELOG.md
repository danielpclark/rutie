# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html)
for the public APIs. `rubysys`, even though shared publicly, is considered a private
API and may have breaking changes during a teeny version change.

## [Unreleased]
### Added
- Methods `VM::yield_object` and `VM::yield_splat`
- `Enumerator` object
- `Array.to_enum`
- `TryConvert` for `AnyException`
- `VM::error_info` and `VM::clear_error_info`
- Documentation for `VM::protect`

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
