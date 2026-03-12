use std::time::Instant;

extern crate cryptominisat;
use cryptominisat::Solver as CMSSolver;
use cryptominisat::{Lit, Lbool};

use rand::prelude::*;
use rand::seq::IteratorRandom;

use crate::problem::Problem;
use crate::{Args, CTRL};
use crate::tree_decomposition::compute_restrictions;

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
        let mut restrictions = compute_restrictions(args, self.problem.clone());
        log::info!("Number of restrictions computed: {}", restrictions.len());
        if !restrictions.is_empty() {
            log::info!("Starting lower-bound computations with {} seconds remaining", CTRL.remaining(args.timeout));
            let nb_restrictions = restrictions.len() as f64;
            let n_calls = nb_restrictions.log2().ceil() as usize;
            let chunk_size = (nb_restrictions / n_calls as f64).ceil() as usize;
            log::info!("Chunk size for restriction removal is {}", chunk_size);
            while !restrictions.is_empty() && CTRL.remaining(args.timeout) > 0 {
                if let Some(lb) = args.counter().lower_bound(&self.problem, &restrictions, CTRL.remaining(args.timeout)) {
                    log::info!("Lower bound on the log-model-count {} ({} restrictions)", lb, restrictions.len());
                    self.bounds.push((CTRL.elapsed(), lb));
                } 
                let new_length = if restrictions.len() > chunk_size { restrictions.len() - chunk_size } else { 0 };
                restrictions.truncate(new_length);
            }
        }
        if CTRL.remaining(args.timeout) > 0 {
            log::trace!("Computing the true model count");
            if let Some(model_count) = args.counter().lower_bound(&self.problem, &restrictions, CTRL.remaining(args.timeout)) {
                log::info!("Exact log-model-count is {}", model_count);
                self.bounds.push((CTRL.elapsed(), model_count));
                self.exact = true;
            }
        }
    }

    fn count_xor(solver: &mut CMSSolver, vars: &Vec<Lit>, args: &Args) -> usize {
        let mut count = 0;
        while count < args.pivot {
            match solver.solve() {
                Lbool::True => {
                    let model = solver.get_model();
                    let mut local_count = 1;
                    let mut blocked: Vec<Lit> = vec![];
                    for i in 0..model.len() {
                        match model[i] {
                            Lbool::True => blocked.push(!vars[i]),
                            Lbool::False => blocked.push(vars[i]),
                            Lbool::Undef => local_count *= 2,
                        }
                    }
                    count += local_count;
                    solver.add_clause(&blocked);
                },
                Lbool::False => {
                    log::trace!("Remaining formula is unsat, breaking at {} models", count);
                    break;
                },
                Lbool::Undef => {
                    panic!("Should not arise");
                }
            }
        }
        count
    }

    fn find_xor_constraints(&self) -> Vec<(Vec<Lit>, bool)> {
        log::trace!("Finding number of XOR constraints such that the problem is UNSAT");
        let mut sat_solver = CMSSolver::new();
        let vars = (0..self.problem.number_var()).map(|_| sat_solver.new_var()).collect::<Vec<_>>();
        for clause in self.problem.iter_clauses() {
            let cms_cls = clause.iter().map(|&l| {
                if l < 0 {
                    !vars[l.unsigned_abs() - 1]
                } else {
                    vars[l.unsigned_abs() - 1]
                }
            }).collect::<Vec<_>>();
            sat_solver.add_clause(&cms_cls);
        }
        let mut xor_clauses: Vec<(Vec<Lit>, bool)> = vec![];
        let mut number_xor = 1;
        let mut rng = rand::rng();
        loop {
            while xor_clauses.len() < number_xor {
                let var_indexes = self.problem.iter_independent_set().sample(&mut rng, 3).iter().map(|&v| vars[v]).collect::<Vec<Lit>>();
                let polarity = rand::random::<bool>();
                sat_solver.add_xor_literal_clause(&var_indexes, polarity);
                xor_clauses.push((var_indexes, polarity));
            }
            if sat_solver.solve() == Lbool::False {
                log::trace!("Problem with {} XOR constraint is UNSAT", number_xor);
                break;
            }
            log::trace!("Problem with {} XOR constraint is SAT", number_xor);
            number_xor *= 2;
        }
        xor_clauses
    }

    pub fn xor_solve(&mut self, args: &Args) {
        log::trace!("XOR solving the problem");
        let xor_clauses = self.find_xor_constraints();
    }

    #[allow(dead_code)]
    pub fn iter_bounds(&self) -> impl Iterator<Item = (u64, f64)> {
        self.bounds.iter().copied()
    }
}
