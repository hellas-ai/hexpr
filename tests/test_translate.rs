use hexpr::{parse, OperationType};

use std::collections::HashMap;

#[test]
fn test_parse_with_signature() {
    // Create a signature with a simple addition operation
    let mut signature = HashMap::new();
    signature.insert(
        "+".to_string(),
        OperationType::new(
            vec!["ℝ".to_string(), "ℝ".to_string()],
            vec!["ℝ".to_string()],
        ),
    );

    // Test with a frobenius structure connected to an operation
    let result = parse("([x . x x] +)", signature);
    assert!(result.is_ok());

    let hypergraph = result.unwrap();
    // Should have created nodes and edges for both the frobenius and the operation
    assert!(!hypergraph.hypergraph.nodes.is_empty());
    assert!(!hypergraph.hypergraph.edges.is_empty());

    // All nodes should now be String labels (no HObject enum)
    // They should all be "ℝ" since that's what the + operation uses
    let all_real = hypergraph.hypergraph.nodes.iter().all(|n| n == "ℝ");
    assert!(all_real);

    // All edge labels should now be String labels (no HOperation enum)
    // They should all be "+" since that's the operation name
    let all_plus = hypergraph.hypergraph.edges.iter().all(|e| e == "+");
    assert!(all_plus);
}

#[test]
fn test_parse_empty_signature() {
    use crate::parse;
    use std::collections::HashMap;

    // Test with empty signature (only frobenius structures)
    // This should fail because Unknown nodes remain after inference
    let signature = HashMap::new();
    let result = parse("[x x . x]", signature);
    assert!(result.is_err());

    // Should get an error about Unknown object types remaining
    if let Err(e) = result {
        assert!(e.to_string().contains("Unknown object type remains"));
    }
}

#[test]
fn test_parse_parse_error() {
    use crate::parse;
    use std::collections::HashMap;

    let signature = HashMap::new();
    let result = parse("invalid[syntax", signature);
    assert!(result.is_err());
}
