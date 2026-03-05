mod problem;
mod tree_decomposition;
mod restricted;
mod utils;
mod counter;

use clap::Parser;
use clap_verbosity_flag::{Verbosity, InfoLevel};

use std::path::PathBuf;
use std::time::Instant;
use std::sync::LazyLock;

use restricted::{RestrictedSolver};
use counter::Counter;
use tree_decomposition::ContractionHeuristic;

#[derive(Parser)]
#[clap(name="Couaincre", version, author, about)]
pub struct Args {
    #[clap(short, long, value_parser)]
    /// The input CNF in DIMACS format
    input: PathBuf,
    #[clap(long, default_value_t=10)]
    /// Timeout for the pre-processing
    preproc_timeout: usize,
    #[clap(long, default_value_t=u64::MAX)]
    /// Timeout for the solving
    timeout: u64,
    #[clap(long, default_value_t=50)]
    /// Maximum width of a tree decomposition at which it is consider it can be solved exactly
    td_threshold: usize,
    #[clap(long, value_enum, default_value_t=Counter::D4)]
    /// Which model counter to use when computing the model count of restricted and relaxed
    /// formulas
    counter: Counter,
    #[clap(long, value_enum, default_value_t=ContractionHeuristic::MaxDegMostCommon)]
    contraction_heuristic: ContractionHeuristic,
    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,
}

impl Args {

    pub fn counter(&self) -> &Counter {
        &self.counter
    }
}

pub struct Controller {
    start: Instant,
}

impl Controller {
    fn remaining(&self, timeout: u64) -> u64 {
        let elapsed = self.start.elapsed().as_secs();
        if elapsed >= timeout {
            0
        } else {
            timeout - elapsed
        }
    }

    fn elapsed(&self) -> u64 {
        self.start.elapsed().as_secs()
    }
}

static CTRL: LazyLock<Controller> = LazyLock::new(|| {
    Controller {
        start: Instant::now(),
    }
});

fn main() {
    let args = Args::parse();
    env_logger::Builder::new().filter_level(args.verbose.log_level_filter()).init();
    utils::check_executables(&args);
    let mut restricted_solver = RestrictedSolver::new(&args);
    restricted_solver.solve(&args);
}
