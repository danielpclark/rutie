use std::convert::From;

use crate::{
    binding::{class, regexp, vm},
    rubysys::regexp::{rb_cMatch, rb_cRegexp},
    types::Value,
    AnyException, AnyObject, Fixnum, NilClass, Object, RString, VerifiedObject,
};

/// `Regexp`
#[derive(Debug)]
#[repr(C)]
pub struct Regexp {
    value: Value,
}

impl Regexp {
    /// `Regexp::IGNORECASE`, the `i` flag.
    pub const IGNORECASE: i32 = 1;
    /// `Regexp::EXTENDED`, the `x` flag.
    pub const EXTENDED: i32 = 2;
    /// `Regexp::MULTILINE`, the `m` flag.
    pub const MULTILINE: i32 = 4;

    /// Compiles `pattern` with `options` (a combination of
    /// `Regexp::IGNORECASE`, `EXTENDED` and `MULTILINE`), or returns the
    /// `RegexpError` for an invalid pattern (`rb_reg_new_str`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Exception, Regexp, VM};
    /// # VM::init();
    ///
    /// let regexp = Regexp::new("h(e)llo", Regexp::IGNORECASE).unwrap();
    ///
    /// assert_eq!(regexp.source().to_str(), "h(e)llo");
    /// assert_eq!(regexp.options() & Regexp::IGNORECASE, Regexp::IGNORECASE);
    ///
    /// assert!(Regexp::new("(unclosed", 0).is_err());
    /// ```
    pub fn new(pattern: &str, options: i32) -> Result<Self, AnyException> {
        vm::protect_value(|| regexp::new(pattern, options))
            .map(Regexp::from)
            .map_err(AnyException::from)
    }

    /// Returns the pattern's source text (Ruby's `source`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, Regexp, VM};
    /// # VM::init();
    ///
    /// let regexp = VM::eval("/a+b/").unwrap().try_convert_to::<Regexp>().unwrap();
    ///
    /// assert_eq!(regexp.source().to_str(), "a+b");
    /// ```
    pub fn source(&self) -> RString {
        RString::from(vm::call_method(self.value(), "source", &[]))
    }

    /// Returns the option bits (`rb_reg_options`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, Regexp, VM};
    /// # VM::init();
    ///
    /// let regexp = VM::eval("/a/mx").unwrap().try_convert_to::<Regexp>().unwrap();
    ///
    /// assert_eq!(regexp.options() & (Regexp::MULTILINE | Regexp::EXTENDED), Regexp::MULTILINE | Regexp::EXTENDED);
    /// ```
    pub fn options(&self) -> i32 {
        regexp::options(self.value())
    }

    /// Returns the character offset of the first match in `string`, or
    /// `None` (Ruby's `=~`, `rb_reg_match`). Returns the error when the
    /// string cannot be matched, such as one with invalid bytes.
    ///
    /// Sets `$~` for the Ruby code that called into Rust.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Regexp, RString, VM};
    /// # VM::init();
    ///
    /// let regexp = Regexp::new("l+", 0).unwrap();
    ///
    /// assert_eq!(regexp.find(&RString::new_utf8("héllo")).unwrap(), Some(2));
    /// assert_eq!(regexp.find(&RString::new_utf8("abc")).unwrap(), None);
    /// ```
    pub fn find(&self, string: &RString) -> Result<Option<usize>, AnyException> {
        let (regexp, string) = (self.value(), string.value());

        vm::protect_value(|| regexp::match_offset(regexp, string))
            .map(|offset| {
                if offset.is_nil() {
                    None
                } else {
                    Some(Fixnum::from(offset).to_i64() as usize)
                }
            })
            .map_err(AnyException::from)
    }

    /// Returns the `MatchData` of the first match in `string`, or `None`
    /// (`rb_reg_match` and `rb_backref_get`). Returns the error when the
    /// string cannot be matched, such as one with invalid bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Regexp, RString, VM};
    /// # VM::init();
    ///
    /// let regexp = Regexp::new(r"(?<key>\w+)=(\d+)", 0).unwrap();
    /// let found = regexp.match_data(&RString::new_utf8("x: count=42;")).unwrap().unwrap();
    ///
    /// assert_eq!(found.matched().to_str(), "count=42");
    /// assert_eq!(found.named("key").unwrap().to_str(), "count");
    /// assert!(regexp.match_data(&RString::new_utf8("none")).unwrap().is_none());
    /// ```
    pub fn match_data(&self, string: &RString) -> Result<Option<MatchData>, AnyException> {
        let (regexp, string) = (self.value(), string.value());

        vm::protect_value(|| {
            let offset = regexp::match_offset(regexp, string);

            if offset.is_nil() {
                offset
            } else {
                regexp::last_match_data()
            }
        })
        .map(|found| {
            if found.is_nil() {
                None
            } else {
                Some(MatchData::from(found))
            }
        })
        .map_err(AnyException::from)
    }
}

