use std::ffi::CString;

use crate::{
    binding::encoding,
    rubysys::encoding::{self as sys, RbEconv},
    types::c_int,
    AnyException, Encoding, EncodingSupport, Exception, Hash, Object, RString,
};

/// A transcoder between two encodings (Ruby's `Encoding::Converter`, the
/// `rb_econv_*` API), for converting text a piece at a time.
///
/// The converter is closed when dropped (`rb_econv_close`). Ruby's encoding
/// database must be loaded (`VM::require("enc/encdb")` and
/// `VM::require("enc/trans/transdb")` in an embedded VM) for conversions other
/// than between the built-in encodings.
///
/// # Examples
///
/// ```
/// use rutie::{EncodingConverter, RString, VM};
/// # VM::init();
/// # VM::init_loadpath();
/// VM::require("enc/encdb");
/// VM::require("enc/trans/transdb");
///
/// let mut converter = EncodingConverter::new("UTF-8", "ISO-8859-1", 0).unwrap();
///
/// let mut latin1 = converter.convert(&RString::new_utf8("café")).unwrap();
/// latin1.append(&converter.finish().unwrap()).unwrap();
///
/// assert_eq!(latin1.to_bytes_unchecked(), b"caf\xE9");
/// ```
#[derive(Debug)]
pub struct EncodingConverter {
    converter: *mut RbEconv,
    destination: CString,
}

/// Where [`EncodingConverter::convert_buffer`](struct.EncodingConverter.html#method.convert_buffer)
/// stopped (`rb_econv_result_t`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConversionResult {
    /// The input has an invalid byte sequence.
    InvalidByteSequence,
    /// The input has a character the destination encoding cannot represent.
    UndefinedConversion,
    /// The output buffer is full.
    DestinationBufferFull,
    /// All the input was converted; more may follow (partial input).
    SourceBufferEmpty,
    /// All the input was converted and the conversion is finished.
    Finished,
    /// Stopped after writing output, as asked by `AFTER_OUTPUT`.
    AfterOutput,
    /// The input ends in the middle of a character.
    IncompleteInput,
}

impl ConversionResult {
    fn from_raw(result: sys::EconvResult) -> Self {
        match result {
            sys::ECONV_RESULT_INVALID_BYTE_SEQUENCE => ConversionResult::InvalidByteSequence,
            sys::ECONV_RESULT_UNDEFINED_CONVERSION => ConversionResult::UndefinedConversion,
            sys::ECONV_RESULT_DESTINATION_BUFFER_FULL => ConversionResult::DestinationBufferFull,
            sys::ECONV_RESULT_SOURCE_BUFFER_EMPTY => ConversionResult::SourceBufferEmpty,
            sys::ECONV_RESULT_FINISHED => ConversionResult::Finished,
            sys::ECONV_RESULT_AFTER_OUTPUT => ConversionResult::AfterOutput,
            _ => ConversionResult::IncompleteInput,
        }
    }
}

