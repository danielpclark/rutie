use crate::{
    binding::{encoding, string},
    rubysys::string::{rstring_end, rstring_ptr},
    types::{c_char, c_int},
    EncodingSupport, Object, RString,
};

/// `CodepointIterator`
#[derive(Debug)]
pub struct CodepointIterator {
    rstring: RString,
    ptr: *const c_char,
}

impl CodepointIterator {
    /// Create new codepoint iterator
    ///
    /// ```
    /// use rutie::{RString, VM, CodepointIterator};
    /// # VM::init();
    ///
    /// let string = RString::new_utf8("aeiou");
    /// let ci = CodepointIterator::new(&string);
    ///
    /// let result: Vec<usize> = ci.into_iter().collect();
    ///
    /// assert_eq!(vec![97, 101, 105, 111, 117], result);
    /// ```
    pub fn new(rstring: &RString) -> Self {
        let fstring = string::new_frozen(rstring.value());

        CodepointIterator {
            rstring: RString::from(fstring),
            ptr: unsafe { rstring_ptr(fstring) },
        }
    }
}

impl Iterator for CodepointIterator {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let mut n: c_int = 0;

        let ptr = self.ptr;
        let end = unsafe { rstring_end(self.rstring.value()) };
        let enc = self.rstring.encoding();

        if ptr < end {
            let result = Some(encoding::next_codepoint(ptr, end, &mut n, enc.value()));
            self.ptr = unsafe { ptr.add(n as usize) };
            result
        } else {
            None
        }
    }
}
