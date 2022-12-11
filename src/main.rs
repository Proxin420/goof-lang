extern crate termion;

mod lexer;
mod interpreter;
mod typechecker;

use std::env;
use std::process;

pub struct Args {
    debug: bool,
}

fn main() {
    let (argv, argc) = (env::args().collect::<Vec<String>>(), env::args().count());
    if argc < 2 {
        println!(
            "{}\n{}\n{}",
            format!("{}Usage{}: put <File> [Options]",
                    termion::color::Fg(termion::color::Yellow),
                    termion::color::Fg(termion::color::Reset)
                   ),
            format!("  Options:"),
            format!("    {}-d{}: Debug mode",
                    termion::color::Fg(termion::color::Green),
                    termion::color::Fg(termion::color::Reset),
                   ),
        );
        process::exit(1);
    }

    let mut args = Args {
        debug: false,
    };

    let mut ctr = 2;
    while ctr < argv.len() {
        if argv[ctr] == "-d" {
            args.debug = true;
        }
        ctr += 1;
    }

    let tokens = lexer::Lexer::tokenize(&argv[1]);
    println!("{:?}", tokens);
    interpreter::Interpreter::run(tokens, args);
}


