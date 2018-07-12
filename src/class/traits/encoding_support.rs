use {AnyException, Encoding};

pub trait EncodingSupport {
    fn encoding(&self) -> Encoding;
    fn force_encoding(&mut self, enc: Encoding) -> Result<Self, AnyException> where Self: Sized;
}

