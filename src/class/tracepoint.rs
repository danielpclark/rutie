use std::{convert::From, marker::PhantomData};

use crate::{
    binding::{tracepoint, vm},
    rubysys::tracepoint as rubysys_tracepoint,
    types::Value,
    AnyException, AnyObject, Array, Binding, Class, Exception, Integer, NilClass, Object, RString,
    Symbol, VerifiedObject,
};

fn protect<F>(func: F) -> Result<Value, AnyException>
where
    F: FnOnce() -> Value,
{
    vm::protect_value(func).map_err(AnyException::from)
}

fn non_nil(value: Value) -> Option<Value> {
    if value.is_nil() {
        None
    } else {
        Some(value)
    }
}

/// `TracePoint`: a hook that Ruby calls for events such as method calls,
/// returns, new lines and raised exceptions, in every thread.
///
/// A `TracePoint` made by [`TracePoint::new`](#method.new) calls a Rust
/// closure with a [`TraceArg`](struct.TraceArg.html) describing the event.
/// It starts disabled; [`enable`](#method.enable) it to start tracing. Ruby
/// does not call hooks from within a hook, so what the closure does is not
/// traced.
#[derive(Debug)]
#[repr(C)]
pub struct TracePoint {
    value: Value,
}

impl TracePoint {
    /// A new line of Ruby code (`:line`).
    pub const LINE: u32 = rubysys_tracepoint::RUBY_EVENT_LINE;
    /// The start of a class or module definition (`:class`).
    pub const CLASS: u32 = rubysys_tracepoint::RUBY_EVENT_CLASS;
    /// The end of a class or module definition (`:end`).
    pub const END: u32 = rubysys_tracepoint::RUBY_EVENT_END;
    /// A call of a method written in Ruby (`:call`).
    pub const CALL: u32 = rubysys_tracepoint::RUBY_EVENT_CALL;
    /// A return from a method written in Ruby (`:return`).
    pub const RETURN: u32 = rubysys_tracepoint::RUBY_EVENT_RETURN;
    /// A call of a method written in C (`:c_call`).
    pub const C_CALL: u32 = rubysys_tracepoint::RUBY_EVENT_C_CALL;
    /// A return from a method written in C (`:c_return`).
    pub const C_RETURN: u32 = rubysys_tracepoint::RUBY_EVENT_C_RETURN;
    /// A raised exception (`:raise`).
    pub const RAISE: u32 = rubysys_tracepoint::RUBY_EVENT_RAISE;
    /// The start of a block (`:b_call`).
    pub const B_CALL: u32 = rubysys_tracepoint::RUBY_EVENT_B_CALL;
    /// The end of a block (`:b_return`).
    pub const B_RETURN: u32 = rubysys_tracepoint::RUBY_EVENT_B_RETURN;
    /// The start of a thread (`:thread_begin`).
    pub const THREAD_BEGIN: u32 = rubysys_tracepoint::RUBY_EVENT_THREAD_BEGIN;
    /// The end of a thread (`:thread_end`).
    pub const THREAD_END: u32 = rubysys_tracepoint::RUBY_EVENT_THREAD_END;
    /// A switch to another fiber (`:fiber_switch`).
    pub const FIBER_SWITCH: u32 = rubysys_tracepoint::RUBY_EVENT_FIBER_SWITCH;
    /// Code compiled by `eval` or loaded from a file (`:script_compiled`).
    pub const SCRIPT_COMPILED: u32 = rubysys_tracepoint::RUBY_EVENT_SCRIPT_COMPILED;
    /// A rescued exception (`:rescue`).
    pub const RESCUE: u32 = rubysys_tracepoint::RUBY_EVENT_RESCUE;
    /// Every event above.
    pub const ALL: u32 = rubysys_tracepoint::RUBY_EVENT_TRACEPOINT_ALL;

