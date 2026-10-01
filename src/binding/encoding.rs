use crate::{
    binding::global::RubySpecialConsts,
    binding::vm,
    rubysys::{
        encoding::{self, EconvResult, RbEconv},
        string,
    },
    types::{
        c_char, c_int, c_long, size_t, EncodingIndex, EncodingType, InternalValue, Value, ValueType,
    },
    util,
};
use std::{
    ffi::{CStr, CString},
    ptr,
};

pub fn default_external() -> Value {
    unsafe { encoding::rb_enc_default_external() }
}

pub fn default_internal() -> Value {
    unsafe { encoding::rb_enc_default_internal() }
}

pub fn force_encoding(s: Value, enc: Value) -> Value {
    unsafe { encoding::rb_enc_associate(s, encoding::rb_to_encoding(enc)) }
}

pub fn coderange_clear(obj: Value) {
    unsafe { encoding::coderange_clear(obj) }
}

// best str1/str2 encoding or nil if incompatible
pub fn compatible_encoding(str1: Value, str2: Value) -> Value {
    unsafe { encoding::rb_enc_from_encoding(encoding::rb_enc_compatible(str1, str2)) }
}

pub fn is_compatible_encoding(str1: Value, str2: Value) -> bool {
    compatible_encoding(str1, str2).ty() != ValueType::Nil
}

pub fn from_encoding_index(idx: EncodingIndex) -> Value {
    unsafe { encoding::rb_enc_from_encoding(encoding::rb_enc_from_index(idx)) }
}

pub fn usascii_encoding() -> Value {
    unsafe { from_encoding_index(encoding::rb_usascii_encindex()) }
}

pub fn utf8_encoding() -> Value {
    unsafe { from_encoding_index(encoding::rb_utf8_encindex()) }
}

pub fn enc_get_index(s: Value) -> EncodingIndex {
    let idx = unsafe { encoding::rb_enc_get_index(s) };

    idx
}

pub fn find_encoding_index(name: &str) -> EncodingIndex {
    let cstr = CString::new(name).unwrap();
    let idx = unsafe { encoding::rb_enc_find_index(cstr.as_ptr()) };

    idx
}

pub fn encode(str: Value, to: Value, ecflags: c_int, ecopts: Value) -> Value {
    unsafe { encoding::rb_str_encode(str, to, ecflags, ecopts) }
}

// Ruby writes the prepared options (what `rb_str_encode` expects as
// `ecopts`) through `opts`.
pub fn econv_prepare_opts(opthash: Value, opts: &mut Value) -> c_int {
    unsafe { encoding::rb_econv_prepare_opts(opthash, opts) }
}

// ptr - pointer for current point in string starting from the beginning
// end - pointer for the end of the string
// len_p - a mutable integer pointer for Ruby to give us how much we need to add on to `ptr`
// enc - the encoding the codepoints will be based on
pub fn next_codepoint(
    ptr: *const c_char,
    end: *const c_char,
    len_p: *mut c_int,
    enc: Value,
) -> usize {
    // `rb_enc_codepoint_len` returns an `unsigned int`.
    unsafe {
        encoding::rb_enc_codepoint_len(ptr, end, len_p, encoding::rb_to_encoding(enc)) as usize
    }
}

pub fn ascii_8bit_encoding() -> Value {
    unsafe { encoding::rb_enc_from_encoding(encoding::rb_ascii8bit_encoding()) }
}

pub fn locale_encoding() -> Value {
    unsafe { encoding::rb_enc_from_encoding(encoding::rb_locale_encoding()) }
}

pub fn filesystem_encoding() -> Value {
    unsafe { encoding::rb_enc_from_encoding(encoding::rb_filesystem_encoding()) }
}

// The `Encoding` of `object`, or `nil` when it has none.
pub fn encoding_of(object: Value) -> Value {
    unsafe {
        let encoding = encoding::rb_enc_get(object);

        if encoding.is_null() {
            Value::from(RubySpecialConsts::Nil as InternalValue)
        } else {
            encoding::rb_enc_from_encoding(encoding)
        }
    }
}

pub fn encoding_index(enc: Value) -> EncodingIndex {
    unsafe { encoding::rb_to_encoding_index(enc) }
}

// Raises `RangeError` for a code point the encoding cannot represent.
pub fn chr(code: u32, enc: Value) -> Value {
    unsafe { encoding::rb_enc_uint_chr(code, encoding::rb_to_encoding(enc)) }
}

// Appends `bytes` (in `enc`) to `string`, converting as `String#<<` does;
// raises `Encoding::CompatibilityError` when they cannot be combined.
pub fn concat_bytes(string: Value, bytes: &[u8], enc: Value) -> Value {
    unsafe {
        encoding::rb_enc_str_buf_cat(
            string,
            bytes.as_ptr() as *const c_char,
            bytes.len() as c_long,
            encoding::rb_to_encoding(enc),
        )
    }
}

