## Rutie

“The tie between Ruby and Rust.”
[![Build Status](https://travis-ci.org/danielpclark/rutie.svg?branch=master)](https://travis-ci.org/danielpclark/rutie)

Integrate Ruby with your Rust application.  Or integrate Rust with your Ruby application.
This project allows you to do either with relative ease.

This project is a continuation of [ruby-sys](https://github.com/steveklabnik/ruby-sys/) (licensed MIT) and [ruru](https://github.com/d-unseductable/ruru/) (licensed MIT).

## Using Ruby in Rust

First add the dependency to your `Cargo.toml` file.

```toml
[dependencies]
rutie = "0.1.4"
```

Then in your Rust program add `VM::init()` to the beginning of its code execution path
and begin to use Rutie.

```rust
extern crate rutie;

use rutie::{Object, RString, VM};

fn try_it(s: &str) -> String {
    let a = RString::new(s);

    // The `send` method returns an AnyObject type.
    let b = a.send("reverse", None);

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

Running `cargo test` should have this test pass.

## Using Rust in Ruby

TODO

## Custom Ruby Objects in Rust

TODO

## Variadic Functions / Splat Operator

A preferred way to integrate a dynamic amount of parameters has not yet been implemented in Rutie.
But you can still manage to get it done in the following way.

## Migrating from Ruru to Rutie

#### &lt;0.1
For using Rutie versions less than 0.1 the change is simple.  Replace all occurences
of the string `ruru` with `rutie` in your program.  And if you would like to use
`ruby-sys` code from Rutie rather than requiring `ruby-sys` you can change all existing
references to `ruby_sys` to `rutie::rubysys`.

#### 0.1

You will have additional considerations to change like `Error` being removed.  For that; change instances of type `ruru::result::Error` to `rutie::AnyException`.

#### 0.2

Migrated `parse_arguments` from `VM` to `util`.

## Troubleshooting

#### rust signal: 11, SIGSEGV: invalid memory reference

This is an indication that you haven't started a Ruby VM in Rust yet with `VM::init();`.  Do this once
before using Ruby code from Rust.

#### error while loading shared libraries: libruby.so.#.#: cannot open shared object file: No such file or directory

This may happen when a Ruby program is trying to link with libruby via Rutie.  Simply disable linking
by setting the environment variable `NO_LINK_RUTIE` before the Rust code is compiled.  This is needed
to be done on the service TravisCI for example.

## LICENSE

MIT LICENSE — see [LICENSE](LICENSE)
