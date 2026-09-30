use crate::{AnyException, AnyObject, Encoding, Hash, Object};

pub trait EncodingSupport {
    fn encode(&self, enc: Encoding, opts: Option<Hash>) -> Self
    where
        Self: Sized;
    fn encoding(&self) -> Encoding;
    fn force_encoding(&mut self, enc: Encoding) -> Result<Self, AnyException>
    where
        Self: Sized;
    fn is_valid_encoding(&self) -> bool;
    fn compatible_with(&self, other: &impl Object) -> bool;
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
