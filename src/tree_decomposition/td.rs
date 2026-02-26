use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::collections::VecDeque;
use std::process::{Command, Stdio};

use rustc_hash::{FxHashSet, FxHashMap};

use crate::Args;
use crate::utils::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct BagIndex(pub usize);

pub struct TreeDecomposition {
    bags: Vec<FxHashSet<usize>>,
    width: usize,
    children: Vec<Vec<BagIndex>>,
    roots: Vec<BagIndex>,
}

impl TreeDecomposition {

    // Implementation for the creation of the tree decomposition

    /// Creates a tree decomposition of the primal graph of the CNF formula in args.input using a
    /// greedy heuristic. The tree-decomposition is computed by finding an elimination order for
    /// the graph's node and creating a chordal graph.
    pub fn new(args: &Args) -> Self {
        log::trace!("Computing tree decomposition...");
        log::trace!("Computing elimination order...");
        let number_var = number_var_from_dimacs(args.input.clone());
        log::trace!("{} variables in the problem", number_var);

        log::trace!("Creating the primal graph...");
        let mut primal_graph: Vec<FxHashSet<usize>> = (0..number_var).map(|_| FxHashSet::default()).collect();

        let reader = BufReader::new(File::open(args.input.clone()).unwrap());
        for line in reader.lines() {
            let line = line.unwrap();
            if line.is_empty() || line.starts_with("p cnf") || line.starts_with('c') {
                continue;
            }
            let variables = line.split_whitespace().map(|l| l.parse::<isize>().unwrap().unsigned_abs()).collect::<Vec<usize>>();
            for i in 0..(variables.len() - 1) {
                for j in (i+1)..(variables.len() - 1) {
                    let u = variables[i] - 1;
                    let v = variables[j] - 1;
                    primal_graph[u].insert(v);
                    primal_graph[v].insert(u);
                }
            }
        }

        log::trace!("Primal graph constructed. Computing elimination order using the {} heuristic...", args.td_heuristic());
        if args.td_validate() {
            Self::write_primal_graph(&primal_graph);
        }

        // order[i] give the node eliminated at iteration i during the elimination process
        let mut order: Vec<usize> = vec![0; number_var];
        // nodes_order[node] gives the elimination order of node
        let mut nodes_order = vec![0; number_var];
        // Bags of the tree decomposition. At the beginning, we create one bag per node in the
        // graph and then reduce the decomposition by merging bags.
        let mut bags: Vec<FxHashSet<usize>> = vec![];

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
        for ord in 0..number_var {
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
                    min_score = min_score.min(new_score);
                    Self::insert_in_bucket(&mut buckets, new_score, node);
                    continue;
                }
            }
            nodes_order[node] = ord;
            order[ord] = node;

