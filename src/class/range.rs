use std::convert::From;

use crate::{
    binding::{class, range, vm},
    rubysys::range::rb_cRange,
    types::Value,
    AnyException, AnyObject, NilClass, Object, VerifiedObject,
};

/// `Range`
#[derive(Debug)]
#[repr(C)]
pub struct Range {
    value: Value,
}

impl Range {
    /// Creates `begin..end`, or `begin...end` when `exclusive` is `true`
    /// (`rb_range_new`). Returns the `ArgumentError` when the bounds cannot
    /// be compared with `<=>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, Range, RString, VM};
    /// # VM::init();
    ///
    /// let range = Range::new(&Fixnum::new(1), &Fixnum::new(5), true).unwrap();
    ///
    /// assert_eq!(range.begin().try_convert_to::<Fixnum>(), Ok(Fixnum::new(1)));
    /// assert!(range.excludes_end());
    ///
    /// assert!(Range::new(&Fixnum::new(1), &RString::new_utf8("z"), false).is_err());
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// 1...5
    /// ```
    pub fn new<B: Object, E: Object>(
        begin: &B,
        end: &E,
        exclusive: bool,
    ) -> Result<Self, AnyException> {
        let (begin, end) = (begin.value(), end.value());

        vm::protect_value(|| range::new(begin, end, exclusive))
            .map(Range::from)
            .map_err(AnyException::from)
    }

    /// Returns the first bound (`rb_range_values`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, Range, VM};
    /// # VM::init();
    ///
    /// let range = VM::eval("3..7").unwrap().try_convert_to::<Range>().unwrap();
    ///
    /// assert_eq!(range.begin().try_convert_to::<Fixnum>(), Ok(Fixnum::new(3)));
    /// ```
    pub fn begin(&self) -> AnyObject {
        self.values().0
    }

    /// Returns the last bound (`rb_range_values`); `nil` for an endless
    /// range (Ruby 2.6+).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, Range, VM};
    /// # VM::init();
    ///
    /// let range = VM::eval("3..7").unwrap().try_convert_to::<Range>().unwrap();
    ///
    /// assert_eq!(range.end().try_convert_to::<Fixnum>(), Ok(Fixnum::new(7)));
    /// ```
    pub fn end(&self) -> AnyObject {
        self.values().1
    }

    /// Returns `true` for `begin...end` (Ruby's `exclude_end?`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, Range, VM};
    /// # VM::init();
    ///
    /// let inclusive = VM::eval("1..2").unwrap().try_convert_to::<Range>().unwrap();
    /// let exclusive = VM::eval("1...2").unwrap().try_convert_to::<Range>().unwrap();
    ///
    /// assert!(!inclusive.excludes_end());
    /// assert!(exclusive.excludes_end());
    /// ```
    pub fn excludes_end(&self) -> bool {
        self.values().2
    }

    fn values(&self) -> (AnyObject, AnyObject, bool) {
        let nil = NilClass::new().value();
        let (begin, end, exclusive) = range::values(self.value()).unwrap_or((nil, nil, false));

        (AnyObject::from(begin), AnyObject::from(end), exclusive)
    }

    /// Returns the `(offset, length)` this range selects from a sequence of
    /// `total` elements, the way `array[range]` does (`rb_range_beg_len`):
    /// negative bounds count from the end and the length is clipped.
    /// `Ok(None)` when the range starts past the end, and an error when the
    /// bounds are not integers.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, Range, VM};
    /// # VM::init();
    ///
    /// let range = VM::eval("1..-2").unwrap().try_convert_to::<Range>().unwrap();
    ///
    /// assert_eq!(range.offset_and_length(5).unwrap(), Some((1, 3)));
    ///
    /// let past = VM::eval("7..9").unwrap().try_convert_to::<Range>().unwrap();
    ///
    /// assert_eq!(past.offset_and_length(5).unwrap(), None);
    ///
    /// let letters = VM::eval("'a'..'c'").unwrap().try_convert_to::<Range>().unwrap();
    ///
    /// assert!(letters.offset_and_length(5).is_err());
    /// ```
    pub fn offset_and_length(&self, total: usize) -> Result<Option<(usize, usize)>, AnyException> {
        let range = self.value();
        let mut result = None;

        vm::protect_value(|| {
            result = range::begin_length(range, total);

            NilClass::new().value()
        })
        .map(|_| result)
        .map_err(AnyException::from)
    }

