/// Creates Rust structure for new Ruby class
///
/// This macro does not define an actual Ruby class. It only creates structs for using
/// the class in Rust. To define the class in Ruby, use `Class` structure.
///
/// # Examples
///
/// ```
/// #[macro_use]
/// extern crate rutie;
///
/// use rutie::{Class, RString, Object, VM};
///
/// class!(Greeter);
///
/// methods!(
///     Greeter,
///     rtself,
///
///     fn anonymous_greeting() -> RString {
///         RString::new_utf8("Hello stranger!")
///     }
///
///     fn friendly_greeting(name: RString) -> RString {
///         let name = name
///             .map(|name| name.to_string())
///             .unwrap_or("Anonymous".to_string());
///
///         let greeting = format!("Hello dear {}!", name);
///
///         RString::new_utf8(&greeting)
///     }
/// );
///
/// fn main() {
///     # VM::init();
///     Class::new("Greeter", None).define(|klass| {
///         klass.def("anonymous_greeting", anonymous_greeting);
///         klass.def("friendly_greeting", friendly_greeting);
///     });
///
///     let greeting = VM::eval("Greeter.new.friendly_greeting('Ruby')").unwrap();
///     assert_eq!(greeting.try_convert_to::<RString>().unwrap().to_str(), "Hello dear Ruby!");
///
///     // A missing or non-String argument falls back to the default.
///     let greeting = VM::eval("Greeter.new.friendly_greeting(1)").unwrap();
///     assert_eq!(greeting.try_convert_to::<RString>().unwrap().to_str(), "Hello dear Anonymous!");
/// }
/// ```
///
/// Ruby:
///
/// ```ruby
/// class Greeter
///   def anonymous_greeting
///     'Hello stranger!'
///   end
///
///   def friendly_greeting(name)
///     default_name = 'Anonymous'
///
///     name = defaut_name unless name.is_a?(String)
///
///     "Hello dear #{name}"
///   end
/// end
/// ```
#[macro_export]
macro_rules! class {
    ($class: ident) => {
        #[repr(C)]
        #[derive(Debug, PartialEq)]
        pub struct $class {
            value: $crate::types::Value,
        }

        impl From<$crate::types::Value> for $class {
            fn from(value: $crate::types::Value) -> Self {
                $class { value: value }
            }
        }

        impl $crate::Object for $class {
            #[inline]
            fn value(&self) -> $crate::types::Value {
                self.value
            }
        }
    };
}

/// Creates Rust structure for new Ruby module
///
/// This macro does not define an actual Ruby module. It only creates structs for using
/// the module in Rust. To define the module in Ruby, use `Module` structure.
///
/// # Examples
///
/// ```
/// #[macro_use]
/// extern crate rutie;
///
/// use rutie::{Module, RString, Object, VM};
///
/// module!(Greeter);
///
/// methods!(
///     Greeter,
///     rtself,
///
///     fn anonymous_greeting() -> RString {
///         RString::new_utf8("Hello stranger!")
///     }
///
///     fn friendly_greeting(name: RString) -> RString {
///         let name = name
///             .map(|name| name.to_string())
///             .unwrap_or("Anonymous".to_string());
///
///         let greeting = format!("Hello dear {}!", name);
///
///         RString::new_utf8(&greeting)
///     }
/// );
///
/// fn main() {
///     # VM::init();
///     Module::new("Greeter").define(|klass| {
///         klass.def("anonymous_greeting", anonymous_greeting);
///         klass.def("friendly_greeting", friendly_greeting);
///     });
///
///     let greeting = VM::eval("Object.new.extend(Greeter).anonymous_greeting").unwrap();
///     assert_eq!(greeting.try_convert_to::<RString>().unwrap().to_str(), "Hello stranger!");
/// }
/// ```
///
/// Ruby:
///
/// ```ruby
/// module Greeter
///   def anonymous_greeting
///     'Hello stranger!'
///   end
///
///   def friendly_greeting(name)
///     default_name = 'Anonymous'
///
///     name = defaut_name unless name.is_a?(String)
///
///     "Hello dear #{name}"
///   end
/// end
/// ```
#[macro_export]
macro_rules! module {
    ($module: ident) => {
        #[repr(C)]
        #[derive(Debug, PartialEq)]
        pub struct $module {
            value: $crate::types::Value,
        }

        impl From<$crate::types::Value> for $module {
            fn from(value: $crate::types::Value) -> Self {
                $module { value: value }
            }
        }

        impl $crate::Object for $module {
            #[inline]
            fn value(&self) -> $crate::types::Value {
                self.value
            }
        }
    };
}

