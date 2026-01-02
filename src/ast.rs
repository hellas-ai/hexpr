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
    /// A named operation
    Operation(Operation),
}

/// Operation names, must match [a-zA-Z0-9-_.*+-/|]+
#[derive(Debug, Clone, PartialEq)]
pub struct Operation(pub(crate) String);

/// Variable names in a Frobenius expression. Must match `[a-zA-Z0-9-_]+`.
#[derive(Debug, Clone, PartialEq)]
pub struct Variable(pub(crate) String);

impl std::str::FromStr for Variable {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Variable(s.to_string()))
    }
}

impl std::fmt::Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for Operation {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Operation(s.to_string()))
    }
}

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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
