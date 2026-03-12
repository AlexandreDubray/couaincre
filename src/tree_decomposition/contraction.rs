use clap::ValueEnum;
use rustc_hash::{FxHashMap, FxHashSet};
use rand::seq::SliceRandom;

use std::fs::File;
use std::io::{BufWriter, Write};
use std::process::{Command, Stdio};


use crate::restricted::{Restriction, RestrictionOp};
use crate::problem::Problem;

#[derive(Clone, ValueEnum)]
pub enum ContractionHeuristic {
    MaxDegMostCommon,
    MinContractionDeg,
    MaxBiClique,
}

impl ContractionHeuristic {

    pub fn compute_restrictions(&self, primal_graph: &FxHashMap<usize, FxHashSet<usize>>, problem: &Problem) -> Vec<Restriction> {
        match self {
            Self::MaxDegMostCommon => {
                let mut contraction_candidates = primal_graph.keys().copied().collect::<Vec<usize>>();
                let mut contracted = FxHashSet::<usize>::default();
                let mut rng = rand::rng();
                contraction_candidates.shuffle(&mut rng);
                let mut equiv = vec![];
                for node in contraction_candidates.iter().copied() {
                    if contracted.contains(&node) || primal_graph[&node].is_empty() {
                        continue;
                    }
                    if let Some((_, contract_to)) = primal_graph[&node].iter().copied().filter(|n| !contracted.contains(n)).map(|n| (primal_graph[&node].intersection(&primal_graph[&n]).count(), n)).max() {
                        contracted.insert(node);
                        contracted.insert(contract_to);
                        equiv.push(Restriction::new(vec![node, contract_to], RestrictionOp::Equal));
                    }
                }
                equiv
            },
            Self::MinContractionDeg => {
                vec![]
            },
            Self::MaxBiClique => {
                log::trace!("Computing restriction by finding bicliques");
                let mut writer = BufWriter::new(File::create("primal.bip").unwrap());
                let number_var = problem.number_var();
                let number_clause = problem.number_clauses();
                let clause_offset = number_var;
                log::trace!("Writing bipartite graph to file");
                for i in 0..number_clause {
                    if problem.is_clause_active(i) {
                        let node_clause = clause_offset + i + 1;
                        let clause = problem.clause_at(i);
                        for variable in clause.iter().map(|&l| l.unsigned_abs() - 1) {
                            writeln!(writer, "{} {}", variable, node_clause).unwrap();
                        }
                    }
                }
                log::trace!("Launching BBK");
                let bbk = Command::new("bbk")
                    .arg("primal.bip")
                    //.stdout(Stdio::piped())
                    //.stderr(Stdio::piped())
                    .output()
                    .unwrap();
                log::trace!("Extracting bicliques");
                let mut bicliques = String::from_utf8_lossy(&bbk.stdout)
                    .split("\n")
                    .map(|bc| {
                        let variables = bc.split(";").next().unwrap().split_whitespace().map(|n| n.parse::<usize>().unwrap()).collect::<Vec<usize>>();
                        let size = variables.len();
                        (size, variables)
                    }).filter(|(size, _)| *size >= 3).collect::<Vec<(usize, Vec<usize>)>>();
                bicliques.sort_unstable();

                let mut restrictions = vec![];
                let mut restricted = vec![false; number_var];
                while let Some((_, variables)) = bicliques.pop() {
                    let vs = variables.iter().copied().filter(|&v| !restricted[v]).collect::<Vec<usize>>();
                    if vs.len() >= 3 {
                        for v in vs.iter().copied() {
                            restricted[v] = true;
                        }
                        restrictions.push(Restriction::new(vs, RestrictionOp::Equal));
                    }
                }
                log::trace!("Computed {} restrictions", restrictions.len());
                restrictions
            }
        }
    }
}
