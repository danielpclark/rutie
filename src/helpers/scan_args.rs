use crate::{AnyObject, Array, Hash, Proc};

/// Arguments of a Ruby method call split up according to an `rb_scan_args`
/// format; returned by [`VM::scan_args`](../struct.VM.html#method.scan_args).
#[derive(Debug)]
pub struct ScannedArgs {
    /// Leading mandatory arguments, in order.
    pub required: Vec<AnyObject>,
    /// Optional arguments, in order; `None` for each one that was not given.
    pub optional: Vec<Option<AnyObject>>,
    /// The remaining arguments when the format has `*`, otherwise `None`.
    pub splat: Option<Array>,
    /// Trailing mandatory arguments (after the splat), in order.
    pub post: Vec<AnyObject>,
    /// The keyword arguments when the format has `:` and keywords were
    /// passed, otherwise `None`.
    pub keywords: Option<Hash>,
    /// The block when the format has `&` and a block was given, otherwise `None`.
    pub block: Option<Proc>,
}

/// Keyword arguments looked up by name; returned by
/// [`VM::get_kwargs`](../struct.VM.html#method.get_kwargs).
#[derive(Debug)]
pub struct KeywordArgs {
    /// Values of the required keywords, in the order they were asked for.
    pub required: Vec<AnyObject>,
    /// Values of the optional keywords, in the order they were asked for;
    /// `None` for each one that was not given.
    pub optional: Vec<Option<AnyObject>>,
    /// The keywords that were not asked for, when extra keywords are allowed.
    pub rest: Option<Hash>,
}

/// The parts of an `rb_scan_args` format string.
#[derive(Debug, Default, PartialEq)]
pub(crate) struct ScanArgsFormat {
    pub required: usize,
    pub optional: usize,
    pub splat: bool,
    pub post: usize,
    pub keywords: bool,
    pub block: bool,
}

impl ScanArgsFormat {
    /// Parses `[required[optional]][*][post][:][&]`, the same grammar
    /// `rb_scan_args` accepts on Ruby 2.5 to 2.7, where each count is a
    /// single digit.
    ///
    /// `rb_scan_args` aborts the process (`rb_fatal`) on a format it cannot
    /// parse, so every format is checked here before it reaches Ruby.
    pub fn parse(format: &str) -> Result<Self, String> {
        let bytes = format.as_bytes();
        let mut spec = ScanArgsFormat::default();
        let mut i = 0;

        let digit = |i: usize| -> Option<usize> {
            bytes
                .get(i)
                .filter(|b| b.is_ascii_digit())
                .map(|b| (b - b'0') as usize)
        };

        if let Some(required) = digit(i) {
            spec.required = required;
            i += 1;

            if let Some(optional) = digit(i) {
                spec.optional = optional;
                i += 1;
            }
        }

        if bytes.get(i) == Some(&b'*') {
            spec.splat = true;
            i += 1;
        }

        if let Some(post) = digit(i) {
            spec.post = post;
            i += 1;
        }

        if bytes.get(i) == Some(&b':') {
            spec.keywords = true;
            i += 1;
        }

        if bytes.get(i) == Some(&b'&') {
            spec.block = true;
            i += 1;
        }

        if i != bytes.len() {
            return Err(format!("bad scan arg format: {}", format));
        }

        Ok(spec)
    }

    /// The number of `VALUE *` outputs `rb_scan_args` writes for this format.
    pub fn variables(&self) -> usize {
        self.required
            + self.optional
            + self.splat as usize
            + self.post
            + self.keywords as usize
            + self.block as usize
    }
}

#[cfg(test)]
mod tests {
    use super::ScanArgsFormat;

    #[test]
    fn test_scan_args_format_parse() {
        let spec = ScanArgsFormat::parse("21*1:&").unwrap();

        assert_eq!(
            spec,
            ScanArgsFormat {
                required: 2,
                optional: 1,
                splat: true,
                post: 1,
                keywords: true,
                block: true,
            }
        );
        assert_eq!(spec.variables(), 7);

        assert_eq!(
            ScanArgsFormat::parse("").unwrap(),
            ScanArgsFormat::default()
        );
        assert_eq!(ScanArgsFormat::parse("*").unwrap().variables(), 1);
        assert_eq!(ScanArgsFormat::parse("*1").unwrap().post, 1);
        assert_eq!(ScanArgsFormat::parse("1&").unwrap().variables(), 2);

        assert_eq!(ScanArgsFormat::parse("123").unwrap().post, 3);

        for bad in &["x", "1x", "&:", "**", "1234", "1:*", "1 ", "-1"] {
            assert!(
                ScanArgsFormat::parse(bad).is_err(),
                "{} should be rejected",
                bad
            );
        }
    }
}
