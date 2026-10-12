use rutie::{rubyclass, rubymethods};

#[rubyclass]
pub struct Counter {
    count: i64,
}

#[rubymethods]
impl Counter {
    pub fn consume(self) -> i64 {
        self.count
    }
}

fn main() {}
