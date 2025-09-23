use std::fmt::Display;

pub mod decimal;
pub mod number;
// pub mod rational;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sign {
    Positive,
    Negative,
}

impl Sign {
    pub fn negate(self) -> Self {
        match self {
            Sign::Positive => Sign::Negative,
            Sign::Negative => Sign::Positive,
        }
    }
}

impl Display for Sign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sign::Positive => write!(f, ""),
            Sign::Negative => write!(f, "-"),
        }
    }
}
