use std::error;
use std::fmt::{self, Display, Formatter};
use std::result;

use Class;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    ArgumentError(String),
    TypeError(String),
}

impl Error {
    /// Converts error to an exception class.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::result::Error;
    /// use rutie::{Class, VM};
    ///
    /// # VM::init();
    /// let argument_error = Error::ArgumentError("Argument is missing".to_string());
    /// let type_error = Error::TypeError("Wrong type".to_string());
    ///
    /// assert_eq!(argument_error.to_exception(), Class::from_existing("ArgumentError"));
    /// assert_eq!(type_error.to_exception(), Class::from_existing("TypeError"));
    /// ```
    pub fn to_exception(&self) -> Class {
        let class_name = match *self {
            Error::ArgumentError(_) => "ArgumentError",
            Error::TypeError(_) => "TypeError",
        };

        Class::from_existing(class_name)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", <Error as error::Error>::description(self))
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::ArgumentError(ref message) | Error::TypeError(ref message) => message,
        }
    }
}
