use std::io::Write;
use std::process::{Command, Stdio, exit};

use clap::ValueEnum;

use crate::restricted::Restriction;
use crate::problem::Problem;

#[derive(Clone, ValueEnum)]
pub enum Counter {
    D4,
    Ganak,
}

impl Counter {

    pub fn lower_bound(&self, problem: &Problem, restrictions: &[Restriction]) -> Option<f64> {
        let number_var = problem.number_var();
        let number_clauses = problem.number_clauses() + restrictions.iter().map(|restriction| restriction.number_of_encoding_clauses()).sum::<usize>();
        let mut counter_proc = match self {
            Self::D4 => Command::new("d4")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap(),
            Self::Ganak => Command::new("ganak")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap(),
        };
        match counter_proc.stdin.take() {
            Some(mut stdin) => {
                log::trace!("Launching model counter on problem with {} variables and {} clauses", number_var, number_clauses);
                writeln!(stdin, "p cnf {} {}", number_var, number_clauses).unwrap();
                for clause in problem.iter_clauses_dimacs() {
                    writeln!(stdin, "{}", clause).unwrap();
                }
                for restriction in restrictions.iter() {
                    writeln!(stdin, "{}", restriction.to_dimacs_lines()).unwrap();
                }
            },
            None => {
                log::error!("Can not take stdin to send problem to counter");
                exit(1);
            }
        }
        let counter_out = counter_proc.wait_with_output().unwrap();
        let mut log_count: Option<f64> = None;
        if counter_out.status.success() {
            match self {
                Self::D4 => {
                    for line in String::from_utf8(counter_out.stdout).unwrap().split('\n') {
                        if line.starts_with("c s log10-estimate") {
                            log_count = Some(line.split_whitespace().last().unwrap().parse::<f64>().unwrap());
                            break;
                        }
                    }
                },
                Self::Ganak => {
                    for line in String::from_utf8(counter_out.stdout).unwrap().split('\n') {
                        if line.starts_with("c s log10-estimate") {
                            log_count = Some(line.split_whitespace().last().unwrap().parse::<f64>().unwrap());
                            break;
                        }
                    }
                },
            }
        }
        log_count
    }
}
