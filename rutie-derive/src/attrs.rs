//! `#[ruby(...)]` helper attributes, taken off the items they annotate.

use syn::{meta::ParseNestedMeta, Attribute, Error, LitStr, Result};

/// The options one `#[ruby(...)]` list may hold; each item kind accepts a
/// subset, checked with [`RubyOptions::only`].
#[derive(Default)]
pub struct RubyOptions {
    pub name: Option<LitStr>,
    pub path: Option<LitStr>,
    pub skip: bool,
    pub constructor: bool,
    pub private: bool,
    pub protected: bool,
    pub module_function: bool,
    pub without_gvl: bool,
    pub keyword: bool,
    pub splat: bool,
    pub block: bool,
    pub get: bool,
    pub set: bool,
    pub no_mark: bool,
    pub base: bool,
    pub constant: bool,
    seen: Vec<(&'static str, proc_macro2::Span)>,
}

impl RubyOptions {
    /// Removes the `#[ruby(...)]` attributes from `attributes` and parses
    /// them.
    pub fn take(attributes: &mut Vec<Attribute>) -> Result<Self> {
        let mut options = RubyOptions::default();
        let mut errors: Option<Error> = None;

        attributes.retain(|attribute| {
            if !attribute.path().is_ident("ruby") {
                return true;
            }

            if let Err(error) = attribute.parse_nested_meta(|meta| options.parse(meta)) {
                match &mut errors {
                    Some(errors) => errors.combine(error),
                    None => errors = Some(error),
                }
            }

            false
        });

        match errors {
            Some(errors) => Err(errors),
            None => Ok(options),
        }
    }

    fn parse(&mut self, meta: ParseNestedMeta) -> Result<()> {
        let span = meta.path.require_ident()?.span();
        let ident = meta.path.require_ident()?.to_string();

        let flag = match ident.as_str() {
            "name" => {
                self.name = Some(meta.value()?.parse()?);
                "name"
            }
            "path" => {
                self.path = Some(meta.value()?.parse()?);
                "path"
            }
            "skip" => set(&mut self.skip, "skip"),
            "constructor" => set(&mut self.constructor, "constructor"),
            "private" => set(&mut self.private, "private"),
            "protected" => set(&mut self.protected, "protected"),
            "module_function" => set(&mut self.module_function, "module_function"),
            "without_gvl" => set(&mut self.without_gvl, "without_gvl"),
            "keyword" => set(&mut self.keyword, "keyword"),
            "splat" => set(&mut self.splat, "splat"),
            "block" => set(&mut self.block, "block"),
            "get" => set(&mut self.get, "get"),
            "set" => set(&mut self.set, "set"),
            "no_mark" => set(&mut self.no_mark, "no_mark"),
            "base" => set(&mut self.base, "base"),
            "constant" => set(&mut self.constant, "constant"),
            _ => return Err(meta.error(format!("unknown `#[ruby({})]` option", ident))),
        };

        self.seen.push((flag, span));

        Ok(())
    }

    /// Fails if an option outside `allowed` was given, naming where it went.
    pub fn only(&self, allowed: &[&str], place: &str) -> Result<()> {
        for (flag, span) in &self.seen {
            if !allowed.contains(flag) {
                return Err(Error::new(
                    *span,
                    format!(
                        "`#[ruby({})]` doesn't apply to {} (expected one of: {})",
                        flag,
                        place,
                        allowed.join(", ")
                    ),
                ));
            }
        }

        Ok(())
    }
}

fn set(flag: &mut bool, name: &'static str) -> &'static str {
    *flag = true;
    name
}

/// Collects errors so a macro can report all of them at once.
#[derive(Default)]
pub struct Errors(Option<Error>);

impl Errors {
    pub fn push(&mut self, error: Error) {
        match &mut self.0 {
            Some(errors) => errors.combine(error),
            None => self.0 = Some(error),
        }
    }

    pub fn check<T>(&mut self, result: Result<T>) -> Option<T> {
        match result {
            Ok(value) => Some(value),
            Err(error) => {
                self.push(error);
                None
            }
        }
    }

    pub fn finish(self) -> Result<()> {
        match self.0 {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}