/// Creates unsafe callbacks for Ruby methods
///
/// This macro is unsafe, because:
///
///  - it uses automatic unsafe conversions for arguments
///     (no guarantee that Ruby objects match the types which you expect);
///  - no bound checks for the array of provided arguments
///     (no guarantee that all the expected arguments are provided);
///     `methods!` is the checked version, giving each argument as a `Result`.
///
/// That is why creating callbacks in unsafe way may cause panics.
///
/// Due to the same reasons unsafe callbacks are faster.
///
/// Use it when:
///
///  - you own the Ruby code which passes arguments to callback;
///  - you are sure that all the object has correct type;
///  - you are sure that all the required arguments are provided;
///  - Ruby code has a good test coverage.
///
/// # Examples
///
/// ```
/// #[macro_use]
/// extern crate rutie;
///
/// use rutie::{Boolean, Class, Fixnum, Object, RString, VM};
///
/// // Creates `string_length_equals` functions
/// unsafe_methods!(
///     RString, // type of `self` object
///     rtself, // name of `self` object which will be used in methods
///
///     fn string_length_equals(expected_length: Fixnum) -> Boolean {
///         let real_length = rtself.to_str().len() as i64;
///
///         Boolean::new(expected_length.to_i64() == real_length)
///     }
/// );
///
/// fn main() {
///     # VM::init();
///     Class::from_existing("String").define(|klass| {
///         klass.def("length_equals?", string_length_equals);
///     });
///
///     let result = VM::eval("'abc'.length_equals?(3)").unwrap();
///     assert!(result.try_convert_to::<Boolean>().unwrap().to_bool());
/// }
/// ```
///
/// Ruby:
///
/// ```ruby
/// class String
///   def blank?
///     # ...
///   end
///
///   def length_equals?(expected_length)
///     # ...
///   end
/// end
/// ```
#[macro_export]
macro_rules! unsafe_methods {
    (
        $rtself_class: ty,
        $rtself_name: ident,
        $(
            fn $method_name: ident
            ($($arg_name: ident: $arg_type: ty),* $(,)?) -> $return_type: ty $body: block
            $(,)?
        )*
    ) => {
        $(
            pub extern "C" fn $method_name(argc: $crate::types::Argc,
                                       argv: *const $crate::AnyObject,
                                       #[allow(unused_mut)]
                                       #[allow(unused_variables)]
                                       mut $rtself_name: $rtself_class) -> $return_type {
                let _arguments = $crate::util::parse_arguments(argc, argv);
                let mut _i = 0;

                $(
                    let $arg_name = unsafe {
                        <$crate::AnyObject as $crate::Object>
                            ::to::<$arg_type>(&_arguments[_i])
                    };

                    _i += 1;
                )*

                $body
            }
        )*
    }
}

