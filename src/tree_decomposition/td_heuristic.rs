use clap::ValueEnum;
use rustc_hash::FxHashSet;

#[derive(Clone, ValueEnum)]
pub enum TDHeuristic {
    MinFill,
    MinDeg,
}

impl TDHeuristic {

    pub fn evaluate_node(&self, graph: &[FxHashSet<usize>], node: usize) -> usize {
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

    pub fn flag_nodes_to_update(&self, graph: &[FxHashSet<usize>], node: usize, flags: &mut [bool]) {
        match self {
            Self::MinFill => {
                // All nodes at a distance of 2 in the graph can have their min-fill heuristic
                // changed.
                for neighbor in graph[node].iter().copied() {
                    flags[neighbor] = true;
                    for neighbor_of_neighbor in graph[neighbor].iter().copied().filter(|n| *n != node) {
                        flags[neighbor_of_neighbor] = true;
                    }
                }
            },
            Self::MinDeg => {
                for neighbor in graph[node].iter().copied() {
                    flags[neighbor] = true;
                }
            },
        }
    }

}

impl std::fmt::Display for TDHeuristic {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TDHeuristic::MinFill => write!(f, "min-fill")?,
            TDHeuristic::MinDeg => write!(f, "min-deg")?,
        };
        Ok(())
    }
}
