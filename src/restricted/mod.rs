mod restricted;

pub use restricted::RestrictedSolver;

pub enum RestrictionOp {
    Equal,
    NotEqual,
    AssignTrue,
    AssignFalse,
}

pub struct Restriction {
    x: Option<usize>,
    y: Option<usize>,
    op: RestrictionOp,
}

impl Restriction {

    pub fn new(x: Option<usize>, y: Option<usize>, op: RestrictionOp) -> Self {
        Self {
            x,
            y,
            op,
        }
    }

    pub fn number_of_encoding_clauses(&self) -> usize {
        match self.op {
            RestrictionOp::AssignTrue => {
                1
            },
            RestrictionOp::AssignFalse => {
                1
            },
            RestrictionOp::Equal => {
                2
            },
            RestrictionOp::NotEqual => {
                2
            },
        }
    }

    pub fn to_dimacs_lines(&self) -> String {
        match self.op {
            RestrictionOp::AssignTrue => {
                debug_assert!(self.x.is_some() && self.y.is_none());
                format!("{} 0", self.x.unwrap() + 1)
            },
            RestrictionOp::AssignFalse => {
                debug_assert!(self.x.is_some() && self.y.is_none());
                format!("-{} 0", self.x.unwrap() + 1)
            },
            RestrictionOp::Equal => {
                debug_assert!(self.x.is_some() && self.y.is_some());
                format!("-{} {} 0\n {} -{} 0", self.x.unwrap() + 1, self.y.unwrap() + 1, self.x.unwrap() + 1, self.y.unwrap() + 1)
            },
            RestrictionOp::NotEqual => {
                debug_assert!(self.x.is_some() && self.y.is_some());
                format!("{} {} 0\n -{} -{} 0", self.x.unwrap() + 1, self.y.unwrap() + 1, self.x.unwrap() + 1, self.y.unwrap() + 1)
            },
        }
    }
}

impl std::fmt::Display for Restriction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.op {
            RestrictionOp::AssignTrue => {
                write!(f, "{} = T", self.x.unwrap() + 1)?;
            },
            RestrictionOp::AssignFalse => {
                write!(f, "{} = F", self.x.unwrap() + 1)?;
            },
            RestrictionOp::Equal => {
                write!(f, "{} = {}", self.x.unwrap() + 1, self.y.unwrap() + 1)?;
            },
            RestrictionOp::NotEqual => {
                write!(f, "{} != {}", self.x.unwrap() + 1, self.y.unwrap() + 1)?;
            }
        }
        Ok(())
    }
}