    /// Creates a disabled TracePoint that calls `func` for each of `events`
    /// (constants such as [`TracePoint::CALL`](#associatedconstant.CALL),
    /// combined with `|`) (`rb_tracepoint_new`).
    ///
    /// What `func` raises (a panic becomes a `RuntimeError`) is raised in
    /// the traced code. `func` may run in any Ruby thread, but never twice
    /// at the same time: a call that would start while another thread is in
    /// `func` (waiting in Ruby code that released the GVL) is skipped.
    /// Ruby objects `func` keeps are not marked by the GC.
    ///
    /// Returns an `ArgumentError` for Ruby's internal events (such as
    /// `RUBY_INTERNAL_EVENT_NEWOBJ`), whose hooks may not use the Ruby API.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, TracePoint, VM};
    /// use std::sync::{Arc, Mutex};
    /// # VM::init();
    ///
    /// VM::eval("def rutie_double(x) = x * 2").unwrap();
    ///
    /// let events = Arc::new(Mutex::new(Vec::new()));
    /// let recorded = events.clone();
    ///
    /// let tracepoint = TracePoint::new(TracePoint::CALL | TracePoint::RETURN, move |arg| {
    ///     let returned = arg.return_value().ok().and_then(|value| value.try_convert_to::<Fixnum>().ok());
    ///
    ///     recorded.lock().unwrap().push((
    ///         arg.event().to_string(),
    ///         arg.method_id().unwrap().to_string(),
    ///         returned.map(|value| value.to_i64()),
    ///     ));
    /// })
    /// .unwrap();
    ///
    /// tracepoint.enable().unwrap();
    /// VM::eval("rutie_double(21)").unwrap();
    /// tracepoint.disable().unwrap();
    ///
    /// // Disabled: not recorded.
    /// VM::eval("rutie_double(1)").unwrap();
    ///
    /// assert_eq!(*events.lock().unwrap(), [
    ///     ("call".to_string(), "rutie_double".to_string(), None),
    ///     ("return".to_string(), "rutie_double".to_string(), Some(42)),
    /// ]);
    ///
    /// assert!(TracePoint::new(0x10_0000, |_| ()).is_err());
    /// ```
    pub fn new<F>(events: u32, mut func: F) -> Result<Self, AnyException>
    where
        F: FnMut(&TraceArg) + Send + 'static,
    {
        if events & rubysys_tracepoint::RUBY_INTERNAL_EVENT_MASK != 0 {
            return Err(AnyException::new(
                "ArgumentError",
                Some("internal events cannot be traced with a Rust closure"),
            ));
        }

        let tracepoint = tracepoint::new(events, move |arg| {
            func(&TraceArg {
                arg,
                _event: PhantomData,
            })
        });

        Ok(TracePoint::from(tracepoint))
    }

    /// Starts tracing (`rb_tracepoint_enable`); enabling an enabled
    /// TracePoint does nothing. Returns the error for a TracePoint that Ruby
    /// code enabled for a target (`enable(target:)`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{TracePoint, VM};
    /// # VM::init();
    ///
    /// let tracepoint = TracePoint::new(TracePoint::LINE, |_| ()).unwrap();
    ///
    /// tracepoint.enable().unwrap();
    /// assert!(tracepoint.is_enabled());
    /// tracepoint.disable().unwrap();
    /// ```
    pub fn enable(&self) -> Result<(), AnyException> {
        let tracepoint = self.value();

        protect(|| {
            tracepoint::enable(tracepoint);

            NilClass::new().value()
        })
        .map(|_| ())
    }

    /// Stops tracing (`rb_tracepoint_disable`). Returns the error for a
    /// TracePoint that Ruby code enabled for a target.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{TracePoint, VM};
    /// # VM::init();
    ///
    /// let tracepoint = TracePoint::new(TracePoint::CALL, |_| ()).unwrap();
    ///
    /// tracepoint.enable().unwrap();
    /// tracepoint.disable().unwrap();
    ///
    /// assert!(!tracepoint.is_enabled());
    /// ```
    pub fn disable(&self) -> Result<(), AnyException> {
        let tracepoint = self.value();

        protect(|| {
            tracepoint::disable(tracepoint);

            NilClass::new().value()
        })
        .map(|_| ())
    }

