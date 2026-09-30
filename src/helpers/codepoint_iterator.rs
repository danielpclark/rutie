use crate::{
    binding::{encoding, string},
    rubysys::string::{rstring_end, rstring_ptr},
    types::{c_char, c_int, InternalValue},
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

#[cfg(test)]
mod tests {
    use super::CodepointIterator;
    use crate::{EncodingSupport, Object, RString, VM};

    #[test]
    fn test_codepoint_iterator() {
        crate::on_ruby_thread(|| {
            let string = RString::new_utf8("aé€😀");
            let codepoints: Vec<usize> = CodepointIterator::new(&string).collect();
            assert_eq!(codepoints, vec![0x61, 0xe9, 0x20ac, 0x1f600]);

            assert_eq!(CodepointIterator::new(&RString::new_utf8("")).count(), 0);

            // Same answer as Ruby's String#codepoints.
            let ruby = VM::eval(r#""a\u00e9\u20ac\u{1f600}".codepoints.sum"#).unwrap();
            let total: usize = codepoints.iter().sum();
            assert_eq!(
                ruby.try_convert_to::<crate::Fixnum>().unwrap().to_i64() as usize,
                total
            );

            assert!(string.is_valid_encoding());
        });
    }
}