// Raises `ArgumentError` when the name is taken.
pub fn define_dummy(name: &str) -> Value {
    let name = util::str_to_cstring(name);

    unsafe { from_encoding_index(encoding::rb_define_dummy_encoding(name.as_ptr())) }
}

// Raises `ArgumentError` when the alias is taken; -1 for an unknown `original`.
pub fn alias(alias: &str, original: &str) -> c_int {
    let (alias, original) = (util::str_to_cstring(alias), util::str_to_cstring(original));

    unsafe { encoding::rb_enc_alias(alias.as_ptr(), original.as_ptr()) }
}

pub fn is_capable(object: Value) -> bool {
    util::c_int_to_bool(unsafe { encoding::rb_enc_capable(object) })
}

pub fn is_dummy(enc: Value) -> bool {
    util::c_int_to_bool(unsafe { encoding::rb_enc_dummy_p(encoding::rb_to_encoding(enc)) })
}

pub fn is_unicode(enc: Value) -> bool {
    util::c_int_to_bool(unsafe { encoding::rb_enc_unicode_p(encoding::rb_to_encoding(enc)) })
}

pub fn locale_charmap() -> Value {
    unsafe { encoding::rb_locale_charmap(Value::from(RubySpecialConsts::Nil as InternalValue)) }
}

pub fn strlen(bytes: &[u8], enc: Value) -> usize {
    let range = bytes.as_ptr_range();

    unsafe {
        string::rb_enc_strlen(
            range.start as *const c_char,
            range.end as *const c_char,
            encoding::rb_to_encoding(enc),
        ) as usize
    }
}

pub fn memsearch(needle: &[u8], haystack: &[u8], enc: Value) -> Option<usize> {
    // Ruby's search reads a few bytes past the end of both (Ruby strings
    // have a terminator there), so search copies padded with NULs.
    let padded = |bytes: &[u8]| {
        let mut copy = Vec::with_capacity(bytes.len() + 4);
        copy.extend_from_slice(bytes);
        copy.extend_from_slice(&[0; 4]);
        copy
    };
    let (needle_copy, haystack_copy) = (padded(needle), padded(haystack));

    let found = unsafe {
        string::rb_memsearch(
            needle_copy.as_ptr() as *const _,
            needle.len() as c_long,
            haystack_copy.as_ptr() as *const _,
            haystack.len() as c_long,
            encoding::rb_to_encoding(enc),
        )
    };

    if found < 0 {
        None
    } else {
        Some(found as usize)
    }
}

fn nil() -> Value {
    Value::from(RubySpecialConsts::Nil as InternalValue)
}

// A transcoder from `source` to `destination`, or the exception (such as
// `Encoding::ConverterNotFoundError`) when there is none. `options` is a
// Hash of `String#encode` options, or `nil` to use `flags`.
pub fn econv_open(
    source: &str,
    destination: &str,
    flags: c_int,
    options: Value,
) -> Result<*mut RbEconv, Value> {
    let (source, destination) = (
        util::str_to_cstring(source),
        util::str_to_cstring(destination),
    );
    let mut converter = ptr::null_mut();

    let result = vm::protect_value(|| unsafe {
        let mut ecflags = flags;

        converter = if options.is_nil() {
            encoding::rb_econv_open(source.as_ptr(), destination.as_ptr(), ecflags)
        } else {
            let mut ecopts = nil();
            ecflags = encoding::rb_econv_prepare_options(options, &mut ecopts, ecflags);

            encoding::rb_econv_open_opts(source.as_ptr(), destination.as_ptr(), ecflags, ecopts)
        };

        if converter.is_null() {
            encoding::rb_econv_open_exc(source.as_ptr(), destination.as_ptr(), ecflags)
        } else {
            nil()
        }
    });

    match result {
        Ok(_) if !converter.is_null() => Ok(converter),
        Ok(exception) | Err(exception) => Err(exception),
    }
}

pub fn econv_close(converter: *mut RbEconv) {
    unsafe { encoding::rb_econv_close(converter) }
}

// Converts `bytes`, appending to a new string in `destination` (or binary
// when Ruby does not know it). Returns the converted text, or the error
// the converter stopped at.
pub fn econv_append(
    converter: *mut RbEconv,
    bytes: &[u8],
    destination: &CStr,
    flags: c_int,
) -> Result<Value, Value> {
    let mut error = nil();

    let converted = vm::protect_value(|| unsafe {
        let result = string::rb_str_buf_new(bytes.len() as c_long);
        let enc = encoding::rb_enc_find(destination.as_ptr());

        if !enc.is_null() {
            encoding::rb_enc_associate(result, enc);
        }

        encoding::rb_econv_append(
            converter,
            bytes.as_ptr() as *const c_char,
            bytes.len() as c_long,
            result,
            flags,
        );
        error = encoding::rb_econv_make_exception(converter);

        result
    })?;

    if error.is_nil() {
        Ok(converted)
    } else {
        Err(error)
    }
}

