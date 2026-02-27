use std::path::PathBuf;
use std::time::Instant;
use std::collections::BinaryHeap;

use rustc_hash::{FxHashMap, FxHashSet};

use super::*;
use super::heuristics::*;
use crate::utils::*;
use crate::problem::Problem;
use crate::Args;
use crate::tree_decomposition::TreeDecomposition;


pub struct RestrictedSolver {
    problem: Problem,
    exact: bool,
    bounds: Vec<(u64, f64)>,
    start_time: Instant,
}

impl RestrictedSolver {

    pub fn new(args: &Args) -> Self {
        let problem = Problem::new(args);
        Self {
            problem,
            exact: false,
            start_time: Instant::now(),
            bounds: vec![],
        }
    }

    pub fn solve(&mut self, args: &Args) {
        let mut restrictions = self.get_restrictions(args, &self.problem);
        for _ in 0..restrictions.len() {
            if let Some(lb) = args.counter().lower_bound(&self.problem, &restrictions) {
                log::info!("Lower bound on the log-model-count {}", lb);
                self.bounds.push((self.start_time.elapsed().as_secs(), lb));
            } 
            let _ = restrictions.pop();
        }
    }

    fn get_restrictions(&self, args: &Args, problem: &Problem) -> Vec<Restriction> {
        let mut td = TreeDecomposition::new(args, problem);
        let mut init_scores = FxHashMap::<(usize, usize), usize>::default();
        for bag in td.iter_bags() {
            let bag_vec = td[bag].iter().copied().collect::<Vec<usize>>();
            for i in 0..bag_vec.len() {
                for j in (i+1)..bag_vec.len() {
                    let u = bag_vec[i].min(bag_vec[j]);
                    let v = bag_vec[i].max(bag_vec[j]);
                    let entry = init_scores.entry((u, v)).or_default();
                    match args.restricted_heuristic {
                        RestrictionHeuristic::Spread => *entry += 1,
                        RestrictionHeuristic::Size => *entry = (*entry).max(bag_vec.len()),
                    }
                }
            }
        }
        let mut heap = BinaryHeap::<(usize, usize, usize)>::new();
        for ((u, v), score) in init_scores.iter() {
            heap.push((*score, *u, *v));
        }

        let mut occurence_map = FxHashMap::<(usize, usize), Vec<usize>>::default();
        while td.width() > args.td_threshold {
            let (u, v, _) = heap.pop().unwrap();
            let pos_u = problem.positive_occurences(u);
            let neg_u = problem.negative_occurences(u);
            let pos_v = problem.positive_occurences(v);
            let neg_v = problem.negative_occurences(v);
            let total_clause_impacted = pos_u.union(neg_u).collect::<FxHashSet<_>>().union(&pos_v.union(neg_v).collect::<FxHashSet<_>>()).count();
            let nb_occ_u = pos_u.len() + neg_u.len();
            let nb_occ_v = pos_v.len() + neg_v.len();
            let similar_occurences = pos_u.intersection(pos_v).count() + neg_u.intersection(neg_v).count();
            let different_occurences = pos_u.intersection(neg_v).count() + neg_u.intersection(pos_v).count();


        }
        vec![]
    }
}
