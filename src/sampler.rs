use std::path::PathBuf;

use rustsat::{
    instances::{fio, ManageVars, SatInstance},
    solvers::{self, Solve, SolveIncremental},
    types::{Assignment, Var, TernaryVal},
};

pub fn sample_solutions(input: PathBuf, n: usize) -> Vec<Vec<TernaryVal>> {
    let instance: SatInstance = SatInstance::from_dimacs_path(input).unwrap();

     let max_var = instance
        .var_manager_ref()
        .max_var()
        .expect("[SAMPLING] expected at least one variable in the instance");

    let (cnf, vm) = instance.into_cnf();

    let mut solver = rustsat_tools::Solver::default();
    solver.reserve(vm.max_var().expect("[SAMPLING] no variables in instance"))
        .expect("[SAMPLING] error reserving memory in solver");
    let _ = solver.add_cnf(cnf);

    let mut solutions: Vec<Vec<TernaryVal>> = vec![];
    for _ in 0..n {
        match solver.solve().expect("[SAMPLING] error while solving") {
            solvers::SolverResult::Sat => {
                let sol = solver
                    .solution(max_var)
                    .expect("[SAMPLING] could not get solution from solver");
                // Add blocking clause to solver
                let bl_cl = sol.clone().into_iter().map(|l| !l).collect();
                solver.add_clause(bl_cl).expect("[SAMPLING] error adding blocking clause to solver");
                let mut solution = vec![TernaryVal::DontCare; max_var.idx() + 1];
                for lit in sol.iter() {
                    if lit.is_pos() {
                        solution[lit.vidx()] = TernaryVal::True;
                    } else {
                        solution[lit.vidx()] = TernaryVal::False;
                    }
                }
                solutions.push(solution);
            }
            solvers::SolverResult::Unsat => {
                break
            },
            solvers::SolverResult::Interrupted => panic!("[SAMPLING] Sat solver interrupted without limits"),
        }
    }
    solutions
}
