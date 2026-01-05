//! Convert Hexprs to open hypergraphs via a "partial signature".
use std::collections::HashMap;
use std::fmt::Display;

use open_hypergraphs::lax::{Interface, NodeId, OpenHypergraph};

use crate::ast::{Hexpr, Operation, Variable};

/// A `Signature` is:
///  - A way to parse operations
///   - A profile: a source/target type for each arrow
pub trait Signature {
    type Arr;
    type Obj;
    type Error;

    fn try_parse_op(&self, op: &Operation) -> Result<Self::Arr, Self::Error>;
    fn profile(&self, op: &Self::Arr) -> (Vec<Option<Self::Obj>>, Vec<Option<Self::Obj>>);
}

#[derive(Debug)]
pub enum Error<E> {
    /// Composition of two hexprs had incorrect arity according to the signature
    Composition(Hexpr, Hexpr),
    /// An operation name was unknown
    Signature(Operation, E),
}

pub fn try_interpret<S: Signature>(
    signature: &S,
    hexpr: &Hexpr,
) -> Result<OpenHypergraph<Option<S::Obj>, S::Arr>, Error<S::Error>> {
    let mut state = OpenHypergraph::empty();
    let mut env = HashMap::new();
    let (sources, targets) = try_interpret_stack(signature, &mut state, &mut env, hexpr)?;
    state.sources = sources;
    state.targets = targets;
    Ok(state)
}

fn try_interpret_stack<S: Signature>(
    signature: &S,
    state: &mut OpenHypergraph<Option<S::Obj>, S::Arr>,
    env: &mut HashMap<Variable, NodeId>,
    hexpr: &Hexpr,
) -> Result<Interface, Error<S::Error>> {
    match hexpr {
        Hexpr::Composition(hexprs) => {
            let mut iter = hexprs.into_iter();
            let mut hexpr = match iter.next() {
                Some(hexpr) => hexpr,
                None => return Ok((vec![], vec![])),
            };

            let (sources, mut current_targets) = try_interpret_stack(signature, state, env, hexpr)?;
            for next_hexpr in iter {
                let (next_sources, next_targets) =
                    try_interpret_stack(signature, state, env, next_hexpr)?;

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
                let (sources, targets) = try_interpret_stack(signature, state, env, hexpr)?;
                all_sources.extend(sources);
                all_targets.extend(targets);
            }

            Ok((all_sources, all_targets))
        }
        Hexpr::Operation(op) => {
            let arr: S::Arr = signature
                .try_parse_op(op)
                .map_err(|e| Error::Signature(op.clone(), e))?;
            let (s, t) = signature.profile(&arr);
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
            Error::Composition(a, b) => write!(f, "failed to compose {} ; {}", a, b),
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