            // Creates the bag with the node and its neighbors
            let mut bag = FxHashSet::<usize>::default();
            bag.insert(node);
            bag.extend(&primal_graph[node]);
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
        let (roots, children) = Self::construct_tree(&mut bags, &nodes_order);
        let width = bags.iter().map(|b| b.len()).max().unwrap() - 1;
        log::trace!("Tree decomposition of size {} with {} bags", width, bags.len());
        if args.td_validate() {
            log::trace!("Validating the tree decomposition");
            Self::write_tree_decomposition(&bags, &children, width, primal_graph.len());
            Self::validate_tree_decomposition();
        }
        Self {
            bags,
            width,
            children,
            roots,
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
    fn construct_tree(bags: &mut Vec<FxHashSet<usize>>, nodes_order: &[usize]) -> (Vec<BagIndex>, Vec<Vec<BagIndex>>) {
        // Each node has a single parent which is defined a follow. Given the bag {v_1, ..., v_n} U {x}
        // created for node x, it has a link to the bag associated with node v_i with i such that
        //      v_i is the node eliminated the closest to x and,
        //      v_i is eliminated after x

        log::trace!("Nodes order is {:?}", nodes_order);
        log::trace!("Bags are \n\t{}", bags.iter().enumerate().map(|(i, b)| format!("Bags {}: {:?}",i, b)).collect::<Vec<String>>().join("\n\t"));
        // For each bag, store the set of children of the node
        let mut children = vec![Vec::<BagIndex>::default(); bags.len()];
        let mut parents: Vec<Option<BagIndex>> = vec![None; bags.len()];
        for node in 0..nodes_order.len() {
            // Bag associated with this node
            let bag_id = nodes_order[node];
            if let Some(parent) = bags[bag_id].iter().copied().map(|n| nodes_order[n]).filter(|order| *order > bag_id).min() {
                log::trace!("Parent of bag {} is {}", bag_id, parent);
                children[parent].push(BagIndex(bag_id));
                parents[bag_id] = Some(BagIndex(parent));
            }
        }
        // Compressing the tree decomposition. A bag can be merged with its parent if it is a
        // subset of it.
        // We merge bags in a bottom-up fashion to do a single pass and avoid recursivity (i.e., a
        // parent can be merged with the children of its children).

        log::trace!("Minimizing the tree decomposition");
        let mut roots = vec![];
        // Queue of bags to consider, implemented as a FIFO queue
        let mut queue = VecDeque::<BagIndex>::new();
        // Flag to indicate if the bag is already queued.
        let mut queued = vec![false; bags.len()];
        // We insert every leaf bags. We know that at least the bag 0 is a leaf
        for bag_id in (0..bags.len()).filter(|b| children[*b].is_empty()) {
            queued[bag_id] = true;
            queue.push_back(BagIndex(bag_id));
        }
        while !queue.is_empty() {
            let bag = queue.pop_front().unwrap();
            if parents[bag.0].is_none() {
                log::trace!("Poping root - bag {}", bag.0);
                roots.push(bag);
                continue;
            }
            log::trace!("Poping internal leaf - bag {}", bag.0);
            let parent = parents[bag.0].unwrap();
            log::trace!("Is subset of its parent? {}", bags[bag.0].is_subset(&bags[parent.0]));
            // If the node is a subset of its parent, we remove node and redirect its children to
            // its parent. Note that this work incrementally because if "child" is not a subset of
            // "node", then it is not a subset of "parent" (or it break tree-decomposition
            // conditions)
            if bags[bag.0].is_subset(&bags[parent.0]) {
                children[parent.0].remove(bag.0);
                for i in 0..children[bag.0].len() {
                    let child = children[bag.0][i];
                    children[parent.0].push(child);
                }
                children[bag.0].clear();
                bags[bag.0].clear();
            }
            if !queued[parent.0] {
                queued[parent.0] = true;
                queue.push_back(parent);
            }
        }

        let mut map = FxHashMap::<BagIndex, BagIndex>::default();
        let mut new_index = 0;
        for (i, bag) in bags.iter().enumerate() {
            if !bag.is_empty() {
                map.insert(BagIndex(i), BagIndex(new_index));
                new_index += 1;
            }
        }
        for i in (0..bags.len()).rev() {
            if bags[i].is_empty() {
                bags.swap_remove(i);
                children.swap_remove(i);
            }
        }
        (roots, children)
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

    fn write_tree_decomposition(bags: &[FxHashSet<usize>], children: &[Vec<BagIndex>], width: usize, number_nodes: usize) {
        let mut file = File::create("decomp.td").unwrap();
        writeln!(file, "s td {} {} {}", bags.len(), width + 1, number_nodes).unwrap();
        for (b_id, bag) in bags.iter().enumerate() {
            writeln!(file, "b {} {}", b_id + 1, bag.iter().map(|node| format!("{}", node + 1)).collect::<Vec<String>>().join(" ")).unwrap();
        }
        for (b_id, children_bag) in children.iter().enumerate() {
            for child in children_bag.iter().copied() {
                writeln!(file, "{} {}", b_id + 1, child.0 + 1).unwrap();
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
            std::process::exit(1);
        }
    }


    // Implemtnation of modifying functions for TD during restriction and relaxations

    pub fn number_roots(&self) -> usize {
        self.roots.len()
    }

    pub fn get_root_at(&self, root_id: usize) -> BagIndex {
        self.roots[root_id]
    }

    /// Merge two variables in a bag and its children (recursively)
    pub fn merge_variables(&mut self, x: usize, y: usize, bag_id: BagIndex) {
        let mut q = vec![bag_id];
        while let Some(b_id) = q.pop() {
            if self[b_id].contains(&x) && self[b_id].contains(&y) {
                self[b_id].remove(&y);
                for child in self.iter_children(b_id) {
                    q.push(child);
                }
            }
        }
    }

    pub fn iter_children(&self, bag: BagIndex) -> impl Iterator<Item = BagIndex> {
        self.children[bag.0].iter().copied()
    }

}

impl std::ops::Index<BagIndex> for TreeDecomposition {
    type Output = FxHashSet<usize>;

    fn index(&self, index: BagIndex) -> &Self::Output {
        &self.bags[index.0]
    }
}

impl std::ops::IndexMut<BagIndex> for TreeDecomposition {
    fn index_mut(&mut self, index: BagIndex) -> &mut Self::Output {
        &mut self.bags[index.0]
    }
}
