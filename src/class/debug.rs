use std::convert::From;

use crate::{
    binding::{debug, vm},
    rubysys::debug as rubysys_debug,
    types::{c_int, Value},
    AnyException, AnyObject, Array, Integer, NilClass, Object, RString,
};

// `nil` as `None`.
fn optional(value: Value) -> Option<Value> {
    if value.is_nil() {
        None
    } else {
        Some(value)
    }
}

/// A frame of a Ruby stack, as a sampling profiler sees it (from
/// [`VM::profile_frames`](struct.VM.html#method.profile_frames), or
/// [`Thread::profile_frames`](struct.Thread.html#method.profile_frames) on
/// Ruby 3.3+).
///
/// A frame is an opaque Ruby internal object (an instruction sequence or a
/// method entry), not one Ruby methods can be called on, so it is only used
/// through these methods (`rb_profile_frame_*`). Like other Ruby objects, it
/// is only kept from the garbage collector while Ruby references it (as it
/// does while the method it belongs to is defined) or it is on the stack.
///
/// # Examples
///
/// ```
/// #[macro_use] extern crate rutie;
///
/// use rutie::{Array, Class, Object, RString, VM};
///
/// methods!(
///     rutie::AnyObject,
///     _rtself,
///     fn profile_labels() -> Array {
///         let mut labels = Array::new();
///
///         for frame in VM::profile_frames(0, 10) {
///             labels.push(frame.full_label().unwrap());
///         }
///
///         labels
///     }
/// );
///
/// fn main() {
///     # VM::init();
///     Class::from_existing("Object").define(|klass| {
///         klass.def_private("profile_labels", profile_labels);
///     });
///
///     let labels = VM::eval("class Report; def self.build = profile_labels; end; Report.build").unwrap();
///     let labels = labels.try_convert_to::<Array>().unwrap();
///
///     let label = |i| labels.at(i).try_convert_to::<RString>().unwrap().to_string();
///     assert_eq!(label(0), "Object#profile_labels");
///     assert_eq!(label(1), "Report.build");
///     // The top frame of `VM::eval` code: `<compiled>` from Ruby 3.4.
///     let top = if cfg!(ruby_gte_3_4) { "<compiled>" } else { "<main>" };
///     assert_eq!(label(2), top);
/// }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct ProfileFrame {
    frame: Value,
    line: c_int,
}

impl ProfileFrame {
    pub(crate) fn from_frames(frames: Vec<(Value, c_int)>) -> Vec<ProfileFrame> {
        frames
            .into_iter()
            .map(|(frame, line)| ProfileFrame { frame, line })
            .collect()
    }

    fn string(value: Value) -> Option<RString> {
        optional(value).map(RString::from)
    }

