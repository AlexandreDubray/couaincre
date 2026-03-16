#[derive(Clone, Copy)]
enum ConstraintType {
    Equality,
    Xor,
}

pub struct Constraint {
    vars: Vec<usize>,
    constraint_type: ConstraintType,
    polarity: bool,
}

impl Constraint {

    pub fn equality(vars: Vec<usize>, polarity: bool) -> Self {
        Self {
            vars,
            constraint_type: ConstraintType::Equality,
            polarity,
        }
    }

    pub fn xor(vars: Vec<usize>, polarity: bool) -> Self {
        Self {
            vars,
            constraint_type: ConstraintType::Xor,
            polarity,
        }
    }

}

impl std::fmt::Display for Constraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.constraint_type {
            ConstraintType::Equality => {
                write!(f, "Equal({}) = {}", self.vars.iter().map(|v| format!("{}", v + 1)).collect::<Vec<String>>().join(","), self.polarity)?;
            },
            ConstraintType::Xor => {
                write!(f, "Xor({}) = {}", self.vars.iter().map(|v| format!("{}", v + 1)).collect::<Vec<String>>().join(","), self.polarity)?;
            },
        }
        Ok(())
    }
}
