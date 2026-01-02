use crate::ast::{Hexpr, Operation, Variable};
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct HExprParser;

impl HExprParser {
    pub fn parse_expr(input: &str) -> Result<Hexpr, Box<pest::error::Error<Rule>>> {
        let pairs = HExprParser::parse(Rule::program, input)?;
        let program = pairs.into_iter().next().unwrap();
        let expr_pair = program.into_inner().next().unwrap();
        Ok(build_expr(expr_pair))
    }
}

fn build_expr(pair: pest::iterators::Pair<Rule>) -> Hexpr {
    match pair.as_rule() {
        Rule::hexpr => {
            let inner = pair.into_inner().next().unwrap();
            build_expr(inner)
        }
        Rule::composition => {
            let exprs = pair.into_inner().map(build_expr).collect();
            Hexpr::Composition(exprs)
        }
        Rule::tensor => {
            let exprs = pair.into_inner().map(build_expr).collect();
            Hexpr::Tensor(exprs)
        }
        Rule::frobenius => {
            let inner = pair.into_inner().next().unwrap();
            match inner.as_rule() {
                Rule::frobenius_full => {
                    let variables = inner.into_inner();
                    let mut inputs = Vec::new();
                    let mut outputs = Vec::new();
                    let mut parsing_outputs = false;

                    for var_pair in variables {
                        match var_pair.as_rule() {
                            Rule::dot => {
                                parsing_outputs = true;
                            }
                            Rule::variable => {
                                let variable = build_variable(var_pair);
                                if parsing_outputs {
                                    outputs.push(variable);
                                } else {
                                    inputs.push(variable);
                                }
                            }
                            _ => {}
                        }
                    }

                    Hexpr::Frobenius { inputs, outputs }
                }
                Rule::frobenius_identity => {
                    let variables: Vec<Variable> = inner.into_inner().map(build_variable).collect();

                    Hexpr::Frobenius {
                        inputs: variables.clone(),
                        outputs: variables,
                    }
                }
                Rule::frobenius_empty => Hexpr::Frobenius {
                    inputs: Vec::new(),
                    outputs: Vec::new(),
                },
                _ => unreachable!(),
            }
        }
        Rule::operation => {
            let name = pair.as_str();
            Hexpr::Operation(Operation(name.to_string()))
        }
        _ => unreachable!(),
    }
}

fn build_variable(pair: pest::iterators::Pair<Rule>) -> Variable {
    match pair.as_rule() {
        Rule::variable => Variable(pair.as_str().to_string()),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operation() -> anyhow::Result<()> {
        let result = HExprParser::parse_expr("add")?;
        assert_eq!(result, Hexpr::Operation(Operation("add".to_string())));
        Ok(())
    }

    #[test]
    fn test_frobenius_identity() {
        let result = HExprParser::parse_expr("[x]").unwrap();
        assert_eq!(
            result,
            Hexpr::Frobenius {
                inputs: vec![Variable("x".to_string())],
                outputs: vec![Variable("x".to_string())],
            }
        );
    }

    #[test]
    fn test_frobenius_full() {
        let result = HExprParser::parse_expr("[x x . x]").unwrap();
        assert_eq!(
            result,
            Hexpr::Frobenius {
                inputs: vec![Variable("x".to_string()), Variable("x".to_string())],
                outputs: vec![Variable("x".to_string())],
            }
        );
    }

    #[test]
    fn test_composition() {
        let result = HExprParser::parse_expr("(add sub)").unwrap();
        assert_eq!(
            result,
            Hexpr::Composition(vec![
                Hexpr::Operation(Operation("add".to_string())),
                Hexpr::Operation(Operation("sub".to_string())),
            ])
        );
    }

    #[test]
    fn test_tensor() {
        let result = HExprParser::parse_expr("{add sub}").unwrap();
        assert_eq!(
            result,
            Hexpr::Tensor(vec![
                Hexpr::Operation(Operation("add".to_string())),
                Hexpr::Operation(Operation("sub".to_string())),
            ])
        );
    }

    #[test]
    fn test_frobenius_empty() {
        let result = HExprParser::parse_expr("[]").unwrap();
        assert_eq!(
            result,
            Hexpr::Frobenius {
                inputs: vec![],
                outputs: vec![],
            }
        );
    }
}
