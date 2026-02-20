use std::path::PathBuf;
use std::time::Instant;

use rustsat::types::{TernaryVal, Var};
use malachite::Natural;

use crate::sampler::Sampler;
use crate::utils::*;
use crate::Args;
use crate::tree_decomposition::TreeDecomposition;

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
        let restrictions = self.get_restrictions(args);
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

    fn get_restrictions(&mut self, args: &Args) -> Vec<Restriction> {
        log::info!("Computing tree decomposition...");
        let td = TreeDecomposition::new(args);
        log::info!("Tree decomposition width is {}", td.width());
        vec![]
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
