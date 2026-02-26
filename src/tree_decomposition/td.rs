use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::collections::VecDeque;
use std::process::{Command, Stdio};

use rustc_hash::{FxHashSet, FxHashMap};

use crate::Args;
use crate::problem::Problem;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct BagIndex(pub usize);

pub struct TreeDecomposition {
    bags: Vec<FxHashSet<isize>>,
    width: usize,
    edges: Vec<FxHashSet<BagIndex>>,
    root: Option<BagIndex>,
}

impl TreeDecomposition {

    // Implementation for the creation of the tree decomposition

    /// Creates a tree decomposition of the primal graph of the CNF formula in args.input using a
    /// greedy heuristic. The tree-decomposition is computed by finding an elimination order for
    /// the graph's node and creating a chordal graph.
    pub fn new(args: &Args, problem: &Problem) -> Self {
        log::trace!("Computing tree decomposition...");
        log::trace!("Computing elimination order...");
        let number_var = problem.number_var();
        log::trace!("{} variables in the problem", number_var);

        log::trace!("Creating the primal graph...");
        let mut primal_graph: Vec<FxHashSet<usize>> = (0..number_var).map(|_| FxHashSet::default()).collect();

        for clause in problem.iter_clauses() {
            log::trace!("Clause from problem {:?}", clause);
            for i in 0..clause.len() {
                for j in (i+1)..clause.len() {
                    let u = clause[i].unsigned_abs() - 1;
                    let v = clause[j].unsigned_abs() - 1;
                    log::trace!("Adding edge between {} and {}", u, v);
                    primal_graph[u].insert(v);
                    primal_graph[v].insert(u);
                }
            }
        }

        log::trace!("Primal graph constructed. Computing elimination order using the {} heuristic...", args.td_heuristic());
        log::trace!("Primal graph adjacency:\n{}", primal_graph.iter().map(|s| format!("{:?}", s)).collect::<Vec<String>>().join("\n"));
        if args.td_validate() {
            Self::write_primal_graph(&primal_graph);
        }

        // order[i] give the node eliminated at iteration i during the elimination process
        let mut order: Vec<usize> = vec![0; number_var];
        // nodes_order[node] gives the elimination order of node
        let mut nodes_order = vec![0; number_var];
        // Bags of the tree decomposition. At the beginning, we create one bag per node in the
        // graph and then reduce the decomposition by merging bags.
        let mut bags: Vec<FxHashSet<isize>> = vec![];

        // Buckets used to compute the order. We place each node in a bucket corresponding to its
        // heuristic score. Then, we can process nodes in increasing order of bucket.
        let mut buckets = FxHashMap::<usize, Vec<usize>>::default();
        // Flag to indicate if the score of a node must be recomputed. We lazily recompute them
        // when poping them from the bucket as this computation can be long (e.g., min-fill)
        //
        // /!\ Note: since we lazily recompute the score when popping nodes from the bucket, we
        // might process nodes in non-greedy order (i.e., process a node that has a worst-score
        // than another non-processed node). However, re-computing the heuristic at each
        // modification of the primal graph is not scalable for large graphs.
        //
        let mut recompute_score = vec![false; number_var];
        // Current minimum score. Since we greedily select nodes based on their minimum score, this
        // give the next bucket to select a node from.
        let mut min_score = usize::MAX;
        // Initialise the buckets
        for candidate in 0..number_var {
            let score = args.td_heuristic().evaluate_node(&primal_graph, candidate);
            min_score = min_score.min(score);
            Self::insert_in_bucket(&mut buckets, score, candidate);
        }
        // We compute the order for each node.
        while bags.len() != number_var {
            // Finds the next non-empty bucket
            while !buckets.contains_key(&min_score) || buckets.get(&min_score).unwrap().is_empty() {
                min_score += 1;
            }
            // Pop a node from the bucket and recompute its score if needed. If the new score is
            // worst than the computed one, put it in the associated bucket.
            let node = buckets.get_mut(&min_score).unwrap().pop().unwrap();
            if recompute_score[node] {
                recompute_score[node] = false;
                let new_score = args.td_heuristic().evaluate_node(&primal_graph, node);
                if new_score > min_score {
                    Self::insert_in_bucket(&mut buckets, new_score, node);
                    continue;
                }
                min_score = min_score.min(new_score);
            }
            let ord = bags.len();
            log::trace!("Elimination process. Eliminating node {} at order {}", node + 1, ord);
            nodes_order[node] = ord;
            order[ord] = node;

            // Creates the bag with the node and its neighbors
            let mut bag = FxHashSet::<isize>::default();
            bag.insert(node as isize + 1);
            bag.extend(primal_graph[node].iter().map(|n| (*n + 1) as isize));
            log::trace!("New bag is {:?}", bag);
            bags.push(bag);
            
            // Apply the node elimination

            // Remove node from the graph (disconnect it from its neighbors) and connect all of its
            // neighbors
            let neighbors = primal_graph[node].iter().copied().collect::<Vec<usize>>();
            for neighbor in neighbors.iter().copied() {
                primal_graph[neighbor].extend(neighbors.iter().copied().filter(|n| *n != neighbor));
                primal_graph[neighbor].remove(&node);
            }
            // Flags node for which the heuristic needs to be recomputed
            args.td_heuristic().flag_nodes_to_update(&primal_graph, node, &mut recompute_score);
            primal_graph[node].clear();
        }
        // Computes the edges between the bags and reduce the resulting tree
        let edges = Self::compute_td_edges(&mut bags, &nodes_order);
        let width = bags.iter().map(|b| b.len()).max().unwrap() - 1;
        log::info!("Tree decomposition of size {} with {} bags", width, bags.len());
        if args.td_validate() {
            log::trace!("Validating the tree decomposition after minimisation");
            Self::write_tree_decomposition(&bags, &edges, width, primal_graph.len());
            Self::validate_tree_decomposition();
        }
        Self {
            bags,
            width,
            edges,
            root: None,
        }
    }

