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
