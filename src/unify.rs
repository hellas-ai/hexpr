use thiserror::Error;

use open_hypergraphs::category::Arrow;
use open_hypergraphs::lax::{NodeId, OpenHypergraph};

#[derive(Debug, Error)]
pub enum UnifyError {
    #[error("Not all node labels known")]
    NotAllLabelsKnown,
    #[error("Could not unify {0:?}")]
    Mismatch(NodeId),
    #[error("Quotient failed")]
    Quotient,
}

/// Unify the variables of an unquotiented open hypergraph with nodes labels `Option<O>`.
pub fn unify<O: Clone + PartialEq, A: Clone>(
    f: OpenHypergraph<Option<O>, A>,
) -> Result<OpenHypergraph<O, A>, UnifyError> {
    // coequalizer of the quotient map
    let coequalizer = f.hypergraph.coequalizer();

    // Compute node values for each equivalence class by merging all values in each class.
    // Done in a single pass over node labels, storing results for each class in class_labels
    let mut class_labels: Vec<Option<O>> = vec![None; coequalizer.target()];
    for (i, node) in f.hypergraph.nodes.iter().enumerate() {
        let class = coequalizer.table[i];
        let label = &mut class_labels[class];
        match (node, label.as_mut()) {
            // update u with value of x
            (Some(x), None) => *label = Some(x.clone()),
            // no new x, do nothing
            (None, _) => (),
            // If x != u, unification error
            (Some(x), Some(u)) => {
                if x != u {
                    return Err(UnifyError::Mismatch(NodeId(i)));
                }
            }
        };
    }

    let class_labels: Vec<O> = class_labels
        .into_iter()
        .collect::<Option<_>>()
        .ok_or(UnifyError::NotAllLabelsKnown)?;
    let nodes = (0..f.hypergraph.nodes.len())
        .map(|i| class_labels[coequalizer.table[i]].clone())
        .collect();
    let mut f = f
        .with_nodes(|_| nodes)
        .ok_or(UnifyError::NotAllLabelsKnown)?;
    f.quotient().map_err(|_| UnifyError::Quotient)?;
    Ok(f)
}
