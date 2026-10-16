//! Convert Hexprs to open hypergraphs via a "partial signature".
use std::collections::HashMap;

use open_hypergraphs::lax::{Interface, NodeId, OpenHypergraph};

use crate::ast::{Hexpr, LVar, Operation, Variable};
use thiserror::Error;

#[derive(Debug)]
pub struct OpenHypergraphWithNames<O, A> {
    pub graph: OpenHypergraph<O, A>,
    pub names: HashMap<NodeId, Vec<Variable>>,
}

/// A `Signature` is:
///  - A way to parse operations
///   - A profile: a source/target type for each arrow
pub trait Signature {
    type Arr;
    type Obj;
    type Error;

    fn try_parse_op(&self, op: &Operation) -> Result<Self::Arr, Self::Error>;
    fn try_parse_object(&self, object: &Hexpr) -> Result<Self::Obj, Self::Error>;
    #[allow(clippy::type_complexity)]
    fn profile(&self, op: &Self::Arr) -> (Vec<Option<Self::Obj>>, Vec<Option<Self::Obj>>);
}

#[derive(Debug, Error)]
pub enum Error<E> {
    #[error("Failed to compose {0} ; {1}")]
    Composition(Hexpr, Hexpr),
    #[error("Couldn't parse op {0}: {1}")]
    Signature(Operation, E),
    #[error("Couldn't parse object {0}: {1}")]
    Object(Hexpr, E),
}

#[allow(clippy::type_complexity)]
pub fn try_interpret<S: Signature>(
    signature: &S,
    hexpr: &Hexpr,
) -> Result<OpenHypergraph<Option<S::Obj>, S::Arr>, Error<S::Error>> {
    Ok(try_interpret_with_names(signature, hexpr)?.graph)
}

#[allow(clippy::type_complexity)]
pub fn try_interpret_with_names<S: Signature>(
    signature: &S,
    hexpr: &Hexpr,
) -> Result<OpenHypergraphWithNames<Option<S::Obj>, S::Arr>, Error<S::Error>> {
    let mut state = OpenHypergraph::empty();
    let mut env = HashMap::new();
    let (sources, targets) = try_interpret_stack(signature, &mut state, &mut env, hexpr)?;
    state.sources = sources;
    state.targets = targets;
    Ok(OpenHypergraphWithNames {
        graph: state,
        names: names_by_node(env),
    })
}

fn names_by_node(env: HashMap<Variable, NodeId>) -> HashMap<NodeId, Vec<Variable>> {
    env.into_iter()
        .fold(HashMap::new(), |mut names, (variable, node)| {
            names.entry(node).or_default().push(variable);
            names
        })
}

fn try_interpret_stack<S: Signature>(
    signature: &S,
    state: &mut OpenHypergraph<Option<S::Obj>, S::Arr>,
    env: &mut HashMap<Variable, NodeId>,
    hexpr: &Hexpr,
) -> Result<Interface, Error<S::Error>> {
    match hexpr {
        Hexpr::Composition(hexprs) => {
            let mut iter = hexprs.iter();
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
        Hexpr::Hole => {
            let node = state.new_node(None);
            Ok((vec![node], vec![node]))
        }
        Hexpr::Wire(label) => {
            let label = signature
                .try_parse_object(label)
                .map_err(|e| Error::Object((**label).clone(), e))?;
            let node = state.new_node(Some(label));
            Ok((vec![node], vec![node]))
        }
        Hexpr::Frobenius { sources, targets } => {
            let source_nodes = process_frobenius_variables(signature, sources, env, state)?;
            let target_nodes = process_frobenius_variables(signature, targets, env, state)?;
            Ok((source_nodes, target_nodes))
        }
    }
}

fn process_frobenius_variables<S: Signature>(
    signature: &S,
    variables: &[LVar],
    env: &mut HashMap<Variable, NodeId>,
    state: &mut OpenHypergraph<Option<S::Obj>, S::Arr>,
) -> Result<Vec<NodeId>, Error<S::Error>> {
    variables
        .iter()
        .map(|var| {
            let label = var
                .label
                .as_ref()
                .map(|label| {
                    signature
                        .try_parse_object(label)
                        .map_err(|e| Error::Object(label.clone(), e))
                })
                .transpose()?;

            let node = if let Some(&existing_node) = env.get(&var.name) {
                existing_node
            } else {
                let new_node = state.new_node(label);
                env.insert(var.name.clone(), new_node);
                return Ok(new_node);
            };

            if let Some(label) = label {
                if state.hypergraph.nodes[node.0].is_none() {
                    state.hypergraph.nodes[node.0] = Some(label);
                } else {
                    let annotation_node = state.new_node(Some(label));
                    state.unify(node, annotation_node);
                }
            }

            Ok(node)
        })
        .collect()
}
