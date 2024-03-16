fn main() {}

#[cfg(test)]
mod tests {

    use rutie::{Object, RString, VM};

    fn try_it(s: &str) -> String {
        let a = RString::new_utf8(s);

        // Send returns an AnyObject type
        let b = unsafe { a.send("reverse", &[]) };

        // We must try to convert the AnyObject
        // type back to our usable type.
        match b.try_convert_to::<RString>() {
            Ok(ruby_string) => ruby_string.to_string(),
            Err(_) => "Fail!".to_string(),
        }
    }

    #[test]
    fn it_works() {
        // Rust projects must start the Ruby VM
        VM::init();

        assert_eq!("selppa", try_it("apples"));
    }
}
