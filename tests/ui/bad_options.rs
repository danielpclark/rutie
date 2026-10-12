use rutie::{rubyclass, rubymethods};

#[rubyclass]
pub struct Counter {
    #[ruby(constructor)]
    count: i64,
}

#[rubymethods]
impl Counter {
    #[ruby(nope)]
    pub fn first(&self) -> i64 {
        self.count
    }

    #[ruby(get)]
    pub fn second(&self) -> i64 {
        self.count
    }
}

fn main() {}
