//! Convert Hexprs to open hypergraphs via a "partial signature".
use std::collections::HashMap;

use open_hypergraphs::lax::{Interface, NodeId, OpenHypergraph};

use crate::ast::{Hexpr, Operation, Variable};

/// A `Signature` is:
///  - a way to extract names from operations
//   - Exact arity/coarity
//   - *Partial* input/output type labels
pub trait Signature<T>: TryFrom<Operation> {
    fn source(&self) -> Vec<Option<T>>;
    fn target(&self) -> Vec<Option<T>>;
}

pub fn try_interpret<O, A: Signature<O>>(
    hexpr: Hexpr,
) -> Result<OpenHypergraph<Option<O>, A>, A::Error> {
    let mut state = OpenHypergraph::empty();
    let mut env = HashMap::new();
    let (sources, targets) = try_interpret_stack(&mut state, &mut env, hexpr)?;
    state.sources = sources;
    state.targets = targets;
    Ok(state)
}

fn try_interpret_stack<O, A: Signature<O>>(
    state: &mut OpenHypergraph<Option<O>, A>,
    _env: &mut HashMap<Variable, NodeId>,
    hexpr: Hexpr,
) -> Result<Interface, A::Error> {
    match hexpr {
        Hexpr::Composition(_hexprs) => todo!(),
        Hexpr::Tensor(_hexprs) => todo!(),
        //Hexpr::Frobenius { sources, targets } => todo!(),
        Hexpr::Operation(operation) => {
            let arr: A = operation.try_into()?;
            let s = arr.source();
            let t = arr.target();
            let (_, interface) = state.new_operation(arr, s, t);
            Ok(interface)
        }
        _ => todo!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Operation;

    enum ArithOp {
        Add,
        Neg,
    }

    #[derive(Debug, thiserror::Error)]
    #[error("{0}")]
    struct Error(String);

    impl TryFrom<Operation> for ArithOp {
        type Error = Error;

        fn try_from(op: Operation) -> Result<Self, Self::Error> {
            match op.0.as_str() {
                "add" => Ok(ArithOp::Add),
                "neg" => Ok(ArithOp::Neg),
                op => Err(Error(format!("invalid op: {}", op))),
            }
        }
    }

    impl Signature<()> for ArithOp {
        fn source(&self) -> Vec<Option<()>> {
            let ob = Some(());
            match self {
                Self::Add => vec![ob, ob],
                Self::Neg => vec![ob],
            }
        }

        fn target(&self) -> Vec<Option<()>> {
            let ob = Some(());
            match self {
                Self::Add => vec![ob],
                Self::Neg => vec![ob],
            }
        }
    }

    #[test]
    fn test_simple_operation() -> anyhow::Result<()> {
        let hexpr = "add".parse()?;
        let result: OpenHypergraph<Option<()>, ArithOp> = try_interpret(hexpr)?;

        assert_eq!(result.sources.len(), 2);
        assert_eq!(result.targets.len(), 1);
        assert_eq!(result.hypergraph.edges.len(), 1);

        Ok(())
    }
}