    fn insert_in_bucket(buckets: &mut FxHashMap<usize, Vec<usize>>, bucket: usize, element: usize) {
        let bucket = buckets.entry(bucket).or_default();
        bucket.push(element);
    }

    pub fn width(&self) -> usize {
        self.width
    }

    /// Given a set of bags and a node elimination order, computes the tree structure and reduce
    /// the tree.
    /// bags are constructed during the elimination process, and nodes_order give for each node its
    /// elimination order (i.e., its bag id)
    ///
    /// This functions modify the bags and returns the vector of children pointers.
    fn compute_td_edges(bags: &mut Vec<FxHashSet<isize>>, nodes_order: &[usize]) -> Vec<FxHashSet<BagIndex>> {
        // We define a link between a bag created when eliminating node x as follows.
        // Given the bag {v_1, ..., v_n} U {x}, it has a link to the bag associated with node v_i with i such that
        //      v_i is the node eliminated the closest to x and,
        //      v_i is eliminated after x

        log::trace!("Nodes order is {:?}", nodes_order);
        log::trace!("Bags are \n\t{}", bags.iter().enumerate().map(|(i, b)| format!("Bags {}: {:?}",i, b)).collect::<Vec<String>>().join("\n\t"));
        // For each bag, store the set of children of the node
        let mut edges = vec![FxHashSet::<BagIndex>::default(); bags.len()];
        for n in 0..nodes_order.len() {
            // Bag associated with this node
            log::trace!("Finding neighbor of bag for node {} with index {}", n, nodes_order[n]);
            let bag = BagIndex(nodes_order[n]);
            // We connect this bag to only one other. We find the node (different from n) in the
            // bag this is the closest in the elimination order from n AND eliminated after n.
            // We map each node to its order, filter the ones eliminated before n and take the min.
            // The order give us the bag_id
            if let Some(b) = bags[bag.0].iter().copied().map(|other_node| nodes_order[(other_node - 1) as usize]).filter(|order| *order > bag.0).min() {
                log::trace!("Linking bag {} with bag {}", bag.0, b);
                edges[bag.0].insert(BagIndex(b));
                edges[b].insert(bag);
            }
        }

        log::trace!("Minimizing the tree decomposition");
        // Compressing the tree decomposition. A bag can be merged with its neighbor if it is a
        // subset of it.
        let mut parents = (0..bags.len()).collect::<Vec<usize>>();

        for bag in 0..bags.len() {
            for neighbor in edges[bag].iter() {
                if bags[bag].is_subset(&bags[neighbor.0]) {
                    let root_bag = Self::find_set(bag, &mut parents);
                    let root_neighbor = Self::find_set(neighbor.0, &mut parents);
                    if root_bag != root_neighbor {
                        parents[root_bag] = root_neighbor;
                        break;
                    }
                }
            }
        }
        log::trace!("Parents of the union-find for tree decomposition minising {:?}", parents);

        let mut map_bag = FxHashMap::<usize, usize>::default();
        let mut new_bag_index = 0;
        for bag in 0..bags.len() {
            if bag == parents[bag] {
                map_bag.insert(bag, new_bag_index);
                new_bag_index += 1;
            }
        }
        // maps the new set of edges for each bag (using the new ids)
        let mut new_edges = vec![FxHashSet::<BagIndex>::default(); new_bag_index];

        for bag in 0..bags.len() {
            // Representative of bag
            let root_bag = parents[bag];
            // The neighbors of the representative, might already exist from another node maps to
            // the root
            let new_neighbors = &mut new_edges[*map_bag.get(&root_bag).unwrap()];

            // For all neighbors, maps their represntative to the adjacency of this one, only if it
            // is not mapped to the same root.
            for neighbor in edges[bag].iter().copied() {
                let root_neighbor = parents[neighbor.0];
                if root_neighbor != root_bag {
                    new_neighbors.insert(BagIndex(*map_bag.get(&root_neighbor).unwrap()));
                }
            }
        }

        let mut bag = 0;
        bags.retain(|_| {
            let should_keep = parents[bag] == bag;
            bag += 1;
            should_keep
        });

        new_edges
    }

