extern crate rutie;

use rutie::VM;
use std::{env, process};

fn main() {
    VM::init();
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        if let Err(e) = VM::eval(&args[1]) {
            println!("{}", e);
            process::exit(1);
        }

        // Shut Ruby down, which runs `at_exit` handlers and flushes
        // `$stdout` (buffered when it is not a terminal).
        process::exit(unsafe { VM::cleanup() });
    } else {
        eprintln!(r#"Usage: eval "puts 'Put ruby code to be evaluated in a string after eval.' ""#);
        process::exit(1);
    }
}
