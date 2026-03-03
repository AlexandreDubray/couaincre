use std::time::Instant;

use crate::problem::Problem;
use crate::Args;
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
        let mut restrictions = compute_restrictions(args, &mut self.problem);
        for _ in 0..restrictions.len() {
            if let Some(lb) = args.counter().lower_bound(&self.problem, &restrictions) {
                log::info!("Lower bound on the log-model-count {}", lb);
                self.bounds.push((self.start_time.elapsed().as_secs(), lb));
            } 
            let _ = restrictions.pop();
        }
        if let Some(exact_count) = args.counter().lower_bound(&self.problem, &restrictions) {
            log::info!("Exact log-model-count is {}", exact_count);
            self.bounds.push((self.start_time.elapsed().as_secs(), exact_count));
            self.exact = true;
        }
    }
}
