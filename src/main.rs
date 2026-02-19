mod problem;
mod tree_decomposition;
mod sampler;
mod restricted;
mod utils;
mod counter;

use clap::Parser;
use clap_verbosity_flag::{Verbosity, InfoLevel};

use std::path::PathBuf;

use restricted::RestrictedSolver;
use counter::Counter;
use tree_decomposition::{TreeDecomposition, TDHeuristic};

#[derive(Parser)]
#[clap(name="Couaincre", version, author, about)]
pub struct Args {
    #[clap(short, long, value_parser)]
    input: PathBuf,
    #[clap(short, long, value_enum, default_value_t=TDHeuristic::MinFill)]
    td_heuristic: TDHeuristic,
    #[clap(long, value_enum, default_value_t=Counter::D4)]
    counter: Counter,
    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,
}

impl Args {

    pub fn counter(&self) -> &Counter {
        &self.counter
    }

    pub fn td_heuristic(&self) -> &TDHeuristic {
        &self.td_heuristic
    }
}

fn main() {
    let args = Args::parse();
    env_logger::Builder::new().filter_level(args.verbose.log_level_filter()).init();
    let td = TreeDecomposition::new(&args);
    println!("{}", td.width());
}
