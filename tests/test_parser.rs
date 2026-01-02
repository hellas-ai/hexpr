use hexpr::ast::Hexpr;
use hexpr::parser::HExprParser;

#[test]
fn test_basic_frobenius_join() -> anyhow::Result<()> {
    let result = HExprParser::parse_expr("[x x . x]").unwrap();
    let expected = Hexpr::Frobenius {
        inputs: vec!["x".parse()?, "x".parse()?],
        outputs: vec!["x".parse()?],
    };

    assert_eq!(result, expected);
    Ok(())
}

#[test]
fn test_basic_frobenius_split() -> anyhow::Result<()> {
    let result = HExprParser::parse_expr("[x . x x]").unwrap();
    assert_eq!(
        result,
        Hexpr::Frobenius {
            inputs: vec!["x".parse()?],
            outputs: vec!["x".parse()?, "x".parse()?],
        }
    );
    Ok(())
}

#[test]
fn test_identity_shorthand() -> anyhow::Result<()> {
    let result = HExprParser::parse_expr("[x y]").unwrap();
    assert_eq!(
        result,
        Hexpr::Frobenius {
            inputs: vec!["x".parse()?, "y".parse()?],
            outputs: vec!["x".parse()?, "y".parse()?],
        }
    );
    Ok(())
}

#[test]
fn test_named_identity() -> anyhow::Result<()> {
    let result = HExprParser::parse_expr("[a]").unwrap();
    assert_eq!(
        result,
        Hexpr::Frobenius {
            inputs: vec!["a".parse()?],
            outputs: vec!["a".parse()?],
        }
    );
    Ok(())
}

#[test]
fn test_identity_via_composition() -> anyhow::Result<()> {
    let result = HExprParser::parse_expr("([x.][.x])").unwrap();
    assert_eq!(
        result,
        Hexpr::Composition(vec![
            Hexpr::Frobenius {
                inputs: vec!["x".parse()?],
                outputs: vec![],
            },
            Hexpr::Frobenius {
                inputs: vec![],
                outputs: vec!["x".parse()?],
            },
        ])
    );
    Ok(())
}

#[test]
fn test_subtraction_pointfree() -> anyhow::Result<()> {
    let result = HExprParser::parse_expr("({[a] -} +)").unwrap();
    assert_eq!(
        result,
        Hexpr::Composition(vec![
            Hexpr::Tensor(vec![
                Hexpr::Frobenius {
                    inputs: vec!["a".parse()?],
                    outputs: vec!["a".parse()?],
                },
                Hexpr::Operation("-".parse()?),
            ]),
            Hexpr::Operation("+".parse()?),
        ])
    );
    Ok(())
}

#[test]
fn test_subtraction_pointed() -> anyhow::Result<()> {
    let result = HExprParser::parse_expr("([x y.] ([.y] - [z.]) [.x z] +)").unwrap();
    assert_eq!(
        result,
        Hexpr::Composition(vec![
            Hexpr::Frobenius {
                inputs: vec!["x".parse()?, "y".parse()?],
                outputs: vec![],
            },
            Hexpr::Composition(vec![
                Hexpr::Frobenius {
                    inputs: vec![],
                    outputs: vec!["y".parse()?],
                },
                Hexpr::Operation("-".parse()?),
                Hexpr::Frobenius {
                    inputs: vec!["z".parse()?],
                    outputs: vec![],
                },
            ]),
            Hexpr::Frobenius {
                inputs: vec![],
                outputs: vec!["x".parse()?, "z".parse()?],
            },
            Hexpr::Operation("+".parse()?),
        ])
    );
    Ok(())
}

#[test]
fn test_explicit_swap_relation() -> anyhow::Result<()> {
    let result = HExprParser::parse_expr("[x y . y x]").unwrap();
    assert_eq!(
        result,
        Hexpr::Frobenius {
            inputs: vec!["x".parse()?, "y".parse()?],
            outputs: vec!["y".parse()?, "x".parse()?],
        }
    );
    Ok(())
}

#[test]
fn test_empty_inputs_outputs() {
    let result = HExprParser::parse_expr("[.]").unwrap();
    assert_eq!(
        result,
        Hexpr::Frobenius {
            inputs: vec![],
            outputs: vec![],
        }
    );
}

#[test]
fn test_discard_variable() -> anyhow::Result<()> {
    let result = HExprParser::parse_expr("[x .]").unwrap();
    assert_eq!(
        result,
        Hexpr::Frobenius {
            inputs: vec!["x".parse()?],
            outputs: vec![],
        }
    );
    Ok(())
}

#[test]
fn test_create_variable() -> anyhow::Result<()> {
    let result = HExprParser::parse_expr("[. x]").unwrap();
    assert_eq!(
        result,
        Hexpr::Frobenius {
            inputs: vec![],
            outputs: vec!["x".parse()?],
        }
    );
    Ok(())
}

#[test]
fn test_dispell_summon_named() -> anyhow::Result<()> {
    let result = HExprParser::parse_expr("[a b . c]").unwrap();
    assert_eq!(
        result,
        Hexpr::Frobenius {
            inputs: vec!["a".parse()?, "b".parse()?],
            outputs: vec!["c".parse()?],
        }
    );
    Ok(())
}

#[test]
fn test_complex_composition() -> anyhow::Result<()> {
    let result = HExprParser::parse_expr("(add sub mul)").unwrap();
    assert_eq!(
        result,
        Hexpr::Composition(vec![
            Hexpr::Operation("add".parse()?),
            Hexpr::Operation("sub".parse()?),
            Hexpr::Operation("mul".parse()?),
        ])
    );
    Ok(())
}

#[test]
fn test_complex_tensor() -> anyhow::Result<()> {
    let result = HExprParser::parse_expr("{add sub mul}").unwrap();
    assert_eq!(
        result,
        Hexpr::Tensor(vec![
            Hexpr::Operation("add".parse()?),
            Hexpr::Operation("sub".parse()?),
            Hexpr::Operation("mul".parse()?),
        ])
    );
    Ok(())
}

#[test]
fn test_nested_expressions() -> anyhow::Result<()> {
    let result = HExprParser::parse_expr("({add} (sub mul))").unwrap();
    assert_eq!(
        result,
        Hexpr::Composition(vec![
            Hexpr::Tensor(vec![Hexpr::Operation("add".parse()?)]),
            Hexpr::Composition(vec![
                Hexpr::Operation("sub".parse()?),
                Hexpr::Operation("mul".parse()?),
            ]),
        ])
    );
    Ok(())
}

#[test]
fn test_names_with_dashes_and_underscores() -> anyhow::Result<()> {
    let result = HExprParser::parse_expr("my-operation_2").unwrap();
    assert_eq!(result, Hexpr::Operation("my-operation_2".parse()?));
    Ok(())
}

#[test]
fn test_whitespace_handling() -> anyhow::Result<()> {
    let result = HExprParser::parse_expr("  ( add   sub  )  ").unwrap();
    assert_eq!(
        result,
        Hexpr::Composition(vec![
            Hexpr::Operation("add".parse()?),
            Hexpr::Operation("sub".parse()?),
        ])
    );
    Ok(())
}

#[test]
fn test_invalid_syntax() {
    assert!(HExprParser::parse_expr("(").is_err());
    assert!(HExprParser::parse_expr("[").is_err());
    assert!(HExprParser::parse_expr("{").is_err());
    assert!(HExprParser::parse_expr("").is_err());
}
