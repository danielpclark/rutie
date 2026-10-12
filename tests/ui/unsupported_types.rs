use rutie::{rubyclass, rubymethods};

pub struct Plain;

#[rubyclass]
pub struct Counter;

#[rubymethods]
impl Counter {
    pub fn take(&self, plain: &Plain) {
        let _ = plain;
    }

    pub fn give(&self) -> Plain {
        Plain
    }

    pub fn change(&self, number: &mut i64) {
        *number += 1;
    }
}

fn main() {}
