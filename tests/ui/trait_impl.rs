use rutie::{rubyclass, rubymethods};

#[rubyclass]
pub struct Counter;

#[rubymethods]
impl Default for Counter {
    fn default() -> Self {
        Counter
    }
}

fn main() {}
