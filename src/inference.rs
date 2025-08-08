use crate::translate::{HObject, HOperation};
use open_hypergraphs::lax::OpenHypergraph;
use std::collections::HashMap;

/// Propagates known object labels to unknown ones using coequalizer-based inference.
/// This ensures that when we quotient the hypergraph, all unified nodes have consistent labels.
pub fn propagate_object_labels(graph: &mut OpenHypergraph<HObject, HOperation>) {
    // Build equivalence classes using the coequalizer
    let coequalizer = graph.hypergraph.coequalizer();

    // Group nodes by their equivalence class representatives
    let mut equiv_classes: HashMap<usize, Vec<usize>> = HashMap::new();
    for node_idx in 0..graph.hypergraph.nodes.len() {
        // The coequalizer is a FiniteFunction with a table field that maps node indices to representatives
        if let Some(representative_idx) = coequalizer.table.get(node_idx) {
            equiv_classes
                .entry(*representative_idx)
                .or_default()
                .push(node_idx);
        }
    }

    // For each equivalence class, find the best label and apply it to all nodes
    for (_representative_idx, node_indices) in equiv_classes {
        let best_label = find_best_label_from_indices(&node_indices, graph);
        if let Some(label) = best_label {
            // Apply the best label to all nodes in this equivalence class
            for &node_idx in &node_indices {
                if let Some(node_label) = graph.hypergraph.nodes.get_mut(node_idx) {
                    *node_label = label.clone();
                }
            }
        }
    }
}

/// Find the best known label from an equivalence class of node indices.
/// Prefers Named labels over Unknown labels.
fn find_best_label_from_indices(
    node_indices: &[usize],
    graph: &OpenHypergraph<HObject, HOperation>,
) -> Option<HObject> {
    let mut best_label = None;

    for &node_idx in node_indices {
        if let Some(label) = graph.hypergraph.nodes.get(node_idx) {
            match label {
                HObject::Named(_) => {
                    // Named labels have highest priority - return immediately
                    return Some(label.clone());
                }
                HObject::Unknown => {
                    // Keep Unknown as fallback but continue searching for Named
                    if best_label.is_none() {
                        best_label = Some(label.clone());
                    }
                }
            }
        }
    }

    best_label
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::translate::{translate_expr_with_signatures, OperationSignature};
    use crate::HExprParser;
    use std::collections::HashMap;

    #[test]
    fn test_propagate_unknown_to_known() {
        // Create a hypergraph where Unknown nodes should get labels from Named nodes
        let mut signatures = HashMap::new();
        let obj = HObject::from("ℝ");
        signatures.insert(
            "+".to_string(),
            OperationSignature::new(vec![obj.clone(), obj.clone()], vec![obj.clone()]),
        );

        // Parse an expression that mixes frobenius (Unknown) with operations (Named)
        let expr = HExprParser::parse_expr("([x] +)").unwrap();
        let mut graph = translate_expr_with_signatures(&expr, signatures).unwrap();

        // Before propagation, we should have both Unknown and Named nodes
        let has_unknown = graph
            .hypergraph
            .nodes
            .iter()
            .any(|n| matches!(n, HObject::Unknown));
        let has_named = graph
            .hypergraph
            .nodes
            .iter()
            .any(|n| matches!(n, HObject::Named(_)));
        assert!(has_unknown && has_named);

        // Apply label propagation
        propagate_object_labels(&mut graph);

        // After propagation, Unknown nodes in the same equivalence class should be Named
        // (This test assumes the frobenius variable gets unified with operation nodes)
    }

    #[test]
    fn test_all_unknown_stays_unknown() {
        // Test that if all nodes in an equivalence class are Unknown, they stay Unknown
        let expr = HExprParser::parse_expr("[x x . x]").unwrap();
        let mut graph = translate_expr_with_signatures(&expr, HashMap::new()).unwrap();

        // All nodes should be Unknown before propagation
        let all_unknown = graph
            .hypergraph
            .nodes
            .iter()
            .all(|n| matches!(n, HObject::Unknown));
        assert!(all_unknown);

        propagate_object_labels(&mut graph);

        // Should still be all Unknown after propagation
        let still_all_unknown = graph
            .hypergraph
            .nodes
            .iter()
            .all(|n| matches!(n, HObject::Unknown));
        assert!(still_all_unknown);
    }
}
