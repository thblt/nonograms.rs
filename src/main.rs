use std::env::args;
use std::fs;
use std::io;
use std::iter::Iterator;
use nonograms::*;

fn go(mut r: impl io::Read) {
    match Parser::parse(&mut r) {
        Ok(n) => {
            println!("Dimensions (w×h) = {}×{}", n.width, n.height);
            println!("{}", n.as_text());
        },
        Err(e) => {println!("Error: {}", e) }
    };
}


fn main() {
    let args = args().skip(1);
    if args.len() == 0 {
        println!()
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
