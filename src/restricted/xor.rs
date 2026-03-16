extern crate cryptominisat;
use cryptominisat::Solver as CMSSolver;
use cryptominisat::{Lit, Lbool};

fn count_xor(solver: &mut CMSSolver, vars: &Vec<Lit>, args: &Args) -> usize {
    let mut count = 0;
    while count < 100 {
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
            let var_indexes = self.problem.iter_independent_set().sample(&mut rng, 2).iter().map(|&v| vars[v]).collect::<Vec<Lit>>();
            sat_solver.add_clause(&[var_indexes[0], !var_indexes[1]]);
            sat_solver.add_clause(&[!var_indexes[0], var_indexes[1]]);
            xor_clauses.push((vec![], true));
            /*
            let polarity = rand::random::<bool>();
            sat_solver.add_xor_literal_clause(&var_indexes, polarity);
            xor_clauses.push((var_indexes, polarity));
            */
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
