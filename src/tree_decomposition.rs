use crate::problem::Problem;
use std::process::{Command, Stdio};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct BagIndex(pub usize);

pub struct TreeDecomposition {
    width: usize,
    bags: Vec<Vec<usize>>,
    edges: Vec<Vec<BagIndex>>,
}

impl TreeDecomposition {

    pub fn from_problem(problem: &Problem, timeout: usize) -> Self {
        let primal_file = "primal.gr";
        println!("Computing tree decomposition");
        println!("Writing primal graph to file");
        problem.primal_graph_to_file(primal_file);
        println!("Launching Flow cutter for {} seconds", timeout);
        let flowcutter_output = Command::new("timeout")
            .arg(timeout.to_string())
            .arg(String::from("flow_cutter_pace17"))
            .arg(String::from(primal_file))
            .stdout(Stdio::piped())
            .output()
            .unwrap();
        let tree_decomposition_str = String::from_utf8(flowcutter_output.stdout).unwrap();
        let mut bags: Vec<Vec<usize>> = vec![];
        let mut edges: Vec<Vec<BagIndex>> = vec![];
        let mut width = 0;
        for line in tree_decomposition_str.split('\n') {
            if line.starts_with('c') || line.is_empty() {
                continue;
            } else if line.starts_with("s td") {
                let split = line.split_whitespace().collect::<Vec<&str>>();
                let number_bags = split[2].parse::<usize>().unwrap();
                bags.resize(number_bags, vec![]);
                edges.resize(number_bags, vec![]);
                width = split[3].parse::<usize>().unwrap() - 1;
            } else if line.starts_with('b') {
                bags.push(line.split_whitespace().skip(1).map(|x| x.parse::<usize>().unwrap()).collect::<Vec<usize>>());
            } else {
                let es = line.split_whitespace().map(|x| BagIndex(x.parse::<usize>().unwrap() - 1)).collect::<Vec<BagIndex>>();
                edges[es[0].0].push(es[1]);
                edges[es[1].0].push(es[0]);
            }
        }
        Self {
            width,
            bags,
            edges,
        }
    }
}
