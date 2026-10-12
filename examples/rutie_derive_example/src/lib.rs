//! A Ruby extension written with Rutie's `derive` feature: a `TextStats`
//! module with a `Document` class, an exception hierarchy and an RBS
//! signature, all declared on ordinary Rust.

use rutie::rubymodule;

#[rubymodule(path = "TextStats")]
mod text_stats {
    use rutie::{rubyclass, rubyerror, rubymethods, Integer, Object, Proc, RString};
    use std::{collections::HashMap, fmt};

    #[ruby(constant)]
    const VERSION: &str = "0.1.0";

    /// `TextStats::Error`, and `TextStats::EmptyError` under it.
    #[rubyerror(path = "TextStats::Error")]
    #[derive(Debug)]
    pub enum StatsError {
        #[ruby(path = "TextStats::EmptyError")]
        Empty,
        TooShort(usize),
    }

    impl fmt::Display for StatsError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                StatsError::Empty => write!(f, "the text has no words"),
                StatsError::TooShort(length) => write!(f, "top must be at least 1, got {}", length),
            }
        }
    }

    /// `TextStats.words(text)`: the words of `text`, lowercased.
    pub fn words(text: &str) -> Vec<String> {
        text.split(|c: char| !c.is_alphanumeric())
            .filter(|word| !word.is_empty())
            .map(str::to_lowercase)
            .collect()
    }

    /// `TextStats.rbs`: this extension's RBS signature.
    #[ruby(name = "rbs")]
    pub fn signature() -> String {
        rbs()
    }

    #[rubyclass(path = "TextStats::Document")]
    #[derive(Clone)]
    pub struct Document {
        #[ruby(get, set)]
        title: String,
        words: Vec<String>,
    }

    #[rubymethods]
    impl Document {
        /// `Document.new(text, title: "untitled")`
        pub fn new(text: &str, #[ruby(keyword)] title: Option<String>) -> Result<Self, StatsError> {
            let words = words(text);

            if words.is_empty() {
                return Err(StatsError::Empty);
            }

            Ok(Document {
                title: title.unwrap_or_else(|| "untitled".to_string()),
                words,
            })
        }

        pub fn word_count(&self) -> usize {
            self.words.len()
        }

        /// The most frequent words and their counts, most frequent first.
        pub fn frequencies(
            &self,
            #[ruby(keyword)] top: Option<usize>,
        ) -> Result<Vec<(String, usize)>, StatsError> {
            let top = top.unwrap_or(10);

            if top == 0 {
                return Err(StatsError::TooShort(top));
            }

            let mut counts: HashMap<&str, usize> = HashMap::new();

            for word in &self.words {
                *counts.entry(word).or_default() += 1;
            }

            let mut counts: Vec<(String, usize)> = counts
                .into_iter()
                .map(|(word, count)| (word.to_string(), count))
                .collect();
            counts.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
            counts.truncate(top);

            Ok(counts)
        }

        /// Yields each word and its position; returns the number of words.
        pub fn each_word(&self, #[ruby(block)] block: Proc) -> usize {
            for (index, word) in self.words.iter().enumerate() {
                block.call(&[
                    RString::new_utf8(word).to_any_object(),
                    Integer::from(index as u64).to_any_object(),
                ]);
            }

            self.words.len()
        }

        /// Runs with the GVL released, so other Ruby threads keep running.
        #[ruby(without_gvl)]
        pub fn longest_word(&self) -> Option<String> {
            self.words
                .iter()
                .max_by_key(|word| word.chars().count())
                .cloned()
        }

        #[ruby(name = "include?")]
        pub fn includes(&self, word: &str) -> bool {
            let word = word.to_lowercase();

            self.words.iter().any(|candidate| *candidate == word)
        }
    }
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn Init_rutie_derive_example() {
    text_stats::define_ruby();
}