/// Creates callbacks for Ruby methods
///
/// Unlike `unsafe_methods!`, this macro is safe, because:
///
///  - it uses safe conversions of arguments (`Object::try_convert_to()`);
///  - it checks if arguments are present;
///
/// Each argument will have type `Result<Object, AnyException>`.
///
/// For example, if you declare `number: Fixnum` in the method definition, it will have actual
/// type `number: Result<Fixnum, AnyException>`.
///
/// See examples below and docs for `Object::try_convert_to()` for more information.
///
/// # Examples
///
/// To launch a server in Rust, you plan to write a simple `Server` class
///
/// ```ruby
/// class Server
///   def start(address)
///     # ...
///   end
/// end
/// ```
///
/// The `address` must be `Hash` with the following structure:
///
/// ```ruby
/// {
///   host: 'localhost',
///   port: 8080,
/// }
/// ```
///
/// You want to extract port from it. Default port is `8080` in case when:
///
///  - `address` is not a `Hash`
///  - `address[:port]` is not present
///  - `address[:port]` is not a `Fixnum`
///
/// ```
/// #[macro_use]
/// extern crate rutie;
///
/// use rutie::{Class, Fixnum, Hash, NilClass, Object, Symbol, VM};
///
/// class!(Server);
///
/// methods!(
///     Server,
///     rtself,
///
///     fn start(address: Hash) -> NilClass {
///         let default_port = 8080;
///
///         let port = address
///             .map(|hash| hash.at(&Symbol::new("port")))
///             .and_then(|port| port.try_convert_to::<Fixnum>())
///             .map(|port| port.to_i64())
///             .unwrap_or(default_port);
///
///         // Start server...
///
///         NilClass::new()
///     }
/// );
///
/// fn main() {
///     # VM::init();
///     Class::new("Server", None).define(|klass| {
///         klass.def("start", start);
///     });
/// }
/// ```
///
/// Ruby:
///
/// ```ruby
/// class Server
///   def start(address)
///     default_port = 8080
///
///     port =
///       if address.is_a?(Hash) && address[:port].is_a?(Fixnum)
///         address[:port]
///       else
///         default_port
///       end
///
///     # Start server...
///   end
/// end
/// ```
///
/// # Variadic methods (splat)
///
/// A last parameter written `*name`, with no type, takes any remaining
/// arguments as an `Array` (Ruby's `*name`). The parameters before it keep
/// their `Result` types.
///
/// ```
/// #[macro_use]
/// extern crate rutie;
///
/// use rutie::{Array, Class, Fixnum, Object, RString, VM};
///
/// class!(Logger);
///
/// methods!(
///     Logger,
///     rtself,
///
///     // def log(level, *parts)
///     fn log(level: RString, *parts) -> RString {
///         let level = level.map(|level| level.to_string()).unwrap_or_default();
///         let parts: Vec<String> = parts
///             .into_iter()
///             .map(|part| part.try_convert_to::<RString>().map(|s| s.to_string()).unwrap_or_default())
///             .collect();
///
///         RString::new_utf8(&format!("[{}] {}", level, parts.join(" ")))
///     }
///
///     // def count(*values)
///     fn count(*values) -> Fixnum {
///         Fixnum::new(values.length() as i64)
///     }
/// );
///
/// fn main() {
///     # VM::init();
///     Class::new("Logger", None).define(|klass| {
///         klass.def("log", log);
///         klass.def("count", count);
///     });
///
///     let line = VM::eval("Logger.new.log('info', 'server', 'started')").unwrap();
///     assert_eq!(line.try_convert_to::<RString>().unwrap().to_str(), "[info] server started");
///
///     let line = VM::eval("Logger.new.log('warn')").unwrap();
///     assert_eq!(line.try_convert_to::<RString>().unwrap().to_str(), "[warn] ");
///
///     let count = VM::eval("Logger.new.count(1, :two, 'three')").unwrap();
///     assert_eq!(count.try_convert_to::<Fixnum>().unwrap().to_i64(), 3);
///
///     let count = VM::eval("Logger.new.count").unwrap();
///     assert_eq!(count.try_convert_to::<Fixnum>().unwrap().to_i64(), 0);
/// }
/// ```
///
/// Ruby:
///
/// ```ruby
/// class Logger
///   def log(level, *parts)
///     "[#{level}] #{parts.join(' ')}"
///   end
///
///   def count(*values)
///     values.length
///   end
/// end
/// ```
///
/// For optional, keyword and block parameters, write a plain
/// `extern "C" fn(Argc, *const AnyObject, Self)` function and use
/// [`VM::scan_args`](struct.VM.html#method.scan_args). Ruby 3 separates
/// keywords from positional arguments: a method defined with `methods!`
/// receives keywords as a trailing `Hash` argument, but `VM::scan_args`
/// only fills its keywords when the caller passed keywords.
#[macro_export]
macro_rules! methods {
    // A trailing `*name` collects the remaining arguments into an `Array`.
    (@method $rtself_class: ty, $rtself_name: ident, $method_name: ident,
        ($($arg_name: ident: $arg_type: ty),+ , * $splat_name: ident $(,)?),
        $return_type: ty, $body: block) => {
        $crate::methods!(@define $rtself_class, $rtself_name, $method_name,
            [$($arg_name: $arg_type),+], [$splat_name], $return_type, $body);
    };
    (@method $rtself_class: ty, $rtself_name: ident, $method_name: ident,
        (* $splat_name: ident $(,)?), $return_type: ty, $body: block) => {
        $crate::methods!(@define $rtself_class, $rtself_name, $method_name,
            [], [$splat_name], $return_type, $body);
    };
    (@method $rtself_class: ty, $rtself_name: ident, $method_name: ident,
        ($($arg_name: ident: $arg_type: ty),* $(,)?), $return_type: ty, $body: block) => {
        $crate::methods!(@define $rtself_class, $rtself_name, $method_name,
            [$($arg_name: $arg_type),*], [], $return_type, $body);
    };
    (@define $rtself_class: ty, $rtself_name: ident, $method_name: ident,
        [$($arg_name: ident: $arg_type: ty),*], [$($splat_name: ident)?],
        $return_type: ty, $body: block) => {
        pub extern "C" fn $method_name(argc: $crate::types::Argc,
                                   argv: *const $crate::AnyObject,
                                   #[allow(unused_mut)]
                                   #[allow(unused_variables)]
                                   mut $rtself_name: $rtself_class) -> $return_type {
            let _arguments = $crate::util::parse_arguments(argc, argv);
            let mut _i = 0;

            $(
                let $arg_name =
                    _arguments
                        .get(_i)
                        .ok_or_else(|| {
                            <$crate::AnyException as $crate::Exception>::new("ArgumentError",
                                Some(&format!(
                                    "Argument '{}: {}' not found for method '{}'",
                                    stringify!($arg_name),
                                    stringify!($arg_type),
                                    stringify!($method_name)
                                ))
                            )
                        }).and_then(|argument| {
                            <$crate::AnyObject as $crate::Object>
                                ::try_convert_to::<$arg_type>(argument)
                        });

                _i += 1;
            )*

            $(
                let $splat_name: $crate::Array = _arguments.iter().skip(_i).cloned().collect();
            )?

            $body
        }
    };
    (
        $rtself_class: ty,
        $rtself_name: ident,
        $(
            fn $method_name: ident
            ($($params: tt)*) -> $return_type: ty $body: block
            $(,)?
        )*
    ) => {
        $(
            $crate::methods!(@method $rtself_class, $rtself_name, $method_name,
                ($($params)*), $return_type, $body);
        )*
    }
}

