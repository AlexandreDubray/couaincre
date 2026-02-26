mod problem;
mod tree_decomposition;
mod sampler;
mod restricted;
mod utils;
mod counter;

use clap::Parser;
use clap_verbosity_flag::{Verbosity, InfoLevel};

use std::path::PathBuf;

use problem::Problem;
use restricted::RestrictedSolver;
use counter::Counter;
use tree_decomposition::TreeDecomposition;
use tree_decomposition::TDHeuristic;

#[derive(Parser)]
#[clap(name="Couaincre", version, author, about)]
pub struct Args {
    #[clap(short, long, value_parser)]
    /// The input CNF in DIMACS format
    input: PathBuf,
    #[clap(long, default_value_t=10)]
    /// Timeout for the pre-processing
    preproc_timeout: usize,
    #[clap(short, long, value_enum, default_value_t=TDHeuristic::MinFill)]
    /// Which heuristic to use during the construction of the tree decomposition
    td_heuristic: TDHeuristic,
    #[clap(long, default_value_t=50)]
    /// Maximum width of a tree decomposition at which it is consider it can be solved exactly
    td_threshold: usize,
    #[clap(long)]
    /// If present, only validate the tree decomposition. Requires the td-validate tools from the
    /// PACE challenge
    td_validate: bool,
    #[clap(long, value_enum, default_value_t=Counter::D4)]
    /// Which model counter to use when computing the model count of restricted and relaxed
    /// formulas
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

    pub fn td_validate(&self) -> bool {
        self.td_validate
    }
}

fn main() {
    let args = Args::parse();
    env_logger::Builder::new().filter_level(args.verbose.log_level_filter()).init();
    utils::check_executables();
    let problem = Problem::new(&args);
    if problem.is_empty() {
        log::info!("UNSAT");
    }
    let td = TreeDecomposition::new(&args, &problem);
}