// As `econv_append`, from a Ruby string.
pub fn econv_str_append(
    converter: *mut RbEconv,
    source: Value,
    destination: &CStr,
    flags: c_int,
) -> Result<Value, Value> {
    let mut error = nil();

    let converted = vm::protect_value(|| unsafe {
        let result = string::rb_str_buf_new(0);
        // A decorator-only converter keeps the source's encoding.
        let enc = if destination.to_bytes().is_empty() {
            encoding::rb_enc_get(source)
        } else {
            encoding::rb_enc_find(destination.as_ptr())
        };

        if !enc.is_null() {
            encoding::rb_enc_associate(result, enc);
        }

        encoding::rb_econv_str_append(converter, source, result, flags);
        error = encoding::rb_econv_make_exception(converter);

        result
    })?;

    if error.is_nil() {
        Ok(converted)
    } else {
        Err(error)
    }
}

// Converts from `source` into `destination`; returns what the converter
// stopped at and the bytes read and written.
pub fn econv_convert(
    converter: *mut RbEconv,
    source: &[u8],
    destination: &mut [u8],
    flags: c_int,
) -> (EconvResult, usize, usize) {
    let source_range = source.as_ptr_range();
    let destination_range = destination.as_mut_ptr_range();
    let mut source_ptr = source_range.start;
    let mut destination_ptr = destination_range.start;

    let result = unsafe {
        encoding::rb_econv_convert(
            converter,
            &mut source_ptr,
            source_range.end,
            &mut destination_ptr,
            destination_range.end,
            flags,
        )
    };

    (
        result,
        source_ptr as usize - source_range.start as usize,
        destination_ptr as usize - destination_range.start as usize,
    )
}

// The exception for the last conversion's error, or `nil`.
pub fn econv_last_error(converter: *mut RbEconv) -> Value {
    unsafe { encoding::rb_econv_make_exception(converter) }
}

pub fn econv_set_replacement(converter: *mut RbEconv, bytes: &[u8], enc_name: &str) -> bool {
    let enc_name = util::str_to_cstring(enc_name);

    unsafe {
        encoding::rb_econv_set_replacement(
            converter,
            bytes.as_ptr(),
            bytes.len() as size_t,
            enc_name.as_ptr(),
        ) == 0
    }
}

pub fn econv_insert_output(converter: *mut RbEconv, bytes: &[u8], enc_name: &str) -> bool {
    let enc_name = util::str_to_cstring(enc_name);

    unsafe {
        encoding::rb_econv_insert_output(
            converter,
            bytes.as_ptr(),
            bytes.len() as size_t,
            enc_name.as_ptr(),
        ) == 0
    }
}

pub fn econv_insert_output_encoding(converter: *mut RbEconv) -> String {
    unsafe { util::cstr_to_string(encoding::rb_econv_encoding_to_insert_output(converter)) }
}

pub fn econv_decorate(converter: *mut RbEconv, decorator: &str, first: bool) -> bool {
    let decorator = util::str_to_cstring(decorator);

    let result = unsafe {
        if first {
            encoding::rb_econv_decorate_at_first(converter, decorator.as_ptr())
        } else {
            encoding::rb_econv_decorate_at_last(converter, decorator.as_ptr())
        }
    };

    result >= 0
}

pub fn econv_binmode(converter: *mut RbEconv) {
    unsafe { encoding::rb_econv_binmode(converter) }
}

pub fn econv_putback(converter: *mut RbEconv) -> Vec<u8> {
    unsafe {
        let count = encoding::rb_econv_putbackable(converter).max(0);
        let mut bytes = vec![0; count as usize];

        encoding::rb_econv_putback(converter, bytes.as_mut_ptr(), count);

        bytes
    }
}

pub fn econv_has_path(source: &str, destination: &str) -> bool {
    let (source, destination) = (
        util::str_to_cstring(source),
        util::str_to_cstring(destination),
    );

    util::c_int_to_bool(unsafe {
        encoding::rb_econv_has_convpath_p(source.as_ptr(), destination.as_ptr())
    })
}

pub fn econv_asciicompat_encoding(name: &str) -> Option<String> {
    let name = util::str_to_cstring(name);

    unsafe {
        let found = encoding::rb_econv_asciicompat_encoding(name.as_ptr());

        if found.is_null() {
            None
        } else {
            Some(util::cstr_to_string(found))
        }
    }
}