/// Makes a Rust struct wrappable for Ruby objects.
///
/// **Note:** Currently to be able to use `wrappable_struct!` macro, you should include
/// `lazy_static` crate to the crate you are working on.
///
/// `Cargo.toml`
///
/// ```toml
/// lazy_static = "0.2.1" # the version is not a strict requirement
/// ```
///
/// Crate root `lib.rs` or `main.rs`
///
/// ```
/// #[macro_use]
/// extern crate lazy_static;
/// # fn main() {}
/// ```
///
/// # Arguments
///
///  - `$struct_name` is name of the actual Rust struct. This structure has to be public (`pub`).
///
///  - `$wrapper` is a name for the structure which will be created to wrap the `$struct_name`.
///
///     The wrapper will be created automatically by the macro.
///
///  - `$static_name` is a name for a static variable which will contain the wrapper.
///
///     The static variable will be created automatically by the macro.
///
///     This variable has to be passed to `wrap_data()` and `get_data()` functions (see examples).
///
///     Also, these variables describe the structure in general, but not some specific object.
///     So you should pass the same static variable when wrapping/getting data of the same
///     type for different ruby objects.
///
///     For example,
///
///     ```
///     # #[macro_use] extern crate rutie;
///     # #[macro_use] extern crate lazy_static;
///     # use rutie::{AnyObject, Class, Object, VM};
///     pub struct Server {
///         port: u16,
///     }
///
///     wrappable_struct!(Server, ServerWrapper, SERVER_WRAPPER);
///
///     # fn main() {
///     # VM::init();
///     let class = Class::new("SharedWrapperServer", None);
///     let server1: AnyObject = class.wrap_data(Server { port: 3000 }, &*SERVER_WRAPPER);
///     let server2: AnyObject = class.wrap_data(Server { port: 3001 }, &*SERVER_WRAPPER);
///
///     assert_eq!(server1.get_data(&*SERVER_WRAPPER).port, 3000);
///     assert_eq!(server2.get_data(&*SERVER_WRAPPER).port, 3001); // <-- the same `SERVER_WRAPPER`
///     # }
///     ```
///
///  - (optional) `mark(data) { ... }` is a block which will be called during the "mark"
///    phase of garbage collection.
///
///    This block must be used if the struct contains any Ruby objects. The objects should
///    be marked with `GC::mark()` to prevent their garbage collection.
///
///    `data` argument will be yielded as a mutable reference to the wrapped struct
///    (`&mut $struct_name`).
///
///    **Notes from the official MRI documentation:**
///
///      - It is not recommended to store Ruby objects in the structs. Try to avoid that
///        if possible.
///
///      - It is not allowed to allocate new Ruby objects in the `mark` function.
///
///  - (optional) `size(data) { ... }` is a block returning the memory used by the
///    struct in bytes, as a `usize` (the `dsize` function). Ruby reports it through
///    `ObjectSpace.memsize_of` and uses it in GC statistics. `data` is a shared
///    reference to the wrapped struct (`&$struct_name`).
///
///  - (optional) `compact(data) { ... }` is a block called after `GC.compact`
///    has moved objects (the `dcompact` function). It must replace every Ruby
///    object the struct holds with `GC::location(..)` of it. `data` is a
///    mutable reference to the wrapped struct (`&mut $struct_name`).
///
///    Without it, objects marked with `GC::mark` are pinned: compaction never
///    moves them. With it, `mark` should use `GC::mark_movable` for the objects
///    `compact` updates (see [`GC::mark_movable`](struct.GC.html#method.mark_movable)).
///
///    `mark`, `size` and `compact` can be given in any order. All run inside
///    Ruby's garbage collector as `extern "C"` functions, so a panic in them
///    aborts the process, and none may allocate Ruby objects.
///
/// The result of `wrappable_struct!` is a wrapper type and a `lazy_static`
/// holding its only value, which implements
/// [`DataTypeWrapper`](typed_data/trait.DataTypeWrapper.html) for the struct:
///
/// ```
/// # #[macro_use] extern crate rutie;
/// # #[macro_use] extern crate lazy_static;
/// use rutie::typed_data::DataTypeWrapper;
///
/// pub struct Server;
///
/// wrappable_struct!(Server, ServerWrapper, SERVER_WRAPPER);
///
/// // produces
/// //
/// // pub struct ServerWrapper<T> { /* ... */ }
/// //
/// // lazy_static! { pub static ref SERVER_WRAPPER: ServerWrapper<Server> = /* ... */; }
///
/// # fn main() {
/// let wrapper: &ServerWrapper<Server> = &*SERVER_WRAPPER;
/// let _: &dyn DataTypeWrapper<Server> = wrapper;
/// # }
/// ```
///
/// # Class
///
/// The class which will be used for wrapping data is `Object` and not `Data`
/// (See [Ruby issue #3072](https://bugs.ruby-lang.org/issues/3072)).
///
/// ```
/// # use rutie::{Class, VM};
/// # VM::init();
/// let data_class = Class::from_existing("Object");
///
/// Class::new("TheNewClass", Some(&data_class));
/// ```
///
/// # Examples
///
/// ## Wrap `Server` structs to `RubyServer` objects
///
/// ```
/// #[macro_use] extern crate rutie;
/// #[macro_use] extern crate lazy_static;
///
/// use rutie::{AnyObject, Class, Fixnum, Object, RString, VM};
///
/// // The structure which we want to wrap
/// pub struct Server {
///     host: String,
///     port: u16,
/// }
///
/// impl Server {
///     fn new(host: String, port: u16) -> Self {
///         Server {
///             host: host,
///             port: port,
///         }
///     }
///
///     fn host(&self) -> &str {
///         &self.host
///     }
///
///     fn port(&self) -> u16 {
///         self.port
///     }
/// }
///
/// wrappable_struct!(Server, ServerWrapper, SERVER_WRAPPER);
///
/// class!(RubyServer);
///
/// methods!(
///     RubyServer,
///     rtself,
///
///     fn ruby_server_new(host: RString, port: Fixnum) -> AnyObject {
///         let server = Server::new(host.unwrap().to_string(),
///                                  port.unwrap().to_i64() as u16);
///
///         Class::from_existing("RubyServer").wrap_data(server, &*SERVER_WRAPPER)
///     }
///
///     fn ruby_server_host() -> RString {
///         let host = rtself.get_data(&*SERVER_WRAPPER).host();
///
///         RString::new_utf8(host)
///     }
///
///     fn ruby_server_port() -> Fixnum {
///         let port = rtself.get_data(&*SERVER_WRAPPER).port();
///
///         Fixnum::new(port as i64)
///     }
/// );
///
/// fn main() {
///     # VM::init();
///     let data_class = Class::from_existing("Object");
///
///     Class::new("RubyServer", Some(&data_class)).define(|klass| {
///         klass.def_self("new", ruby_server_new);
///
///         klass.def("host", ruby_server_host);
///         klass.def("port", ruby_server_port);
///     });
/// }
/// ```
///
/// To use the `RubyServer` class in Ruby:
///
/// ```ruby
/// server = RubyServer.new("127.0.0.1", 3000)
///
/// server.host == "127.0.0.1"
/// server.port == 3000
/// ```
///
/// ## `RustyArray`
///
/// Custom array implementation using a vector which contains `AnyObject`s.
///
/// ```
/// #[macro_use] extern crate rutie;
/// #[macro_use] extern crate lazy_static;
///
/// use std::ops::{Deref, DerefMut};
///
/// use rutie::{AnyObject, Class, Fixnum, GC, NilClass, Object, VM};
///
/// pub struct VectorOfObjects {
///     inner: Vec<AnyObject>,
/// }
///
/// impl VectorOfObjects {
///     fn new() -> Self {
///         VectorOfObjects {
///             inner: Vec::new(),
///         }
///     }
/// }
///
/// impl Deref for VectorOfObjects {
///     type Target = Vec<AnyObject>;
///
///     fn deref(&self) -> &Vec<AnyObject> {
///         &self.inner
///     }
/// }
///
/// impl DerefMut for VectorOfObjects {
///     fn deref_mut(&mut self) -> &mut Vec<AnyObject> {
///         &mut self.inner
///     }
/// }
///
/// wrappable_struct! {
///     VectorOfObjects,
///     VectorOfObjectsWrapper,
///     VECTOR_OF_OBJECTS_WRAPPER,
///
///     // Mark each `AnyObject` element of the `inner` vector to prevent garbage collection.
///     // `data` is a mutable reference to the wrapped data (`&mut VectorOfObjects`).
///     mark(data) {
///         for object in &data.inner {
///             GC::mark(object);
///         }
///     }
/// }
///
/// class!(RustyArray);
///
/// methods! {
///     RustyArray,
///     rtself,
///
///     fn new() -> AnyObject {
///         let vec = VectorOfObjects::new();
///
///         Class::from_existing("RustyArray").wrap_data(vec, &*VECTOR_OF_OBJECTS_WRAPPER)
///     }
///
///     fn push(object: AnyObject) -> NilClass {
///         rtself.get_data_mut(&*VECTOR_OF_OBJECTS_WRAPPER).push(object.unwrap());
///
///         NilClass::new()
///     }
///
///     fn length() -> Fixnum {
///         let length = rtself.get_data(&*VECTOR_OF_OBJECTS_WRAPPER).len() as i64;
///
///         Fixnum::new(length)
///     }
/// }
///
/// fn main() {
///     # VM::init();
///     let data_class = Class::from_existing("Object");
///
///     Class::new("RustyArray", Some(&data_class)).define(|klass| {
///         klass.def_self("new", new);
///
///         klass.def("push", push);
///         klass.def("length", length);
///     });
/// }
/// ```
///
/// To use the `RustyArray` class in Ruby:
///
/// ```ruby
/// array = RustyArray.new
///
/// array.push(1)
/// array.push("string")
/// array.push(:symbol)
///
/// array.length == 3
/// ```
///
/// ## Reporting memory use
///
/// ```
/// #[macro_use] extern crate rutie;
/// #[macro_use] extern crate lazy_static;
///
/// use rutie::{AnyObject, Class, Fixnum, Object, VM};
///
/// pub struct Buffer {
///     bytes: Vec<u8>,
/// }
///
/// wrappable_struct! {
///     Buffer,
///     BufferWrapper,
///     BUFFER_WRAPPER,
///
///     size(data) {
///         std::mem::size_of::<Buffer>() + data.bytes.capacity()
///     }
/// }
///
/// fn main() {
///     # VM::init();
///     let buffer = Buffer { bytes: vec![0; 64 * 1024] };
///     let object: AnyObject = Class::new("RutieBuffer", None).wrap_data(buffer, &*BUFFER_WRAPPER);
///
///     VM::init_loadpath();
///     VM::protect_require("objspace").unwrap();
///
///     let memsize = Class::from_existing("ObjectSpace")
///         .protect_send("memsize_of", &[object.to_any_object()])
///         .unwrap()
///         .try_convert_to::<Fixnum>()
///         .unwrap()
///         .to_i64();
///
///     assert!(memsize >= 64 * 1024);
/// }
/// ```
#[macro_export]
macro_rules! wrappable_struct {
    // Collects the optional `mark(..) { .. }`, `size(..) { .. }` and
    // `compact(..) { .. }` clauses, in any order, then defines the wrapper.
    (@parse $struct_name: ty, $wrapper: ident, $static_name: ident,
        { $($mark: tt)* }, { $($size: tt)* }, { $($compact: tt)* }, $(,)?) => {
        $crate::wrappable_struct!(@define $struct_name, $wrapper, $static_name,
            { $($mark)* }, { $($size)* }, { $($compact)* });
    };
    (@parse $struct_name: ty, $wrapper: ident, $static_name: ident,
        { }, { $($size: tt)* }, { $($compact: tt)* },
        , mark($object: ident) $body: block $($rest: tt)*) => {
        $crate::wrappable_struct!(@parse $struct_name, $wrapper, $static_name,
            { $object $body }, { $($size)* }, { $($compact)* }, $($rest)*);
    };
    (@parse $struct_name: ty, $wrapper: ident, $static_name: ident,
        { $($mark: tt)* }, { }, { $($compact: tt)* },
        , size($object: ident) $body: block $($rest: tt)*) => {
        $crate::wrappable_struct!(@parse $struct_name, $wrapper, $static_name,
            { $($mark)* }, { $object $body }, { $($compact)* }, $($rest)*);
    };
    (@parse $struct_name: ty, $wrapper: ident, $static_name: ident,
        { $($mark: tt)* }, { $($size: tt)* }, { },
        , compact($object: ident) $body: block $($rest: tt)*) => {
        $crate::wrappable_struct!(@parse $struct_name, $wrapper, $static_name,
            { $($mark)* }, { $($size)* }, { $object $body }, $($rest)*);
    };

    (@mark_function_pointer { }) => {
        None as Option<extern "C" fn(*mut $crate::types::c_void)>
    };
    (@mark_function_pointer { $object: ident $body: block }) => {
        Some(Self::mark as extern "C" fn(*mut $crate::types::c_void))
    };
    (@mark_function_definition $struct_name: ty, { }) => {};
    (@mark_function_definition $struct_name: ty, { $object: ident $body: block }) => {
        pub extern "C" fn mark(data: *mut $crate::types::c_void) {
            let mut data = unsafe { (data as *mut $struct_name).as_mut() };

            if let Some(ref mut $object) = data {
                $body
            }
        }
    };

    (@size_function_pointer { }) => {
        None as Option<extern "C" fn(*const $crate::types::c_void) -> $crate::types::size_t>
    };
    (@size_function_pointer { $object: ident $body: block }) => {
        Some(Self::size as extern "C" fn(*const $crate::types::c_void) -> $crate::types::size_t)
    };
    (@size_function_definition $struct_name: ty, { }) => {};
    (@size_function_definition $struct_name: ty, { $object: ident $body: block }) => {
        pub extern "C" fn size(data: *const $crate::types::c_void) -> $crate::types::size_t {
            let data = unsafe { (data as *const $struct_name).as_ref() };

            match data {
                Some($object) => {
                    let size: usize = $body;

                    size as $crate::types::size_t
                }
                None => 0,
            }
        }
    };

    (@compact_function_pointer { }) => {
        ::std::ptr::null_mut()
    };
    (@compact_function_pointer { $object: ident $body: block }) => {
        Self::compact as extern "C" fn(*mut $crate::types::c_void) as *mut $crate::types::c_void
    };
    (@compact_function_definition $struct_name: ty, { }) => {};
    (@compact_function_definition $struct_name: ty, { $object: ident $body: block }) => {
        pub extern "C" fn compact(data: *mut $crate::types::c_void) {
            let mut data = unsafe { (data as *mut $struct_name).as_mut() };

            if let Some(ref mut $object) = data {
                $body
            }
        }
    };

    (@define $struct_name: ty, $wrapper: ident, $static_name: ident,
        { $($mark: tt)* }, { $($size: tt)* }, { $($compact: tt)* }) => {
        pub struct $wrapper<T> {
            data_type: $crate::types::DataType,
            _marker: ::std::marker::PhantomData<T>,
        }

        ::lazy_static::lazy_static! {
            pub static ref $static_name: $wrapper<$struct_name> = $wrapper::new();
        }

        impl<T> $wrapper<T> {
            fn new() -> $wrapper<T> {
                let name = concat!("Rutie/", stringify!($struct_name));
                let name = $crate::util::str_to_cstring(name);
                // `reserved[0]` is `dcompact`. Without a `compact` clause it is
                // null, and objects marked with `GC::mark` are pinned by
                // `GC.compact`.
                let dcompact = $crate::wrappable_struct!(@compact_function_pointer { $($compact)* });
                let reserved_bytes: [*mut $crate::types::c_void; 2] = [dcompact, ::std::ptr::null_mut()];

                let dmark = $crate::wrappable_struct!(@mark_function_pointer { $($mark)* });
                let dsize = $crate::wrappable_struct!(@size_function_pointer { $($size)* });

                let data_type = $crate::types::DataType {
                    wrap_struct_name: name.into_raw(),
                    parent: ::std::ptr::null(),
                    data: ::std::ptr::null_mut(),
                    flags: $crate::types::Value::from(0),

                    function: $crate::types::DataTypeFunction {
                        dmark: dmark,
                        dfree: Some($crate::typed_data::free::<T>),
                        dsize: dsize,
                        reserved: reserved_bytes,
                    },
                };

                $wrapper {
                    data_type: data_type,
                    _marker: ::std::marker::PhantomData,
                }
            }

            $crate::wrappable_struct!(@mark_function_definition $struct_name, { $($mark)* });
            $crate::wrappable_struct!(@size_function_definition $struct_name, { $($size)* });
            $crate::wrappable_struct!(@compact_function_definition $struct_name, { $($compact)* });
        }

        unsafe impl<T> Sync for $wrapper<T> {}

        // Set constraint to be able to wrap and get data only for type `T`
        impl<T> $crate::typed_data::DataTypeWrapper<T> for $wrapper<T> {
            fn data_type(&self) -> &$crate::types::DataType {
                &self.data_type
            }
        }
    };

    ($struct_name: ty, $wrapper: ident, $static_name: ident $($tail: tt)*) => {
        $crate::wrappable_struct!(@parse $struct_name, $wrapper, $static_name, { }, { }, { }, $($tail)*);
    };
}

