use std::path::PathBuf;
use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::Args;

pub fn clauses_from_dimacs(input: PathBuf) -> Vec<String> {
    let file = File::open(input).unwrap();
    let reader = BufReader::new(file);
    reader.lines().map(|l| l.unwrap()).filter(|line| !line.starts_with('c') && !line.starts_with("p cnf")).collect()
}

pub fn number_var_from_dimacs(input: PathBuf) -> usize {
    let file = File::open(input).unwrap();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line.unwrap();
        if line.starts_with("p cnf") {
            return line.split_whitespace().skip(2).next().unwrap().parse::<usize>().unwrap();
        }
    }
    log::error!("No header found in DIMACS file when looking for metadata");
    panic!();
}

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
