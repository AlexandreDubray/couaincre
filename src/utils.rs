use std::path::PathBuf;
use std::fs::File;
use std::io::{BufRead, BufReader};

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
    panic!("[UTILS] No header found in DIMACS file when looking for number of variables");
}
