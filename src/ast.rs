//! # H-Expression abstract syntax tree

#[derive(Debug, Clone, PartialEq)]
pub enum Hexpr {
    /// Sequential (categorical) composition of hexprs
    Composition(Vec<Hexpr>),
    /// Parallel (tensor) composition of hexprs
    Tensor(Vec<Hexpr>),
    /// A Frobenius spider
    Frobenius {
        inputs: Vec<Variable>,
        outputs: Vec<Variable>,
    },
    Operation(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Variable {
    Named(String),
}

impl std::str::FromStr for Variable {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Variable::Named(s.to_string()))
    }
}

impl std::fmt::Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Variable::Named(name) => write!(f, "{}", name),
        }
    }
}

impl std::fmt::Display for Hexpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Hexpr::Composition(exprs) => {
                write!(f, "(")?;
                for (i, expr) in exprs.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", expr)?;
                }
                write!(f, ")")
            }
            Hexpr::Tensor(exprs) => {
                write!(f, "{{")?;
                for (i, expr) in exprs.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", expr)?;
                }
                write!(f, "}}")
            }
            Hexpr::Frobenius { inputs, outputs } => {
                // Special case for empty frobenius
                if inputs.is_empty() && outputs.is_empty() {
                    write!(f, "[]")
                } else {
                    write!(f, "[")?;
                    for (i, var) in inputs.iter().enumerate() {
                        if i > 0 {
                            write!(f, " ")?;
                        }
                        write!(f, "{}", var)?;
                    }
                    write!(f, " . ")?;
                    for (i, var) in outputs.iter().enumerate() {
                        if i > 0 {
                            write!(f, " ")?;
                        }
                        write!(f, "{}", var)?;
                    }
                    write!(f, "]")
                }
            }
            Hexpr::Operation(name) => write!(f, "{}", name),
        }
    }
}
