use crate::ast::{Expr, Variable};
use crate::parser::HExprParser;

#[test]
fn test_basic_frobenius_join() {
    let result = HExprParser::parse_expr("[x x . x]").unwrap();
    assert_eq!(
        result,
        Expr::Frobenius {
            inputs: vec![
                Variable::Named("x".to_string()),
                Variable::Named("x".to_string())
            ],
            outputs: vec![Variable::Named("x".to_string())],
        }
    );
}

#[test]
fn test_basic_frobenius_split() {
    let result = HExprParser::parse_expr("[x . x x]").unwrap();
    assert_eq!(
        result,
        Expr::Frobenius {
            inputs: vec![Variable::Named("x".to_string())],
            outputs: vec![
                Variable::Named("x".to_string()),
                Variable::Named("x".to_string())
            ],
        }
    );
}

#[test]
fn test_identity_shorthand() {
    let result = HExprParser::parse_expr("[x y]").unwrap();
    assert_eq!(
        result,
        Expr::Frobenius {
            inputs: vec![
                Variable::Named("x".to_string()),
                Variable::Named("y".to_string())
            ],
            outputs: vec![
                Variable::Named("x".to_string()),
                Variable::Named("y".to_string())
            ],
        }
    );
}

#[test]
fn test_named_identity() {
    let result = HExprParser::parse_expr("[a]").unwrap();
    assert_eq!(
        result,
        Expr::Frobenius {
            inputs: vec![Variable::Named("a".to_string())],
            outputs: vec![Variable::Named("a".to_string())],
        }
    );
}

#[test]
fn test_identity_via_composition() {
    let result = HExprParser::parse_expr("([x.][.x])").unwrap();
    assert_eq!(
        result,
        Expr::Composition(vec![
            Expr::Frobenius {
                inputs: vec![Variable::Named("x".to_string())],
                outputs: vec![],
            },
            Expr::Frobenius {
                inputs: vec![],
                outputs: vec![Variable::Named("x".to_string())],
            },
        ])
    );
}

#[test]
fn test_subtraction_pointfree() {
    let result = HExprParser::parse_expr("({[a] -} +)").unwrap();
    assert_eq!(
        result,
        Expr::Composition(vec![
            Expr::Tensor(vec![
                Expr::Frobenius {
                    inputs: vec![Variable::Named("a".to_string())],
                    outputs: vec![Variable::Named("a".to_string())],
                },
                Expr::Operation("-".to_string()),
            ]),
            Expr::Operation("+".to_string()),
        ])
    );
}

#[test]
fn test_subtraction_pointed() {
    let result = HExprParser::parse_expr("([x y.] ([.y] - [z.]) [.x z] +)").unwrap();
    assert_eq!(
        result,
        Expr::Composition(vec![
            Expr::Frobenius {
                inputs: vec![
                    Variable::Named("x".to_string()),
                    Variable::Named("y".to_string())
                ],
                outputs: vec![],
            },
            Expr::Composition(vec![
                Expr::Frobenius {
                    inputs: vec![],
                    outputs: vec![Variable::Named("y".to_string())],
                },
                Expr::Operation("-".to_string()),
                Expr::Frobenius {
                    inputs: vec![Variable::Named("z".to_string())],
                    outputs: vec![],
                },
            ]),
            Expr::Frobenius {
                inputs: vec![],
                outputs: vec![
                    Variable::Named("x".to_string()),
                    Variable::Named("z".to_string())
                ],
            },
            Expr::Operation("+".to_string()),
        ])
    );
}

#[test]
fn test_explicit_swap_relation() {
    let result = HExprParser::parse_expr("[x y . y x]").unwrap();
    assert_eq!(
        result,
        Expr::Frobenius {
            inputs: vec![
                Variable::Named("x".to_string()),
                Variable::Named("y".to_string())
            ],
            outputs: vec![
                Variable::Named("y".to_string()),
                Variable::Named("x".to_string())
            ],
        }
    );
}

#[test]
fn test_empty_inputs_outputs() {
    let result = HExprParser::parse_expr("[.]").unwrap();
    assert_eq!(
        result,
        Expr::Frobenius {
            inputs: vec![],
            outputs: vec![],
        }
    );
}

#[test]
fn test_discard_variable() {
    let result = HExprParser::parse_expr("[x .]").unwrap();
    assert_eq!(
        result,
        Expr::Frobenius {
            inputs: vec![Variable::Named("x".to_string())],
            outputs: vec![],
        }
    );
}

#[test]
fn test_create_variable() {
    let result = HExprParser::parse_expr("[. x]").unwrap();
    assert_eq!(
        result,
        Expr::Frobenius {
            inputs: vec![],
            outputs: vec![Variable::Named("x".to_string())],
        }
    );
}

#[test]
fn test_dispell_summon_named() {
    let result = HExprParser::parse_expr("[a b . c]").unwrap();
    assert_eq!(
        result,
        Expr::Frobenius {
            inputs: vec![
                Variable::Named("a".to_string()),
                Variable::Named("b".to_string())
            ],
            outputs: vec![Variable::Named("c".to_string())],
        }
    );
}

#[test]
fn test_complex_composition() {
    let result = HExprParser::parse_expr("(add sub mul)").unwrap();
    assert_eq!(
        result,
        Expr::Composition(vec![
            Expr::Operation("add".to_string()),
            Expr::Operation("sub".to_string()),
            Expr::Operation("mul".to_string()),
        ])
    );
}

#[test]
fn test_complex_tensor() {
    let result = HExprParser::parse_expr("{add sub mul}").unwrap();
    assert_eq!(
        result,
        Expr::Tensor(vec![
            Expr::Operation("add".to_string()),
            Expr::Operation("sub".to_string()),
            Expr::Operation("mul".to_string()),
        ])
    );
}

#[test]
fn test_nested_expressions() {
    let result = HExprParser::parse_expr("({add} (sub mul))").unwrap();
    assert_eq!(
        result,
        Expr::Composition(vec![
            Expr::Tensor(vec![Expr::Operation("add".to_string())]),
            Expr::Composition(vec![
                Expr::Operation("sub".to_string()),
                Expr::Operation("mul".to_string()),
            ]),
        ])
    );
}

#[test]
fn test_names_with_dashes_and_underscores() {
    let result = HExprParser::parse_expr("my-operation_2").unwrap();
    assert_eq!(result, Expr::Operation("my-operation_2".to_string()));
}

#[test]
fn test_whitespace_handling() {
    let result = HExprParser::parse_expr("  ( add   sub  )  ").unwrap();
    assert_eq!(
        result,
        Expr::Composition(vec![
            Expr::Operation("add".to_string()),
            Expr::Operation("sub".to_string()),
        ])
    );
}

#[test]
fn test_invalid_syntax() {
    assert!(HExprParser::parse_expr("(").is_err());
    assert!(HExprParser::parse_expr("[").is_err());
    assert!(HExprParser::parse_expr("{").is_err());
    assert!(HExprParser::parse_expr("").is_err());
}

#[test]
fn test_parse_with_signature() {
    use crate::{parse, OperationType};
    use std::collections::HashMap;

    // Create a signature with a simple addition operation
    let mut signature = HashMap::new();
    signature.insert(
        "+".to_string(),
        OperationType::new(vec!["ℝ".to_string(), "ℝ".to_string()], vec!["ℝ".to_string()]),
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
