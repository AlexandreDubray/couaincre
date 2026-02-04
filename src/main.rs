mod problem;
mod tree_decomposition;

use clap::Parser;
use std::path::PathBuf;

use problem::Problem;
use tree_decomposition::TreeDecomposition;

#[derive(Parser)]
#[clap(name="Couaincre", version, author, about)]
pub struct Args {
    #[clap(short, long, value_parser)]
    input: PathBuf,
    #[clap(short, long, value_parser, default_value_t=5)]
    td_timeout: usize,
}


fn main() {
    let args = Args::parse();
    let problem = Problem::from_file(&args.input);
    let td = TreeDecomposition::from_problem(&problem, args.td_timeout);
}