/// eval(string [, binding [, filename [,lineno]]]) → obj
///
/// # Examples
/// ```
/// #[macro_use]
/// extern crate rutie;
/// use rutie::{Object, Integer, Binding, VM};
///
/// fn main() {
///     # VM::init();
///
///     let binding = eval!("asdf = 1; binding").unwrap().
///       try_convert_to::<Binding>().unwrap();
///
///     let result = eval!("asdf", binding).unwrap();
///
///     match result.try_convert_to::<Integer>() {
///         Ok(v) => assert_eq!(1, v.to_i64()),
///         Err(_) => unreachable!(),
///     }
/// }
/// ```
#[macro_export]
macro_rules! eval {
    ($string_arg:expr) => {{
        $crate::VM::eval($string_arg)
    }};
    ($string_arg:expr, $binding_arg:expr) => {{
        let eval_str: $crate::AnyObject = $crate::RString::from($string_arg).into();
        let bndng: $crate::AnyObject = $binding_arg.into();
        let arguments = &[eval_str, bndng];

        $crate::Class::from_existing("Kernel").protect_send("eval", arguments)
    }};
    ($string_arg:expr, $binding_arg:expr, $filename:expr) => {{
        let eval_str: $crate::AnyObject = $crate::RString::from($string_arg).into();
        let bndng: $crate::AnyObject = $binding_arg.into();
        let filename: $crate::AnyObject = $crate::RString::from($filename).into();
        let arguments = &[eval_str, bndng, filename];

        $crate::Class::from_existing("Kernel").protect_send("eval", arguments)
    }};
    ($string_arg:expr, $binding_arg:expr, $filename:expr, $linenumber:expr) => {{
        let eval_str: $crate::AnyObject = $crate::RString::from($string_arg).into();
        let bndng: $crate::AnyObject = $binding_arg.into();
        let filename: $crate::AnyObject = $crate::RString::from($filename).into();
        let linenumber: $crate::AnyObject = $crate::Integer::from($linenumber as i64).into();
        let arguments = &[eval_str, bndng, filename, linenumber];

        $crate::Class::from_existing("Kernel").protect_send("eval", arguments)
    }};
}