    /// Returns whether the TracePoint is enabled (`rb_tracepoint_enabled_p`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, TracePoint, VM};
    /// # VM::init();
    ///
    /// // A TracePoint made by Ruby code works too.
    /// let tracepoint = VM::eval("TracePoint.new(:line) {}").unwrap().try_convert_to::<TracePoint>().unwrap();
    ///
    /// assert!(!tracepoint.is_enabled());
    /// ```
    pub fn is_enabled(&self) -> bool {
        tracepoint::is_enabled(self.value())
    }
}

impl From<Value> for TracePoint {
    fn from(value: Value) -> Self {
        TracePoint { value }
    }
}

impl Into<Value> for TracePoint {
    fn into(self) -> Value {
        self.value
    }
}

impl Into<AnyObject> for TracePoint {
    fn into(self) -> AnyObject {
        AnyObject::from(self.value)
    }
}

impl Object for TracePoint {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for TracePoint {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        Class::from_existing("TracePoint").case_equals(object)
    }

    fn error_message() -> &'static str {
        "Error converting to TracePoint"
    }
}

impl PartialEq for TracePoint {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

/// The event a [`TracePoint`](struct.TracePoint.html) closure is called for
/// (`rb_trace_arg_t`, what Ruby passes to a `TracePoint.new` block). It is
/// only valid during the call.
///
/// The methods for details that only some events have (such as
/// [`return_value`](#method.return_value)) return Ruby's `RuntimeError` for
/// the other events.
pub struct TraceArg<'a> {
    arg: *mut rubysys_tracepoint::TraceArg,
    _event: PhantomData<&'a ()>,
}

impl<'a> TraceArg<'a> {
    /// Returns the event's name, such as `:line` or `:call`
    /// (`rb_tracearg_event`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{TracePoint, VM};
    /// use std::sync::{Arc, Mutex};
    /// # VM::init();
    ///
    /// let names = Arc::new(Mutex::new(Vec::new()));
    /// let recorded = names.clone();
    /// let tracepoint = TracePoint::new(TracePoint::C_CALL, move |arg| {
    ///     recorded.lock().unwrap().push(arg.event().to_string());
    /// }).unwrap();
    ///
    /// tracepoint.enable().unwrap();
    /// VM::eval("[].size").unwrap();
    /// tracepoint.disable().unwrap();
    ///
    /// assert!(names.lock().unwrap().iter().all(|name| name == "c_call"));
    /// assert!(!names.lock().unwrap().is_empty());
    /// ```
    pub fn event(&self) -> Symbol {
        Symbol::from(tracepoint::event(self.arg))
    }

    /// Returns the event as its flag, such as
    /// [`TracePoint::LINE`](struct.TracePoint.html#associatedconstant.LINE)
    /// (`rb_tracearg_event_flag`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{TracePoint, VM};
    /// use std::sync::{Arc, Mutex};
    /// # VM::init();
    ///
    /// let flags = Arc::new(Mutex::new(0));
    /// let seen = flags.clone();
    /// let tracepoint = TracePoint::new(TracePoint::RAISE | TracePoint::RESCUE, move |arg| {
    ///     *seen.lock().unwrap() |= arg.event_flag();
    /// }).unwrap();
    ///
    /// tracepoint.enable().unwrap();
    /// VM::eval("begin; raise 'x'; rescue; end").unwrap();
    /// tracepoint.disable().unwrap();
    ///
    /// assert_eq!(*flags.lock().unwrap(), TracePoint::RAISE | TracePoint::RESCUE);
    /// ```
    pub fn event_flag(&self) -> u32 {
        tracepoint::event_flag(self.arg)
    }

