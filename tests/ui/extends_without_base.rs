use rutie::rubyclass;

#[rubyclass]
pub struct Shape;

#[rubyclass(extends = Shape)]
pub struct Square {
    length: f64,
}

fn main() {}
