use std::path::PathBuf;
use std::time::Instant;

use rustsat::types::{TernaryVal, Var};
use malachite::Natural;

use crate::sampler::Sampler;
use crate::utils::*;
use crate::Args;

pub struct RestrictedSolver {
    input: PathBuf,
    model_count: Natural,
    exact: bool,
    bounds: Vec<(u64, Natural)>,
    start_time: Instant,
}

impl RestrictedSolver {

    pub fn new(input: PathBuf) -> Self {
        Self {
            input,
            model_count: Natural::from(0u32),
            exact: false,
            start_time: Instant::now(),
            bounds: vec![],
        }
    }

    pub fn solve(&mut self, args: &Args) {
        println!("Computing restrictions with number_sample = 1000");
        let restrictions = self.get_restrictions(1000);
        if self.exact {
            let time = self.start_time.elapsed().as_secs();
            println!("c s exact arb int {} ({} secs)", self.model_count, time);
            self.bounds.push((time, self.model_count.clone()));
            return;
        }
        let number_var = number_var_from_dimacs(self.input.clone());
        let clauses = clauses_from_dimacs(self.input.clone());
        for i in 0..restrictions.len() {
            let mut restrictions_clauses = vec![];
            for j in 0..(restrictions.len() - i) {
                restrictions_clauses.extend(restrictions[j].to_dimacs_lines());
            }
            self.model_count = args.counter().count(number_var, &clauses, &restrictions_clauses);
            println!("Lower bound {} found in {} seconds", self.model_count, self.start_time.elapsed().as_secs());
            self.bounds.push((self.start_time.elapsed().as_secs(), self.model_count.clone()));
        }
    }

    fn get_restrictions(&mut self, n: usize) -> Vec<Restriction> {
        let number_var = number_var_from_dimacs(self.input.clone());
        // We map each assignment (four possibilities) to a single index. For an
        // assignment (i, j) with i, j in {0, 1}, we compute  the index as follows:
        //      index = i + 2j
        //  which maps the following way:
        //  (0, 0) -> 0
        //  (1, 0) -> 1
        //  (0, 1) -> 2
        //  (1, 1) -> 3
        let mut stats: Vec<Vec<Vec<usize>>> = (0..number_var).map(|u| {
            (0..(u+1)).map(|_| {
                vec![0; 4]
            }).collect()
        }).collect();
        let sampler = Sampler::new(self.input.clone());
        let t = Vec::from([1]);
        let f = Vec::from([0]);
        let dc = Vec::from([0, 1]);
        for model in sampler.sample_solutions(n) {
            self.model_count += Natural::from(1u32);
            // compute co-occurences for each variable pairs
            for u in 0..number_var {
                match model.var_value(Var::new(u as u32)) {
                    TernaryVal::True => stats[u][u][3] += 1,
                    TernaryVal::False => stats[u][u][0] += 1,
                    TernaryVal::DontCare => {
                        stats[u][u][0] += 1;
                        stats[u][u][3] += 1;
                    },
                };
                let dom_u = match model.var_value(Var::new(u as u32)) {
                    TernaryVal::True => &t,
                    TernaryVal::False => &f,
                    TernaryVal::DontCare => &dc,
                };
                for v in 0..u {
                    let dom_v = match model.var_value(Var::new(v as u32)) {
                        TernaryVal::True => &t,
                        TernaryVal::False => &f,
                        TernaryVal::DontCare => &dc,
                    };
                    for i in dom_u.iter() {
                        for j in dom_v.iter() {
                            let index = i + 2*j;
                            stats[u][v][index] += 1;
                        }
                    }
                }
            }
        }
        if self.model_count < n {
            self.exact = true;
            vec![]
        } else {
            let mut restrictions: Vec<(usize, Restriction)> = vec![];
            for u in 0..number_var {
                restrictions.push((stats[u][u][3], Restriction::new(Some(u), None, RestrictionOp::AssignTrue)));
                restrictions.push((stats[u][u][0], Restriction::new(Some(u), None, RestrictionOp::AssignFalse)));
                for v in 0..u {
                    restrictions.push((stats[u][v][0] + stats[u][v][3], Restriction::new(Some(u), Some(v), RestrictionOp::Equal)));
                    restrictions.push((stats[u][v][1] + stats[u][v][2], Restriction::new(Some(u), Some(v), RestrictionOp::NotEqual)));
                }
            }
            restrictions.sort_unstable_by_key(|x| x.0);
            restrictions.truncate(10);
            restrictions.into_iter().map(|(_, y)| y).collect()
        }
    }
}

enum RestrictionOp {
    Equal,
    NotEqual,
    AssignTrue,
    AssignFalse,
}

struct Restriction {
    x: Option<usize>,
    y: Option<usize>,
    op: RestrictionOp,
}

impl Restriction {

    fn new(x: Option<usize>, y: Option<usize>, op: RestrictionOp) -> Self {
        Self {
            x,
            y,
            op,
        }
    }

    fn flip(&self) -> Self {
        let newOp = match self.op {
            RestrictionOp::Equal => RestrictionOp::NotEqual,
            RestrictionOp::NotEqual => RestrictionOp::Equal,
            RestrictionOp::AssignTrue => RestrictionOp::AssignFalse,
            RestrictionOp::AssignFalse => RestrictionOp::AssignTrue,
        };
        Self {
            x: self.x,
            y: self.y,
            op: newOp,
        }
    }

    fn to_dimacs_lines(&self) -> Vec<String> {
        let mut lines = vec![];
        match self.op {
            RestrictionOp::AssignTrue => {
                debug_assert!(self.x.is_some() && self.y.is_none());
                lines.push(format!("{} 0", self.x.unwrap() + 1));
            },
            RestrictionOp::AssignFalse => {
                debug_assert!(self.x.is_some() && self.y.is_none());
                lines.push(format!("-{} 0", self.x.unwrap() + 1));
            },
            RestrictionOp::Equal => {
                debug_assert!(self.x.is_some() && self.y.is_some());
                lines.push(format!("-{} {} 0", self.x.unwrap() + 1, self.y.unwrap() + 1));
                lines.push(format!("{} -{} 0", self.x.unwrap() + 1, self.y.unwrap() + 1));
            },
            RestrictionOp::NotEqual => {
                debug_assert!(self.x.is_some() && self.y.is_some());
                lines.push(format!("{} {} 0", self.x.unwrap() + 1, self.y.unwrap() + 1));
                lines.push(format!("-{} -{} 0", self.x.unwrap() + 1, self.y.unwrap() + 1));
            },
        }
        lines
    }

}
