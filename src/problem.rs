use rustc_hash::{FxHashSet, FxHashMap};
use std::fs::File;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};

use crate::Args;
use crate::utils::metadata_from_header;

pub struct Problem {
    number_var: usize,
    clauses: Vec<Vec<isize>>,
    active: Vec<bool>,
    var_pos_occ: Vec<FxHashSet<usize>>,
    var_neg_occ: Vec<FxHashSet<usize>>,
}

impl Problem {

    pub fn new(args: &Args) -> Self {
        {
            let (number_var, number_cls) = metadata_from_header(args);
            log::info!("CNF file with {} variables and {} clauses before preprocess", number_var, number_cls);
        }
        // We launch a SAT solver to verify that the formula is SAT.
        log::trace!("Checking satisfiability of the formula");
        let sat_status = Command::new("cadical")
            .arg(args.input.clone())
            .stdout(Stdio::null())
            .status()
            .expect("Fail to run the cadical SAT solver");
        if let Some(code) = sat_status.code() {
            // Code for UNSAT in cadical
            if code == 20 {
                log::info!("Formula is UNSAT");
                return Self { number_var: 0, clauses: vec![], active: vec![], var_pos_occ: vec![], var_neg_occ: vec![] };
            }
        }
        log::info!("Formula is SAT");
        // First, we pre-process the formula using the B+E tool available at https://www.cril.univ-artois.fr/kc/bpe2.html
        // This tool takes a CNF formula in DIMACS file as input and return a new formula in DIMACS
        // format.
        log::trace!("Launche B+E pre-processing with {} seconds time limit", args.preproc_timeout);
        let dimacs = Command::new("bpe")
            .arg(args.input.clone())
            .arg(format!("-cpu-lim={}", args.preproc_timeout))
            .stdout(Stdio::piped())
            .output()
            .unwrap();
        log::trace!("Pre-processing finished");
        let bpe_out = String::from_utf8(dimacs.stdout).unwrap();
        let lines: Box<dyn Iterator<Item = String>> = if !bpe_out.ends_with("s UNSATISFIABLE\n") {
            log::trace!("B+E found a pre-processed formula");
            Box::new(bpe_out.lines().map(|s| s.to_string()))
        } else {
            log::info!("BPE timed out but the formula is SAT. Bypassing pre-processing");
            // We know that the formula is SAT. BPE timed out before finding any pre-processing.
            let reader = BufReader::new(File::open(args.input.clone()).unwrap());
            Box::new(reader.lines().map(|line| line.unwrap()))
        };

        // Then, we extract the clauses from the output of bpe
        let mut clauses: Vec<Vec<isize>> = vec![];
        // At the same time, we detect assignment (i.e., unit clause)
        let mut queue: Vec<(usize, bool)> = vec![];
        let mut map_clause_variables = FxHashMap::<usize, Vec<usize>>::default();
        let mut max_var = 0;
        for line in lines {
            if line.starts_with("p cnf") {
                let mut split = line.split_whitespace().skip(2);
                let number_vars = split.next().unwrap().parse::<usize>().unwrap();
                let number_clauses = split.next().unwrap().parse::<usize>().unwrap();
                log::info!("After preprocess : {} variables and {} clauses", number_vars, number_clauses);
                continue;
            }
            if !line.starts_with('c') && !line.starts_with('p') {
                // Note: the space before the 0 is important so that clauses like "1 -10 0" are correctly splitted
                for clause in line.trim_end().split(" 0").filter(|cl| !cl.is_empty()) {
                    let cls = clause.split_whitespace().map(|x| x.parse::<isize>().unwrap()).collect::<Vec<isize>>();
                    if cls.len() > 1 {
                        let clause_id = clauses.len();
                        for variable in cls.iter().map(|l| l.unsigned_abs()) {
                            max_var = max_var.max(variable);
                            let occurences = map_clause_variables.entry(variable).or_default();
                            occurences.push(clause_id);
                        }
                        clauses.push(cls);
                    } else {
                        debug_assert!(cls.len() == 1);
                        let variable = cls[0].unsigned_abs();
                        max_var = max_var.max(variable);
                        let assignment = cls[0] > 0;
                        queue.push((variable, assignment));
                    }
                }
            }
        }

        // Before creating the problem, we apply a single boolean unit propagation step to further
        // reduce the problem
        let mut flags = vec![false; max_var + 1];
        while let Some((variable, assignment)) = queue.pop() {
            if flags[variable] {
                continue;
            }
            flags[variable] = true;
            if let Some(clause_ids) = map_clause_variables.get(&variable) {
                for clause_id in clause_ids.iter().copied() {
                    // The clause might have been already cleared in a previous propagation
                    if clauses[clause_id].is_empty() {
                        continue;
                    }
                    // First, we find the occurence of variable in the clause
                    let (i, v) = clauses[clause_id].iter().copied().enumerate().find(|(_, v)| v.unsigned_abs() == variable).unwrap();
                    // Two possibilities:
                    //      1. The polarity of the variables aligns with the assignment
                    //         (i.e., -v and assignment is false or v and assignment is
                    //         true). Then the clause is respected and can be cleared
                    //      2. The polarity does not align, then we have to remove the
                    //         variable from the clause. If the clause becomes unit, we add
                    //         the remaining variable to the propagation queue.
                    if (v < 0 && !assignment) || (v > 0 && assignment) {
                        clauses[clause_id].clear();
                    } else {
                        clauses[clause_id].swap_remove(i);
                        if clauses[clause_id].len() == 1 {
                            let remaining_var = clauses[clause_id][0].unsigned_abs();
                            let assignment = clauses[clause_id][0] > 0;
                            queue.push((remaining_var, assignment));
                        }
                    }
                }
            }
        }

        let mut map_variable = FxHashMap::<usize, usize>::default();
        let mut new_variable_index = 1;
        clauses.retain(|cls| !cls.is_empty());
        for clause in clauses.iter_mut() {
            for variable in clause.iter_mut() {
                let v = variable.unsigned_abs();
                if !map_variable.contains_key(&v) {
                    map_variable.insert(v, new_variable_index);
                    new_variable_index += 1;
                }
                let new_v = *map_variable.get(&v).unwrap() as isize;
                if *variable < 0 {
                    *variable = -new_v;
                } else {
                    *variable = new_v;
                }
            }
        }
        log::info!("After initial BUP, {} variables and {} clauses remaining", map_variable.len(), clauses.len());
        let mut var_pos_occ = vec![FxHashSet::default(); new_variable_index - 1];
        let mut var_neg_occ = vec![FxHashSet::default(); new_variable_index - 1];
        for (clause_id, clause) in clauses.iter().enumerate() {
            for literal in clause.iter().copied() {
                if literal < 0 {
                    var_neg_occ[literal.unsigned_abs() - 1].insert(clause_id);
                } else {
                    var_pos_occ[literal.unsigned_abs() - 1].insert(clause_id);
                }
            }
        }
        let nb_clauses = clauses.len();
        Self {
            number_var: new_variable_index - 1,
            clauses,
            active: vec![true; nb_clauses],
            var_pos_occ,
            var_neg_occ,
        }
    }

