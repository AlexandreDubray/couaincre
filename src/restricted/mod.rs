mod restricted;
mod constraint;
mod equals;
mod xor;

use clap::ValueEnum;

pub use restricted::RestrictedSolver;

#[derive(Clone, ValueEnum)]
pub enum EqualityHeuristic {
    MaxDegMostCommon,
    MinContractionDeg,
}

#[derive(Clone, ValueEnum)]
pub enum RestrictedMethod {
    Equality,
    Xor,
}

impl std::fmt::Display for RestrictedMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Equality => {
                write!(f, "equality")?;
            },
            Self::Xor => {
                write!(f, "xor")?;
            },
        }
        Ok(())
    }
}