#[cfg(test)]
mod tests {
    use crate::{AnyObject, Array, Class, Exception, Fixnum, Object, RString, GC, VM};

    crate::class!(RutieDslSplat);

    crate::methods!(
        RutieDslSplat,
        rtself,

        fn dsl_rest_only(*rest) -> Array {
            rest
        }

        fn dsl_first_and_rest(first: Fixnum, *rest,) -> Array {
            let mut result = Array::new();
            result.push(first.map(|n| n.to_any_object()).unwrap_or_else(|e| e.to_any_object()));
            result.push(rest);
            result
        }

        fn dsl_plain(a: Fixnum, b: Fixnum) -> Fixnum {
            Fixnum::new(a.unwrap().to_i64() + b.unwrap().to_i64())
        }
    );

    crate::unsafe_methods!(
        RutieDslSplat,
        rtself,
        fn dsl_unsafe_two(a: Fixnum, b: Fixnum) -> Fixnum {
            Fixnum::new(a.to_i64() * b.to_i64())
        }
    );

    pub struct Payload {
        objects: Vec<AnyObject>,
        extra: usize,
    }

    crate::wrappable_struct! {
        Payload,
        PayloadWrapper,
        PAYLOAD_WRAPPER,

        size(data) {
            std::mem::size_of::<Payload>() + data.extra
        },

        mark(data) {
            for object in &data.objects {
                GC::mark(object);
            }
        },
    }

