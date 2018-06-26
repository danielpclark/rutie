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

To create a Ruby object in Rust that can be returned directly to Ruby
it needs just a few simple things.

Here's an example excerpt of code from [FasterPath](https://github.com/danielpclark/faster_path).

```rust
use rutie::{
  RString,
  AnyObject,
  Object,
  Class,
  VerifiedObject
};


pub struct Pathname {
  value: Value
}

impl Pathname {
  pub fn new(path: &str) -> Pathname {
    let mut instance = Class::from_existing("Pathname").allocate();
    instance.instance_variable_set("@path", RString::new(path).to_any_object());

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
    object.value().ty() == ValueType::Class &&
      Class::from_existing("Pathname").case_equals(object)
  }

  fn error_message() -> &'static str {
    "Error converting to Pathname"
  }
}
```

If the class does not yet exist in Ruby you'll need to account for creating
it before generating a new instance of it.  This object is now compatible to
be returned into Ruby directly from Rust/Rutie.

## Variadic Functions / Splat Operator

A preferred way to integrate a dynamic amount of parameters has not yet been implemented in Rutie.
But you can still manage to get it done in the following way.

```rust
use rutie::{AnyObject, Array, Object, AnyException};
use rutie::types::{Argc, Value};
use rutie::util::str_to_cstring;
use rutie::rubysys::util;
use std::mem;

pub extern fn example_method(argc: Argc, argv: *const AnyObject, _: AnyObject) -> AnyObject {
    let args = Value::from(0);
  
    unsafe {
        let p_argv: *const Value = mem::transmute(argv);
  
        util::rb_scan_args(
            argc,
            p_argv,
            str_to_cstring("*").as_ptr(),
            &args
        )   
    };  
  
    let arguments = Array::from(args);

    let output = // YOUR CODE HERE.  Use arguments as you see fit.

    output.to_any_object() // When mapping a method to Ruby
                           // the return object needs to be AnyObject
}
```

This style of code is meant to be used outside of the `methods!` macro for now.
You may place this method on a class or module as you normally would from a `methods!` macro definition.

```
#[macro_use]
extern crate rutie;

use rutie::{Class, Object, VM};

class!(Example);

// Code from above

fn main() {
    # VM::init();
    Class::new("Example", None).define(|itself| {
        itself.def("example_method", example_method);
    });
}

The Rutie project has in its plans to remove the need for anyone to write unsafe code for
variadic support and will likely be updating the `methods!` macro to support this natively.

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
