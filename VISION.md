The vision for Rutie development has two values.

## 1) Idiomatic Rust

`Result<AnyObject, AnyException>` preferred when exceptions are possible

When Ruby object may potentially raise an error, emphasize use of methods that return
a `Result<AnyObject, AnyException>` to prevent panics which crash the application.  Such
methods are `protect_send`, `public_protect_send`, or `eval`.

The existing methods that don't protect against exception raising will be left here with
the understanding that a developer can be responsible enough to write code that won't break
code execution.

## 2) Ruby Objects First

At the center of object handled with this library will be a Ruby C object known
as a Value.