impl EncodingConverter {
    /// Replace invalid input with the replacement (`invalid: :replace`).
    pub const INVALID_REPLACE: i32 = sys::ECONV_INVALID_REPLACE;
    /// Replace characters the destination lacks with the replacement
    /// (`undef: :replace`).
    pub const UNDEF_REPLACE: i32 = sys::ECONV_UNDEF_REPLACE;
    /// Replace characters the destination lacks with XML character
    /// references (`&#x...;`).
    pub const UNDEF_HEX_CHARREF: i32 = sys::ECONV_UNDEF_HEX_CHARREF;
    /// Convert CRLF and CR to LF (`universal_newline: true`).
    pub const UNIVERSAL_NEWLINE_DECORATOR: i32 = sys::ECONV_UNIVERSAL_NEWLINE_DECORATOR;
    /// Convert LF to CRLF (`crlf_newline: true`).
    pub const CRLF_NEWLINE_DECORATOR: i32 = sys::ECONV_CRLF_NEWLINE_DECORATOR;
    /// Convert LF to CR (`cr_newline: true`).
    pub const CR_NEWLINE_DECORATOR: i32 = sys::ECONV_CR_NEWLINE_DECORATOR;
    /// Convert CRLF and CR to LF on output (`lf_newline: true`).
    pub const LF_NEWLINE_DECORATOR: i32 = sys::ECONV_LF_NEWLINE_DECORATOR;
    /// Escape `&`, `<` and `>` as XML text (`xml: :text`).
    pub const XML_TEXT_DECORATOR: i32 = sys::ECONV_XML_TEXT_DECORATOR;
    /// Escape `&`, `<`, `>` and `"` for an XML attribute (`xml: :attr`).
    pub const XML_ATTR_CONTENT_DECORATOR: i32 = sys::ECONV_XML_ATTR_CONTENT_DECORATOR;
    /// Quote the output with `"` (with `XML_ATTR_CONTENT_DECORATOR`).
    pub const XML_ATTR_QUOTE_DECORATOR: i32 = sys::ECONV_XML_ATTR_QUOTE_DECORATOR;
    /// For [`convert_buffer`](#method.convert_buffer): more input follows.
    pub const PARTIAL_INPUT: i32 = sys::ECONV_PARTIAL_INPUT;
    /// For [`convert_buffer`](#method.convert_buffer): stop after writing
    /// some output.
    pub const AFTER_OUTPUT: i32 = sys::ECONV_AFTER_OUTPUT;

    /// Opens a converter from encoding `source` to `destination` with
    /// `flags` (a combination of the constants above), or returns the
    /// `Encoding::ConverterNotFoundError` (`rb_econv_open`,
    /// `rb_econv_open_exc`). As in Ruby, there is no converter from an
    /// encoding to itself; empty names for both make one that only applies
    /// the decorators (newline conversion, XML escaping).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{EncodingConverter, Exception, VM};
    /// # VM::init();
    ///
    /// assert!(EncodingConverter::new("", "", EncodingConverter::UNIVERSAL_NEWLINE_DECORATOR).is_ok());
    /// assert!(EncodingConverter::new("UTF-8", "UTF-8", 0).is_err());
    ///
    /// let error = EncodingConverter::new("UTF-8", "NO-SUCH-ENCODING", 0).unwrap_err();
    /// assert!(error.message().contains("NO-SUCH-ENCODING"));
    /// ```
    pub fn new(source: &str, destination: &str, flags: i32) -> Result<Self, AnyException> {
        Self::open(
            source,
            destination,
            flags as c_int,
            crate::NilClass::new().value(),
        )
    }

    /// Opens a converter from encoding `source` to `destination` configured
    /// with the options of Ruby's `String#encode` (`invalid: :replace`,
    /// `replace: "?"`, `xml: :text`, `crlf_newline: true`, ...)
    /// (`rb_econv_prepare_options`, `rb_econv_open_opts`). Returns the error
    /// for bad options or a missing converter.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{EncodingConverter, Hash, RString, Symbol, VM};
    /// # VM::init();
    /// # VM::init_loadpath();
    /// VM::require("enc/encdb");
    /// VM::require("enc/trans/transdb");
    ///
    /// let mut options = Hash::new();
    /// options.store(Symbol::new("undef"), Symbol::new("replace"));
    /// options.store(Symbol::new("replace"), RString::new_utf8("?"));
    ///
    /// let mut converter = EncodingConverter::with_options("UTF-8", "US-ASCII", &options).unwrap();
    ///
    /// assert_eq!(converter.convert(&RString::new_utf8("naïve")).unwrap().to_str(), "na?ve");
    /// ```
    pub fn with_options(
        source: &str,
        destination: &str,
        options: &Hash,
    ) -> Result<Self, AnyException> {
        Self::open(source, destination, 0, options.value())
    }

    fn open(
        source: &str,
        destination: &str,
        flags: c_int,
        options: crate::types::Value,
    ) -> Result<Self, AnyException> {
        let converter = encoding::econv_open(source, destination, flags, options)
            .map_err(AnyException::from)?;

        Ok(EncodingConverter {
            converter,
            destination: crate::util::str_to_cstring(destination),
        })
    }

