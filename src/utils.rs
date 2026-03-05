use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio, exit};

use crate::Args;
use crate::counter::Counter;

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

fn check_cmd(cmd: &str, args: &[&str]) {
    if let Err(_) = Command::new(cmd)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status() {
        log::error!("Command {} not found", cmd);
        exit(1);
    }
}

pub fn check_executables(args: &Args) {
    check_cmd("cadical", &["--help"]);
    check_cmd("bpe", &["--help"]);
    match args.counter {
        Counter::D4 => check_cmd("d4", &["<<<", "p cnf 1 1 1 0"]),
        Counter::Ganak => check_cmd("ganak", &["--help"]),
    }
}