    /// Returns true if the problem is empty
    pub fn is_empty(&self) -> bool {
        self.clauses.is_empty()
    }

    /// Returns the number of variable in the problem
    pub fn number_var(&self) -> usize {
        self.number_var
    }

    /// Returns the number of clauses in the problem
    pub fn number_clauses(&self) -> usize {
        self.clauses.len()
    }

    /// Iterates on the clauses of the problem
    pub fn iter_clauses(&self) -> impl Iterator<Item = &Vec<isize>> {
        self.clauses.iter().enumerate().filter(|(index, _)| self.active[*index]).map(|c| c.1)
    }

    /// Iterates on the clauses of the problem in DIMACS format
    pub fn iter_clauses_dimacs(&self) -> impl Iterator<Item = String> {
        self.iter_clauses().map(|clause| format!("{} 0", clause.iter().map(|l| l.to_string()).collect::<Vec<String>>().join(" ")))
    }

    pub fn positive_occurences(&self, variable: usize) -> &FxHashSet<usize> {
        &self.var_pos_occ[variable]
    }

    pub fn negative_occurences(&self, variable: usize) -> &FxHashSet<usize> {
        &self.var_neg_occ[variable]
    }

    pub fn make_equal(&mut self, u: usize, v: usize) {
        for clause_id in self.var_pos_occ[u].intersection(&self.var_neg_occ[v]).copied() {
            self.active[clause_id] = false;
        }
        for clause_id in self.var_pos_occ[v].intersection(&self.var_neg_occ[u]).copied() {
            self.active[clause_id] = false;
        }
    }

    pub fn make_not_equal(&mut self, u: usize, v: usize) {
        for clause_id in self.var_pos_occ[u].intersection(&self.var_pos_occ[v]).copied() {
            self.active[clause_id] = false;
        }
        for clause_id in self.var_neg_occ[u].intersection(&self.var_neg_occ[v]).copied() {
            self.active[clause_id] = false;
        }
    }

}
