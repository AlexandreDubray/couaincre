use std::fs::File;
use std::io::Write;
use std::process::{Command, Stdio};

use malachite::Natural;
use clap::ValueEnum;

#[derive(Clone, ValueEnum)]
pub enum Counter {
    D4,
}

impl Counter {

    pub fn count(&self, number_var: usize, clauses: &Vec<String>, restrictions: &Vec<String>) -> Natural {
        let mut file = File::create("restricted.cnf").unwrap();
        writeln!(file, "p cnf {} {}", number_var, clauses.len() + restrictions.len()).unwrap();
        for clause in clauses.iter() {
            writeln!(file, "{}", clause).unwrap();
        }
        for clause in restrictions.iter() {
            writeln!(file, "{}", clause).unwrap();
        }
        match self {
            Counter::D4 => count_d4("restricted.cnf"),
        }
    }
}

fn count_d4(file: &str) -> Natural {
    let counter_out = Command::new("d4").arg(file).stdout(Stdio::piped()).output().unwrap();
    let mut count: Option<Natural> = None;
    for line in String::from_utf8(counter_out.stdout).unwrap().split('\n') {
        if line.starts_with("c s exact") {
            count = Some(line.split_whitespace().last().unwrap().parse::<Natural>().unwrap());
            break;
        }
    }
    count.unwrap()
}
