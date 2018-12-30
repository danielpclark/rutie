use {AnyException, Encoding, Hash, AnyObject};

pub trait EncodingSupport {
    fn encode(&self, enc: Encoding, opts: Option<Hash>) -> Self where Self: Sized;
    fn encoding(&self) -> Encoding;
    fn force_encoding(&mut self, enc: Encoding) -> Result<Self, AnyException> where Self: Sized;
    fn is_valid_encoding(&self) -> bool;
    fn compatible_with<T>(&self, other: T) -> bool where T: Into<AnyObject>;
    fn compatible_encoding<T>(&self, other: T) -> AnyObject where T: Into<AnyObject>;
}

