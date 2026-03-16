mod restricted;

use clap::ValueEnum;

pub use restricted::RestrictedSolver;

#[derive(Clone, ValueEnum)]
pub enum RestrictedMethod {
    Equality,
    Xor,
}

#[derive(Clone, Copy)]
pub enum RestrictionOp {
    Equal,
}

pub struct Restriction {
    vars: Vec<usize>,
    op: RestrictionOp,
}

impl Restriction {

    pub fn new(vars: Vec<usize>, op: RestrictionOp) -> Self {
        Self {
            vars,
            op,
        }
    }

    pub fn vars(&self) -> &Vec<usize> {
        &self.vars
    }

    pub fn op(&self) -> RestrictionOp {
        self.op
    }

    pub fn number_of_encoding_clauses(&self) -> usize {
        match self.op {
            RestrictionOp::Equal => {
                2*(self.vars.len() - 1)
            },
        }
    }

    pub fn to_dimacs_lines(&self) -> String {
        match self.op {
            RestrictionOp::Equal => {
                let mut out = String::new();
                for i in 0..(self.vars.len() - 1) {
                    let x = (self.vars[i] + 1) as isize;
                    let y = (self.vars[i+1] + 1) as isize;
                    out.push_str(&format!("{} {} 0\n {} {} 0\n", -x, y, x, -y));
                }
                out
            },
        }
    }
}

impl std::fmt::Display for Restriction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.op {
            RestrictionOp::Equal => {
                write!(f, "Equal({})", self.vars.iter().map(|v| format!("{}", v + 1)).collect::<Vec<String>>().join(","))?;
            },
        }
        Ok(())
    }
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
