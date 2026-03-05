use std::time::Instant;

use crate::problem::Problem;
use crate::{Args, CTRL};
use crate::tree_decomposition::compute_restrictions;

pub struct RestrictedSolver {
    problem: Problem,
    exact: bool,
    bounds: Vec<(u64, f64)>,
    start_time: Instant,
}

impl RestrictedSolver {

    pub fn new(args: &Args) -> Self {
        let problem = Problem::new(args);
        if problem.is_empty() {
            log::info!("UNSAT");
            return Self {
                problem,
                exact: true,
                start_time: Instant::now(),
                bounds: vec![(0, f64::NEG_INFINITY)],
            };
        }
        Self {
            problem,
            exact: false,
            start_time: Instant::now(),
            bounds: vec![],
        }
    }

    pub fn solve(&mut self, args: &Args) {
        let mut restrictions = compute_restrictions(args, self.problem.clone());
        log::info!("Number of restrictions computed: {}", restrictions.len());
        if !restrictions.is_empty() {
            log::info!("Starting lower-bound computations with {} seconds remaining", CTRL.remaining(args.timeout));
            let nb_restrictions = restrictions.len() as f64;
            let n_calls = nb_restrictions.log2().ceil() as usize;
            let chunk_size = (nb_restrictions / n_calls as f64).ceil() as usize;
            log::info!("Chunk size for restriction removal is {}", chunk_size);
            while !restrictions.is_empty() && CTRL.remaining(args.timeout) > 0 {
                if let Some(lb) = args.counter().lower_bound(&self.problem, &restrictions) {
                    log::info!("Lower bound on the log-model-count {} ({} restrictions)", lb, restrictions.len());
                    self.bounds.push((self.start_time.elapsed().as_secs(), lb));
                } 
                let new_length = if restrictions.len() > chunk_size { restrictions.len() - chunk_size } else { 0 };
                restrictions.truncate(new_length);
            }
        }
        if CTRL.remaining(args.timeout) > 0 {
            log::trace!("Computing the true model count");
            if let Some(model_count) = args.counter().lower_bound(&self.problem, &restrictions) {
                log::info!("Exact log-model-count is {}", model_count);
                self.bounds.push((self.start_time.elapsed().as_secs(), model_count));
                self.exact = true;
            }
        }
    }

    #[allow(dead_code)]
    pub fn iter_bounds(&self) -> impl Iterator<Item = (u64, f64)> {
        self.bounds.iter().copied()
    }
}