    /// Returns the line number of the event, or 0 outside of Ruby code
    /// (`rb_tracearg_lineno`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{TracePoint, VM};
    /// use std::sync::{Arc, Mutex};
    /// # VM::init();
    ///
    /// let lines = Arc::new(Mutex::new(Vec::new()));
    /// let recorded = lines.clone();
    /// let tracepoint = TracePoint::new(TracePoint::LINE, move |arg| {
    ///     recorded.lock().unwrap().push((arg.lineno(), arg.path().map(|path| path.to_string())));
    /// }).unwrap();
    ///
    /// tracepoint.enable().unwrap();
    /// VM::eval("a = 1\nb = 2\n").unwrap();
    /// tracepoint.disable().unwrap();
    ///
    /// assert_eq!(*lines.lock().unwrap(), [(1, Some("eval".to_string())), (2, Some("eval".to_string()))]);
    /// ```
    pub fn lineno(&self) -> i64 {
        Integer::from(tracepoint::lineno(self.arg)).to_i64()
    }

    /// Returns the path of the file of the event, or `None` outside of Ruby
    /// code (`rb_tracearg_path`). See [`lineno`](#method.lineno).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{TracePoint, VM};
    /// use std::sync::{Arc, Mutex};
    /// # VM::init();
    ///
    /// let paths = Arc::new(Mutex::new(Vec::new()));
    /// let recorded = paths.clone();
    /// let tracepoint = TracePoint::new(TracePoint::LINE, move |arg| {
    ///     recorded.lock().unwrap().push(arg.path().unwrap().to_string());
    /// }).unwrap();
    ///
    /// tracepoint.enable().unwrap();
    /// VM::eval("eval('1', nil, 'virtual.rb')").unwrap();
    /// tracepoint.disable().unwrap();
    ///
    /// assert!(paths.lock().unwrap().contains(&"virtual.rb".to_string()));
    /// ```
    pub fn path(&self) -> Option<RString> {
        non_nil(tracepoint::path(self.arg)).map(RString::from)
    }

    /// Returns the name of the method being called or returned from, as it
    /// was defined, or `None` (`rb_tracearg_method_id`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{TracePoint, VM};
    /// use std::sync::{Arc, Mutex};
    /// # VM::init();
    ///
    /// VM::eval("def rutie_original = 1; alias rutie_alias rutie_original").unwrap();
    ///
    /// let names = Arc::new(Mutex::new(Vec::new()));
    /// let recorded = names.clone();
    /// let tracepoint = TracePoint::new(TracePoint::CALL, move |arg| {
    ///     let method = arg.method_id().unwrap().to_string();
    ///     let callee = arg.callee_id().unwrap().to_string();
    ///     recorded.lock().unwrap().push((method, callee));
    /// }).unwrap();
    ///
    /// tracepoint.enable().unwrap();
    /// VM::eval("rutie_alias").unwrap();
    /// tracepoint.disable().unwrap();
    ///
    /// assert_eq!(*names.lock().unwrap(), [("rutie_original".to_string(), "rutie_alias".to_string())]);
    /// ```
    pub fn method_id(&self) -> Option<Symbol> {
        non_nil(tracepoint::method_id(self.arg)).map(Symbol::from)
    }

    /// Returns the name the method was called by (which differs from
    /// [`method_id`](#method.method_id) for an alias), or `None`
    /// (`rb_tracearg_callee_id`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{TracePoint, VM};
    /// use std::sync::{Arc, Mutex};
    /// # VM::init();
    ///
    /// let callees = Arc::new(Mutex::new(Vec::new()));
    /// let recorded = callees.clone();
    /// let tracepoint = TracePoint::new(TracePoint::C_CALL, move |arg| {
    ///     recorded.lock().unwrap().push(arg.callee_id().map(|id| id.to_string()));
    /// }).unwrap();
    ///
    /// tracepoint.enable().unwrap();
    /// VM::eval("[1].length").unwrap();
    /// tracepoint.disable().unwrap();
    ///
    /// assert!(callees.lock().unwrap().contains(&Some("length".to_string())));
    /// ```
    pub fn callee_id(&self) -> Option<Symbol> {
        non_nil(tracepoint::callee_id(self.arg)).map(Symbol::from)
    }

