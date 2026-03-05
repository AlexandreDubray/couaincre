use rustc_hash::{FxHashSet, FxHashMap};

use crate::Args;
use crate::problem::Problem;
use crate::restricted::{Restriction, RestrictionOp};

fn fill_in_score(graph: &FxHashMap<usize, FxHashSet<usize>>, node: usize) -> usize {
    if graph[&node].is_empty() {
        return 0;
    }
    let number_neighbors = graph[&node].len();
    let mut missing_edges = (number_neighbors * (number_neighbors - 1)) / 2;
    let neighbors = graph[&node].iter().copied().collect::<Vec<usize>>();
    for i in 0..neighbors.len() {
        for j in (i+1)..neighbors.len() {
            if graph[&neighbors[i]].contains(&neighbors[j]) {
                missing_edges -= 1;
            }
        }
    }
    missing_edges
}

fn compute_treewidth(graph: &mut FxHashMap<usize, FxHashSet<usize>>) -> usize {
    let number_var = graph.len();

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
    let mut recompute_score = FxHashSet::<usize>::default();
    // Current minimum score. Since we greedily select nodes based on their minimum score, this
    // give the next bucket to select a node from.
    let mut min_score = usize::MAX;
    // Initialise the buckets
    for (candidate, _) in graph.iter() {
        let score = fill_in_score(&graph, *candidate);
        min_score = min_score.min(score);
        insert_in_bucket(&mut buckets, score, *candidate);
    }
    // We compute the order for each node.
    let mut eliminated = 0;
    let mut treewidth = 0;
    while eliminated != number_var {
        // Finds the next non-empty bucket
        while !buckets.contains_key(&min_score) || buckets.get(&min_score).unwrap().is_empty() {
            min_score += 1;
        }
        // Pop a node from the bucket and recompute its score if needed. If the new score is
        // worst than the computed one, put it in the associated bucket.
        let node = buckets.get_mut(&min_score).unwrap().pop().unwrap();
        if recompute_score.contains(&node) {
            recompute_score.remove(&node);
            let new_score = fill_in_score(&graph, node);
            if new_score > min_score {
                insert_in_bucket(&mut buckets, new_score, node);
                continue;
            }
            min_score = min_score.min(new_score);
        }
        eliminated += 1;

        // Clique size is primal_graph[node] + 1 (the neighbors and the node) but we remove 1 for
        // the treewidth.
        treewidth = treewidth.max(graph[&node].len());
        
        // Apply the node elimination

        // Remove node from the graph (disconnect it from its neighbors) and connect all of its
        // neighbors
        let neighbors = graph[&node].iter().copied().collect::<Vec<usize>>();
        for neighbor in neighbors.iter().copied() {
            graph.get_mut(&neighbor).unwrap().extend(neighbors.iter().copied().filter(|n| *n != neighbor));
            graph.get_mut(&neighbor).unwrap().remove(&node);
        }
        // Flags node for which the heuristic needs to be recomputed
        // All nodes at a distance of 2 in the graph can have their min-fill heuristic
        // changed.
        for neighbor in graph[&node].iter().copied() {
            recompute_score.insert(neighbor);
            for neighbor_of_neighbor in graph[&neighbor].iter().copied().filter(|n| *n != node) {
                recompute_score.insert(neighbor_of_neighbor);
            }
        }
        graph.get_mut(&node).unwrap().clear();
    }
    treewidth
}

fn compute_primal_graph(problem: &Problem) -> FxHashMap<usize, FxHashSet<usize>> {
    let mut graph = FxHashMap::<usize, FxHashSet<usize>>::default();
    for clause in problem.iter_clauses() {
        for i in 0..clause.len() {
            for j in (i+1)..clause.len() {
                let u = clause[i].unsigned_abs() - 1;
                let v = clause[j].unsigned_abs() - 1;
                graph.entry(u).or_default().insert(v);
                graph.entry(v).or_default().insert(u);
            }
        }
    }
    graph
}

pub fn compute_restrictions(args: &Args, mut problem: Problem) -> Vec<Restriction> {
    log::trace!("Computing restrictions for lower bound computation");
    let mut primal_graph = compute_primal_graph(&problem);
    let mut treewidth = compute_treewidth(&mut primal_graph);
    log::info!("Initial treewidth is {}", treewidth);
    let mut restrictions = vec![];
    while treewidth > args.td_threshold {
        primal_graph = compute_primal_graph(&problem);
        let (u, v) = args.contraction_heuristic.edge_to_contract(&primal_graph);
        let pos_u = problem.positive_occurences(u);
        let neg_u = problem.negative_occurences(u);
        let pos_v = problem.positive_occurences(v);
        let neg_v = problem.negative_occurences(v);

        let impact_equal = pos_u.intersection(neg_v).count() + neg_u.intersection(pos_v).count();
        let impact_not_equal = pos_u.intersection(pos_v).count() + neg_u.intersection(neg_v).count();

        if impact_equal > impact_not_equal {
            restrictions.push(Restriction::new(Some(u), Some(v), RestrictionOp::Equal));
            problem.make_equal(u, v);
        } else {
            restrictions.push(Restriction::new(Some(u), Some(v), RestrictionOp::NotEqual));
            problem.make_not_equal(u, v);
        }

        primal_graph = compute_primal_graph(&problem);
        treewidth = compute_treewidth(&mut primal_graph);
        log::trace!("Updated treewidth: {}", treewidth);
    }
    log::trace!("Restrictions computed: {}", restrictions.iter().map(|r| format!("{}", r)).collect::<Vec<String>>().join(", "));
    restrictions
}

fn insert_in_bucket(buckets: &mut FxHashMap<usize, Vec<usize>>, bucket: usize, element: usize) {
    let bucket = buckets.entry(bucket).or_default();
    bucket.push(element);
}
