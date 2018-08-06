use {AnyException, Encoding, Hash};

pub trait EncodingSupport {
    fn encode(&mut self, enc: Encoding, opts: Option<Hash>) -> Self where Self: Sized;
    fn encoding(&self) -> Encoding;
    fn force_encoding(&mut self, enc: Encoding) -> Result<Self, AnyException> where Self: Sized;
}

