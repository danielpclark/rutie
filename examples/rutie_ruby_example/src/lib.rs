#[macro_use]
extern crate rutie;

use rutie::{Class, Object, RString, VM, Marshal, AnyObject, NilClass};

class!(RutieExample);

methods!(
    RutieExample,
    _rtself,
    fn pub_reverse(input: RString) -> RString {
        let ruby_string = input.map_err(|e| VM::raise_ex(e)).unwrap();

        RString::new_utf8(&ruby_string.to_string().chars().rev().collect::<String>())
    }

    fn pub_dump(input: AnyObject) -> RString {
        Marshal::dump(input.unwrap(), NilClass::new().into())
    }

    fn pub_load(input: RString) -> AnyObject {
        Marshal::load(input.unwrap())
    }
);

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn Init_rutie_ruby_example() {
    Class::new("RutieExample", None).define(|klass| {
        klass.def_self("reverse", pub_reverse);
        klass.def_self("dump", pub_dump);
        klass.def_self("load", pub_load);
    });
}
