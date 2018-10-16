# Building with Rutie

In many cases just including Rutie in your `Cargo.toml` dependencies will
just work.  The build allows for much more custom options in how you want
to have the build work.

There are two main building options for linking with Ruby and also an
option to not use any additional build configurations.

## Disabling Linker Options

When building for a Ruby program it may well be the case that you don't
need the linker to pass any commands during compilation.  To disable the
build linking options simply set the environment variable `NO_LINK_RUTIE`.

## Using PKG-CONFIG

In many cases the build script will find the `ruby*.pc` file in the
libruby directory.  When that happens pkg-config will be used for the
linking options.

If the the build `.pc` file isn't in that location or the name `ruby`
isn't what it's being looked up with then pkg-config will be skipped and
the default build commands will be used.  If the folder `pkgconfig` isn't
in the Ruby `libdir` then you may need to set the environment variable
`PKG_CONFIG_PATH` if you'd like to use Ruby's pkg-config.  If the
pkg-config isn't locating `ruby` in it's list as the `ruby` wrapper for
a specific version wasn't included then you may need to set `LIBRUBY_NAME`
to something like `ruby-2.5` to use pkg-config.  Run `pkg-config
--list-all | grep -e ruby` to see what Ruby library versions of its name
are available for `pkg-config` for the build.

For more options on how to set the linking between dynamic or static
linking with Ruby please see the documentation for
[pkg-config-rs](https://github.com/alexcrichton/pkg-config-rs).

## Disabling PKG-CONFIG

Disabling the use of pkg-config will guarantee using the default fallback
settings which were written in to the Rutie library.  You can disable the
use of pkg-config in Rutie with setting the environment variable
`RUTIE_NO_PKG_CONFIG`.

With the fallback settings you can set dynamic or static linking with the
environment variable `RUBY_STATIC` to force static linking.  Otherwise the
value of `ENABLE_SHARED` from Ruby's `RbConfig::CONFIG` will determine
whether static or dynamic linking is used.

## Static Builds

Set `RUBY_STATIC` environment variable first and read the following.

Ruby removed the static ruby library with [issue #12845](https://bugs.ruby-lang.org/issues/12845)
by default.  Because of this you will need to provide a path where this
static library `libruby-static.a` can be found in `RUBY_STATIC_PATH` if you
wish to build a statically linked build.  If you compile Ruby with RVM then
the build script will work without you providing the `RUBY_STATIC_PATH` env
var.

NOTE: there are a three methods as of this writing that don't work when using
statically built code.  They are:

- `class::string::RString::from_bytes`
- `class::string::RString::is_valid_encoding`
- `class::vm::VM::init_loadpath`

NOTE: code seems to run much slower when using static builds.

## Logging

Currently when `CI_STDERR_LOG` is set logging is mapped to the builds
standard error output which is written into a file `stderr` in
a subdirectory under `target/debug/build/rutie-*/`.  The output detail is
available on TravisCI builds if you care to look at those logs.
