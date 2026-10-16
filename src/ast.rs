//! # H-Expression abstract syntax tree

use crate::parser::{HExprParser, Rule};
use pest::Parser;

#[derive(Debug, Clone, PartialEq)]
pub enum Hexpr {
    /// Sequential (categorical) composition of hexprs
    Composition(Vec<Hexpr>),
    /// Parallel (tensor) composition of hexprs
    Tensor(Vec<Hexpr>),
    /// A Frobenius spider
    Frobenius {
        sources: Vec<LVar>,
        targets: Vec<LVar>,
    },
    /// An unlabeled identity wire.
    Hole,
    /// A labeled identity wire.
    Wire(Box<Hexpr>),
    /// A named operation
    Operation(Operation),
}

/// Operation names consist of alphanumerics or `-_.*+/|><:=!?`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Operation(pub(crate) String);

impl Operation {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Variable names in a Frobenius expression. Must match `[a-zA-Z0-9-_]+`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Variable(pub(crate) String);

/// A variable occurrence in a Frobenius expression, optionally labeled.
#[derive(Debug, Clone, PartialEq)]
pub struct LVar {
    pub name: Variable,
    pub label: Option<Hexpr>,
}

impl From<Variable> for LVar {
    fn from(name: Variable) -> Self {
        LVar { name, label: None }
    }
}

impl std::str::FromStr for Variable {
    type Err = Box<pest::error::Error<Rule>>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pairs = HExprParser::parse(Rule::variable, s)?;
        let variable_pair = pairs.into_iter().next().unwrap();
        Ok(Variable(variable_pair.as_str().to_string()))
    }
}

impl std::str::FromStr for LVar {
    type Err = crate::parser::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parser::HExprParser::parse_lvar(s)
    }
}

impl std::fmt::Display for LVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)?;
        if let Some(label) = &self.label {
            write!(f, "^{}", label)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for Operation {
    type Err = Box<pest::error::Error<Rule>>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pairs = HExprParser::parse(Rule::operation, s)?;
        let operation_pair = pairs.into_iter().next().unwrap();
        Ok(Operation(operation_pair.as_str().to_string()))
    }
}

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for Hexpr {
    type Err = crate::parser::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parser::HExprParser::parse_hexpr(s)
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
            Hexpr::Frobenius { sources, targets } => {
                // Special case for empty frobenius
                if sources.is_empty() && targets.is_empty() {
                    write!(f, "[]")
                } else {
                    write!(f, "[")?;
                    for (i, var) in sources.iter().enumerate() {
                        if i > 0 {
                            write!(f, " ")?;
                        }
                        write!(f, "{}", var)?;
                    }
                    write!(f, " . ")?;
                    for (i, var) in targets.iter().enumerate() {
                        if i > 0 {
                            write!(f, " ")?;
                        }
                        write!(f, "{}", var)?;
                    }
                    write!(f, "]")
                }
            }
            Hexpr::Hole => write!(f, "_"),
            Hexpr::Wire(label) => write!(f, "^{}", label),
            Hexpr::Operation(name) => write!(f, "{}", name),
        }
    }
}
