use std::fs::File;
use std::io::{BufRead, BufReader};

use clap::ValueEnum;
use rustc_hash::{FxHashSet, FxHashMap};

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
        println!("[Tree decomposition] Computing elimination order...");
        let number_var = number_var_from_dimacs(args.input.clone());
        println!("[Tree decomposition] {} variables in the problem", number_var);

        println!("[Tree decomposition] Creating the primal graph...");
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

        println!("[Tree decomposition] Primal graph constructed. Computing elimination order...");
        // Computes the order of elimination for the tree-decomposition
        let mut width = 0;
        let mut order: Vec<usize> = vec![];
        let mut buckets = FxHashMap::<usize, FxHashSet<usize>>::default();
        let mut map_node_bucket = FxHashMap::<usize, usize>::default();
        let mut recompute_score = vec![false; number_var];
        let mut min_score = usize::MAX;
        for candidate in 0..number_var {
            let score = args.td_heuristic().evaluate_node(&primal_graph, candidate);
            map_node_bucket.insert(candidate, score);
            min_score = min_score.min(score);
            Self::insert_in_bucket(&mut buckets, score, candidate);
        }
        while order.len() < number_var {
            while !buckets.contains_key(&min_score) || buckets.get(&min_score).unwrap().is_empty() {
                min_score += 1;
            }
            let node = *buckets.get(&min_score).unwrap().iter().next().unwrap();
            if recompute_score[node] {
                recompute_score[node] = false;
                let new_score = args.td_heuristic().evaluate_node(&primal_graph, node);
                if new_score > min_score {
                    min_score = min_score.min(new_score);
                    buckets.get_mut(&min_score).unwrap().remove(&node);
                    Self::insert_in_bucket(&mut buckets, new_score, node);
                    map_node_bucket.insert(node, new_score);
                    continue;
                }
            }
            buckets.get_mut(&min_score).unwrap().remove(&node);
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
            // Re-evaluate nodes for which the heuristic has changed
            let nodes_to_update = args.td_heuristic().get_nodes_to_update(&primal_graph, node);
            for n in nodes_to_update.iter().copied() {
                recompute_score[n] = true;
            }
            primal_graph[node].clear();
        }
        debug_assert!(order.len() == order.iter().copied().collect::<FxHashSet<usize>>().len());

        Self {
            order,
            width: width as usize,
        }
    }

    fn insert_in_bucket(buckets: &mut FxHashMap<usize, FxHashSet<usize>>, bucket: usize, element: usize) {
        if !buckets.contains_key(&bucket) {
            buckets.insert(bucket, FxHashSet::<usize>::default());
        }
        buckets.get_mut(&bucket).unwrap().insert(element);
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn order(&self) -> &[usize] {
        &self.order
    }
}

impl TDHeuristic {

    fn evaluate_node(&self, graph: &[FxHashSet<usize>], node: usize) -> usize {
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

    fn get_nodes_to_update(&self, graph: &[FxHashSet<usize>], node: usize) -> Vec<usize> {
        match self {
            Self::MinFill => {
                // All nodes at a distance of 2 in the graph can have their min-fill heuristic
                // changed.
                let mut to_update = FxHashSet::<usize>::default();
                for neighbor in graph[node].iter().copied() {
                    to_update.insert(neighbor);
                    for neighbor_of_neighbor in graph[neighbor].iter().copied().filter(|n| *n != node) {
                        to_update.insert(neighbor_of_neighbor);
                    }
                }
                to_update.iter().copied().collect::<Vec<usize>>()
            },
            Self::MinDeg => {
                graph[node].iter().copied().collect::<Vec<usize>>()
            },
        }
    }

    fn max_score(&self, number_var: usize) -> usize {
        match self {
            Self::MinFill => (number_var * (number_var - 1)) / 2,
            Self::MinDeg => number_var,
        }
    }
}