    /// Returns `(begin, end, step, exclude_end)` of a range or an
    /// arithmetic sequence such as `(1..10).step(3)`, or `None` for
    /// anything else (`rb_arithmetic_sequence_extract`).
    ///
    /// Only available on Ruby 2.6 and later.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, Range, VM};
    /// # VM::init();
    ///
    /// # #[cfg(ruby_gte_2_6)]
    /// # {
    /// let sequence = VM::eval("(1..10).step(3)").unwrap();
    /// let (begin, end, step, exclusive) = Range::arithmetic_sequence(&sequence).unwrap();
    ///
    /// assert_eq!(step.try_convert_to::<Fixnum>(), Ok(Fixnum::new(3)));
    /// assert!(!exclusive);
    /// assert!(Range::arithmetic_sequence(&Fixnum::new(1)).is_none());
    /// # }
    /// ```
    #[cfg(ruby_gte_2_6)]
    pub fn arithmetic_sequence<T: Object>(
        object: &T,
    ) -> Option<(AnyObject, AnyObject, AnyObject, bool)> {
        range::arithmetic_sequence(object.value()).map(|(begin, end, step, exclusive)| {
            (
                AnyObject::from(begin),
                AnyObject::from(end),
                AnyObject::from(step),
                exclusive,
            )
        })
    }
}

impl From<Value> for Range {
    fn from(value: Value) -> Self {
        Range { value }
    }
}

impl Into<Value> for Range {
    fn into(self) -> Value {
        self.value
    }
}

impl Into<AnyObject> for Range {
    fn into(self) -> AnyObject {
        AnyObject::from(self.value)
    }
}

impl Object for Range {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for Range {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        class::is_kind_of(object.value(), unsafe { rb_cRange })
    }

    fn error_message() -> &'static str {
        "Error converting to Range"
    }
}

impl PartialEq for Range {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Fixnum, Object, RString, Range, GC, VM};

    #[test]
    fn test_range() {
        crate::on_ruby_thread(|| {
            let range = Range::new(&Fixnum::new(-3), &Fixnum::new(-1), false).unwrap();
            GC::start();

            assert_eq!(
                range.begin().try_convert_to::<Fixnum>(),
                Ok(Fixnum::new(-3))
            );
            assert_eq!(range.end().try_convert_to::<Fixnum>(), Ok(Fixnum::new(-1)));
            assert!(!range.excludes_end());
            assert_eq!(range.offset_and_length(10).unwrap(), Some((7, 3)));
            assert_eq!(range.offset_and_length(2).unwrap(), None);

            let empty = Range::new(&Fixnum::new(2), &Fixnum::new(2), true).unwrap();
            assert_eq!(empty.offset_and_length(5).unwrap(), Some((2, 0)));

            let strings =
                Range::new(&RString::new_utf8("a"), &RString::new_utf8("e"), true).unwrap();
            assert!(strings.offset_and_length(3).is_err());

            let from_ruby = VM::eval("(1..4)").unwrap();
            assert!(from_ruby.try_convert_to::<Range>().is_ok());
            // A Struct is not a Range even though Ruby 2 stores ranges as structs.
            assert!(VM::eval("Struct.new(:a).new(1)")
                .unwrap()
                .try_convert_to::<Range>()
                .is_err());
        });
    }

    #[cfg(ruby_gte_2_6)]
    #[test]
    fn test_endless_range_and_arithmetic_sequence() {
        crate::on_ruby_thread(|| {
            let endless = VM::eval("(3..)")
                .unwrap()
                .try_convert_to::<Range>()
                .unwrap();
            assert!(endless.end().is_nil());
            assert_eq!(endless.offset_and_length(5).unwrap(), Some((3, 2)));

            let (begin, end, step, exclusive) =
                Range::arithmetic_sequence(&VM::eval("(0...9).step(2)").unwrap()).unwrap();
            assert_eq!(begin.try_convert_to::<Fixnum>(), Ok(Fixnum::new(0)));
            assert_eq!(end.try_convert_to::<Fixnum>(), Ok(Fixnum::new(9)));
            assert_eq!(step.try_convert_to::<Fixnum>(), Ok(Fixnum::new(2)));
            assert!(exclusive);
        });
    }
}
