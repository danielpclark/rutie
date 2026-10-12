use rutie::rubyclass;

#[rubyclass(path = "Generic")]
pub struct Generic<T> {
    value: T,
}

fn main() {}
