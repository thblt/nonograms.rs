use nonograms::parser::*;
use nonograms::solver::*;
use nonograms::*;
use std::env::args;
use std::fs;
use std::io;
use std::iter::Iterator;

fn go(mut r: impl io::Read) {
    let mut nono = Parser::new().parse(&mut r);
    match nono {
        Ok(mut n) => {
            println!("Dimensions (w×h) = {}×{}", n.width, n.height);
            println!("{}", n.as_text());
            Solver::new(&mut n).solve();
            println!("{}", n.as_text());
        }
        Err(e) => {
            println!("Error: {}", e)
        }
    };
}

fn test_cands() {
    let constraint = vec![1,2,2,3,2,1,2];
    let cands = nonograms::solver::candidates(&constraint, 20);
    let consensus = find_consensus(&cands);

    for cand in cands {
        println!("{}", mask_as_string(&cand));
    }
    println!("Consensus:\n{}", mask_as_string(&consensus));
}

fn mask_as_string(mask: &Vec<CellState>) -> String {
    mask.iter()
        .map(|b| match *b {
            CellState::Undecided => "?",
            CellState::Empty => "_",
            CellState::Filled => "█",
        })
        .collect::<String>()
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
