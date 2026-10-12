//! What generated methods run: argument handling, the protected call into
//! the user's code, and handing the result (or exception) back to Ruby.

use std::{
    marker::PhantomData,
    panic::{self, AssertUnwindSafe},
};

use super::{
    class::RubyClass,
    convert::{FromArgument, IntoRubyObject},
};
use crate::{
    binding::vm,
    rubysys::class::{
        rb_define_method, rb_define_module_function, rb_define_private_method,
        rb_define_protected_method, rb_define_singleton_method,
    },
    types::{c_int, c_void, Argc, ValueType},
    util, AnyException, AnyObject, Class, Exception, Hash, NilClass, Object, Proc, Symbol, Thread,
    VM,
};

/// Why a generated method didn't return normally: an exception to raise, or
/// a Ruby jump (`raise`, `throw`, `break`) out of the user's code, resumed
/// once the method's values are dropped.
#[doc(hidden)]
pub enum Failure {
    Exception(AnyException),
    Jump(c_int),
}

impl From<AnyException> for Failure {
    fn from(exception: AnyException) -> Self {
        Failure::Exception(exception)
    }
}

fn panic_exception(payload: Box<dyn std::any::Any + Send>) -> AnyException {
    AnyException::from(vm::panic_to_exception(payload))
}

/// Runs a generated method body and hands its result to Ruby. Everything the
/// body owns (arguments, borrows of `#[rubyclass]` structs) is dropped before
/// an exception is raised or a jump resumed, and a panic is raised as a
/// `RuntimeError`.
#[doc(hidden)]
pub fn invoke<F>(body: F) -> AnyObject
where
    F: FnOnce() -> Result<AnyObject, Failure>,
{
    let result = match panic::catch_unwind(AssertUnwindSafe(body)) {
        Ok(result) => result,
        Err(payload) => Err(Failure::Exception(panic_exception(payload))),
    };

    match result {
        Ok(object) => object,
        Err(Failure::Exception(exception)) => vm::raise_ex(exception.value()),
        Err(Failure::Jump(state)) => vm::jump_tag(state),
    }
}

/// Calls the user's code under `rb_protect`, so a Ruby exception raised by
/// Ruby code it calls stops here instead of skipping the generated method's
/// cleanup. (The user's own frames are still skipped by such a jump; return
/// `Err` to raise without that.)
#[doc(hidden)]
pub fn call<R, F>(func: F) -> Result<R, Failure>
where
    F: FnOnce() -> R,
{
    let mut func = Some(func);
    let mut outcome = None;

    let protected = vm::protect(|| {
        if let Some(func) = func.take() {
            outcome = Some(panic::catch_unwind(AssertUnwindSafe(func)));
        }

        NilClass::new().to_any_object()
    });

    match protected {
        Err(state) => Err(Failure::Jump(state)),
        Ok(_) => match outcome {
            Some(Ok(result)) => Ok(result),
            Some(Err(payload)) => Err(Failure::Exception(panic_exception(payload))),
            None => unreachable!("rb_protect returned without calling the function"),
        },
    }
}

/// Calls the user's code with the GVL released (`#[ruby(without_gvl)]`).
///
/// Ruby can't be used while it runs, and it can't be interrupted: Ruby's
/// pending interrupts are handled once it returns.
#[doc(hidden)]
pub fn call_without_gvl<R, F>(func: F) -> Result<R, Failure>
where
    F: FnOnce() -> R,
{
    let mut func = Some(func);

    let outcome = call(|| {
        Thread::call_without_gvl(
            || {
                func.take()
                    .map(|func| panic::catch_unwind(AssertUnwindSafe(func)))
            },
            None::<fn()>,
        )
    })?;

    match outcome {
        Some(Ok(result)) => Ok(result),
        Some(Err(payload)) => Err(Failure::Exception(panic_exception(payload))),
        None => unreachable!("the function ran twice"),
    }
}

/// The arguments Ruby passed to a variadic method, without copying them.
///
/// Only for generated callbacks, which pass on what Ruby gave them; it is
/// safe so code using the attributes can `#![forbid(unsafe_code)]`.
#[doc(hidden)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn arguments<'a>(argc: Argc, argv: *const AnyObject) -> &'a [AnyObject] {
    if argc <= 0 || argv.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(argv, argc as usize) }
    }
}

/// The `ArgumentError` for a call with the wrong number of arguments, worded
/// as Ruby words it. `optional` is `None` for a splat (no maximum).
#[doc(hidden)]
pub fn check_arity(
    given: usize,
    required: usize,
    optional: Option<usize>,
) -> Result<(), AnyException> {
    let fits = given >= required && optional.is_none_or(|optional| given <= required + optional);

    if fits {
        return Ok(());
    }

    let expected = match optional {
        None => format!("{}+", required),
        Some(0) => required.to_string(),
        Some(optional) => format!("{}..{}", required, required + optional),
    };
    let message = format!(
        "wrong number of arguments (given {}, expected {})",
        given, expected
    );

    Err(AnyException::new("ArgumentError", Some(&message)))
}