    /// Returns the class or module that defines the method of the event, or
    /// `None` (`rb_tracearg_defined_class`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Object, TracePoint, VM};
    /// use std::sync::{Arc, Mutex};
    /// # VM::init();
    ///
    /// VM::eval("class RutieTraced; def run = 1; end").unwrap();
    ///
    /// let classes = Arc::new(Mutex::new(Vec::new()));
    /// let recorded = classes.clone();
    /// let tracepoint = TracePoint::new(TracePoint::CALL, move |arg| {
    ///     recorded.lock().unwrap().push(arg.defined_class().unwrap().inspect_object().to_string());
    /// }).unwrap();
    ///
    /// tracepoint.enable().unwrap();
    /// VM::eval("RutieTraced.new.run").unwrap();
    /// tracepoint.disable().unwrap();
    ///
    /// assert_eq!(*classes.lock().unwrap(), ["RutieTraced"]);
    /// ```
    pub fn defined_class(&self) -> Option<AnyObject> {
        non_nil(tracepoint::defined_class(self.arg)).map(AnyObject::from)
    }

    /// Returns a binding of the code of the event, or `None` when it is not
    /// Ruby code (`rb_tracearg_binding`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, TracePoint, VM};
    /// use std::sync::{Arc, Mutex};
    /// # VM::init();
    ///
    /// VM::eval("def rutie_local(x) = x").unwrap();
    ///
    /// let values = Arc::new(Mutex::new(Vec::new()));
    /// let recorded = values.clone();
    /// let tracepoint = TracePoint::new(TracePoint::CALL, move |arg| {
    ///     let x = arg.binding().unwrap().local_variable_get("x").unwrap();
    ///     recorded.lock().unwrap().push(x.try_convert_to::<Fixnum>().unwrap().to_i64());
    /// }).unwrap();
    ///
    /// tracepoint.enable().unwrap();
    /// VM::eval("rutie_local(9)").unwrap();
    /// tracepoint.disable().unwrap();
    ///
    /// assert_eq!(*values.lock().unwrap(), [9]);
    /// ```
    pub fn binding(&self) -> Option<Binding> {
        non_nil(tracepoint::binding(self.arg)).map(Binding::from)
    }

    /// Returns `self` of the code of the event (Ruby's `TracePoint#self`,
    /// `rb_tracearg_self`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, TracePoint, VM};
    /// use std::sync::{Arc, Mutex};
    /// # VM::init();
    ///
    /// let receivers = Arc::new(Mutex::new(Vec::new()));
    /// let recorded = receivers.clone();
    /// let tracepoint = TracePoint::new(TracePoint::C_CALL, move |arg| {
    ///     recorded.lock().unwrap().push(arg.receiver().inspect_object().to_string());
    /// }).unwrap();
    ///
    /// tracepoint.enable().unwrap();
    /// VM::eval("'rutie'.size").unwrap();
    /// tracepoint.disable().unwrap();
    ///
    /// assert!(receivers.lock().unwrap().contains(&"\"rutie\"".to_string()));
    /// ```
    pub fn receiver(&self) -> AnyObject {
        AnyObject::from(tracepoint::receiver(self.arg))
    }

