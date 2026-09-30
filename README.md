## Rutie

### Ruby 3.0, 3.1 and 3.2 (Ruby 2: use 0.10)

[![GitHub Actions Status](https://github.com/danielpclark/rutie/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/danielpclark/rutie/actions/workflows/ci.yml)
[![Ruby 3.0–3.2 Compatible](https://img.shields.io/badge/Supports-Ruby%203.0%20%7C%203.1%20%7C%203.2-brightgreen)](https://github.com/danielpclark/rutie#ruby-version-support)
[![Maintenance](https://img.shields.io/maintenance/yes/2026.svg)](https://github.com/danielpclark/rutie/commits/master)
[![GitHub contributors](https://img.shields.io/github/contributors/danielpclark/rutie.svg)](https://github.com/danielpclark/rutie/graphs/contributors)
[![license](https://img.shields.io/github/license/danielpclark/rutie.svg)](https://github.com/danielpclark/rutie/blob/master/LICENSE)
[![crates.io version](https://img.shields.io/crates/v/rutie.svg)](https://crates.io/crates/rutie)

> Rutie — /ro&#x035E;oˈˌtī/rOOˈˌtɪ/rüˈˌtaɪ/

Integrate Ruby with your Rust application.  Or integrate Rust with your Ruby application.
This project allows you to do either with relative ease.

You are highly encouraged to read the source code for this project.  Every method that has been
mapped from Ruby for public use in `src/class/*` is **very well documented** with example code.
This is the best way to take off running with Rutie.  There are also integration examples in the
examples directory which are based off of this README.

This project is a continuation of:
* [ruru](https://github.com/d-unseductable/ruru/) (licensed MIT)
* [ruby-sys](https://github.com/steveklabnik/ruby-sys/) (licensed MIT)

## Index

* [Using Ruby in Rust](https://github.com/danielpclark/rutie#using-ruby-in-rust)
* [Using Rust in Ruby](https://github.com/danielpclark/rutie#using-rust-in-ruby)
* [Custom Ruby Objects in Rust](https://github.com/danielpclark/rutie#custom-ruby-objects-in-rust)
* [Variadic Functions / Splat Operator](https://github.com/danielpclark/rutie#variadic-functions--splat-operator)
* [Migrating from Ruru to Rutie](https://github.com/danielpclark/rutie#migrating-from-ruru-to-rutie)
* [Safety — The Rutie Philosophy vs The Rust Philosophy on Safety](https://github.com/danielpclark/rutie/blob/master/README.md#safety--the-rutie-philosophy-vs-the-rust-philosophy-on-safety)
* [Troubleshooting](https://github.com/danielpclark/rutie#troubleshooting)
  * [It panics for some Rubies on CI server tests](https://github.com/danielpclark/rutie#it-panics-for-some-rubies-on-ci-server-tests)
  * [Rust signal: 11, SIGSEGV: invalid memory reference](https://github.com/danielpclark/rutie#rust-signal-11-sigsegv-invalid-memory-reference)
  * [Error while loading shared libraries: libruby.so.#.#: cannot open shared object file: No such file or directory](https://github.com/danielpclark/rutie#error-while-loading-shared-libraries-librubyso-cannot-open-shared-object-file-no-such-file-or-directory)
  * [Calling methods from other methods within the `methods!` macro doesn't work](https://github.com/danielpclark/rutie#calling-methods-from-other-methods-within-the-methods-macro-doesnt-work)
  * [Handling exceptions raised from Ruby in Rust code](https://github.com/danielpclark/rutie#handling-exceptions-raised-from-ruby-in-rust-code)
  * [Segfault during GC when using a Ruby method written in C](https://github.com/danielpclark/rutie/blob/master/README.md#segfault-during-gc-when-using-a-ruby-method-written-in-c)
* [Operating System Requirements](https://github.com/danielpclark/rutie#operating-system-requirements)
* [Contributing](https://github.com/danielpclark/rutie#contributing)
* [Rutie's Future](https://github.com/danielpclark/rutie#ruties-future)
  * [SemVer](https://github.com/danielpclark/rutie#semver)
* [Additional Project History](https://github.com/danielpclark/rutie#additional-project-history)
* [LICENSE](https://github.com/danielpclark/rutie#license)

## Using Ruby in Rust

First add the dependency to your `Cargo.toml` file.

```toml
[dependencies]
rutie = "0.11.0"
```

Then in your Rust program add `VM::init()` to the beginning of its code execution path
and begin to use Rutie.

```rust
extern crate rutie;

use rutie::{Object, RString, VM};

fn try_it(s: &str) -> String {
    let a = RString::new_utf8(s);

    // The `send` method returns an AnyObject type.
    let b = unsafe { a.send("reverse", &[]) };

    // We must try to convert the AnyObject
    // type back to our usable type.
    match b.try_convert_to::<RString>() {
        Ok(ruby_string) => ruby_string.to_string(),
        Err(_) => "Fail!".to_string(),
    }
}

#[test]
fn it_works() {

    // Rust projects must start the Ruby VM
    VM::init();

    assert_eq!("selppa", try_it("apples"));
}

fn main() {}
```

> NOTE: Currently in **Linux** you need to set `LD_LIBRARY_PATH` to point at the directory of your current Ruby library and in **Mac** you need to set `DYLD_LIBRARY_PATH` with that info.  You can get the path information with the following command:

    ruby -e "puts RbConfig::CONFIG['libdir']"

This should let you run `cargo test` and `cargo run`.

Running `cargo test` should have this test pass.

## Using Rust in Ruby

You can start a Ruby project with `bundle gem rutie_ruby_example` and then once
you change into that directory run `cargo init --lib`.  Remove the TODOs from the gemspec
file. Add Rutie to the `Cargo.toml` file and define the lib type.

```toml
[dependencies]
rutie = {version="xxx"}

[lib]
name = "rutie_ruby_example"
crate-type = ["cdylib"]
```

Then edit your `src/lib.rs` file for your Rutie code.

```rust
#[macro_use]
extern crate rutie;

use rutie::{Class, Object, RString, VM};

class!(RutieExample);

methods!(
    RutieExample,
    _rtself,

    fn pub_reverse(input: RString) -> RString {
        let ruby_string = input.
          map_err(|e| VM::raise_ex(e) ).
          unwrap();

        RString::new_utf8(
          &ruby_string.
          to_string().
          chars().
          rev().
          collect::<String>()
        )
    }
);

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn Init_rutie_ruby_example() {
    Class::new("RutieExample", None).define(|klass| {
        klass.def_self("reverse", pub_reverse);
    });
}
```

And that's it for the Rust side.  When using the `methods!` macro or `extern` functions
make sure the method name won't clash with any others.  This is why this example is prefixed with `pub_`.

Now you just need to load the library in Ruby.  Add the `rutie` gem to your gemspec or Gemfile.

```ruby
# gemspec
spec.add_dependency 'rutie', '~> 0.0.4'

# Gemfile
gem 'rutie', '~> 0.0.4'
```

And then load the library in your main project file `lib/rutie_ruby_example.rb`.

```ruby
require 'rutie_ruby_example/version'
require 'rutie'

module RutieRubyExample
  Rutie.new(:rutie_ruby_example).init 'Init_rutie_ruby_example', __dir__
end
```

That's all you need to load your Ruby things from Rust.  Now to write the test in
`test/rutie_ruby_example_test.rb`:

```ruby
require_relative "test_helper"
require "rutie_ruby_example"

class RutieRubyExampleTest < Minitest::Test
  def test_it_reverses
    assert_equal "selppa", RutieExample.reverse("apples")
  end
end
```

Write the following in `test/test_helper.rb`:

```ruby
require 'minitest/autorun'
```

Add the following snippet to your already scaffolded `Rakefile`:

```ruby
require "rake/testtask"

Rake::TestTask.new do |t|
  t.libs << 'test'
  t.pattern = 'test/**/*_test.rb'
  t.verbose = true
end
```

And to properly test it you will always need to run `cargo build --release` whenever
you make **any** changes to the Rust code.  Run the test with:

```bash
cargo build --release; rake test
```

Or better yet change your `Rakefile` to always run the `cargo build --release` before
every test suite run.  Feel free to change the test input to prove it fails because
the above test works as is.

## Custom Ruby Objects in Rust

To create a Ruby object in Rust that can be returned directly to Ruby
it needs just a few simple things.

Here's an example excerpt of code from [FasterPath](https://github.com/danielpclark/faster_path).

```rust
use rutie::types::{ Value, ValueType };
use rutie::{ RString, AnyObject, Object, Class, VerifiedObject };

pub struct Pathname {
    value: Value
}

impl Pathname {
    pub fn new(path: &str) -> Pathname {
        let arguments = [RString::new_utf8(path).to_any_object()];
        let instance = Class::from_existing("Pathname").new_instance(Some(&arguments));

        Pathname { value: instance.value() }
    }

    pub fn to_any_object(&self) -> AnyObject {
        AnyObject::from(self.value())
    }
}

impl From<Value> for Pathname {
    fn from(value: Value) -> Self {
        Pathname { value }
    }
}

impl Object for Pathname {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for Pathname {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        Class::from_existing("Pathname").case_equals(object)
    }

    fn error_message() -> &'static str {
        "Error converting to Pathname"
    }
}
```

If the class does not yet exist in Ruby you'll need to account for creating
it before generating a new instance of it.  This object is now compatible to
be returned into Ruby directly from Rust/Rutie.  _Note that this definition is
merely a Rust compatible representation of the Ruby object and doesn't define
any Ruby methods which can be used from Ruby._

## Variadic Functions / Splat Operator

Since 0.10 the `methods!` macro takes a Ruby-style splat: a last parameter written
`*name` (with no type) receives the remaining arguments as an `Array`. The
parameters before it keep their usual `Result` types.

```rust
#[macro_use]
extern crate rutie;

use rutie::{Array, Class, Object, RString, VM};

class!(Example);

methods!(
    Example,
    rtself,

    // def example_method(name, *rest)
    fn example_method(name: RString, *rest) -> Array {
        let mut result = Array::new();

        if let Ok(name) = name {
            result.push(name);
        }

        result.push(rest);
        result
    }
);

fn main() {
    VM::init();
    Class::new("Example", None).define(|klass| {
        klass.def("example_method", example_method);
    });
}
```

For optional, keyword and block parameters, write a plain
`extern fn(Argc, *const AnyObject, Self)` function and let
[`VM::scan_args`](https://docs.rs/rutie/latest/rutie/struct.VM.html#method.scan_args)
parse the arguments with an `rb_scan_args` format such as `"11*:&"`. It returns
`ScannedArgs` (required, optional, splat, post, keywords and block) without any
unsafe code; see its documentation for an example.

## Migrating from Ruru to Rutie

#### &lt;0.1

For using Rutie versions less than 0.1 the change is simple.  Replace all occurrences
of the string `ruru` with `rutie` in your program.  And if you would like to use
`ruby-sys` code from Rutie rather than requiring `ruby-sys` you can change all existing
references to `ruby_sys` to `rutie::rubysys`.

#### 0.1

You will have additional considerations to change like `Error` being removed.  For that; change instances of type `ruru::result::Error` to `rutie::AnyException`.

#### 0.2

Migrated `parse_arguments` from `VM` to `util`.

#### 0.3

Internal changes `util` from `binding` and `rubysys` have been replaced to reduce confusion and reduce duplication.

#### 0.10

0.10 targets Ruby 2 (2.5, 2.6, 2.7) and continues the 0.8/0.9 line. It adds
a large amount of API (see the CHANGELOG) and has one breaking change to
migrate from 0.9.x:

- `VM::at_exit` now does what its name says: the closure runs when the Ruby VM
  shuts down (Ruby's `at_exit`), not immediately. It must be `'static` (move
  captured values into it). Programs embedding Ruby run these handlers by calling
  `unsafe { VM::cleanup() }` at the end. If you relied on the old immediate call,
  use `VM::call_protected`, which keeps that behaviour (and no longer crashes:
  closures that captured variables used to read a bad pointer).

Builds also work with current Cargo without setting `LD_LIBRARY_PATH` for
`cargo test`.

If you were depending on the unpublished `rb-sys`-based `master` (self-labelled
0.10.0, February–September 2025) through a git dependency: that tree is
discontinued. Its `link-ruby` and `ruby-static` cargo features do not exist in
0.10; use the `no-link` feature (or `NO_LINK_RUTIE`) and the `RUBY_STATIC`
environment variable as documented below, and expect Ruby 2, not Ruby 3.

#### 0.11

0.11 supports Ruby 3.0, 3.1 and 3.2 and drops Ruby 2 (stay on 0.10.x for
Ruby 2.5–2.7). Migrating from 0.10:

- **Build.** `build.rs` fails with a message naming the right Rutie line when
  it finds a Ruby other than 3.0–3.2. The cfg flags are now `ruby_3_0`,
  `ruby_3_1`, `ruby_3_2` and `ruby_gte_3_0`…`ruby_gte_3_2`; the `ruby_2_*`
  flags are gone. Switching Ruby (a new `PATH`, `RUBY`, `RBENV_VERSION` or
  `ASDF_RUBY_VERSION`) now rebuilds Rutie; run `cargo clean -p rutie` if you
  switch some other way.
- **Keyword arguments** follow Ruby 3. `VM::scan_args` only fills the `:`
  keywords when the method was called with keywords; a trailing `Hash`
  passed positionally stays positional. `VM::scan_args_with_keywords` keeps
  Ruby 2's behaviour of taking a trailing `Hash` as keywords.
  `Object::send_with_keywords` and `VM::is_keyword_given` are available on
  every supported Ruby. `methods!` receives keywords as a trailing `Hash`.
- **Removed:** `Class::data` (Ruby 3 no longer exports `rb_cData`).
- **Changed behaviour:**
  - `IO::binmode` also sets the external encoding to ASCII-8BIT, as Ruby's
    `IO#binmode` does.
  - `VM::run_file` loads the script like `VM::load` after setting `$0` and
    `ARGV`, so it can run more than once and returns errors (including
    `SystemExit`) instead of exiting.
  - `VM::init` processes Ruby's command line once (with RubyGems and
    `RUBYOPT` off), because Ruby 3 loads part of its core library there, so
    `$0` can be assigned.
  - Methods an embedded VM defines through Rutie are not Ractor-safe: calling
    one outside the main Ractor raises `Ractor::UnsafeError`, as it already
    did in an extension. `unsafe { VM::ext_ractor_safe(true) }` opts in.
- **No-ops and deprecations:** `GC::force_recycle` does nothing from Ruby 3.1.
  `Thread::wait_fd` and `Thread::wait_fd_writable` wrap C functions Ruby
  deprecates from 3.1; use `Thread::wait_readable` and
  `Thread::wait_writable`. Both old methods get `#[deprecated]` in 0.12.
- **New:** GC compaction support (`GC::mark_movable`, `GC::location`,
  `GC::compact` and a `compact` clause in `wrappable_struct!`),
  `Fiber::with_storage` (Ruby 3.2), `Hash::with_capacity`, and the Ractor and
  wait methods above. See the CHANGELOG for the full list.


## Safety — The Rutie Philosophy vs The Rust Philosophy on Safety

I'm writing this section to bring to light that, as of this writing, the safety that Rust likes to guarantee for its crates and the Rutie crate aren't currently aligned.  The typical Rust safety for libraries wrapping C code is to have one unsafe crate with a `-sys` extension in its name and then a crate that wraps that to make it safe.

Rutie is an official fork of the project Ruru and because of this a great deal of the decisions in design for the project remain unchanged.  Rutie also brought in the `ruby-sys` crate and treats it as an internal private API/module; and yet shares it openly for other developers to have full control to design their own API on top of it.

One of the glaring things that Rutie has that goes against the Rust Philosophy on Safety is that any of the methods that call Ruby code, can potentially raise an exception, and don't return the type `Option<AnyObject, AnyException>` will panic when an exception is raised from Ruby… which kills the application process running.  The way to avoid panics is simple: either guarantee the Ruby code you're running will never raise an exception, or [Handling exceptions raised from Ruby in Rust code](https://github.com/danielpclark/rutie#handling-exceptions-raised-from-ruby-in-rust-code) with "protect" methods that return the type `Option<AnyObject, AnyException>`.  Anyone can implement this safety by reading and understanding how the **protect** methods are written in this library and working with them.

The important thing to consider as to **“why doesn't every method guarantee the safety as Rust would prescribe to?”** is that exception handling in Ruby is **not a zero cost abstraction**.  So there is a cost in performance when you need to implement it.  One can easily argue that the guarantee of safety is far more important than leaving the risk in the hands of inexperienced developers.  But one could also argue that it is better to leave the choice of performance cost, and the fact that exception capturing is occasionally unnecessary, up to the developer.  Seeing how the legacy of design decisions is largely inherited this project leans towards the latter argument where the choice of being absolutely safe everywhere vs some extra speed in performance is up to the developer.

I'm not opposed to this project being 100% safe, but that will be a major change and a totally different API with many decisions that need to come into play.  Also since this project doesn't strictly adhere to Rust safety principles, as a crate library would be expected to be, this project will not reach the stable 1.0 release as the idea of stability and safety are hand in hand.

I do like safety guarantees and as much as possible new features and language APIs will be built toward this.  You can see what the design of a safe API would look like by examining the [Enumerator features](https://github.com/danielpclark/rutie/blob/master/src/class/enumerator.rs) that have been implemented in this way (largely wrapping method names with calls to `protect_send`).

## Troubleshooting

#### It panics for some Rubies on CI server tests

Sometimes the Ruby binary built isn't the best for the system.  Be sure to compile Ruby
for that system if this is the issue.   With RVM do `rvm reinstall --disable-binary` with
your choice of Ruby version.

#### Rust signal: 11, SIGSEGV: invalid memory reference

This is an indication that you haven't started a Ruby VM in Rust yet with `VM::init();`.  Do this once
before using Ruby code from Rust.

#### Error while loading shared libraries: libruby.so.#.#: cannot open shared object file: No such file or directory

This happens when the Rutie build is trying to link with `libruby`,
but it's not found on your library search path. Either add it to
`LD_LIBRARY_PATH`/`DYLD_LIBRARY_PATH` if you're building a standalone
program that calls `VM::init()`, or if you're building a library to
load into a running Ruby VM then you can disable linking by either
setting the environment variable `NO_LINK_RUTIE`, or enabling the
cargo feature `no-link` for Rutie in your `Cargo.toml` like this:

```toml
[dependencies]
rutie = {version="xxx", features=["no-link"]}
```

#### Calling methods from other methods within the `methods!` macro doesn't work

The way the macro is designed doesn't use the same parameter signatures you've provided and
therefore it is recommended to implement any methods you want to re-use in Rust with
functions outside of the `methods!` macro.  You can simply call that new external
method in the `methods!` macro when defining methods for Ruby to use.

#### Handling exceptions raised from Ruby in Rust code

If you're using any method that doesn't return a `Result<AnyObject, AnyException>` then
any exception raised from the Ruby side will interfere with that Ruby thread and cause
Rust to panic and stop.  Ruby internally uses exceptions to effect the entire thread through
an internal thread global value.  To handle places where Ruby may raise an exception during Rust
code execution you should use methods that are designed to handle that.

* `VM::eval`
* `Object.protect_send`
* `Object.protect_public_send`

If you are writing lower level code and want to work more directly with the internal Ruby
exception you may use `VM::protect` and read the source code for `Object.protect_send` to
see how it's done.

#### Segfault during GC when using a Ruby method written in C

One possible issue that may cause this is when you store an item in Rust in heap memory rather than the stack.

An example case that caused this issue is the following:

```rust
Class::from_existing("Pathname").new_instance(&vec![RString::new_utf8(path).to_any_object()])
```

> Ruby's GC traces objects from the stack. Rust's Vec, on the other hand, stores elements in the heap. So Ruby's GC may not be able to find the string you created and may release it. — @irxground

To rememdy the issue it required not using Vec but rather Rust's array type to store the argument on the stack rather than the heap.

```rust
let arguments = [RString::new_utf8(path).to_any_object()];
Class::from_existing("Pathname").new_instance(&arguments)
```

## Operating System Requirements

Everything is tested against 64 bit operating systems with 64 bit Ruby & Rust builds.  32 bit isn't currently supported.  

### Ruby version support

| Rutie | Ruby | Notes |
|---|---|---|
| 0.11.x | 3.0, 3.1, 3.2 | current; dynamic linking on Linux is the supported configuration, macOS and Windows are best-effort |
| 0.10.x | 2.5, 2.6, 2.7 | the Ruby 2 line |
| 0.9.x | 2.5, 2.6, 2.7 | still works on Ruby 2, but superseded by 0.10.0 (`VM::at_exit` crash fix, current-Cargo build fix) |
| 0.8.x | 2.5, 2.6, 2.7 | older Ruby 2 line |

### Version roadmap

Each Rutie minor version supports exactly three Ruby minor versions:

| Rutie | Ruby | Status |
|---|---|---|
| 0.10 | 2.5, 2.6, 2.7 | released: Ruby 2 |
| 0.11 | 3.0, 3.1, 3.2 | current: first Ruby 3 release, drops Ruby 2 |
| 0.12 | 3.1, 3.2, 3.3 | planned: drops 3.0, adds 3.3 |
| 0.13 | 3.2, 3.3, 3.4 | planned: drops 3.1, adds 3.4 |

The Ruby 3 work is planned in `docs/ruby3-upgrade-plan.md`.

An `rb-sys`-based rewrite lived on `master` between February and September
2025 (self-labelled 0.10.0, never published to crates.io); it has been
reverted and is not supported. If you depended on it through a
`git = "..."` dependency, move to 0.11 (Ruby 3) or 0.10.x (Ruby 2).

Prebuilt Rubies (`ruby/setup-ruby`, RVM's binaries, distribution packages)
are the easiest way to get each version. Building Ruby 3.0 from source needs
OpenSSL 1.1 for its `openssl` extension; 3.1 and later build with OpenSSL 3.

#### Linux & Mac

- A current stable Rust (CI tests stable and beta)
- Ruby (64 bit) 3.0, 3.1 or 3.2, built with `--enable-shared` (see "Ruby version support")

#### Windows
- A current stable Rust
- Ruby 3.0–3.2 from RubyInstaller (64 bit) — best-effort: CI builds it, but failures do not block
- MS Visual Studio (Build Tools)

#### Dynamic vs Static Builds

Rutie links Ruby's shared library (`libruby`), which gives the best performance and
support, so Ruby should be compiled with the `--enable-shared` option.

If using RBENV then the following is recommended:

    CONFIGURE_OPTS=--enable-shared rbenv install 3.2.6

You can check if your Ruby is compiled to be dynamically linked to by running the following and getting a `"yes"` response.

    ruby -e "pp RbConfig::CONFIG['ENABLE_SHARED']"

A Ruby built without a shared library (`"no"` above; `ruby/setup-ruby`'s macOS Ruby 3.1 is
one) is linked statically instead, as a fallback. Static support is incomplete, and two
things differ from a shared Ruby:

- A program that embeds Ruby must export libruby's functions for Ruby's own extensions
  (`enc/encdb` is loaded at boot), or Ruby crashes loading them. Rutie does this for its
  own tests and examples, and publishes the linker flag as `DEP_RUBY_LINK_ARG` for your
  build script, as `examples/rutie_rust_example/build.rs` uses it:

  ```rust
  if let Ok(arg) = std::env::var("DEP_RUBY_LINK_ARG") {
      println!("cargo:rustc-link-arg={}", arg);
  }
  ```

- A library loaded into Ruby (a gem's extension) must not link libruby at all, or it
  carries a second VM that was never started. Build it with the `no-link` feature or
  `NO_LINK_RUTIE` set (on macOS with `-C link-args=-Wl,-undefined,dynamic_lookup`).

`RUBY_STATIC` forces static linking, and `RUBY_STATIC_PATH` names the directory holding the
static library when it is not in Ruby's `libdir`. If you still run into
`ld: library not found for -lruby-static` issue, try running `cargo clean`. This'll clean
any artifacts from previous attempts.

If you'd like to make a pull request for adding static build support there are currently 3 methods not working with it and linking to the proper name of the ruby static lib file & path needs to be updated.

## Contributing

Contributors are welcome!

The code is organized in 3 main layers.  The `rubysys` folder is the raw mapping to Ruby C code and
all the methods from there are unsafe.  The `binding` folder is where we wrap those methods to abstract
away all the unsafe methods to safe methods.  The `class` folder is where the public API is implemented
for using Ruby with Rust code.  These methods in the `class` folder must all be documented and tested within
the documentation.  There is a subfolder under `class` for traits called `traits`.

Macros for abstracting away complexity are in `src/dsl.rs`.

Ruby's helper gem is in the submodule folder `gem`.

### Testing against several Rubies

Changes must pass on Ruby 3.0, 3.1 and 3.2. `build.rs` links against the `ruby`
first on your `PATH` (or the one named by `$RUBY`), so install each version under
its own prefix and give each its own Cargo target directory, so switching does
not rebuild everything.

```sh
for version in 3.0.7 3.1.7 3.2.9; do
  PATH="/opt/rb/$version/bin:$PATH" \
  CARGO_TARGET_DIR="$HOME/rt-$version/target" \
  cargo test
done
```

`CARGO_TARGET_DIR` must end in `/target`. `python3 ci/check_rubysys.py` checks
the `rubysys` declarations against the Ruby on your `PATH` (headers and, on
Linux, `libruby`'s exports), as CI does. Any way of installing the Rubies
works: RVM, rbenv/ruby-build, release tarballs, or RVM's prebuilt binaries.
Build them with `--enable-shared`, which RVM does by default.

Unit tests live in a `#[cfg(test)] mod tests` at the bottom of each file and run
their body through `crate::on_ruby_thread(|| { ... })`, because Ruby must be
used from the thread that started it. Every public item also needs a doctest
that runs and asserts.

## Rutie's Future

Rutie will continue to be improved upon to be more and more compatible with every aspect of Ruby.  It
will also gradually change toward Rust safety, semantics, and best practices.

I imagine a future where Rutie is the stepping stone that helps Ruby switch from C to Rust.

#### SemVer

As this package has taken 1.0 to mean both stable and safe and won't likely make a 1.0, then there can
be breaking changes expected in each MINOR version update.  These MINOR version breaking changes will
occur in the public API of `src/class/*` and `src/helpers/*`.  For private APIs there can be breaking
changes in each PATCH version update which includes `src/rubysys/*`, `src/binding/*`, and
`src/util.rs`.

## Additional Project History

If you need some more examples of usage or the git blame history please look at the [Ruru](https://github.com/d-unseductable/ruru)
project as Rutie has had the README completely rewritten and this first git commit is from Ruru.
Note that there are some fundamental changes which that README won't account for.
This project also had [ruby-sys](https://github.com/steveklabnik/ruby-sys) merged in which may have some additional beneficial git history.

## LICENSE

Both projects that were merged into this project contained identifiers under the MIT license.
This project follows with the same licensing.  **ruby-sys** marked MIT as the license in the
`Cargo.toml` file whereas **ruru** had that and included a LICENSE file.  This projects LICENSE
has credited the original author by preserving the MIT license author line and appending new
author(s) which is permitted by the MIT LICENSE.

MIT LICENSE — see [LICENSE](LICENSE)
