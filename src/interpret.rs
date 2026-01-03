//! Convert Hexprs to open hypergraphs via a "partial signature".
use std::collections::HashMap;
use std::fmt::Display;

use open_hypergraphs::lax::{Interface, NodeId, OpenHypergraph};

use crate::ast::{Hexpr, Operation, Variable};

/// A `Signature` is:
///  - a way to extract names from operations
//   - Exact arity/coarity
//   - *Partial* input/output type labels
pub trait Signature<T>: for<'a> TryFrom<&'a Operation> {
    fn source(&self) -> Vec<Option<T>>;
    fn target(&self) -> Vec<Option<T>>;
}

#[derive(Debug)]
pub enum Error<E> {
    /// Composition of two hexprs had incorrect arity according to the signature
    Composition(Hexpr, Hexpr),
    /// An operation name was unknown
    Signature(Operation, E),
}

pub fn try_interpret<O, A: Signature<O>>(
    hexpr: &Hexpr,
) -> Result<OpenHypergraph<Option<O>, A>, Error<<A as TryFrom<&Operation>>::Error>> {
    let mut state = OpenHypergraph::empty();
    let mut env = HashMap::new();
    let (sources, targets) = try_interpret_stack(&mut state, &mut env, hexpr)?;
    state.sources = sources;
    state.targets = targets;
    Ok(state)
}

fn try_interpret_stack<'a, O, A: Signature<O>>(
    state: &mut OpenHypergraph<Option<O>, A>,
    env: &mut HashMap<Variable, NodeId>,
    hexpr: &'a Hexpr,
) -> Result<Interface, Error<<A as TryFrom<&'a Operation>>::Error>> {
    match hexpr {
        Hexpr::Composition(hexprs) => {
            let mut iter = hexprs.into_iter();
            let mut hexpr = match iter.next() {
                Some(hexpr) => hexpr,
                None => return Ok((vec![], vec![])),
            };

            let (sources, mut current_targets) = try_interpret_stack(state, env, hexpr)?;
            for next_hexpr in iter {
                let (next_sources, next_targets) = try_interpret_stack(state, env, next_hexpr)?;

                // Check if targets of current match sources of next
                if current_targets.len() != next_sources.len() {
                    return Err(Error::Composition(hexpr.clone(), next_hexpr.clone()));
                }

                // Unify targets of current with sources of next
                for (&target, &source) in current_targets.iter().zip(&next_sources) {
                    state.unify(target, source);
                }

                // Update current interface
                current_targets = next_targets;
                hexpr = next_hexpr;
            }

            Ok((sources, current_targets))
        }
        Hexpr::Tensor(hexprs) => {
            let mut all_sources = vec![];
            let mut all_targets = vec![];

            for hexpr in hexprs {
                let (sources, targets) = try_interpret_stack(state, env, hexpr)?;
                all_sources.extend(sources);
                all_targets.extend(targets);
            }

            Ok((all_sources, all_targets))
        }
        Hexpr::Operation(op) => {
            let arr: A = op.try_into().map_err(|e| Error::Signature(op.clone(), e))?;
            let s = arr.source();
            let t = arr.target();
            let (_, interface) = state.new_operation(arr, s, t);
            Ok(interface)
        }
        Hexpr::Frobenius { sources, targets } => {
            let source_nodes = process_frobenius_variables(sources, env, state);
            let target_nodes = process_frobenius_variables(targets, env, state);
            Ok((source_nodes, target_nodes))
        }
    }
}

fn process_frobenius_variables<O, A>(
    variables: &[Variable],
    env: &mut HashMap<Variable, NodeId>,
    state: &mut OpenHypergraph<Option<O>, A>,
) -> Vec<NodeId> {
    variables
        .iter()
        .map(|var| {
            if let Some(&existing_node) = env.get(var) {
                // Variable already exists - reuse its node (creates unification)
                existing_node
            } else {
                // First occurrence of this variable - create new node with None type
                let new_node = state.new_node(None);
                env.insert(var.clone(), new_node);
                new_node
            }
        })
        .collect()
}

impl<E: Display> Display for Error<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Composition(a, b) => write!(f, "failed to compose {:?} ; {:?}", a, b),
            Error::Signature(op, err) => write!(f, "couldn't parse op {}: {}", op, err),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for Error<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Signature(_, e) => Some(e),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Operation;

    #[derive(Debug, Clone)]
    enum ArithOp {
        Add,
        Neg,
    }

    #[derive(Debug, thiserror::Error)]
    #[error("{0}")]
    struct Error(String);

    impl TryFrom<&Operation> for ArithOp {
        type Error = Error;

        fn try_from(op: &Operation) -> Result<Self, Self::Error> {
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
        let result: OpenHypergraph<Option<()>, ArithOp> = try_interpret(&hexpr)?;

        assert_eq!(result.sources.len(), 2);
        assert_eq!(result.targets.len(), 1);
        assert_eq!(result.hypergraph.edges.len(), 1);

        Ok(())
    }

    #[test]
    fn test_composition() -> anyhow::Result<()> {
        let hexpr = "(add neg)".parse()?;
        let result: OpenHypergraph<Option<()>, ArithOp> = try_interpret(&hexpr)?;

        assert_eq!(result.sources.len(), 2);
        assert_eq!(result.targets.len(), 1);
        assert_eq!(result.hypergraph.edges.len(), 2);

        Ok(())
    }

    #[test]
    fn test_frobenius() -> anyhow::Result<()> {
        let hexpr = "[x y . x]".parse()?;
        let result: OpenHypergraph<Option<()>, ArithOp> = try_interpret(&hexpr)?;
        assert_eq!(result.sources.len(), 2);
        assert_eq!(result.targets.len(), 1);
        assert_eq!(result.hypergraph.edges.len(), 0);

        Ok(())
    }

    #[test]
    fn test_all() -> anyhow::Result<()> {
        let hexpr = "({[x y . x] neg} add neg [y])".parse()?;
        let result: OpenHypergraph<Option<()>, ArithOp> = try_interpret(&hexpr)?;
        let mut result = result.map_nodes(|_| ()); // erase None nodes

        // NOTE: this will panic if nodes cannot be quotiented!
        result.quotient();

        assert_eq!(result.sources.len(), 3);
        assert_eq!(result.targets.len(), 1);
        assert_eq!(result.hypergraph.edges.len(), 3);
        assert_eq!(result.hypergraph.nodes.len(), 5);

        Ok(())
    }
}