    fn find_set(node: usize, parents: &mut Vec<usize>) -> usize {
        if node == parents[node] {
            return node;
        }
        let new_parent = Self::find_set(parents[node], parents);
        parents[node] = new_parent;
        new_parent
    }

    fn write_primal_graph(primal_graph: &[FxHashSet<usize>]) {
        let mut edges: Vec<(usize, usize)> = vec![];
        for (u, neighbors) in  primal_graph.iter().enumerate() {
            for v in neighbors.iter().copied() {
                if u < v {
                    edges.push((u + 1, v + 1));
                }
            }
        }
        let mut file = File::create("primal.gr").unwrap();
        writeln!(file, "p tw {} {}", primal_graph.len(), edges.len()).unwrap();
        write!(file, "{}", edges.iter().map(|(u, v)| format!("{} {}", u, v)).collect::<Vec<String>>().join("\n")).unwrap();
    }

    fn write_tree_decomposition(bags: &[FxHashSet<isize>], edges: &[FxHashSet<BagIndex>], width: usize, number_nodes: usize) {
        let mut file = File::create("decomp.td").unwrap();
        writeln!(file, "s td {} {} {}", bags.len(), width + 1, number_nodes).unwrap();
        for (b_id, bag) in bags.iter().enumerate() {
            writeln!(file, "b {} {}", b_id + 1, bag.iter().map(|node| format!("{}", node)).collect::<Vec<String>>().join(" ")).unwrap();
        }
        for (b_id, neighbors) in edges.iter().enumerate() {
            for neighbor in neighbors.iter().copied() {
                if b_id < neighbor.0 {
                    writeln!(file, "{} {}", b_id + 1, neighbor.0 + 1).unwrap();
                }
            }
        }
    }

    fn validate_tree_decomposition() {
        let process_status = Command::new("td-validate")
            .args(["primal.gr", "decomp.td"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .status()
            .expect("Failed to execute td-validate command");
        if !process_status.success() {
            log::error!("Tree decomposition not valid");
            std::process::exit(1);
        }
        log::info!("Tree decomposition valid");
    }

    // Implemtnation of modifying functions for TD during restriction and relaxations

    /// Merge two variables in a bag and its children (recursively)
    pub fn merge_variables(&mut self, x: usize, y: usize, bag_id: BagIndex) {
    }

}

impl std::ops::Index<BagIndex> for TreeDecomposition {
    type Output = FxHashSet<isize>;

    fn index(&self, index: BagIndex) -> &Self::Output {
        &self.bags[index.0]
    }
}

impl std::ops::IndexMut<BagIndex> for TreeDecomposition {
    fn index_mut(&mut self, index: BagIndex) -> &mut Self::Output {
        &mut self.bags[index.0]
    }
}