    /// Returns `true` if Ruby knows a conversion path from `source` to
    /// `destination` (`rb_econv_has_convpath_p`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{EncodingConverter, VM};
    /// # VM::init();
    /// # VM::init_loadpath();
    /// VM::require("enc/encdb");
    /// VM::require("enc/trans/transdb");
    ///
    /// assert!(EncodingConverter::has_path("UTF-8", "EUC-JP"));
    /// assert!(!EncodingConverter::has_path("UTF-8", "NO-SUCH-ENCODING"));
    /// ```
    pub fn has_path(source: &str, destination: &str) -> bool {
        encoding::econv_has_path(source, destination)
    }

    /// Returns the ASCII-compatible encoding Ruby converts the
    /// ASCII-incompatible encoding `name` through, such as `"UTF-8"` for
    /// `"UTF-16BE"`, or `None` (Ruby's `Encoding::Converter.asciicompat_encoding`,
    /// `rb_econv_asciicompat_encoding`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{EncodingConverter, VM};
    /// # VM::init();
    /// # VM::init_loadpath();
    /// VM::require("enc/encdb");
    /// VM::require("enc/trans/transdb");
    ///
    /// assert_eq!(EncodingConverter::ascii_compatible_encoding("UTF-16BE").unwrap(), "UTF-8");
    /// assert!(EncodingConverter::ascii_compatible_encoding("UTF-8").is_none());
    /// ```
    pub fn ascii_compatible_encoding(name: &str) -> Option<String> {
        encoding::econv_asciicompat_encoding(name)
    }

    /// Converts `source`, keeping an incomplete character at its end for the
    /// next call (Ruby's `Encoding::Converter#convert`,
    /// `rb_econv_str_append`). Returns the error (such as
    /// `Encoding::InvalidByteSequenceError`) where the conversion stopped.
    ///
    /// The result is in the destination encoding (the source's for a
    /// decorator-only converter).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, EncodingConverter, EncodingSupport, RString, VM};
    /// # VM::init();
    /// # VM::init_loadpath();
    /// VM::require("enc/encdb");
    /// VM::require("enc/trans/transdb");
    ///
    /// let mut converter = EncodingConverter::new("UTF-8", "UTF-16BE", 0).unwrap();
    ///
    /// let utf16 = converter.convert(&RString::new_utf8("hé")).unwrap();
    /// assert_eq!(utf16.to_bytes_unchecked(), b"\x00h\x00\xE9");
    /// assert_eq!(utf16.encoding(), Encoding::find("UTF-16BE").unwrap());
    ///
    /// let broken = RString::from_bytes(b"\xFF", &Encoding::utf8());
    /// assert!(converter.convert(&broken).is_err());
    /// ```
    pub fn convert(&mut self, source: &RString) -> Result<RString, AnyException> {
        encoding::econv_str_append(
            self.converter,
            source.value(),
            &self.destination,
            sys::ECONV_PARTIAL_INPUT,
        )
        .map(RString::from)
        .map_err(AnyException::from)
    }

    /// Converts `bytes` in the source encoding, as
    /// [`convert`](#method.convert) does (`rb_econv_append`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{EncodingConverter, VM};
    /// # VM::init();
    /// # VM::init_loadpath();
    /// VM::require("enc/encdb");
    /// VM::require("enc/trans/transdb");
    ///
    /// let mut converter = EncodingConverter::new("ISO-8859-1", "UTF-8", 0).unwrap();
    ///
    /// assert_eq!(converter.convert_bytes(b"caf\xE9").unwrap().to_str(), "café");
    /// ```
    pub fn convert_bytes(&mut self, bytes: &[u8]) -> Result<RString, AnyException> {
        encoding::econv_append(
            self.converter,
            bytes,
            &self.destination,
            sys::ECONV_PARTIAL_INPUT,
        )
        .map(RString::from)
        .map_err(AnyException::from)
    }