    pub struct Movable {
        strings: Vec<RString>,
    }

    static MOVED: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

    crate::wrappable_struct! {
        Movable,
        MovableWrapper,
        MOVABLE_WRAPPER,

        compact(data) {
            for string in data.strings.iter_mut() {
                let moved = GC::location(string);

                if moved.value().value != string.value().value {
                    MOVED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                }

                *string = moved;
            }
        },

        mark(data) {
            for string in &data.strings {
                GC::mark_movable(string);
            }
        },
    }

    // Not inlined, so no copy of the strings stays on the (pinning) stack.
    #[inline(never)]
    fn movable_object() -> AnyObject {
        let strings = (0..200)
            .map(|i| RString::new_utf8(&format!("s{}", i)))
            .collect();

        Class::new("RutieDslMovable", None).wrap_data(Movable { strings }, &*MOVABLE_WRAPPER)
    }

    pub struct Plain;

    crate::wrappable_struct!(Plain, PlainWrapper, PLAIN_WRAPPER);

    fn eval_array(code: &str) -> Array {
        VM::eval(code).unwrap().try_convert_to::<Array>().unwrap()
    }

    #[test]
    fn test_methods_splat() {
        crate::on_ruby_thread(|| {
            Class::new("RutieDslSplat", None).define(|klass| {
                klass.def("rest_only", dsl_rest_only);
                klass.def("first_and_rest", dsl_first_and_rest);
                klass.def("plain", dsl_plain);
                klass.def("unsafe_two", dsl_unsafe_two);
            });

            assert_eq!(eval_array("RutieDslSplat.new.rest_only").length(), 0);
            assert_eq!(
                eval_array("RutieDslSplat.new.rest_only(1, 2, 3)").length(),
                3
            );

            let result = eval_array("RutieDslSplat.new.first_and_rest(1, :a, :b)");
            assert_eq!(result.at(0).try_convert_to::<Fixnum>(), Ok(Fixnum::new(1)));
            assert_eq!(result.at(1).try_convert_to::<Array>().unwrap().length(), 2);

            // A missing positional argument is an error value, and the rest is empty.
            let result = eval_array("RutieDslSplat.new.first_and_rest");
            assert!(Class::argument_error().case_equals(&result.at(0)));
            assert_eq!(result.at(1).try_convert_to::<Array>().unwrap().length(), 0);

            let sum = VM::eval("RutieDslSplat.new.plain(2, 3)").unwrap();
            assert_eq!(sum.try_convert_to::<Fixnum>(), Ok(Fixnum::new(5)));

            // Keywords arrive as a trailing `Hash`, as the docs say.
            let result = eval_array("RutieDslSplat.new.rest_only(1, mode: :fast)");
            assert_eq!(result.length(), 2);
            assert!(result.at(1).try_convert_to::<crate::Hash>().is_ok());
        });
    }