    /// Returns the value returned, for a `:return`, `:c_return` or
    /// `:b_return` event (`rb_tracearg_return_value`). See
    /// [`TracePoint::new`](struct.TracePoint.html#method.new).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, TracePoint, VM};
    /// use std::sync::{Arc, Mutex};
    /// # VM::init();
    ///
    /// let results = Arc::new(Mutex::new(Vec::new()));
    /// let recorded = results.clone();
    /// let tracepoint = TracePoint::new(TracePoint::B_CALL | TracePoint::B_RETURN, move |arg| {
    ///     let value = arg.return_value().map(|value| value.try_convert_to::<Fixnum>().unwrap().to_i64());
    ///     recorded.lock().unwrap().push(value.ok());
    /// }).unwrap();
    ///
    /// tracepoint.enable().unwrap();
    /// VM::eval("[5].map { |x| x + 1 }").unwrap();
    /// tracepoint.disable().unwrap();
    ///
    /// // The b_call event has no return value.
    /// assert_eq!(*results.lock().unwrap(), [None, Some(6)]);
    /// ```
    pub fn return_value(&self) -> Result<AnyObject, AnyException> {
        let arg = self.arg;

        protect(|| tracepoint::return_value(arg)).map(AnyObject::from)
    }

    /// Returns the exception of a `:raise` or `:rescue` event
    /// (`rb_tracearg_raised_exception`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Exception, TracePoint, VM};
    /// use std::sync::{Arc, Mutex};
    /// # VM::init();
    ///
    /// let messages = Arc::new(Mutex::new(Vec::new()));
    /// let recorded = messages.clone();
    /// let tracepoint = TracePoint::new(TracePoint::RAISE, move |arg| {
    ///     recorded.lock().unwrap().push(arg.raised_exception().unwrap().message());
    /// }).unwrap();
    ///
    /// tracepoint.enable().unwrap();
    /// let _ = VM::eval("raise 'traced failure'");
    /// tracepoint.disable().unwrap();
    ///
    /// assert_eq!(*messages.lock().unwrap(), ["traced failure"]);
    /// ```
    pub fn raised_exception(&self) -> Result<AnyException, AnyException> {
        let arg = self.arg;

        protect(|| tracepoint::raised_exception(arg)).map(AnyException::from)
    }

    /// Returns the parameters of the method or block of a call or return
    /// event, like `Method#parameters` (`rb_tracearg_parameters`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, TracePoint, VM};
    /// use std::sync::{Arc, Mutex};
    /// # VM::init();
    ///
    /// VM::eval("def rutie_params(a, b = 1, *rest, key:) = nil").unwrap();
    ///
    /// let parameters = Arc::new(Mutex::new(String::new()));
    /// let recorded = parameters.clone();
    /// let tracepoint = TracePoint::new(TracePoint::CALL, move |arg| {
    ///     *recorded.lock().unwrap() = arg.parameters().unwrap().inspect_object().to_string();
    /// }).unwrap();
    ///
    /// tracepoint.enable().unwrap();
    /// VM::eval("rutie_params(1, key: 2)").unwrap();
    /// tracepoint.disable().unwrap();
    ///
    /// assert_eq!(*parameters.lock().unwrap(), "[[:req, :a], [:opt, :b], [:rest, :rest], [:keyreq, :key]]");
    /// ```
    pub fn parameters(&self) -> Result<Array, AnyException> {
        let arg = self.arg;

        protect(|| tracepoint::parameters(arg)).map(Array::from)
    }

    /// Returns the source of a `:script_compiled` event, or `None` when the
    /// code was loaded from a file (`rb_tracearg_eval_script`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{TracePoint, VM};
    /// use std::sync::{Arc, Mutex};
    /// # VM::init();
    ///
    /// let scripts = Arc::new(Mutex::new(Vec::new()));
    /// let recorded = scripts.clone();
    /// let tracepoint = TracePoint::new(TracePoint::SCRIPT_COMPILED, move |arg| {
    ///     recorded.lock().unwrap().push(arg.eval_script().unwrap().map(|script| script.to_string()));
    /// }).unwrap();
    ///
    /// tracepoint.enable().unwrap();
    /// VM::eval("eval('6 * 7')").unwrap();
    /// tracepoint.disable().unwrap();
    ///
    /// assert!(scripts.lock().unwrap().contains(&Some("6 * 7".to_string())));
    /// ```
    pub fn eval_script(&self) -> Result<Option<RString>, AnyException> {
        let arg = self.arg;

        protect(|| tracepoint::eval_script(arg)).map(|script| non_nil(script).map(RString::from))
    }

