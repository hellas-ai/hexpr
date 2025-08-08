#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Composition(Vec<Expr>),
    Tensor(Vec<Expr>),
    Frobenius { inputs: Vec<Variable>, outputs: Vec<Variable> },
    Operation(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Variable {
    Named(String),
    Anonymous,
}

impl std::str::FromStr for Variable {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "_" {
            Ok(Variable::Anonymous)
        } else {
            Ok(Variable::Named(s.to_string()))
        }
    }
}

impl std::fmt::Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Variable::Named(name) => write!(f, "{}", name),
            Variable::Anonymous => write!(f, "_"),
        }
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Composition(exprs) => {
                write!(f, "(")?;
                for (i, expr) in exprs.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", expr)?;
                }
                write!(f, ")")
            }
            Expr::Tensor(exprs) => {
                write!(f, "{{")?;
                for (i, expr) in exprs.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", expr)?;
                }
                write!(f, "}}")
            }
            Expr::Frobenius { inputs, outputs } => {
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
            Expr::Operation(name) => write!(f, "{}", name),
        }
    }
}