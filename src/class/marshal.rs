use crate::{binding::marshal, types::Value, AnyObject, NilClass};

/// `Marshal`
#[derive(Debug)]
#[repr(C)]
pub struct Marshal {
    value: Value,
}

impl Marshal {
    /// Dump a Ruby object and load back
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Boolean, VM, NilClass, Marshal};
    /// # VM::init();
    ///
    /// let dumped = Marshal::dump(Boolean::new(true).into(), NilClass::new().into());
    ///
    /// assert_eq!(Marshal::load(dumped), Boolean::new(true).into());
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// dumped =  Marshal::dump(true)
    ///
    /// Marshal::load(dumped) == true
    /// ```
    pub fn load(port: AnyObject) -> AnyObject {
        marshal::marshal_load(port.into()).into()
    }

    pub fn dump(val: AnyObject, port: AnyObject) -> AnyObject {
        marshal::marshal_dump(val.into(), port.into()).into()
    }
}