    /// Returns the `RubyVM::InstructionSequence` of a `:script_compiled`
    /// event (`rb_tracearg_instruction_sequence`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, TracePoint, VM};
    /// use std::sync::{Arc, Mutex};
    /// # VM::init();
    ///
    /// let classes = Arc::new(Mutex::new(Vec::new()));
    /// let recorded = classes.clone();
    /// let tracepoint = TracePoint::new(TracePoint::SCRIPT_COMPILED | TracePoint::C_CALL, move |arg| {
    ///     recorded.lock().unwrap().push(arg.instruction_sequence().map(|iseq| iseq.class_name()).ok());
    /// }).unwrap();
    ///
    /// tracepoint.enable().unwrap();
    /// VM::eval("eval('1')").unwrap();
    /// tracepoint.disable().unwrap();
    ///
    /// let classes = classes.lock().unwrap();
    /// assert!(classes.contains(&Some("RubyVM::InstructionSequence".to_string())));
    /// // Other events have none.
    /// assert!(classes.contains(&None));
    /// ```
    pub fn instruction_sequence(&self) -> Result<AnyObject, AnyException> {
        let arg = self.arg;

        protect(|| tracepoint::instruction_sequence(arg)).map(AnyObject::from)
    }

    /// Returns the object of an object allocation or freeing event
    /// (`rb_tracearg_object`). Those are internal events, which a Rust
    /// closure cannot trace, so this always returns the `RuntimeError`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{TracePoint, VM};
    /// use std::sync::{Arc, Mutex};
    /// # VM::init();
    ///
    /// let failed = Arc::new(Mutex::new(false));
    /// let recorded = failed.clone();
    /// let tracepoint = TracePoint::new(TracePoint::C_CALL, move |arg| {
    ///     *recorded.lock().unwrap() = arg.object().is_err();
    /// }).unwrap();
    ///
    /// tracepoint.enable().unwrap();
    /// VM::eval("Object.new").unwrap();
    /// tracepoint.disable().unwrap();
    ///
    /// assert!(*failed.lock().unwrap());
    /// ```
    pub fn object(&self) -> Result<AnyObject, AnyException> {
        let arg = self.arg;

        protect(|| tracepoint::object(arg)).map(AnyObject::from)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Class, Exception, Object, Symbol, TracePoint, VM};
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    };

    #[test]
    fn test_tracepoint_events() {
        crate::on_ruby_thread(|| {
            VM::eval("def rutie_tp_add(a, b) = a + b").unwrap();

            let events = Arc::new(Mutex::new(Vec::new()));
            let recorded = events.clone();
            let tracepoint = TracePoint::new(
                TracePoint::CALL | TracePoint::RETURN | TracePoint::LINE,
                move |arg| {
                    let flag = arg.event_flag();
                    let name = arg.method_id().map(|id| id.to_string());
                    let params = arg.parameters().map(|params| params.length()).ok();
                    recorded
                        .lock()
                        .unwrap()
                        .push((flag, name, params, arg.lineno() > 0));
                },
            )
            .unwrap();
            assert!(tracepoint.class_name() == "TracePoint");
            assert!(!tracepoint.is_enabled());

            tracepoint.enable().unwrap();
            tracepoint.enable().unwrap();
            VM::eval("rutie_tp_add(1, 2)").unwrap();
            tracepoint.disable().unwrap();
            assert!(!tracepoint.is_enabled());

            let events = events.lock().unwrap();
            let calls: Vec<_> = events
                .iter()
                .filter(|(flag, _, _, _)| *flag != TracePoint::LINE)
                .cloned()
                .collect();
            let add = Some("rutie_tp_add".to_string());
            assert_eq!(
                calls,
                [
                    (TracePoint::CALL, add.clone(), Some(2), true),
                    (TracePoint::RETURN, add, Some(2), true)
                ]
            );
            // Line events have no parameters.
            assert!(events
                .iter()
                .any(|(flag, _, params, _)| *flag == TracePoint::LINE && params.is_none()));
        });
    }

    #[test]
    fn test_tracepoint_errors_panics_and_lifetime() {
        crate::on_ruby_thread(|| {
            assert!(TracePoint::new(TracePoint::LINE | 0x20_0000, |_| ()).is_err());

            // A Ruby exception raised by the closure reaches the traced code.
            let raising = TracePoint::new(TracePoint::C_CALL, |arg| {
                if arg.method_id().map(|id| id.to_string()) == Some("rutie_marker".to_string()) {
                    VM::raise(Class::from_existing("IndexError"), "from the hook");
                }
            })
            .unwrap();
            VM::eval("def Object.rutie_marker = 1; Object.define_singleton_method(:rutie_c_marker, &:to_s)").unwrap();
            raising.enable().unwrap();
            let result = VM::eval("[].size");
            raising.disable().unwrap();
            assert!(result.is_ok());

            let panicking = TracePoint::new(TracePoint::CALL, |arg| {
                if arg.method_id().map(|id| id.to_string()) == Some("rutie_marker".to_string()) {
                    panic!("hook panic");
                }
            })
            .unwrap();
            panicking.enable().unwrap();
            let error = VM::eval("Object.rutie_marker").unwrap_err();
            panicking.disable().unwrap();
            assert!(Class::from_existing("RuntimeError").case_equals(&error));
            assert!(error.message().contains("hook panic"));

            // The hook still works after raising.
            let count = Arc::new(AtomicUsize::new(0));
            let counter = count.clone();
            let counting = TracePoint::new(TracePoint::CALL, move |_| {
                counter.fetch_add(1, Ordering::SeqCst);
            })
            .unwrap();
            counting.enable().unwrap();
            VM::eval("Object.rutie_marker; Object.rutie_marker").unwrap();
            counting.disable().unwrap();
            assert_eq!(count.load(Ordering::SeqCst), 2);

            // The closure stays alive with the TracePoint through GCs.
            let survivor = Arc::new(AtomicUsize::new(0));
            let counter = survivor.clone();
            let tracepoint = TracePoint::new(TracePoint::RAISE, move |arg| {
                assert_eq!(arg.raised_exception().unwrap().message(), "x");
                assert!(arg.return_value().is_err());
                counter.fetch_add(1, Ordering::SeqCst);
            })
            .unwrap();
            tracepoint.enable().unwrap();
            crate::GC::start();
            crate::GC::start();
            let _ = VM::eval("raise 'x'");
            tracepoint.disable().unwrap();
            assert_eq!(survivor.load(Ordering::SeqCst), 1);

            // Ruby's own TracePoints convert.
            let ruby = VM::eval("TracePoint.new(:call) {}").unwrap();
            assert!(ruby.try_convert_to::<TracePoint>().is_ok());
            assert!(Symbol::new("x").try_convert_to::<TracePoint>().is_err());
        });
    }

    #[test]
    fn test_tracepoint_compiled_scripts() {
        crate::on_ruby_thread(|| {
            let seen = Arc::new(Mutex::new(Vec::new()));
            let recorded = seen.clone();
            let tracepoint = TracePoint::new(TracePoint::SCRIPT_COMPILED, move |arg| {
                let script = arg.eval_script().unwrap().map(|script| script.to_string());
                let iseq = arg.instruction_sequence().unwrap().class_name();
                assert!(arg.parameters().is_err());
                recorded
                    .lock()
                    .unwrap()
                    .push((script, iseq, arg.event().to_string()));
            })
            .unwrap();

            tracepoint.enable().unwrap();
            VM::eval("eval('1 + 1')").unwrap();
            tracepoint.disable().unwrap();

            assert!(seen.lock().unwrap().contains(&(
                Some("1 + 1".to_string()),
                "RubyVM::InstructionSequence".to_string(),
                "script_compiled".to_string()
            )));
        });
    }
}
