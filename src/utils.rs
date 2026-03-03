use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio, exit};

use crate::Args;

/// Returns the number of variables and clauses in the input file
pub fn metadata_from_header(args: &Args) -> (usize, usize) {
    let file = File::open(args.input.clone()).unwrap();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line.unwrap();
        if line.starts_with("p cnf") {
            let mut split = line.split_whitespace().skip(2);
            let number_var = split.next().unwrap().parse::<usize>().unwrap();
            let number_clauses = split.next().unwrap().parse::<usize>().unwrap();
            return (number_var, number_clauses);
        }
    }
    log::error!("No header found in DIMACS file when looking for metadata");
    panic!();
}

pub fn check_executables() {
    if let Err(_) = Command::new("cadical")
        .arg("--help")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status() {
        log::error!("No executable cadical found");
        exit(1);
    }
    if let Err(_) = Command::new("bpe")
        .arg("--help")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status() {
        log::error!("No executable bpe found");
        exit(1);
    }
}