    /// Ends the conversion and returns what is left to write, such as the
    /// reset sequence of a stateful encoding (Ruby's
    /// `Encoding::Converter#finish`). Returns the
    /// `Encoding::InvalidByteSequenceError` if the input ended in the middle
    /// of a character.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{EncodingConverter, RString, VM};
    /// # VM::init();
    /// # VM::init_loadpath();
    /// VM::require("enc/encdb");
    /// VM::require("enc/trans/transdb");
    ///
    /// let mut converter = EncodingConverter::new("UTF-8", "ISO-2022-JP", 0).unwrap();
    ///
    /// let mut output = converter.convert(&RString::new_utf8("日本")).unwrap();
    /// output.append(&converter.finish().unwrap()).unwrap();
    ///
    /// // Escape sequences switch to JIS X 0208 and back to ASCII.
    /// assert_eq!(output.to_bytes_unchecked(), b"\x1B$BF|K\\\x1B(B");
    ///
    /// let mut incomplete = EncodingConverter::new("UTF-8", "UTF-16LE", 0).unwrap();
    /// incomplete.convert_bytes(b"\xE6").unwrap();
    /// assert!(incomplete.finish().is_err());
    /// ```
    pub fn finish(&mut self) -> Result<RString, AnyException> {
        encoding::econv_append(self.converter, &[], &self.destination, 0)
            .map(RString::from)
            .map_err(AnyException::from)
    }

    /// Converts from `source` into `destination` (Ruby's
    /// `Encoding::Converter#primitive_convert`, `rb_econv_convert`) and
    /// returns where it stopped with the number of bytes read and written.
    /// `flags` may hold `PARTIAL_INPUT` and `AFTER_OUTPUT`.
    ///
    /// After `InvalidByteSequence` or `UndefinedConversion`,
    /// [`last_error`](#method.last_error) describes the problem; calling
    /// again continues after it.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{ConversionResult, EncodingConverter, VM};
    /// # VM::init();
    /// # VM::init_loadpath();
    /// VM::require("enc/encdb");
    /// VM::require("enc/trans/transdb");
    ///
    /// let mut converter = EncodingConverter::new("UTF-8", "UTF-16LE", 0).unwrap();
    /// let mut output = [0u8; 4];
    ///
    /// let (result, read, written) = converter.convert_buffer("aé!".as_bytes(), &mut output, 0);
    /// assert_eq!(result, ConversionResult::DestinationBufferFull);
    /// assert_eq!((read, written), (4, 4));
    /// assert_eq!(output, [b'a', 0, 0xE9, 0]);
    ///
    /// // The converter holds the "!" it read; an empty source finishes.
    /// let (result, read, written) = converter.convert_buffer(&[], &mut output, 0);
    /// assert_eq!(result, ConversionResult::Finished);
    /// assert_eq!((read, written), (0, 2));
    /// assert_eq!(&output[..2], &[b'!', 0]);
    /// ```
    pub fn convert_buffer(
        &mut self,
        source: &[u8],
        destination: &mut [u8],
        flags: i32,
    ) -> (ConversionResult, usize, usize) {
        let (result, read, written) =
            encoding::econv_convert(self.converter, source, destination, flags as c_int);

        (ConversionResult::from_raw(result), read, written)
    }

    /// Returns the error the last conversion stopped at, or `None`
    /// (`rb_econv_make_exception`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{ConversionResult, EncodingConverter, Exception, VM};
    /// # VM::init();
    /// # VM::init_loadpath();
    /// VM::require("enc/encdb");
    /// VM::require("enc/trans/transdb");
    ///
    /// let mut converter = EncodingConverter::new("UTF-8", "US-ASCII", 0).unwrap();
    /// let mut output = [0u8; 8];
    ///
    /// let (result, _, _) = converter.convert_buffer("é".as_bytes(), &mut output, 0);
    ///
    /// assert_eq!(result, ConversionResult::UndefinedConversion);
    /// assert!(converter.last_error().unwrap().message().contains("U+00E9"));
    /// ```
    pub fn last_error(&self) -> Option<AnyException> {
        let error = encoding::econv_last_error(self.converter);

        if error.is_nil() {
            None
        } else {
            Some(AnyException::from(error))
        }
    }