impl From<Value> for Regexp {
    fn from(value: Value) -> Self {
        Regexp { value }
    }
}

impl Into<Value> for Regexp {
    fn into(self) -> Value {
        self.value
    }
}

impl Into<AnyObject> for Regexp {
    fn into(self) -> AnyObject {
        AnyObject::from(self.value)
    }
}

impl Object for Regexp {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for Regexp {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        class::is_kind_of(object.value(), unsafe { rb_cRegexp })
    }

    fn error_message() -> &'static str {
        "Error converting to Regexp"
    }
}

impl PartialEq for Regexp {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

/// `MatchData`, the result of a successful `Regexp` match.
#[derive(Debug)]
#[repr(C)]
pub struct MatchData {
    value: Value,
}

impl MatchData {
    /// Returns `$~`, the last match in the calling Ruby frame
    /// (`rb_backref_get`), or `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{MatchData, Regexp, RString, VM};
    /// # VM::init();
    ///
    /// Regexp::new("b", 0).unwrap().find(&RString::new_utf8("abc")).unwrap();
    ///
    /// assert_eq!(MatchData::last().unwrap().pre_match().to_str(), "a");
    /// ```
    pub fn last() -> Option<MatchData> {
        let found = regexp::last_match_data();

        if found.is_nil() {
            None
        } else {
            Some(MatchData::from(found))
        }
    }

    /// Sets (or with `None`, clears) `$~` for the calling Ruby frame
    /// (`rb_backref_set`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{MatchData, Regexp, RString, VM};
    /// # VM::init();
    ///
    /// Regexp::new("b", 0).unwrap().find(&RString::new_utf8("abc")).unwrap();
    /// MatchData::set_last(None);
    ///
    /// assert!(MatchData::last().is_none());
    /// ```
    pub fn set_last(match_data: Option<&MatchData>) {
        let value = match_data
            .map(Object::value)
            .unwrap_or_else(|| NilClass::new().value());

        regexp::set_last_match_data(value)
    }

    /// Returns capture group `n` (`0` is the whole match), or `None` if it
    /// did not take part in the match or does not exist
    /// (`rb_reg_nth_match`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Regexp, RString, VM};
    /// # VM::init();
    ///
    /// let found = Regexp::new("(a)(x)?(c)", 0).unwrap()
    ///     .match_data(&RString::new_utf8("ac")).unwrap().unwrap();
    ///
    /// assert_eq!(found.nth(1).unwrap().to_str(), "a");
    /// assert!(found.nth(2).is_none());
    /// assert_eq!(found.nth(3).unwrap().to_str(), "c");
    /// assert!(found.nth(9).is_none());
    /// ```
    pub fn nth(&self, n: i32) -> Option<RString> {
        let group = regexp::nth_match(self.value(), n);

        if group.is_nil() {
            None
        } else {
            Some(RString::from(group))
        }
    }

    /// Returns the named capture group `name`, or `None` if it did not take
    /// part in the match or there is no such group
    /// (`rb_reg_backref_number`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Regexp, RString, VM};
    /// # VM::init();
    ///
    /// let found = Regexp::new("(?<year>\\d{4})", 0).unwrap()
    ///     .match_data(&RString::new_utf8("in 2019")).unwrap().unwrap();
    ///
    /// assert_eq!(found.named("year").unwrap().to_str(), "2019");
    /// assert!(found.named("month").is_none());
    /// ```
    pub fn named(&self, name: &str) -> Option<RString> {
        let match_data = self.value();
        let mut number = 0;

        vm::protect_value(|| {
            number = regexp::backref_number(match_data, name);

            NilClass::new().value()
        })
        .ok()?;

        self.nth(number)
    }

