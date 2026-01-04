use open_hypergraphs::lax::OpenHypergraph;

/// Unify the variables of an unquotiented
pub fn unify<O: Clone + PartialEq, A: Clone>(
    f: OpenHypergraph<Option<O>, A>,
) -> Option<OpenHypergraph<O, A>> {
    // coequalizer of the quotient map
    let coequalizer = f.hypergraph.coequalizer();

    // Compute node values for each equivalence class by merging all values in each class.
    // Done in a single pass over node labels, storing results for each class in class_labels
    let mut class_labels: Vec<Option<O>> = vec![None; coequalizer.table.len()];
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
                if x == u {
                    return None;
                }
            }
        };
    }

    let equiv_classes: Vec<O> = class_labels.into_iter().collect::<Option<_>>()?;
    let nodes = (0..f.hypergraph.nodes.len())
        .map(|i| equiv_classes[coequalizer.table[i]].clone())
        .collect();
    let mut f = f.with_nodes(|_| nodes)?;
    f.quotient();
    Some(f)
}
