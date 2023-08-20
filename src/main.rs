use nonograms::{Parser,Solver};
use std::env::args;
use std::fs;
use std::io;
use std::iter::Iterator;

fn go(mut r: impl io::Read) {
    let parser = Parser::new().parse(&mut r);
    match parser {
        Ok(mut n) => {
            println!("Dimensions (w×h) = {}×{}", n.width(), n.height());
            Solver::new(&mut n).solve();
            println!("{}", n.as_text());
        }
        Err(e) => {
            println!("Error: {}", e)
        }
    };
}

fn main() {
    let args = args().skip(1);
    if args.len() == 0 {
        go(std::io::stdin());
    } else {
        for (fname, fd) in args.map(|fname| (fname.clone(), fs::File::open(&fname))) {
            println!("File: {}", fname);
            match fd {
                Ok(mut fd) => go(&mut fd),
                Err(err) => eprintln!("Cannot read {}: {}", fname, err),
            }
        }
    }
}
