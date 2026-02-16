mod problem;
mod tree_decomposition;
mod sampler;
mod restricted;
mod utils;
mod counter;

use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use restricted::RestrictedSolver;
use counter::Counter;

#[derive(Clone, ValueEnum)]
enum TDHeuristic {
    MinFill,
    MinDeg,
}

#[derive(Parser)]
#[clap(name="Couaincre", version, author, about)]
pub struct Args {
    #[clap(short, long, value_parser)]
    input: PathBuf,
    #[clap(short, long, value_enum, default_value_t=TDHeuristic::MinFill)]
    td_heuristic: TDHeuristic,
    #[clap(long, value_enum, default_value_t=Counter::D4)]
    counter: Counter,
}

impl Args {

    pub fn counter(&self) -> &Counter {
        &self.counter
    }
}

fn main() {
    let args = Args::parse();
    let mut restricted_solver = RestrictedSolver::new(args.input.clone());
    restricted_solver.solve(&args);
}