    #[test]
    fn test_unsafe_methods() {
        crate::on_ruby_thread(|| {
            Class::new("RutieDslSplat", None).define(|klass| {
                klass.def("unsafe_two", dsl_unsafe_two);
            });

            let product = VM::eval("RutieDslSplat.new.unsafe_two(6, 7)").unwrap();
            assert_eq!(product.try_convert_to::<Fixnum>(), Ok(Fixnum::new(42)));

            // Extra arguments are ignored. (Too few is the caller's
            // responsibility with `unsafe_methods!`; `methods!` checks.)
            let product = VM::eval("RutieDslSplat.new.unsafe_two(6, 7, 8)").unwrap();
            assert_eq!(product.try_convert_to::<Fixnum>(), Ok(Fixnum::new(42)));
        });
    }

    #[test]
    fn test_wrappable_struct_mark_and_size() {
        crate::on_ruby_thread(|| {
            VM::init_loadpath();
            VM::protect_require("objspace").unwrap();

            let class = Class::new("RutieDslPayload", None);
            let payload = Payload {
                objects: vec![RString::new_utf8("kept alive").to_any_object()],
                extra: 1 << 20,
            };
            let object: AnyObject = class.wrap_data(payload, &*PAYLOAD_WRAPPER);

            GC::start();

            let kept = object.get_data(&*PAYLOAD_WRAPPER).objects[0].clone();
            assert_eq!(
                kept.try_convert_to::<RString>().unwrap().to_str(),
                "kept alive"
            );

            let memsize = |object: &AnyObject| {
                Class::from_existing("ObjectSpace")
                    .protect_send("memsize_of", &[object.clone()])
                    .unwrap()
                    .try_convert_to::<Fixnum>()
                    .unwrap()
                    .to_i64()
            };

            assert!(memsize(&object) >= 1 << 20);

            let plain: AnyObject = class.wrap_data(Plain, &*PLAIN_WRAPPER);
            assert!(memsize(&plain) < 1 << 20);
        });
    }

    #[test]
    fn test_wrappable_struct_compact() {
        crate::on_ruby_thread(|| {
            let object = movable_object();

            let compacted = VM::eval(
                "begin
                   GC.verify_compaction_references(toward: :empty, double_heap: true)
                   true
                 rescue NotImplementedError
                   false
                 end",
            )
            .unwrap();

            let strings = &object.get_data(&*MOVABLE_WRAPPER).strings;

            for (i, string) in strings.iter().enumerate() {
                assert_eq!(string.to_str(), format!("s{}", i));
            }

            if compacted.value().is_true() {
                assert!(MOVED.load(std::sync::atomic::Ordering::SeqCst) > 0);
            }
        });
    }
}
