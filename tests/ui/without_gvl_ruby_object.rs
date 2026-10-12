use rutie::{rubyclass, rubymethods, RString};

#[rubyclass]
pub struct Worker;

#[rubymethods]
impl Worker {
    #[ruby(without_gvl)]
    pub fn length(&self, text: &RString) -> usize {
        text.to_str().len()
    }
}

fn main() {}
