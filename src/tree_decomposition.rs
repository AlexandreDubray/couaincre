use std::fs::File;
use std::io::{BufRead, BufReader};

use clap::ValueEnum;
use rustc_hash::FxHashSet;

use crate::Args;
use crate::utils::*;

pub struct TreeDecomposition {
    order: Vec<usize>,
    width: usize,
}

#[derive(Clone, ValueEnum)]
pub enum TDHeuristic {
    MinFill,
    MinDeg,
}

impl TreeDecomposition {

    pub fn new(args: &Args) -> Self {
        let number_var = number_var_from_dimacs(args.input.clone());

        let mut primal_graph: Vec<FxHashSet<usize>> = (0..number_var).map(|_| FxHashSet::default()).collect();

        let reader = BufReader::new(File::open(args.input.clone()).unwrap());
        for line in reader.lines() {
            let line = line.unwrap();
            if line.is_empty() || line.starts_with("p cnf") || line.starts_with('c') {
                continue;
            }
            let variables = line.split_whitespace().map(|l| l.parse::<isize>().unwrap().abs() as usize).collect::<Vec<usize>>();
            for i in 0..(variables.len() - 1) {
                for j in (i+1)..(variables.len() - 1) {
                    let u = variables[i] - 1;
                    let v = variables[j] - 1;
                    primal_graph[u].insert(v);
                    primal_graph[v].insert(u);
                }
            }
        }

        // Computes the order of elimination for the tree-decomposition
        let mut width = 0;
        let mut order: Vec<usize> = vec![];
        let mut candidates = (0..number_var).collect::<Vec<usize>>();
        while !candidates.is_empty() {
            let mut best = 0;
            let mut best_score = usize::MAX;
            for (index, candidate) in candidates.iter().copied().enumerate() {
                let score = args.td_heuristic().evaluate_node(&primal_graph, candidate);
                if score < best_score {
                    best = index;
                    best_score = score;
                }
            }
            let node = candidates[best];
            candidates.swap_remove(best);
            order.push(node);
            
            // Apply the node elimination, and compute the size of the created clique
            let clique_size = primal_graph[node].len() + 1;
            width = width.max(clique_size);
            // Remove node from the graph (disconnect it from its neighbors) and connect all of its
            // neighbors
            let neighbors = primal_graph[node].iter().copied().collect::<Vec<usize>>();
            for neighbor in neighbors.iter().copied() {
                primal_graph[neighbor].extend(neighbors.iter().copied().filter(|n| *n != neighbor));
                primal_graph[neighbor].remove(&node);
            }
            primal_graph[node].clear();
        }

        Self {
            order,
            width: width as usize,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn order(&self) -> &[usize] {
        &self.order
    }
}

impl TDHeuristic {

    fn evaluate_node(&self, graph: &Vec<FxHashSet<usize>>, node: usize) -> usize {
        match self {
            Self::MinFill => {
                if graph[node].is_empty() {
                    return 0;
                }
                let number_neighbors = graph[node].len();
                let mut missing_edges = (number_neighbors * (number_neighbors - 1)) / 2;
                let neighbors = graph[node].iter().copied().collect::<Vec<usize>>();
                for i in 0..neighbors.len() {
                    for j in (i+1)..neighbors.len() {
                        if graph[neighbors[i]].contains(&neighbors[j]) {
                            missing_edges -= 1;
                        }
                    }
                }
                missing_edges
            },
            Self::MinDeg => {
                graph[node].len()
            }
        }
    }

}
