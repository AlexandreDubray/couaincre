use std::path::PathBuf;

use rustsat::{
    instances::{ManageVars, SatInstance},
    solvers::{self, Solve},
    types::{Assignment, Var},
};
use rustsat_tools::Solver;

pub struct Sampler {
    input: PathBuf,
}

impl Sampler {

    pub fn new(input: PathBuf) -> Self {
        Self { input }
    }

    pub fn sample_solutions(&self, n: usize) -> Enumerator {
        let instance: SatInstance = SatInstance::from_dimacs_path(self.input.clone()).unwrap();

         let max_var = instance
            .var_manager_ref()
            .max_var()
            .expect("[SAMPLING] expected at least one variable in the instance");

        let (cnf, vm) = instance.into_cnf();

        let mut solver = rustsat_tools::Solver::default();
        solver.reserve(vm.max_var().expect("[SAMPLING] no variables in instance"))
            .expect("[SAMPLING] error reserving memory in solver");
        let _ = solver.add_cnf(cnf);
        Enumerator {
            solver,
            max_var,
            n,
        }
    }
}

pub struct Enumerator {
    solver: Solver,
    max_var: Var,
    n: usize,
}

impl Iterator for Enumerator {

    type Item = Assignment;

    fn next(&mut self) -> Option<Self::Item> {
        if self.n == 0 {
            None
        } else {
            self.n -= 1;
            match self.solver.solve().expect("[SAMPLING] Error while solving formula") {
                solvers::SolverResult::Sat => {
                    let sol = self.solver
                        .solution(self.max_var)
                        .expect("[SAMPLING] could not get solution from solver");
                    // Add blocking clause to solver
                    let bl_cl = sol.clone().into_iter().map(|l| !l).collect();
                    self.solver.add_clause(bl_cl).expect("[SAMPLING] error adding blocking clause to solver");
                    Some(sol)
                },
                solvers::SolverResult::Unsat => {
                    None
                },
                solvers::SolverResult::Interrupted => panic!("[SAMPLING] SAT solver interrupted without limits"),
            }
        }
    }
}
