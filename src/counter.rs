use std::io::Write;
use std::process::{Command, Stdio, exit};

use clap::ValueEnum;
use rustc_hash::FxHashMap;

use crate::restricted::{Restriction, RestrictionOp};
use crate::problem::Problem;

#[derive(Clone, ValueEnum)]
pub enum Counter {
    D4,
    Ganak,
}

impl Counter {

    fn union(map: &mut FxHashMap<usize, usize>, x: usize, y: usize) {
        let repr_x = Self::find(map, x);
        let repr_y = Self::find(map, y);
        if repr_x != repr_y {
            map.insert(repr_x, repr_y);
        }
    }

    fn find(map: &mut FxHashMap<usize, usize>, x: usize) -> usize {
        let repr = *map.entry(x).or_insert(x);
        if repr == x {
            return repr;
        }
        let repr = Self::find(map, map[&x]);
        map.insert(x, repr);
        repr
    }

    pub fn lower_bound(&self, problem: &Problem, restrictions: &[Restriction], timeout: u64) -> Option<f64> {
        let mut mapping = FxHashMap::<usize, usize>::default();
        for restriction in restrictions.iter() {
            match restriction.op() {
                RestrictionOp::Equal => {
                    let vars = restriction.vars();
                    for i in 0..(vars.len() - 1) {
                        for j in (i+1)..vars.len() {
                            let x = vars[i];
                            let y = vars[j];
                            Self::union(&mut mapping, x, y);
                        }
                    }
                },
            };
        }
        for restriction in restrictions.iter() {
            match restriction.op() {
                RestrictionOp::Equal => {
                    let vars = restriction.vars();
                    for i in 0..(vars.len() - 1) {
                        for j in (i+1)..vars.len() {
                            let x = vars[i];
                            let y = vars[j];
                            Self::find(&mut mapping, x);
                            Self::find(&mut mapping, y);
                        }
                    }
                },
            };
        }
        for (_, repr) in mapping.iter() {
            assert!(mapping[repr] == *repr);
        }
        let number_var = problem.number_var();
        let number_clauses = problem.number_clauses();
        let mut counter_proc = match self {
            Self::D4 => Command::new("timeout")
                .arg(format!("{}", timeout))
                .arg("d4")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap(),
            Self::Ganak => Command::new("timeout")
                .arg(format!("{}", timeout))
                .arg("ganak")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap(),
        };
        match counter_proc.stdin.take() {
            Some(mut stdin) => {
                log::trace!("Launching model counter on problem with {} variables and {} clauses with {} seconds timeout", number_var, number_clauses, timeout);
                writeln!(stdin, "p cnf {} {}", number_var, number_clauses).unwrap();
                writeln!(stdin, "c p show {} 0", problem.iter_independent_set().map(|v| format!("{}", v + 1)).collect::<Vec<String>>().join(" ")).unwrap();
                for clause in problem.iter_clauses() {
                    let clause_str = clause.iter().map(|&l| {
                        let var = match mapping.get(&(l.unsigned_abs() - 1)) {
                            Some(&v) => v + 1,
                            None => l.unsigned_abs(),
                        };
                        if l < 0 {
                            format!("-{}", var)
                        } else {
                            format!("{}", var)
                        }
                    }).collect::<Vec<String>>().join(" ");
                    writeln!(stdin, "{} 0", clause_str).unwrap();
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