// Ruby 2.7 says whether the caller passed keywords.
#[cfg(ruby_gte_2_7)]
fn keywords_given(_arguments: &[AnyObject]) -> bool {
    unsafe { crate::rubysys::vm::rb_keyword_given_p() != 0 }
}

// Ruby 2.5 and 2.6 pass keywords as a last `Hash`, and a last `Hash` whose
// keys are all Symbols is taken as keywords, as Ruby methods take it.
#[cfg(not(ruby_gte_2_7))]
fn keywords_given(arguments: &[AnyObject]) -> bool {
    match arguments.last() {
        Some(last) if last.ty() == ValueType::Hash => {
            let mut symbols = true;
            Hash::from(last.value()).each(|key, _| symbols &= key.ty() == ValueType::Symbol);

            symbols
        }
        _ => false,
    }
}

/// Keyword arguments, taken off the end of the arguments when the caller
/// passed keywords.
#[doc(hidden)]
pub struct Keywords {
    pairs: Vec<(String, AnyObject)>,
    used: Vec<bool>,
}

impl Keywords {
    /// Splits the keyword `Hash` off `arguments` if the caller passed
    /// keywords: a `**`/`key:` call on Ruby 2.7, a last `Hash` with Symbol
    /// keys on Ruby 2.5 and 2.6.
    pub fn split(arguments: &[AnyObject]) -> (&[AnyObject], Keywords) {
        let given = keywords_given(arguments);

        match arguments.split_last() {
            Some((last, rest)) if given && last.ty() == ValueType::Hash => {
                let mut pairs = Vec::new();

                Hash::from(last.value()).each(|key, value| {
                    let name = if key.ty() == ValueType::Symbol {
                        Symbol::from(key.value()).to_string()
                    } else {
                        // Not a keyword name; reported as unknown.
                        String::from("\0")
                    };

                    pairs.push((name, value));
                });

                let used = vec![false; pairs.len()];

                (rest, Keywords { pairs, used })
            }
            _ => (
                arguments,
                Keywords {
                    pairs: Vec::new(),
                    used: Vec::new(),
                },
            ),
        }
    }

    /// The value for `name`, if it was given (`nil` counts as given).
    pub fn take(&mut self, name: &str) -> Option<AnyObject> {
        let index = self.pairs.iter().position(|(key, _)| key == name)?;
        self.used[index] = true;

        Some(self.pairs[index].1.clone())
    }

    /// Converts the keyword `name`, which must be given.
    /// `None` (with `name` noted in `missing`) if it wasn't; `finish`
    /// then reports it.
    pub fn required<T: FromArgument + ?Sized>(
        &mut self,
        name: &'static str,
        missing: &mut Vec<&'static str>,
    ) -> Result<Option<T::Holder>, AnyException> {
        match self.take(name) {
            Some(value) => T::hold(value).map(Some),
            None => {
                missing.push(name);
                Ok(None)
            }
        }
    }

    /// Converts the keyword `name` if it was given and isn't `nil`.
    pub fn optional<T: FromArgument + ?Sized>(
        &mut self,
        name: &str,
    ) -> Result<Option<T::Holder>, AnyException> {
        match self.take(name) {
            Some(value) if !value.is_nil() => T::hold(value).map(Some),
            _ => Ok(None),
        }
    }

    /// Fails on missing required keywords, then on unknown keywords.
    pub fn finish(self, missing: Vec<&'static str>) -> Result<(), AnyException> {
        if !missing.is_empty() {
            return Err(keyword_error(
                "missing",
                &missing
                    .iter()
                    .map(|name| name.to_string())
                    .collect::<Vec<_>>(),
            ));
        }

        let unknown: Vec<String> = self
            .pairs
            .iter()
            .zip(self.used)
            .filter(|(_, used)| !used)
            .map(|((name, _), _)| name.clone())
            .collect();

        if unknown.is_empty() {
            Ok(())
        } else {
            Err(keyword_error("unknown", &unknown))
        }
    }
}

fn keyword_error(kind: &str, names: &[String]) -> AnyException {
    let names: Vec<String> = names
        .iter()
        .map(|name| {
            if name == "\0" {
                "(not a Symbol)".to_string()
            } else {
                format!(":{}", name)
            }
        })
        .collect();
    let plural = if names.len() == 1 { "" } else { "s" };
    let message = format!("{} keyword{}: {}", kind, plural, names.join(", "));

    AnyException::new("ArgumentError", Some(&message))
}

/// The block passed to the method, if any (`#[ruby(block)]`).
#[doc(hidden)]
pub fn block() -> Option<Proc> {
    if VM::is_block_given() {
        Some(VM::block_proc())
    } else {
        None
    }
}

#[doc(hidden)]
pub fn require_block(block: Option<Proc>) -> Result<Proc, AnyException> {
    block.ok_or_else(|| AnyException::new("LocalJumpError", Some("no block given")))
}

/// How a generated method is defined.
#[doc(hidden)]
#[derive(Clone, Copy)]
pub enum Definition {
    Public,
    Private,
    Protected,
    Singleton,
    ModuleFunction,
}

