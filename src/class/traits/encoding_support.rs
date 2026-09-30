use crate::{AnyException, AnyObject, Encoding, Hash, Object};

/// Encoding operations for Ruby objects that carry an encoding
/// (implemented by [`RString`](../struct.RString.html)).
pub trait EncodingSupport {
    /// Transcodes to `enc` (Ruby's `String#encode`), with `opts` as
    /// `String#encode` options.
    ///
    /// ```
    /// use rutie::{Encoding, EncodingSupport, RString, VM};
    /// # VM::init();
    ///
    /// let ascii = RString::new_utf8("plain").encode(Encoding::us_ascii(), None);
    ///
    /// assert_eq!(ascii.encoding().name(), "US-ASCII");
    /// ```
    fn encode(&self, enc: Encoding, opts: Option<Hash>) -> Self
    where
        Self: Sized;

    /// Returns the object's encoding.
    ///
    /// ```
    /// use rutie::{EncodingSupport, RString, VM};
    /// # VM::init();
    ///
    /// assert_eq!(RString::new_utf8("é").encoding().name(), "UTF-8");
    /// ```
    fn encoding(&self) -> Encoding;

    /// Relabels the object's bytes as `enc` without converting them
    /// (Ruby's `String#force_encoding`); fails on a frozen object.
    ///
    /// ```
    /// use rutie::{Encoding, EncodingSupport, RString, VM};
    /// # VM::init();
    ///
    /// let mut string = RString::new_utf8("é");
    /// let binary = string.force_encoding(Encoding::find("ASCII-8BIT").unwrap()).unwrap();
    ///
    /// assert_eq!(binary.encoding().name(), "ASCII-8BIT");
    /// assert_eq!(binary.to_bytes_unchecked(), "é".as_bytes());
    /// ```
    fn force_encoding(&mut self, enc: Encoding) -> Result<Self, AnyException>
    where
        Self: Sized;

    /// Returns whether the bytes are valid in the object's encoding
    /// (Ruby's `String#valid_encoding?`).
    ///
    /// ```
    /// use rutie::{Encoding, EncodingSupport, RString, VM};
    /// # VM::init();
    ///
    /// assert!(RString::new_utf8("ok").is_valid_encoding());
    /// assert!(!RString::from_bytes(&[0xff], &Encoding::utf8()).is_valid_encoding());
    /// ```
    fn is_valid_encoding(&self) -> bool;

    /// Returns whether `other` can be combined with this object
    /// (`rb_enc_compatible`).
    ///
    /// ```
    /// use rutie::{EncodingSupport, RString, VM};
    /// # VM::init();
    ///
    /// let utf8 = RString::new_utf8("é");
    /// let binary = VM::eval(r#""\xFF".b"#).unwrap();
    ///
    /// assert!(utf8.compatible_with(&RString::new_utf8("ascii")));
    /// assert!(!utf8.compatible_with(&binary));
    /// ```
    fn compatible_with(&self, other: &impl Object) -> bool;

    /// Returns the encoding that combining `obj1` and `obj2` would have, or
    /// `nil` when they are incompatible (Ruby's `Encoding.compatible?`).
    ///
    /// ```
    /// use rutie::{Encoding, EncodingSupport, Object, RString, VM};
    /// # VM::init();
    ///
    /// let utf8 = RString::new_utf8("é");
    /// let binary = VM::eval(r#""\xFF".b"#).unwrap();
    ///
    /// let combined = RString::compatible_encoding(&utf8, &RString::new_utf8("a"));
    /// assert_eq!(combined.try_convert_to::<Encoding>().unwrap().name(), "UTF-8");
    /// assert!(RString::compatible_encoding(&utf8, &binary).is_nil());
    /// ```
    fn compatible_encoding(obj1: &impl Object, obj2: &impl Object) -> AnyObject;
}

#[cfg(test)]
mod tests {
    use crate::{Encoding, EncodingSupport, Hash, Object, RString, Symbol, VM};

    #[test]
    fn test_encoding_support() {
        crate::on_ruby_thread(|| {
            VM::init_loadpath();
            VM::require("enc/encdb");
            VM::require("enc/trans/transdb");

            let text = RString::new_utf8("héllo");
            assert_eq!(text.encoding().name(), "UTF-8");
            assert!(text.is_valid_encoding());

            // Replace what US-ASCII can't hold.
            let mut options = Hash::new();
            options.store(Symbol::new("undef"), Symbol::new("replace"));
            let ascii = text.encode(Encoding::us_ascii(), Some(options));
            assert_eq!(ascii.to_str(), "h?llo");
            assert_eq!(ascii.encoding().name(), "US-ASCII");

            let same = RString::new_utf8("plain").encode(Encoding::utf8(), None);
            assert_eq!(same.to_str(), "plain");

            let mut bytes = RString::new_utf8("é");
            let binary = bytes
                .force_encoding(Encoding::find("ASCII-8BIT").unwrap())
                .unwrap();
            assert_eq!(binary.encoding().name(), "ASCII-8BIT");
            assert_eq!(binary.to_bytes_unchecked(), "é".as_bytes());

            let mut frozen = RString::new_utf8("x").freeze();
            assert!(frozen.force_encoding(Encoding::utf8()).is_err());

            let invalid = RString::from_bytes(&[0xff], &Encoding::utf8());
            assert!(!invalid.is_valid_encoding());

            let ascii_text = RString::new_utf8("abc");
            assert!(text.compatible_with(&ascii_text));
            assert!(!text.compatible_with(&binary));

            let compatible = RString::compatible_encoding(&text, &ascii_text);
            assert_eq!(
                compatible.try_convert_to::<Encoding>().unwrap().name(),
                "UTF-8"
            );
            assert!(RString::compatible_encoding(&text, &binary).is_nil());
        });
    }
}
