use rand::prelude::*;
use rand::seq::IteratorRandom;

use crate::problem::Problem;
use crate::{Args, CTRL};
use crate::tree_decomposition::compute_restrictions;
use super::constraint::*;

pub struct RestrictedSolver {
    problem: Problem,
    exact: bool,
    bounds: Vec<(u64, f64)>,
}

impl RestrictedSolver {

    pub fn new(args: &Args) -> Self {
        let problem = Problem::new(args);
        if problem.is_empty() {
            log::info!("UNSAT");
            return Self {
                problem,
                exact: true,
                bounds: vec![(0, f64::NEG_INFINITY)],
            };
        }
        // Before starting, the first bound is given by the log-10 factor found during the
        // computation of the independent set as we know the formula is SAT, there is at least one
        // model
        let first_bound = problem.log10_mult_factor();
        let elapsed = CTRL.elapsed();
        Self {
            problem,
            exact: false,
            bounds: vec![(elapsed, first_bound)],
        }
    }

    pub fn solve(&mut self, args: &Args) {
        let mut constraints: Vec<Constraint> = vec![];
        log::info!("Number of constraints to partition the space: {}", constraints.len());
        if !constraints.is_empty() {
            log::info!("Starting lower-bound computations with {} seconds remaining", CTRL.remaining(args.timeout));
            let nb_restrictions = constraints.len() as f64;
            let n_calls = nb_restrictions.log2().ceil() as usize;
            let chunk_size = (nb_restrictions / n_calls as f64).ceil() as usize;
            log::info!("Chunk size for restriction removal is {}", chunk_size);
            while !constraints.is_empty() && CTRL.remaining(args.timeout) > 0 {
                if let Some(lb) = args.counter().lower_bound(&self.problem, &constraints, CTRL.remaining(args.timeout)) {
                    log::info!("Lower bound on the log-model-count {} ({} restrictions)", lb, constraints.len());
                    self.bounds.push((CTRL.elapsed(), lb));
                } 
                let new_length = if constraints.len() > chunk_size { constraints.len() - chunk_size } else { 0 };
                constraints.truncate(new_length);
            }
        }
        if CTRL.remaining(args.timeout) > 0 {
            log::trace!("Computing the true model count");
            if let Some(model_count) = args.counter().lower_bound(&self.problem, &constraints, CTRL.remaining(args.timeout)) {
                log::info!("Exact log-model-count is {}", model_count);
                self.bounds.push((CTRL.elapsed(), model_count));
                self.exact = true;
            }
        }
    }

    #[allow(dead_code)]
    pub fn iter_bounds(&self) -> impl Iterator<Item = (u64, f64)> {
        self.bounds.iter().copied()
    }
}
