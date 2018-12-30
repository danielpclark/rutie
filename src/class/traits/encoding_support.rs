use {AnyException, Encoding, Hash, AnyObject, Object, RString};

pub trait EncodingSupport {
    fn encode(&self, enc: Encoding, opts: Option<Hash>) -> Self where Self: Sized;
    fn encoding(&self) -> Encoding;
    fn force_encoding(&mut self, enc: Encoding) -> Result<Self, AnyException> where Self: Sized;
    fn is_valid_encoding(&self) -> bool;
    fn compatible_with(&self, other: &impl Object) -> bool;
    fn compatible_encoding(&self, other: &impl Object) -> AnyObject;
}