    /// Sets the text used by `INVALID_REPLACE` and `UNDEF_REPLACE`
    /// (Ruby's `Encoding::Converter#replacement=`,
    /// `rb_econv_set_replacement`). Returns the
    /// `Encoding::UndefinedConversionError` if it cannot be converted to the
    /// destination encoding.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{EncodingConverter, RString, VM};
    /// # VM::init();
    /// # VM::init_loadpath();
    /// VM::require("enc/encdb");
    /// VM::require("enc/trans/transdb");
    ///
    /// let mut converter =
    ///     EncodingConverter::new("UTF-8", "US-ASCII", EncodingConverter::UNDEF_REPLACE).unwrap();
    /// converter.set_replacement(&RString::new_utf8("<?>")).unwrap();
    ///
    /// assert_eq!(converter.convert(&RString::new_utf8("aéb")).unwrap().to_str(), "a<?>b");
    /// assert!(converter.set_replacement(&RString::new_utf8("é")).is_err());
    /// ```
    pub fn set_replacement(&mut self, replacement: &RString) -> Result<(), AnyException> {
        let enc_name = replacement.encoding().name();

        if encoding::econv_set_replacement(
            self.converter,
            replacement.to_bytes_unchecked(),
            &enc_name,
        ) {
            Ok(())
        } else {
            Err(AnyException::new(
                "Encoding::UndefinedConversionError",
                Some("replacement string cannot be converted"),
            ))
        }
    }

    /// Adds the decorator `name` (such as `"universal_newline"`,
    /// `"crlf_newline"` or `"xml_text_escape"`) after the conversion
    /// (`rb_econv_decorate_at_last`). Returns the `ArgumentError` for an
    /// unknown decorator or one added after the conversion started.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{EncodingConverter, RString, VM};
    /// # VM::init();
    /// # VM::init_loadpath();
    /// VM::require("enc/encdb");
    /// VM::require("enc/trans/transdb");
    ///
    /// let mut converter = EncodingConverter::new("", "", 0).unwrap();
    /// converter.decorate_at_last("xml_text_escape").unwrap();
    ///
    /// assert_eq!(converter.convert(&RString::new_utf8("a<b")).unwrap().to_str(), "a&lt;b");
    /// assert!(converter.decorate_at_last("no_such_decorator").is_err());
    /// ```
    pub fn decorate_at_last(&mut self, name: &str) -> Result<(), AnyException> {
        self.decorate(name, false)
    }

    /// Adds the decorator `name` before the conversion
    /// (`rb_econv_decorate_at_first`); see
    /// [`decorate_at_last`](#method.decorate_at_last).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{EncodingConverter, RString, VM};
    /// # VM::init();
    /// # VM::init_loadpath();
    /// VM::require("enc/encdb");
    /// VM::require("enc/trans/transdb");
    ///
    /// let mut converter = EncodingConverter::new("", "", 0).unwrap();
    /// converter.decorate_at_first("universal_newline").unwrap();
    ///
    /// assert_eq!(converter.convert(&RString::new_utf8("a\r\nb")).unwrap().to_str(), "a\nb");
    /// ```
    pub fn decorate_at_first(&mut self, name: &str) -> Result<(), AnyException> {
        self.decorate(name, true)
    }

    fn decorate(&mut self, name: &str, first: bool) -> Result<(), AnyException> {
        if encoding::econv_decorate(self.converter, name, first) {
            Ok(())
        } else {
            let message = format!("cannot add the decorator {}", name);

            Err(AnyException::new("ArgumentError", Some(&message)))
        }
    }

    /// Removes the newline decorators, so newlines pass through unchanged
    /// (`rb_econv_binmode`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{EncodingConverter, RString, VM};
    /// # VM::init();
    ///
    /// let mut converter =
    ///     EncodingConverter::new("", "", EncodingConverter::CRLF_NEWLINE_DECORATOR).unwrap();
    /// converter.binmode();
    ///
    /// assert_eq!(converter.convert(&RString::new_utf8("a\nb")).unwrap().to_str(), "a\nb");
    /// ```
    pub fn binmode(&mut self) {
        encoding::econv_binmode(self.converter)
    }

