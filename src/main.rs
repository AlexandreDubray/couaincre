mod problem;
mod tree_decomposition;
mod sampler;

use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use std::time::Instant;

use problem::Problem;
use tree_decomposition::TreeDecomposition;

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
}


fn main() {
    let args = Args::parse();
    let n = 1000;
    let timer = Instant::now();
    let models = sampler::sample_solutions(args.input, n);
    println!("Sample models in {} seconds", timer.elapsed().as_secs());
    if models.len() < n {
        println!("Model count is {}", models.len());
    }
    /*
    for model in models.iter() {
        println!("{:?}", model);
    }
    */
}
