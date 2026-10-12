use rutie::{rubyclass, rubymethods};

#[rubyclass]
pub struct Counter;

#[rubymethods]
impl Counter {
    pub fn add(&self, first: Option<i64>, second: i64) -> i64 {
        first.unwrap_or(0) + second
    }

    pub fn change(&self, other: Option<&mut Counter>) {
        let _ = other;
    }
}

fn main() {}
