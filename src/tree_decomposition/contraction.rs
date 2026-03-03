use clap::ValueEnum;
use rustc_hash::{FxHashMap, FxHashSet};

#[derive(Clone, ValueEnum)]
pub enum ContractionHeuristic {
    MaxDegMostCommon,
    MinContractionDeg,
}

impl ContractionHeuristic {

    pub fn edge_to_contract(&self, graph: &FxHashMap<usize, FxHashSet<usize>>) -> (usize, usize) {
        match self {
            Self::MaxDegMostCommon => {
                // First we find the node with the highest degree
                let u = *graph.iter().filter(|(_, ns)| !ns.is_empty()).map(|(node, neighbors)| (neighbors.len(), node)).max().unwrap().1;
                // Second we select the node that shares the most edges with u
                let v = graph[&u].iter().map(|v| (graph[&u].intersection(&graph[v]).count(), *v)).max().unwrap().1;
                (u, v)
            },
            Self::MinContractionDeg => {
                let mut u = 0;
                let mut v = 0;
                let mut min_deg = usize::MAX;
                for (node, neighbors) in graph.iter().filter(|(_, ns)| !ns.is_empty()) {
                    for neighbor in neighbors.iter().copied().filter(|v| !graph[v].is_empty() && *v > *node) {
                        let contracted_node_deg = graph[node].union(&graph[&neighbor]).count() - 2;
                        if contracted_node_deg < min_deg {
                            u = *node;
                            v = neighbor;
                            min_deg = contracted_node_deg;
                        }
                    }
                }
                (u, v)
            },
        }
    }
}
