#[macro_use]
extern crate rutie;

use rutie::{AnyObject, Class, Fixnum, NilClass, Object, RString, Thread};

#[cfg(unix)]
use std::os::unix::{io::AsRawFd, net::UnixStream};
use std::sync::mpsc;

class!(RutieExample);

methods! {
    RutieExample,
    _rtself,
    fn heap_allocated_returning_input() -> RString {
        let input = "Object".to_string();
        let handler = move || {
            assert_eq!("Object", &input);
            input.clone()
        };
        let ret = Thread::call_without_gvl(handler, Some(|| {}));
        RString::new_utf8(&ret)
    }

    fn stack_allocated_returning_input() -> Fixnum {
        let input = 42;
        let handler = move || {
            assert_eq!(42, input);
            input
        };
        let ret = Thread::call_without_gvl(handler, Some(|| {}));
        Fixnum::new(ret)
    }

    fn heap_allocated_returning_from_closure(n: Fixnum) -> Fixnum {
        let input = n.unwrap().to_i64() as u32;
        let input2 = "Object".to_string();
        let handler = move || {
            assert_eq!(5, input);
            assert_eq!("Object", &input2);
            fibonacci(input)
        };
        let ret = Thread::call_without_gvl(handler, Some(|| {}));
        Fixnum::new(ret as i64)
    }

    fn stack_allocated_returning_from_closure(n: Fixnum) -> RString {
        let input = n.unwrap().to_i64() as u32;
        let handler = move || {
            assert_eq!(5, input);
            fibonacci(input).to_string()
        };
        let ret = Thread::call_without_gvl(handler, Some(|| {}));
        RString::new_utf8(&ret)
    }

    fn call_ruby_in_call_with_gvl() -> AnyObject {
        let class = "Object".to_string();
        let b = Thread::call_without_gvl(
            move || {
                let _n = fibonacci(5);
                let class = class.clone();
                Thread::call_with_gvl(move || {
                    let ruby_class = Class::from_existing(&class);
                    unsafe { ruby_class.send("name", &[]) }
                })
            },
            Some(|| {}),
        );
        b
    }

    fn create_thread() -> AnyObject {
        let (tx, rx) = mpsc::channel();
        Thread::new(move || {
            let ruby_class = Class::from_existing("Object");
            let name = unsafe { ruby_class.send("name", &[]) };
            tx.send(name).unwrap();
            NilClass::new()
        });
        let (unix_socket, _) = UnixStream::pair().unwrap();
        loop {
            if let Ok(ret) = rx.try_recv() {
                return ret;
            } else {
                Thread::wait_fd(unix_socket.as_raw_fd());
            }
        }
    }
}

fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 1,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn Init_rutie_ruby_gvl_example() {
    Class::new("RutieExample", None).define(|klass| {
        klass.def_self(
            "stack_allocated_returning_input",
            stack_allocated_returning_input,
        );
        klass.def_self(
            "stack_allocated_returning_from_closure",
            stack_allocated_returning_from_closure,
        );
        klass.def_self(
            "heap_allocated_returning_input",
            heap_allocated_returning_input,
        );
        klass.def_self(
            "heap_allocated_returning_from_closure",
            heap_allocated_returning_from_closure,
        );
        klass.def_self("call_ruby_in_call_with_gvl", call_ruby_in_call_with_gvl);
        klass.def_self("create_thread", create_thread);
    });
}
