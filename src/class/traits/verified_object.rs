use Object;

/// Interface for safe conversions between types
///
/// This trait is required by `Object::convert_to()` function.
///
/// All built-in types like `Hash`, `RString` and others implement it.
///
/// **You should implement this trait for custom classes which you receive from Ruby
/// if at least one of the following statements is false**:
///
///  - you own the Ruby code which passes the object to Rust;
///  - you are sure that the object always has correct type;
///  - your Ruby code has a good test coverage.
///
/// Various techniques can be used to check if the object has correct type:
///
///  - check the class of the object;
///  - check ancestors of the object's class;
///  - check if object is one of built-in objects (when it is inherited from one of those);
///  - use duck typing to check if object responds to required methods;
///  - etc
///
/// # Examples
///
/// ```
/// #[macro_use]
/// extern crate rutie;
///
/// use rutie::types::ValueType;
/// use rutie::{Class, Object, RString, VerifiedObject, VM};
///
/// // Check the class of the object
/// class!(Server);
///
/// impl VerifiedObject for Server {
///     fn is_correct_type<T: Object>(object: &T) -> bool {
///         object.class() == Class::from_existing("Server")
///     }
///
///     fn error_message() -> &'static str {
///         "Error converting to Server"
///     }
/// }
///
/// // Check presence of required methods (duck typing)
/// class!(Request);
///
/// methods!(
///     Request,
///     itself,
///
///     fn protocol() -> RString { RString::new("HTTP") }
///     fn body() -> RString { RString::new("request body") }
/// );
///
/// impl VerifiedObject for Request {
///     fn is_correct_type<T: Object>(object: &T) -> bool {
///         object.respond_to("protocol") && object.respond_to("body")
///     }
///
///     fn error_message() -> &'static str {
///         "Error converting to Request"
///     }
/// }
///
/// // Check if class inherits/includes some class or module
/// class!(Response);
///
/// impl VerifiedObject for Response {
///     fn is_correct_type<T: Object>(object: &T) -> bool {
///         object.class().ancestors().iter()
///             .any(|class| *class == Class::from_existing("BasicResponse"))
///     }
///
///     fn error_message() -> &'static str {
///         "Error converting to Response"
///     }
/// }
///
/// // Check if class was inherited from built-in classes
/// class!(Headers);
///
/// impl VerifiedObject for Headers {
///     fn is_correct_type<T: Object>(object: &T) -> bool {
///         object.value().ty() == ValueType::Hash
///     }
///
///     fn error_message() -> &'static str {
///         "Error converting to Headers"
///     }
/// }
///
/// fn main() {
///     # VM::init();
///     Class::new("Server", None);
///     Class::new("Response", Some(&Class::new("BasicResponse", None)));
///     Class::new("Headers", Some(&Class::from_existing("Hash")));
///     Class::new("Request", None).define(|itself| {
///         itself.def("protocol", protocol);
///         itself.def("body", body);
///     });
///
///     // Create new instances of classes and convert them to `AnyObject`s
///     // (make their type unknown)
///     let server = Class::from_existing("Server").new_instance(None).to_any_object();
///     let request = Class::from_existing("Request").new_instance(None).to_any_object();
///     let response = Class::from_existing("Response").new_instance(None).to_any_object();
///     let headers = Class::from_existing("Headers").new_instance(None).to_any_object();
///
///     assert!(server.try_convert_to::<Server>().is_ok());
///     assert!(request.try_convert_to::<Request>().is_ok());
///     assert!(response.try_convert_to::<Response>().is_ok());
///     assert!(headers.try_convert_to::<Headers>().is_ok());
///
///     // P.S.
///     // The following is possible to compile, but the program will panic
///     // if you perform any actions with these objects.
///     // Try to avoid unsafe conversions.
///     let bad_request = unsafe { server.to::<Request>() };
///     let bad_server = unsafe { headers.to::<Server>() };
/// }
/// ```
pub trait VerifiedObject: Object {
    fn is_correct_type<T: Object>(object: &T) -> bool;
    fn error_message() -> &'static str;
}
