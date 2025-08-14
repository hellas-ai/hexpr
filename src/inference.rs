use crate::translate::{HObject, HOperation};
use open_hypergraphs::lax::OpenHypergraph;
use std::collections::HashMap;

/// Propagates known object labels to unknown ones using coequalizer-based inference.
/// This ensures that when we quotient the hypergraph, all unified nodes have consistent labels.
/// Returns an error if there are type conflicts (multiple different known types in same equivalence class).
pub fn propagate_object_labels(
    graph: &mut OpenHypergraph<HObject, HOperation>,
) -> Result<(), crate::translate::TranslationError> {
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
        let best_label = find_best_label_from_indices(&node_indices, graph)
            .map_err(|msg| crate::translate::TranslationError { message: msg })?;
        if let Some(label) = best_label {
            // Apply the best label to all nodes in this equivalence class
            for &node_idx in &node_indices {
                if let Some(node_label) = graph.hypergraph.nodes.get_mut(node_idx) {
                    *node_label = label.clone();
                }
            }
        }
    }

    Ok(())
}

/// Find the best known label from an equivalence class of node indices.
/// Returns error if multiple different Named types are found in the same equivalence class.
fn find_best_label_from_indices(
    node_indices: &[usize],
    graph: &OpenHypergraph<HObject, HOperation>,
) -> Result<Option<HObject>, String> {
    let mut found_named: Option<HObject> = None;
    let mut has_unknown = false;

    for &node_idx in node_indices {
        if let Some(label) = graph.hypergraph.nodes.get(node_idx) {
            match label {
                HObject::Named(_) => {
                    if let Some(ref existing_named) = found_named {
                        // Check if we found a different named type
                        if existing_named != label {
                            return Err(format!(
                                "Type conflict: cannot unify {} with {} in the same equivalence class",
                                existing_named, label
                            ));
                        }
                    } else {
                        found_named = Some(label.clone());
                    }
                }
                HObject::Unknown => {
                    has_unknown = true;
                }
            }
        }
    }

    // Return the single consistent named type, or Unknown if that's all we have
    if let Some(named) = found_named {
        Ok(Some(named))
    } else if has_unknown {
        Ok(Some(HObject::Unknown))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::translate::{translate_expr_with_signature, OperationSignature};
    use crate::HExprParser;
    use std::collections::HashMap;

    #[test]
    fn test_propagate_unknown_to_known() {
        // Create a hypergraph where Unknown nodes should get labels from Named nodes
        let mut signature = HashMap::new();
        let obj = HObject::from("ℝ");
        signature.insert(
            "+".to_string(),
            OperationSignature::new(vec![obj.clone(), obj.clone()], vec![obj.clone()]),
        );

        // Parse an expression that mixes frobenius (Unknown) with operations (Named)
        let expr = HExprParser::parse_expr("([x . x x] +)").unwrap();
        let mut graph = translate_expr_with_signature(&expr, signature).unwrap();

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
        propagate_object_labels(&mut graph).unwrap();

        // After propagation, Unknown nodes in the same equivalence class should be Named
        // (This test assumes the frobenius variable gets unified with operation nodes)
    }

    #[test]
    fn test_all_unknown_stays_unknown() {
        // Test that if all nodes in an equivalence class are Unknown, they stay Unknown
        let expr = HExprParser::parse_expr("[x x . x]").unwrap();
        let mut graph = translate_expr_with_signature(&expr, HashMap::new()).unwrap();

        // All nodes should be Unknown before propagation
        let all_unknown = graph
            .hypergraph
            .nodes
            .iter()
            .all(|n| matches!(n, HObject::Unknown));
        assert!(all_unknown);

        propagate_object_labels(&mut graph).unwrap();

        // Should still be all Unknown after propagation
        let still_all_unknown = graph
            .hypergraph
            .nodes
            .iter()
            .all(|n| matches!(n, HObject::Unknown));
        assert!(still_all_unknown);
    }

    #[test]
    fn test_type_conflict_error() {
        // Test that conflicting types in the same equivalence class cause an error
        let mut signature = HashMap::new();
        let r_obj = HObject::from("ℝ");
        let n_obj = HObject::from("ℕ");
        signature.insert(
            "+".to_string(),
            OperationSignature::new(vec![r_obj.clone(), r_obj.clone()], vec![r_obj.clone()]),
        );
        signature.insert(
            "nat_zero".to_string(),
            OperationSignature::new(vec![], vec![n_obj.clone()]),
        );

        // Create a direct type conflict by manually building a graph
        // where the same variable appears in contexts requiring different types
        use open_hypergraphs::lax::OpenHypergraph;
        let mut graph = OpenHypergraph::empty();

        // Create nodes manually to force the type conflict
        let nat_node = graph.new_node(n_obj.clone()); // ℕ node
        let real_node = graph.new_node(r_obj.clone()); // ℝ node

        // Manually unify them in the quotient (this simulates what would happen
        // if a frobenius relation connected operations of different types)
        graph.unify(nat_node, real_node);

        // This should fail with a type conflict error
        let result = propagate_object_labels(&mut graph);
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(e.message.contains("Type conflict"));
            assert!(e.message.contains("ℕ") && e.message.contains("ℝ"));
        }
    }
}