    /// Queues `text` to be written to the output before what is converted
    /// next, converting it to the destination encoding
    /// (`rb_econv_insert_output`). Returns the
    /// `Encoding::UndefinedConversionError` if it cannot be converted.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{EncodingConverter, RString, VM};
    /// # VM::init();
    /// # VM::init_loadpath();
    /// VM::require("enc/encdb");
    /// VM::require("enc/trans/transdb");
    ///
    /// let mut converter = EncodingConverter::new("UTF-8", "ISO-8859-1", 0).unwrap();
    /// converter.insert_output(&RString::new_utf8("é:")).unwrap();
    ///
    /// assert_eq!(converter.convert(&RString::new_utf8("x")).unwrap().to_bytes_unchecked(), b"\xE9:x");
    /// ```
    pub fn insert_output(&mut self, text: &RString) -> Result<(), AnyException> {
        let enc_name = text.encoding().name();

        if encoding::econv_insert_output(self.converter, text.to_bytes_unchecked(), &enc_name) {
            Ok(())
        } else {
            Err(AnyException::new(
                "Encoding::UndefinedConversionError",
                Some("inserted text cannot be converted"),
            ))
        }
    }

    /// Returns the name of the encoding text given to
    /// [`insert_output`](#method.insert_output) is converted to
    /// (`rb_econv_encoding_to_insert_output`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{EncodingConverter, VM};
    /// # VM::init();
    /// # VM::init_loadpath();
    /// VM::require("enc/encdb");
    /// VM::require("enc/trans/transdb");
    ///
    /// let converter = EncodingConverter::new("UTF-8", "ISO-8859-1", 0).unwrap();
    ///
    /// assert_eq!(converter.insert_output_encoding(), "ISO-8859-1");
    /// ```
    pub fn insert_output_encoding(&self) -> String {
        encoding::econv_insert_output_encoding(self.converter)
    }

    /// Takes back the bytes the converter read past an error, so they can
    /// be handled again (Ruby's `Encoding::Converter#putback`,
    /// `rb_econv_putbackable` and `rb_econv_putback`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{ConversionResult, EncodingConverter, VM};
    /// # VM::init();
    /// # VM::init_loadpath();
    /// VM::require("enc/encdb");
    /// VM::require("enc/trans/transdb");
    ///
    /// let mut converter = EncodingConverter::new("UTF-16LE", "UTF-8", 0).unwrap();
    /// let mut output = [0u8; 8];
    ///
    /// // A lone high surrogate followed by "a".
    /// let (result, _, _) = converter.convert_buffer(b"\x00\xD8a\x00", &mut output, 0);
    ///
    /// assert_eq!(result, ConversionResult::InvalidByteSequence);
    /// assert_eq!(converter.putback(), b"a\x00");
    /// assert!(converter.putback().is_empty());
    /// ```
    pub fn putback(&mut self) -> Vec<u8> {
        encoding::econv_putback(self.converter)
    }

    /// Returns the destination encoding, if Ruby knows it.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, EncodingConverter, VM};
    /// # VM::init();
    ///
    /// let converter = EncodingConverter::new("UTF-8", "US-ASCII", 0).unwrap();
    ///
    /// assert_eq!(converter.destination_encoding(), Some(Encoding::us_ascii()));
    /// ```
    pub fn destination_encoding(&self) -> Option<Encoding> {
        Encoding::find(self.destination.to_str().unwrap_or("")).ok()
    }
}

impl Drop for EncodingConverter {
    fn drop(&mut self) {
        encoding::econv_close(self.converter);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ConversionResult, Encoding, EncodingConverter, EncodingSupport, Exception, Hash, Object,
        RString, Symbol, VM,
    };

    fn load_transcoders() {
        VM::init_loadpath();
        VM::require("enc/encdb");
        VM::require("enc/trans/transdb");
    }

    #[test]
    fn test_converter_streaming() {
        crate::on_ruby_thread(|| {
            load_transcoders();

            let mut converter = EncodingConverter::new("UTF-8", "EUC-JP", 0).unwrap();
            let bytes = "日本語".as_bytes();

            // A character split across calls is kept for the next one.
            let first = converter.convert_bytes(&bytes[..4]).unwrap();
            let second = converter.convert_bytes(&bytes[4..]).unwrap();
            let rest = converter.finish().unwrap();

            let mut output = first.to_bytes_unchecked().to_vec();
            output.extend_from_slice(second.to_bytes_unchecked());
            output.extend_from_slice(rest.to_bytes_unchecked());

            let expected = VM::eval("'日本語'.force_encoding('UTF-8').encode('EUC-JP').b").unwrap();
            let expected = expected.try_convert_to::<RString>().unwrap();
            assert_eq!(&output[..], expected.to_bytes_unchecked());
            assert_eq!(first.encoding().name(), "EUC-JP");
            assert_eq!(first.bytesize(), 2);
        });
    }