    /// Returns the whole matched text (`rb_reg_last_match`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Regexp, RString, VM};
    /// # VM::init();
    ///
    /// let found = Regexp::new("b+", 0).unwrap()
    ///     .match_data(&RString::new_utf8("abbc")).unwrap().unwrap();
    ///
    /// assert_eq!(found.matched().to_str(), "bb");
    /// ```
    pub fn matched(&self) -> RString {
        RString::from(regexp::matched(self.value()))
    }

    /// Returns the text before the match (`rb_reg_match_pre`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Regexp, RString, VM};
    /// # VM::init();
    ///
    /// let found = Regexp::new("b+", 0).unwrap()
    ///     .match_data(&RString::new_utf8("abbc")).unwrap().unwrap();
    ///
    /// assert_eq!(found.pre_match().to_str(), "a");
    /// ```
    pub fn pre_match(&self) -> RString {
        RString::from(regexp::pre_match(self.value()))
    }

    /// Returns the text after the match (`rb_reg_match_post`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Regexp, RString, VM};
    /// # VM::init();
    ///
    /// let found = Regexp::new("b+", 0).unwrap()
    ///     .match_data(&RString::new_utf8("abbc")).unwrap().unwrap();
    ///
    /// assert_eq!(found.post_match().to_str(), "c");
    /// ```
    pub fn post_match(&self) -> RString {
        RString::from(regexp::post_match(self.value()))
    }
}

impl From<Value> for MatchData {
    fn from(value: Value) -> Self {
        MatchData { value }
    }
}

impl Into<Value> for MatchData {
    fn into(self) -> Value {
        self.value
    }
}

impl Into<AnyObject> for MatchData {
    fn into(self) -> AnyObject {
        AnyObject::from(self.value)
    }
}

impl Object for MatchData {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for MatchData {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        class::is_kind_of(object.value(), unsafe { rb_cMatch })
    }

    fn error_message() -> &'static str {
        "Error converting to MatchData"
    }
}

impl PartialEq for MatchData {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Encoding, MatchData, Object, RString, Regexp, VM};

    #[test]
    fn test_regexp_and_match_data() {
        crate::on_ruby_thread(|| {
            let regexp = Regexp::new("(?<word>[a-z]+)(?<digit>\\d)?", Regexp::IGNORECASE).unwrap();
            let subject = RString::new_utf8("123 ABC4 def");

            assert_eq!(regexp.find(&subject).unwrap(), Some(4));
            let found = regexp.match_data(&subject).unwrap().unwrap();
            assert_eq!(found.matched().to_str(), "ABC4");
            assert_eq!(found.named("word").unwrap().to_str(), "ABC");
            assert_eq!(found.named("digit").unwrap().to_str(), "4");
            assert_eq!(found.nth(2).unwrap().to_str(), "4");
            assert_eq!(found.pre_match().to_str(), "123 ");
            assert_eq!(found.post_match().to_str(), " def");
            assert!(found.named("missing").is_none());

            assert_eq!(MatchData::last().unwrap().matched().to_str(), "ABC4");
            MatchData::set_last(Some(&found));
            assert!(MatchData::last().is_some());

            assert_eq!(regexp.find(&RString::new_utf8("123")).unwrap(), None);
            assert!(regexp
                .match_data(&RString::new_utf8("123"))
                .unwrap()
                .is_none());

            // Matching a string with invalid bytes is an error, not a crash.
            let broken = RString::from_bytes(b"\xFFabc", &Encoding::utf8());
            assert!(regexp.find(&broken).is_err());

            let error = Regexp::new("[", 0).unwrap_err();
            assert!(crate::Class::from_existing("RegexpError").case_equals(&error));

            let literal = VM::eval("/x/i")
                .unwrap()
                .try_convert_to::<Regexp>()
                .unwrap();
            assert_eq!(literal.options() & Regexp::IGNORECASE, Regexp::IGNORECASE);
            assert!(VM::eval("'x'").unwrap().try_convert_to::<Regexp>().is_err());
        });
    }

    #[test]
    fn test_regexp_source() {
        crate::on_ruby_thread(|| {
            let regexp = Regexp::new(r"\d+(?<unit>px)?", 0).unwrap();
            assert_eq!(regexp.source().to_str(), r"\d+(?<unit>px)?");

            let from_ruby = VM::eval("/a\\/b/i")
                .unwrap()
                .try_convert_to::<Regexp>()
                .unwrap();
            // Ruby drops the escape before `/` from the source.
            assert_eq!(from_ruby.source().to_str(), "a/b");
        });
    }
}
