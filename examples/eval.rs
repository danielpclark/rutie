extern crate rutie;

use rutie::VM;
use std::{env, process};

fn main() {
    VM::init();
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        match VM::eval(&args[1]) {
            Ok(_) => (),
            Err(e) => {
                println!("{}", e);
                process::exit(1);
            },
        }
    } else {
        eprintln!(r#"Usage: eval "puts 'Put ruby code to be evaluated in a string after eval.' ""#);
        process::exit(1);
    }
}