    #[test]
    fn test_converter_errors_and_recovery() {
        crate::on_ruby_thread(|| {
            load_transcoders();

            let mut converter = EncodingConverter::new("UTF-8", "ISO-8859-1", 0).unwrap();

            let error = converter.convert(&RString::new_utf8("a🦀")).unwrap_err();
            assert!(crate::Class::from_existing("Encoding")
                .get_nested_class("UndefinedConversionError")
                .case_equals(&error));
            assert!(converter.last_error().is_some());

            // The converter keeps working after an error.
            let ok = converter.convert(&RString::new_utf8("é")).unwrap();
            assert_eq!(ok.to_bytes_unchecked(), b"\xE9");
            assert!(converter.last_error().is_none());

            assert!(EncodingConverter::new("NO-SUCH", "UTF-8", 0).is_err());

            let mut bad_options = Hash::new();
            bad_options.store(Symbol::new("invalid"), Symbol::new("bogus"));
            assert!(EncodingConverter::with_options("UTF-8", "UTF-16LE", &bad_options).is_err());

            let mut options = Hash::new();
            options.store(Symbol::new("xml"), Symbol::new("attr"));
            let mut xml = EncodingConverter::with_options("", "", &options).unwrap();
            assert_eq!(
                xml.convert(&RString::new_utf8("a\"b")).unwrap().to_str(),
                "\"a&quot;b"
            );
            assert_eq!(xml.finish().unwrap().to_str(), "\"");

            let mut hex =
                EncodingConverter::new("UTF-8", "US-ASCII", EncodingConverter::UNDEF_HEX_CHARREF)
                    .unwrap();
            assert_eq!(
                hex.convert(&RString::new_utf8("é")).unwrap().to_str(),
                "&#xE9;"
            );
        });
    }

    #[test]
    fn test_converter_buffers() {
        crate::on_ruby_thread(|| {
            load_transcoders();

            let mut converter = EncodingConverter::new("UTF-8", "UTF-16BE", 0).unwrap();

            // An empty source finishes the conversion.
            let mut output = [0u8; 2];
            let (result, read, written) = converter.convert_buffer(b"", &mut output, 0);
            assert_eq!(result, ConversionResult::Finished);
            assert_eq!((read, written), (0, 0));

            let mut converter = EncodingConverter::new("UTF-8", "UTF-16BE", 0).unwrap();
            let (result, read, written) = converter.convert_buffer(
                b"ab\xE6",
                &mut [0u8; 16],
                EncodingConverter::PARTIAL_INPUT,
            );
            assert_eq!(result, ConversionResult::SourceBufferEmpty);
            assert_eq!((read, written), (3, 4));

            let mut empty: [u8; 0] = [];
            let (result, _, written) = converter.convert_buffer(b"\x97\xA5", &mut empty, 0);
            assert_eq!(result, ConversionResult::DestinationBufferFull);
            assert_eq!(written, 0);

            assert!(EncodingConverter::has_path("UTF-8", "Shift_JIS"));
            assert_eq!(
                EncodingConverter::ascii_compatible_encoding("UTF-32LE").as_deref(),
                Some("UTF-8")
            );
            assert_eq!(
                EncodingConverter::new("UTF-8", "UTF-16LE", 0)
                    .unwrap()
                    .destination_encoding(),
                Encoding::find("UTF-16LE").ok()
            );

            let mut started = EncodingConverter::new("", "", 0).unwrap();
            started.convert(&RString::new_utf8("x")).unwrap();
            let error = started.decorate_at_first("crlf_newline").unwrap_err();
            assert!(error.message().contains("crlf_newline"));
        });
    }
}