/// Defines a method from a generated callback: `arity` is the number of
/// arguments Ruby checks and passes as separate parameters, or -1 for
/// `(argc, argv, self)`.
///
/// Only for generated callbacks, whose signature matches `arity`; it is safe
/// so code using the attributes can `#![forbid(unsafe_code)]`.
#[doc(hidden)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn define(
    target: &impl Object,
    name: &str,
    callback: *const c_void,
    arity: c_int,
    how: Definition,
) {
    let name = util::str_to_cstring(name);
    let target = target.value();

    unsafe {
        match how {
            Definition::Public => rb_define_method(target, name.as_ptr(), callback, arity),
            Definition::Private => rb_define_private_method(target, name.as_ptr(), callback, arity),
            Definition::Protected => {
                rb_define_protected_method(target, name.as_ptr(), callback, arity)
            }
            Definition::Singleton => {
                rb_define_singleton_method(target, name.as_ptr(), callback, arity)
            }
            Definition::ModuleFunction => {
                rb_define_module_function(target, name.as_ptr(), callback, arity)
            }
        }
    }
}

/// One `#[rubymethods]` block of a `#[rubyclass]` struct; `define_ruby()`
/// defines blocks 0 to 7 that exist.
pub trait RubyMethods<const N: usize> {
    /// Defines the block's methods on `class`.
    fn define_methods(class: &mut Class);

    /// The block's methods as RBS declarations.
    fn rbs_methods(out: &mut Vec<String>);
}

#[doc(hidden)]
pub struct Part<S, const N: usize>(PhantomData<S>);

#[allow(clippy::new_without_default)]
impl<S, const N: usize> Part<S, N> {
    pub fn new() -> Self {
        Part(PhantomData)
    }
}

#[doc(hidden)]
pub trait DefinePart {
    fn define_part(&self, class: &mut Class);
    fn rbs_part(&self, out: &mut Vec<String>);
}

impl<S: RubyMethods<N>, const N: usize> DefinePart for Part<S, N> {
    fn define_part(&self, class: &mut Class) {
        S::define_methods(class);
    }

    fn rbs_part(&self, out: &mut Vec<String>) {
        S::rbs_methods(out);
    }
}

#[doc(hidden)]
pub trait DefinePartFallback {
    fn define_part(&self, class: &mut Class);
    fn rbs_part(&self, out: &mut Vec<String>);
}

impl<S, const N: usize> DefinePartFallback for &Part<S, N> {
    fn define_part(&self, _class: &mut Class) {}
    fn rbs_part(&self, _out: &mut Vec<String>) {}
}

/// `Some(Clone::clone)` when `T: Clone` (for `initialize_copy`).
#[doc(hidden)]
pub struct CloneProbe<T>(PhantomData<T>);

#[allow(clippy::new_without_default)]
impl<T> CloneProbe<T> {
    pub fn new() -> Self {
        CloneProbe(PhantomData)
    }
}

#[doc(hidden)]
pub trait ProbeClone<T> {
    fn clone_fn(&self) -> Option<fn(&T) -> T>;
}

impl<T: Clone> ProbeClone<T> for CloneProbe<T> {
    fn clone_fn(&self) -> Option<fn(&T) -> T> {
        Some(T::clone)
    }
}

#[doc(hidden)]
pub trait ProbeCloneFallback<T> {
    fn clone_fn(&self) -> Option<fn(&T) -> T>;
}

impl<T> ProbeCloneFallback<T> for &CloneProbe<T> {
    fn clone_fn(&self) -> Option<fn(&T) -> T> {
        None
    }
}

/// `initialize_copy` for `dup` and `clone`: copies `original`'s struct into
/// `copy`, which Ruby has just allocated.
#[doc(hidden)]
pub fn initialize_copy<T: RubyClass>(
    copy: &AnyObject,
    original: &AnyObject,
    clone: Option<fn(&T) -> T>,
) -> Result<AnyObject, Failure> {
    let clone = clone.ok_or_else(|| {
        let message = format!("can't copy {} (the Rust struct is not Clone)", T::PATH);

        AnyException::new("TypeError", Some(&message))
    })?;
    let value = {
        let original = T::borrow_from(original)?;

        call(|| clone(&original))?
    };

    T::initialize_object(copy, value)?;

    Ok(copy.clone())
}

/// Wraps a `#[rubyclass]` value returned by a method; `receiver` (a class,
/// for a singleton method) picks a subclass.
#[doc(hidden)]
pub fn wrap<T: RubyClass>(value: T, receiver: &AnyObject) -> Result<AnyObject, AnyException> {
    let class = T::class()?;

    let class = match receiver.try_convert_to::<Class>() {
        Ok(receiver) if receiver.inherits(&class) == Some(true) => receiver,
        _ => class,
    };

    Ok(value.into_object_of(&class))
}

/// Sets a constant of a `#[rubymodule]`.
#[doc(hidden)]
pub fn constant<V: IntoRubyObject>(module: &mut crate::Module, name: &str, value: V) {
    match value.into_ruby_object(&module.to_any_object()) {
        Ok(value) => module.const_set(name, &value),
        Err(exception) => vm::raise_ex(exception.value()),
    }
}
