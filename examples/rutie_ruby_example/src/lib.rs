use rutie::{class, methods, Class, Object, RString, VM};

class!(RutieExample);

methods!(
    RutieExample,
    _rtself,
    fn pub_reverse(input: RString) -> RString {
        let ruby_string = input.map_err(VM::raise_ex).unwrap();

        RString::new_utf8(&ruby_string.to_string().chars().rev().collect::<String>())
    }
);

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn Init_rutie_ruby_example() {
    Class::new("RutieExample", None).define(|klass| {
        klass.def_self("reverse", pub_reverse);
    });
}
