use rustc_hash::{FxHashSet, FxHashMap};
use std::fs::File;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};

use crate::{Args, CTRL};
use crate::utils::metadata_from_header;

#[derive(Clone)]
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
                return Self {
                    number_var: 0,
                    clauses: vec![],
                    active: vec![],
                    var_pos_occ: vec![],
                    var_neg_occ: vec![],
                };
            }
        }
        log::info!("Formula is SAT. {} seconds elapsed since start", CTRL.elapsed());
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
        log::trace!("Pre-processing finished. {} seconds elapsed since start", CTRL.elapsed());
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
        let mut map_clause_variables = FxHashMap::<usize, Vec<usize>>::default();
        let mut number_var_after_preproc = 0;
        let mut max_var = 0;
        for line in lines {
            if line.starts_with("p cnf") {
                let mut split = line.split_whitespace().skip(2);
                number_var_after_preproc = split.next().unwrap().parse::<usize>().unwrap();
                let number_clauses = split.next().unwrap().parse::<usize>().unwrap();
                log::info!("After preprocess : {} variables and {} clauses", number_var_after_preproc, number_clauses);
                continue;
            }
            if !line.starts_with('c') && !line.starts_with('p') {
                // Note: the space before the 0 is important so that clauses like "1 -10 0" are correctly splitted
                for clause in line.trim_end().split(" 0").filter(|cl| !cl.is_empty()) {
                    let cls = clause.split_whitespace().map(|x| x.parse::<isize>().unwrap()).collect::<Vec<isize>>();
                    let clause_id = clauses.len();
                    for variable in cls.iter().map(|l| l.unsigned_abs()) {
                        max_var = max_var.max(variable);
                        let occurences = map_clause_variables.entry(variable).or_default();
                        occurences.push(clause_id);
                    }
                    clauses.push(cls);
                }
            }
        }

        let mut var_pos_occ = vec![FxHashSet::default(); number_var_after_preproc];
        let mut var_neg_occ = vec![FxHashSet::default(); number_var_after_preproc];
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
            number_var: number_var_after_preproc,
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
        for clause_id in self.var_neg_occ[v].iter().copied() {
            if self.var_pos_occ[u].contains(&clause_id) {
                self.active[clause_id] = false;
            } else {
                let index = self.clauses[clause_id].iter().position(|&l| l.unsigned_abs() - 1 == v).unwrap();
                self.clauses[clause_id].swap_remove(index);
            }
        }
        for clause_id in self.var_pos_occ[v].iter().copied() {
            if self.var_neg_occ[u].contains(&clause_id) {
                self.active[clause_id] = false;
            } else {
                let index = self.clauses[clause_id].iter().position(|&l| l.unsigned_abs() - 1 == v).unwrap();
                self.clauses[clause_id].swap_remove(index);
            }
        }
    }

    pub fn make_not_equal(&mut self, u: usize, v: usize) {
        for clause_id in self.var_neg_occ[v].iter().copied() {
            if self.var_neg_occ[u].contains(&clause_id) {
                self.active[clause_id] = false;
            } else {
                let index = self.clauses[clause_id].iter().position(|&l| l.unsigned_abs() - 1 == v).unwrap();
                self.clauses[clause_id].swap_remove(index);
            }
        }
        for clause_id in self.var_pos_occ[v].iter().copied() {
            if self.var_pos_occ[u].contains(&clause_id) {
                self.active[clause_id] = false;
            } else {
                let index = self.clauses[clause_id].iter().position(|&l| l.unsigned_abs() - 1 == v).unwrap();
                self.clauses[clause_id].swap_remove(index);
            }
        }
    }

}