    /// Returns the line the frame is running (`0` for a method written in
    /// C).
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Class, Fixnum, Object, VM};
    ///
    /// methods!(
    ///     rutie::AnyObject,
    ///     _rtself,
    ///     fn caller_line() -> Fixnum {
    ///         let frames = VM::profile_frames(0, 2);
    ///
    ///         // Frame 0 is this method itself, written in Rust (C, to Ruby).
    ///         assert_eq!(frames[0].line(), 0);
    ///
    ///         Fixnum::new(frames[1].line() as i64)
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::from_existing("Object").define(|klass| {
    ///         klass.def_private("caller_line", caller_line);
    ///     });
    ///
    ///     let line = VM::eval("x = 1\ny = 2\ncaller_line").unwrap();
    ///     assert_eq!(line, Fixnum::new(3).into());
    /// }
    /// ```
    pub fn line(&self) -> i32 {
        self.line
    }

    /// Returns the path of the file the frame's code is from, as it was
    /// loaded (`rb_profile_frame_path`), or `None` for a method written in C.
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Class, NilClass, Object, RString, VM};
    ///
    /// methods!(
    ///     rutie::AnyObject,
    ///     _rtself,
    ///     fn frame_paths() -> rutie::Array {
    ///         let mut paths = rutie::Array::new();
    ///         for frame in VM::profile_frames(0, 2) {
    ///             match frame.path() {
    ///                 Some(path) => paths.push(path),
    ///                 None => paths.push(NilClass::new()),
    ///             };
    ///         }
    ///         paths
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::from_existing("Object").define(|klass| {
    ///         klass.def_private("frame_paths", frame_paths);
    ///     });
    ///
    ///     let paths = VM::eval("eval('frame_paths', nil, 'report.rb')").unwrap();
    ///     let paths = paths.try_convert_to::<rutie::Array>().unwrap();
    ///
    ///     // `frame_paths` is written in Rust (C, to Ruby).
    ///     assert!(paths.at(0).is_nil());
    ///     assert_eq!(paths.at(1).try_convert_to::<RString>().unwrap().to_str(), "report.rb");
    /// }
    /// ```
    pub fn path(&self) -> Option<RString> {
        Self::string(debug::profile_frame_path(self.frame))
    }

    /// Returns the expanded path of the file the frame's code is from
    /// (`rb_profile_frame_absolute_path`): `"<cfunc>"` for a method written
    /// in C, `None` when there is no file (such as code run with `eval`).
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Class, NilClass, Object, RString, VM};
    ///
    /// methods!(
    ///     rutie::AnyObject,
    ///     _rtself,
    ///     fn own_absolute_path() -> rutie::AnyObject {
    ///         match VM::profile_frames(0, 1)[0].absolute_path() {
    ///             Some(path) => path.into(),
    ///             None => NilClass::new().into(),
    ///         }
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::from_existing("Object").define(|klass| {
    ///         klass.def_private("own_absolute_path", own_absolute_path);
    ///     });
    ///
    ///     let path = VM::eval("own_absolute_path").unwrap();
    ///     assert_eq!(path.try_convert_to::<RString>().unwrap().to_str(), "<cfunc>");
    /// }
    /// ```
    pub fn absolute_path(&self) -> Option<RString> {
        Self::string(debug::profile_frame_absolute_path(self.frame))
    }

    /// Returns the frame's label, as in a backtrace (`rb_profile_frame_label`):
    /// the method name, `"<main>"`, `"<class:...>"`...; `None` for a method
    /// written in C. (A block is reported as the method it is in.)
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Class, Object, RString, VM};
    ///
    /// methods!(
    ///     rutie::AnyObject,
    ///     _rtself,
    ///     fn caller_label() -> RString {
    ///         let frames = VM::profile_frames(0, 2);
    ///         assert!(frames[0].label().is_none());
    ///
    ///         frames[1].label().unwrap()
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::from_existing("Object").define(|klass| {
    ///         klass.def_private("caller_label", caller_label);
    ///     });
    ///
    ///     let label = VM::eval("def run = caller_label; run").unwrap();
    ///     assert_eq!(label.try_convert_to::<RString>().unwrap().to_str(), "run");
    ///
    ///     let label = VM::eval("class Setup; caller_label; end").unwrap();
    ///     assert_eq!(label.try_convert_to::<RString>().unwrap().to_str(), "<class:Setup>");
    /// }
    /// ```
    pub fn label(&self) -> Option<RString> {
        Self::string(debug::profile_frame_label(self.frame))
    }

    /// Like [`label`](#method.label), without qualifiers such as
    /// `"block in "` (`rb_profile_frame_base_label`).
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Class, Object, RString, VM};
    ///
    /// methods!(
    ///     rutie::AnyObject,
    ///     _rtself,
    ///     fn caller_base_label() -> RString {
    ///         VM::profile_frames(1, 1)[0].base_label().unwrap()
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::from_existing("Object").define(|klass| {
    ///         klass.def_private("caller_base_label", caller_base_label);
    ///     });
    ///
    ///     let label = VM::eval("def run = caller_base_label; run").unwrap();
    ///     assert_eq!(label.try_convert_to::<RString>().unwrap().to_str(), "run");
    /// }
    /// ```
    pub fn base_label(&self) -> Option<RString> {
        Self::string(debug::profile_frame_base_label(self.frame))
    }

    /// Like [`label`](#method.label), qualified with the class
    /// (`rb_profile_frame_full_label`), such as `"Report.build"` or
    /// `"Report#rows"`. Methods written in C have one too.
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Class, Object, RString, VM};
    ///
    /// methods!(
    ///     rutie::AnyObject,
    ///     _rtself,
    ///     fn full_labels() -> rutie::Array {
    ///         let mut labels = rutie::Array::new();
    ///         for frame in VM::profile_frames(0, 2) {
    ///             labels.push(frame.full_label().unwrap());
    ///         }
    ///         labels
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::from_existing("Object").define(|klass| {
    ///         klass.def_private("full_labels", full_labels);
    ///     });
    ///
    ///     let labels = VM::eval("class Report; def rows = full_labels; end; Report.new.rows").unwrap();
    ///     let labels = labels.try_convert_to::<rutie::Array>().unwrap();
    ///
    ///     assert_eq!(labels.at(0).try_convert_to::<RString>().unwrap().to_str(), "Object#full_labels");
    ///     assert_eq!(labels.at(1).try_convert_to::<RString>().unwrap().to_str(), "Report#rows");
    /// }
    /// ```
    pub fn full_label(&self) -> Option<RString> {
        Self::string(debug::profile_frame_full_label(self.frame))
    }

    /// Returns the line the frame's method or block starts on
    /// (`rb_profile_frame_first_lineno`), or `None` for a method written in
    /// C.
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Class, Fixnum, Object, VM};
    ///
    /// methods!(
    ///     rutie::AnyObject,
    ///     _rtself,
    ///     fn caller_first_line() -> Fixnum {
    ///         let frames = VM::profile_frames(0, 2);
    ///         assert_eq!(frames[0].first_lineno(), None);
    ///
    ///         Fixnum::new(frames[1].first_lineno().unwrap())
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::from_existing("Object").define(|klass| {
    ///         klass.def_private("caller_first_line", caller_first_line);
    ///     });
    ///
    ///     let line = VM::eval("\n\ndef run\n  caller_first_line\nend\nrun").unwrap();
    ///     assert_eq!(line, Fixnum::new(3).into());
    /// }
    /// ```
    pub fn first_lineno(&self) -> Option<i64> {
        optional(debug::profile_frame_first_lineno(self.frame))
            .map(|line| Integer::from(line).to_i64())
    }

    /// Returns the name of the class or module of the frame's method
    /// (`rb_profile_frame_classpath`), or `None` (for the top level).
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Class, Object, RString, VM};
    ///
    /// methods!(
    ///     rutie::AnyObject,
    ///     _rtself,
    ///     fn caller_class() -> RString {
    ///         VM::profile_frames(1, 1)[0].classpath().unwrap()
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::from_existing("Object").define(|klass| {
    ///         klass.def_private("caller_class", caller_class);
    ///     });
    ///
    ///     let class = VM::eval("module Billing; class Invoice; def total = caller_class; end; end; Billing::Invoice.new.total").unwrap();
    ///     assert_eq!(class.try_convert_to::<RString>().unwrap().to_str(), "Billing::Invoice");
    /// }
    /// ```
    pub fn classpath(&self) -> Option<RString> {
        Self::string(debug::profile_frame_classpath(self.frame))
    }

    /// Returns whether the frame's method is a singleton method, such as a
    /// class method (`rb_profile_frame_singleton_method_p`).
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Boolean, Class, Object, VM};
    ///
    /// methods!(
    ///     rutie::AnyObject,
    ///     _rtself,
    ///     fn called_from_singleton() -> Boolean {
    ///         Boolean::new(VM::profile_frames(1, 1)[0].is_singleton_method())
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::from_existing("Object").define(|klass| {
    ///         klass.def_private("called_from_singleton", called_from_singleton);
    ///     });
    ///
    ///     VM::eval("class Job; def self.build = called_from_singleton; def run = called_from_singleton; end").unwrap();
    ///     assert!(VM::eval("Job.build").unwrap().is_true());
    ///     assert!(VM::eval("Job.new.run").unwrap().is_false());
    /// }
    /// ```
    pub fn is_singleton_method(&self) -> bool {
        debug::profile_frame_is_singleton_method(self.frame)
    }

    /// Returns the name of the frame's method (`rb_profile_frame_method_name`),
    /// or `None` when the frame is not in a method.
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Class, NilClass, Object, RString, VM};
    ///
    /// methods!(
    ///     rutie::AnyObject,
    ///     _rtself,
    ///     fn caller_method() -> rutie::AnyObject {
    ///         match VM::profile_frames(1, 1)[0].method_name() {
    ///             Some(name) => name.into(),
    ///             None => NilClass::new().into(),
    ///         }
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::from_existing("Object").define(|klass| {
    ///         klass.def_private("caller_method", caller_method);
    ///     });
    ///
    ///     let name = VM::eval("def checkout = caller_method; checkout").unwrap();
    ///     assert_eq!(name.try_convert_to::<RString>().unwrap().to_str(), "checkout");
    ///
    ///     // The top level is not a method.
    ///     assert!(VM::eval("caller_method").unwrap().is_nil());
    /// }
    /// ```
    pub fn method_name(&self) -> Option<RString> {
        Self::string(debug::profile_frame_method_name(self.frame))
    }

    /// Like [`method_name`](#method.method_name), qualified with the class
    /// (`rb_profile_frame_qualified_method_name`), such as `"Job.build"` or
    /// `"Job#run"`.
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Class, Object, RString, VM};
    ///
    /// methods!(
    ///     rutie::AnyObject,
    ///     _rtself,
    ///     fn caller_qualified_method() -> RString {
    ///         VM::profile_frames(1, 1)[0].qualified_method_name().unwrap()
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::from_existing("Object").define(|klass| {
    ///         klass.def_private("caller_qualified_method", caller_qualified_method);
    ///     });
    ///
    ///     VM::eval("class Job; def self.build = caller_qualified_method; def run = caller_qualified_method; end").unwrap();
    ///     assert_eq!(VM::eval("Job.build").unwrap().try_convert_to::<RString>().unwrap().to_str(), "Job.build");
    ///     assert_eq!(VM::eval("Job.new.run").unwrap().try_convert_to::<RString>().unwrap().to_str(), "Job#run");
    /// }
    /// ```
    pub fn qualified_method_name(&self) -> Option<RString> {
        Self::string(debug::profile_frame_qualified_method_name(self.frame))
    }
}

/// A view of the Ruby stack for debuggers, only available inside
/// [`DebugInspector::open`](#method.open) (`rb_debug_inspector_*`).
///
/// Frames are numbered from the top (0, the innermost) like
/// [`backtrace_locations`](#method.backtrace_locations); the methods taking
/// a frame index return `None` for one out of range.
///
/// # Examples
///
/// ```
/// #[macro_use] extern crate rutie;
///
/// use rutie::{AnyObject, Class, DebugInspector, Object, VM};
///
/// methods!(
///     rutie::AnyObject,
///     _rtself,
///     fn caller_self() -> AnyObject {
///         // Frame 0 is this method; frame 1 is its caller.
///         DebugInspector::open(|inspector| inspector.frame_self(1).unwrap()).unwrap()
///     }
/// );
///
/// fn main() {
///     # VM::init();
///     Class::from_existing("Object").define(|klass| {
///         klass.def_private("caller_self", caller_self);
///     });
///
///     let same = VM::eval("class Order; def who = caller_self; end; o = Order.new; o.who.equal?(o)").unwrap();
///     assert!(same.is_true());
/// }
/// ```
pub struct DebugInspector {
    context: *const rubysys_debug::DebugInspector,
    frames: usize,
}

impl DebugInspector {
    /// Calls `func` with a debug inspector for the current stack
    /// (`rb_debug_inspector_open`) and returns its result, or the exception
    /// it raised.
    ///
    /// The inspector cannot outlive the call.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{DebugInspector, Object, VM};
    /// # VM::init();
    ///
    /// let (count, locations) = DebugInspector::open(|inspector| {
    ///     (inspector.frame_count(), inspector.backtrace_locations().length())
    /// })
    /// .unwrap();
    /// assert_eq!(count, locations);
    ///
    /// let error = DebugInspector::open(|_| {
    ///     VM::raise(rutie::Class::from_existing("ArgumentError"), "no frames wanted");
    /// })
    /// .unwrap_err();
    /// assert_eq!(rutie::Exception::message(&error), "no frames wanted");
    /// ```
    pub fn open<F, R>(func: F) -> Result<R, AnyException>
    where
        F: FnOnce(&DebugInspector) -> R,
    {
        let mut result = None;

        vm::protect_value(|| {
            debug::debug_inspector_open(|context| {
                let locations = unsafe { debug::debug_inspector_backtrace_locations(context) };
                let inspector = DebugInspector {
                    context,
                    frames: Array::from(locations).length(),
                };

                result = Some(func(&inspector));

                NilClass::new().value()
            })
        })
        .map_err(AnyException::from)?;

        Ok(result.expect("the debug inspector did not run"))
    }

    /// Returns the frames as `Thread::Backtrace::Location`s, as Ruby's
    /// `caller_locations(0)` would at this point
    /// (`rb_debug_inspector_backtrace_locations`).
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Array, Class, DebugInspector, Object, RString, VM};
    ///
    /// methods!(
    ///     rutie::AnyObject,
    ///     _rtself,
    ///     fn locations() -> Array {
    ///         DebugInspector::open(|inspector| inspector.backtrace_locations()).unwrap()
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::from_existing("Object").define(|klass| {
    ///         klass.def_private("locations", locations);
    ///     });
    ///
    ///     // `base_label`: Ruby 3.4's `label` adds the owner (`Object#outer`).
    ///     let labels = VM::eval("def outer = locations; outer.map(&:base_label)").unwrap();
    ///     let labels = labels.try_convert_to::<Array>().unwrap();
    ///
    ///     assert_eq!(labels.at(0).try_convert_to::<RString>().unwrap().to_str(), "locations");
    ///     assert_eq!(labels.at(1).try_convert_to::<RString>().unwrap().to_str(), "outer");
    /// }
    /// ```
    pub fn backtrace_locations(&self) -> Array {
        Array::from(unsafe { debug::debug_inspector_backtrace_locations(self.context) })
    }

    /// Returns the number of frames (the length of
    /// [`backtrace_locations`](#method.backtrace_locations)).
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Class, DebugInspector, Fixnum, Object, VM};
    ///
    /// methods!(
    ///     rutie::AnyObject,
    ///     _rtself,
    ///     fn frame_count() -> Fixnum {
    ///         Fixnum::new(DebugInspector::open(|inspector| inspector.frame_count()).unwrap() as i64)
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::from_existing("Object").define(|klass| {
    ///         klass.def_private("frame_count", frame_count);
    ///     });
    ///
    ///     let counts = VM::eval("def deeper = frame_count; [frame_count, deeper]").unwrap();
    ///     let counts = counts.try_convert_to::<rutie::Array>().unwrap();
    ///     let count = |i| counts.at(i).try_convert_to::<Fixnum>().unwrap().to_i64();
    ///
    ///     assert_eq!(count(1), count(0) + 1);
    /// }
    /// ```
    pub fn frame_count(&self) -> usize {
        self.frames
    }

    fn frame<F>(&self, index: usize, get: F) -> Option<AnyObject>
    where
        F: FnOnce(*const rubysys_debug::DebugInspector, usize) -> Value,
    {
        if index < self.frames {
            Some(AnyObject::from(get(self.context, index)))
        } else {
            None
        }
    }

    /// Returns `self` in the frame at `index`
    /// (`rb_debug_inspector_frame_self_get`).
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{AnyObject, Class, DebugInspector, Fixnum, Object, VM};
    ///
    /// methods!(
    ///     rutie::AnyObject,
    ///     _rtself,
    ///     fn block_self() -> AnyObject {
    ///         // 0: this method, 1: the block, 2: `Array#map`.
    ///         DebugInspector::open(|inspector| {
    ///             assert!(inspector.frame_self(1000).is_none());
    ///
    ///             inspector.frame_self(2).unwrap()
    ///         })
    ///         .unwrap()
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::from_existing("Object").define(|klass| {
    ///         klass.def_private("block_self", block_self);
    ///     });
    ///
    ///     let receiver = VM::eval("[7].map { block_self }.first").unwrap();
    ///     assert_eq!(VM::eval("[7]").unwrap().protect_send("==", &[receiver]).unwrap().is_true(), true);
    /// }
    /// ```
    pub fn frame_self(&self, index: usize) -> Option<AnyObject> {
        self.frame(index, |context, index| unsafe {
            debug::debug_inspector_frame_self(context, index)
        })
    }

    /// Returns the class of the method running in the frame at `index`
    /// (`rb_debug_inspector_frame_class_get`), `nil` at the top level.
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{AnyObject, Class, DebugInspector, Object, VM};
    ///
    /// methods!(
    ///     rutie::AnyObject,
    ///     _rtself,
    ///     fn caller_class() -> AnyObject {
    ///         DebugInspector::open(|inspector| inspector.frame_class(1).unwrap()).unwrap()
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::from_existing("Object").define(|klass| {
    ///         klass.def_private("caller_class", caller_class);
    ///     });
    ///
    ///     let class = VM::eval("class Base; def who = caller_class; end; class Child < Base; end; Child.new.who").unwrap();
    ///     assert_eq!(class, Class::from_existing("Base").into());
    ///
    ///     assert!(VM::eval("caller_class").unwrap().is_nil());
    /// }
    /// ```
    pub fn frame_class(&self, index: usize) -> Option<AnyObject> {
        self.frame(index, |context, index| unsafe {
            debug::debug_inspector_frame_class(context, index)
        })
    }

    /// Returns a `Binding` of the frame at `index`, to read its local
    /// variables (`rb_debug_inspector_frame_binding_get`), or `nil` for a
    /// frame of a method written in C.
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{AnyObject, Class, DebugInspector, Fixnum, Object, Symbol, VM};
    ///
    /// methods!(
    ///     rutie::AnyObject,
    ///     _rtself,
    ///     fn caller_total() -> AnyObject {
    ///         DebugInspector::open(|inspector| {
    ///             assert!(inspector.frame_binding(0).unwrap().is_nil());
    ///
    ///             let binding = inspector.frame_binding(1).unwrap();
    ///             binding.protect_send("local_variable_get", &[Symbol::new("total").into()]).unwrap()
    ///         })
    ///         .unwrap()
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::from_existing("Object").define(|klass| {
    ///         klass.def_private("caller_total", caller_total);
    ///     });
    ///
    ///     let total = VM::eval("def sum; total = 40 + 2; caller_total; end; sum").unwrap();
    ///     assert_eq!(total, Fixnum::new(42).into());
    /// }
    /// ```
    pub fn frame_binding(&self, index: usize) -> Option<AnyObject> {
        self.frame(index, |context, index| unsafe {
            debug::debug_inspector_frame_binding(context, index)
        })
    }

    /// Returns the `RubyVM::InstructionSequence` running in the frame at
    /// `index` (`rb_debug_inspector_frame_iseq_get`), or `nil` for a frame
    /// of a method written in C.
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{AnyObject, Class, DebugInspector, Object, RString, VM};
    ///
    /// methods!(
    ///     rutie::AnyObject,
    ///     _rtself,
    ///     fn caller_iseq_label() -> AnyObject {
    ///         DebugInspector::open(|inspector| {
    ///             assert!(inspector.frame_iseq(0).unwrap().is_nil());
    ///
    ///             inspector.frame_iseq(1).unwrap().protect_send("label", &[]).unwrap()
    ///         })
    ///         .unwrap()
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::from_existing("Object").define(|klass| {
    ///         klass.def_private("caller_iseq_label", caller_iseq_label);
    ///     });
    ///
    ///     let label = VM::eval("def compute = caller_iseq_label; compute").unwrap();
    ///     assert_eq!(label.try_convert_to::<RString>().unwrap().to_str(), "compute");
    /// }
    /// ```
    pub fn frame_iseq(&self, index: usize) -> Option<AnyObject> {
        self.frame(index, |context, index| unsafe {
            debug::debug_inspector_frame_iseq(context, index)
        })
    }

    /// Returns the stack depth of the frame at `index`
    /// (`rb_debug_inspector_frame_depth`, Ruby 3.2+). Unlike the index, it
    /// counts the internal frames the inspector skips; compare it with
    /// [`DebugInspector::current_depth`](#method.current_depth).
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Array, Class, DebugInspector, Fixnum, Object, VM};
    ///
    /// methods!(
    ///     rutie::AnyObject,
    ///     _rtself,
    ///     fn depths() -> Array {
    ///         DebugInspector::open(|inspector| {
    ///             assert!(inspector.frame_depth(inspector.frame_count()).is_none());
    ///
    ///             let mut depths = Array::new();
    ///             for i in 0..inspector.frame_count() {
    ///                 depths.push(Fixnum::new(inspector.frame_depth(i).unwrap()));
    ///             }
    ///             depths
    ///         })
    ///         .unwrap()
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::from_existing("Object").define(|klass| {
    ///         klass.def_private("depths", depths);
    ///     });
    ///
    ///     let depths = VM::eval("def outer = depths; outer").unwrap().try_convert_to::<Array>().unwrap();
    ///     let depth = |i| depths.at(i).try_convert_to::<Fixnum>().unwrap().to_i64();
    ///
    ///     // The innermost frame is the deepest.
    ///     assert_eq!(depths.length(), 3);
    ///     assert!(depth(0) > depth(1) && depth(1) > depth(2));
    /// }
    /// ```
    pub fn frame_depth(&self, index: usize) -> Option<i64> {
        self.frame(index, |context, index| unsafe {
            debug::debug_inspector_frame_depth(context, index)
        })
        .map(|depth| Integer::from(depth.value()).to_i64())
    }

    /// Returns the stack depth of the current frame
    /// (`rb_debug_inspector_current_depth`, Ruby 3.2+), counting every frame
    /// like [`frame_depth`](#method.frame_depth).
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Class, DebugInspector, Fixnum, Object, VM};
    ///
    /// methods!(
    ///     rutie::AnyObject,
    ///     _rtself,
    ///     fn depth() -> Fixnum {
    ///         let current = DebugInspector::current_depth();
    ///         let top = DebugInspector::open(|inspector| inspector.frame_depth(0).unwrap()).unwrap();
    ///         assert_eq!(current, top);
    ///
    ///         Fixnum::new(current)
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::from_existing("Object").define(|klass| {
    ///         klass.def_private("depth", depth);
    ///     });
    ///
    ///     let depths = VM::eval("def nested = depth; [depth, nested]").unwrap();
    ///     let depths = depths.try_convert_to::<rutie::Array>().unwrap();
    ///     let at = |i| depths.at(i).try_convert_to::<Fixnum>().unwrap().to_i64();
    ///
    ///     assert_eq!(at(1), at(0) + 1);
    /// }
    /// ```
    pub fn current_depth() -> i64 {
        Integer::from(debug::debug_inspector_current_depth()).to_i64()
    }
}

/// A job Ruby runs, with the GVL, the next time it checks for interrupts
/// after it is triggered (Ruby 3.3+, `rb_postponed_job_*`). Triggering is
/// async signal safe and works from any thread, so a signal handler (as in a
/// sampling profiler) or a thread without the GVL can have Ruby run code at
/// a safe point.
///
/// # Examples
///
/// ```
/// use std::sync::atomic::{AtomicUsize, Ordering};
/// use std::sync::Arc;
///
/// use rutie::{PostponedJob, Thread, VM};
/// # VM::init();
///
/// let runs = Arc::new(AtomicUsize::new(0));
///
/// let job = {
///     let runs = runs.clone();
///
///     PostponedJob::preregister(move || {
///         runs.fetch_add(1, Ordering::SeqCst);
///     })
///     .unwrap()
/// };
///
/// // Triggered from another (non-Ruby) thread.
/// std::thread::spawn(move || job.trigger()).join().unwrap();
///
/// Thread::check_interrupts();
/// assert_eq!(runs.load(Ordering::SeqCst), 1);
/// ```
#[cfg(ruby_gte_3_3)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PostponedJob(rubysys_debug::PostponedJobHandle);

#[cfg(ruby_gte_3_3)]
impl PostponedJob {
    /// Registers `func` as a postponed job (`rb_postponed_job_preregister`).
    /// Returns `None` when Ruby's job table (32 entries for the whole
    /// process, never freed) is full.
    ///
    /// Ruby keeps one entry per callback function, and Rutie has one per
    /// closure type: preregistering another closure of the same type
    /// replaces the previous one (which is kept allocated) and returns the
    /// same job. Closures are kept until the process exits.
    ///
    /// `func` runs on the Ruby thread that next checks for interrupts, with
    /// the GVL; an exception or panic in it is discarded.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::atomic::{AtomicI64, Ordering};
    /// use std::sync::Arc;
    ///
    /// use rutie::{PostponedJob, Thread, VM};
    /// # VM::init();
    ///
    /// let last = Arc::new(AtomicI64::new(0));
    ///
    /// let register = |n: i64| {
    ///     let last = last.clone();
    ///
    ///     PostponedJob::preregister(move || last.store(n, Ordering::SeqCst)).unwrap()
    /// };
    ///
    /// let first = register(1);
    /// let second = register(2);
    /// assert_eq!(first, second);
    ///
    /// first.trigger();
    /// Thread::check_interrupts();
    /// assert_eq!(last.load(Ordering::SeqCst), 2);
    /// ```
    pub fn preregister<F>(func: F) -> Option<Self>
    where
        F: Fn() + Send + Sync + 'static,
    {
        debug::postponed_job_preregister(func).map(PostponedJob)
    }

    /// Schedules the job to run the next time Ruby checks for interrupts
    /// (`rb_postponed_job_trigger`). Triggering it again before it runs
    /// runs it only once.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::atomic::{AtomicUsize, Ordering};
    /// use std::sync::Arc;
    ///
    /// use rutie::{PostponedJob, Thread, VM};
    /// # VM::init();
    ///
    /// let runs = Arc::new(AtomicUsize::new(0));
    ///
    /// let job = {
    ///     let runs = runs.clone();
    ///
    ///     PostponedJob::preregister(move || {
    ///         runs.fetch_add(1, Ordering::SeqCst);
    ///     })
    ///     .unwrap()
    /// };
    ///
    /// job.trigger();
    /// job.trigger();
    /// Thread::check_interrupts();
    /// assert_eq!(runs.load(Ordering::SeqCst), 1);
    ///
    /// // Ruby also checks for interrupts as it runs code.
    /// job.trigger();
    /// VM::eval("100.times { [1].map { _1 } }").unwrap();
    /// assert_eq!(runs.load(Ordering::SeqCst), 2);
    /// ```
    pub fn trigger(&self) {
        unsafe { debug::postponed_job_trigger(self.0) }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        AnyException, AnyObject, Array, Class, DebugInspector, Exception, Fixnum, Object, RString,
        VM,
    };

    crate::methods!(
        AnyObject,
        _rtself,
        fn rutie_test_profile() -> Array {
            let mut rows = Array::new();

            for frame in VM::profile_frames(0, 16) {
                let mut row = Array::new();
                let text = |value: Option<RString>| match value {
                    Some(text) => text.to_any_object(),
                    None => crate::NilClass::new().to_any_object(),
                };

                row.push(Fixnum::new(frame.line() as i64));
                row.push(text(frame.path()));
                row.push(text(frame.absolute_path()));
                row.push(text(frame.label()));
                row.push(text(frame.base_label()));
                row.push(text(frame.full_label()));
                row.push(match frame.first_lineno() {
                    Some(line) => Fixnum::new(line).to_any_object(),
                    None => crate::NilClass::new().to_any_object(),
                });
                row.push(text(frame.classpath()));
                row.push(crate::Boolean::new(frame.is_singleton_method()));
                row.push(text(frame.method_name()));
                row.push(text(frame.qualified_method_name()));
                rows.push(row);
            }

            rows
        },
        fn rutie_test_inspect() -> Array {
            DebugInspector::open(|inspector| {
                let mut rows = Array::new();
                let count = inspector.frame_count();
                assert_eq!(count, inspector.backtrace_locations().length());
                assert!(inspector.frame_self(count).is_none());
                assert!(inspector.frame_class(count).is_none());
                assert!(inspector.frame_binding(count).is_none());
                assert!(inspector.frame_iseq(count).is_none());

                for i in 0..count {
                    let mut row = Array::new();
                    row.push(inspector.frame_self(i).unwrap());
                    row.push(inspector.frame_class(i).unwrap());
                    row.push(inspector.frame_binding(i).unwrap());
                    row.push(inspector.frame_iseq(i).unwrap());
                    rows.push(row);
                }

                {
                    assert!(inspector.frame_depth(count).is_none());
                    assert_eq!(
                        inspector.frame_depth(0),
                        Some(DebugInspector::current_depth())
                    );
                    let depths: Vec<i64> = (0..count)
                        .map(|i| inspector.frame_depth(i).unwrap())
                        .collect();
                    assert!(depths.windows(2).all(|pair| pair[0] > pair[1]));
                }

                rows
            })
            .unwrap()
        }
    );

    fn string(object: AnyObject) -> Option<String> {
        object
            .try_convert_to::<RString>()
            .ok()
            .map(|text| text.to_string())
    }

    #[test]
    fn test_profile_frames() {
        crate::on_ruby_thread(|| {
            Class::from_existing("Object").define(|klass| {
                klass.def_private("rutie_test_profile", rutie_test_profile);
            });

            let rows = VM::eval(
                "module RutieProfile
                   class Sample
                     def self.collect
                       rutie_test_profile
                     end
                   end
                 end
                 RutieProfile::Sample.collect",
            )
            .unwrap()
            .try_convert_to::<Array>()
            .unwrap();
            let row = |i: usize| rows.at(i as i64).try_convert_to::<Array>().unwrap();

            // The Rust method, seen as a C one.
            let native = row(0);
            assert_eq!(native.at(0), Fixnum::new(0).into());
            assert!(native.at(1).is_nil());
            assert_eq!(string(native.at(2)).unwrap(), "<cfunc>");
            assert!(native.at(3).is_nil());
            assert_eq!(string(native.at(5)).unwrap(), "Object#rutie_test_profile");
            assert!(native.at(6).is_nil());
            assert_eq!(string(native.at(7)).unwrap(), "Object");
            assert!(native.at(8).is_false());
            assert_eq!(string(native.at(9)).unwrap(), "rutie_test_profile");

            let ruby = row(1);
            assert_eq!(ruby.at(0), Fixnum::new(4).into());
            assert_eq!(string(ruby.at(1)).unwrap(), "eval");
            assert_eq!(string(ruby.at(3)).unwrap(), "collect");
            assert_eq!(string(ruby.at(4)).unwrap(), "collect");
            assert_eq!(string(ruby.at(5)).unwrap(), "RutieProfile::Sample.collect");
            assert_eq!(ruby.at(6), Fixnum::new(3).into());
            assert_eq!(string(ruby.at(7)).unwrap(), "RutieProfile::Sample");
            assert!(ruby.at(8).is_true());
            assert_eq!(string(ruby.at(9)).unwrap(), "collect");
            assert_eq!(string(ruby.at(10)).unwrap(), "RutieProfile::Sample.collect");

            // Ruby 3.4 labels the top frame of `rb_eval_string` code
            // `<compiled>`; 3.2 and 3.3 call it `<main>`.
            let top = row(2);
            let top_label = if cfg!(ruby_gte_3_4) {
                "<compiled>"
            } else {
                "<main>"
            };
            assert_eq!(string(top.at(3)).unwrap(), top_label);
            assert!(top.at(9).is_nil());

            // Outside Ruby code there is no Ruby frame (Ruby 3.1 has one
            // for the embedding program).
            assert!(VM::profile_frames(0, 8).len() <= 1);
            assert!(VM::profile_frames(0, 0).is_empty());
        });
    }

    #[test]
    fn test_debug_inspector() {
        crate::on_ruby_thread(|| {
            Class::from_existing("Object").define(|klass| {
                klass.def_private("rutie_test_inspect", rutie_test_inspect);
            });

            let rows = VM::eval(
                "class RutieInspected
                   def run
                     answer = 42
                     [1].map { rutie_test_inspect }.first
                   end
                 end
                 RutieInspected.new.run",
            )
            .unwrap()
            .try_convert_to::<Array>()
            .unwrap();
            let row = |i: i64| rows.at(i).try_convert_to::<Array>().unwrap();

            // 0: the Rust method, 1: the block, 2: `Array#map`, 3: `run`.
            assert!(row(0).at(2).is_nil());
            assert!(row(0).at(3).is_nil());
            assert_eq!(row(1).at(1), Class::from_existing("RutieInspected").into());
            assert_eq!(row(2).at(1), Class::from_existing("Array").into());
            assert!(row(2).at(2).is_nil());

            let binding = row(3).at(2);
            let answer = binding
                .protect_send("local_variable_get", &[crate::Symbol::new("answer").into()])
                .unwrap();
            assert_eq!(answer, Fixnum::new(42).into());
            let label = row(3).at(3).protect_send("label", &[]).unwrap();
            assert_eq!(string(label).unwrap(), "run");

            // Results and exceptions come back.
            assert_eq!(DebugInspector::open(|_| 7).unwrap(), 7);
            let error = DebugInspector::open(|_| {
                VM::raise_ex(AnyException::new("TypeError", Some("in inspector")));
            })
            .unwrap_err();
            assert_eq!(error.message(), "in inspector");
            let panic = DebugInspector::open(|_| panic!("inspector panic")).unwrap_err();
            assert_eq!(panic.message(), "Rust panic: inspector panic");
        });
    }

    #[cfg(ruby_gte_3_3)]
    #[test]
    fn test_postponed_job() {
        use crate::{PostponedJob, Thread};
        use std::sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        };

        crate::on_ruby_thread(|| {
            let runs = Arc::new(AtomicUsize::new(0));
            let job = {
                let runs = runs.clone();

                PostponedJob::preregister(move || {
                    if runs.fetch_add(1, Ordering::SeqCst) == 1 {
                        // Exceptions and panics in a job are discarded.
                        VM::raise_ex(AnyException::new("RuntimeError", Some("discarded")));
                    }
                })
                .unwrap()
            };

            Thread::check_interrupts();
            assert_eq!(runs.load(Ordering::SeqCst), 0);

            job.trigger();
            job.trigger();
            Thread::check_interrupts();
            assert_eq!(runs.load(Ordering::SeqCst), 1);

            job.trigger();
            Thread::check_interrupts();
            assert_eq!(runs.load(Ordering::SeqCst), 2);
            assert!(VM::error_info().is_err());

            // From threads without the GVL.
            let handles: Vec<_> = (0..4)
                .map(|_| std::thread::spawn(move || job.trigger()))
                .collect();
            for handle in handles {
                handle.join().unwrap();
            }
            Thread::check_interrupts();
            assert_eq!(runs.load(Ordering::SeqCst), 3);

            let panicking = PostponedJob::preregister(|| panic!("job panic")).unwrap();
            assert_ne!(panicking, job);
            panicking.trigger();
            Thread::check_interrupts();
            assert_eq!(VM::eval("1 + 1").unwrap(), Fixnum::new(2).into());
        });
    }
}
