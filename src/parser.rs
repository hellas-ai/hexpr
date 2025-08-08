use pest::Parser;
use pest_derive::Parser;
use crate::ast::{Expr, Variable};

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct HExprParser;

impl HExprParser {
    pub fn parse_expr(input: &str) -> Result<Expr, Box<pest::error::Error<Rule>>> {
        let pairs = HExprParser::parse(Rule::program, input)?;
        let program = pairs.into_iter().next().unwrap();
        let expr_pair = program.into_inner().next().unwrap();
        Ok(build_expr(expr_pair))
    }
}

fn build_expr(pair: pest::iterators::Pair<Rule>) -> Expr {
    match pair.as_rule() {
        Rule::expr => {
            let inner = pair.into_inner().next().unwrap();
            build_expr(inner)
        }
        Rule::composition => {
            let exprs = pair
                .into_inner()
                .map(build_expr)
                .collect();
            Expr::Composition(exprs)
        }
        Rule::tensor => {
            let exprs = pair
                .into_inner()
                .map(build_expr)
                .collect();
            Expr::Tensor(exprs)
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
                    
                    Expr::Frobenius { inputs, outputs }
                }
                Rule::frobenius_identity => {
                    let variables: Vec<Variable> = inner
                        .into_inner()
                        .map(build_variable)
                        .collect();
                    
                    Expr::Frobenius {
                        inputs: variables.clone(),
                        outputs: variables,
                    }
                }
                _ => unreachable!()
            }
        }
        Rule::operation => {
            let name = pair.into_inner().next().unwrap().as_str();
            Expr::Operation(name.to_string())
        }
        _ => unreachable!()
    }
}

fn build_variable(pair: pest::iterators::Pair<Rule>) -> Variable {
    match pair.as_rule() {
        Rule::variable => {
            if pair.as_str() == "_" {
                Variable::Anonymous
            } else {
                let inner = pair.into_inner().next().unwrap();
                match inner.as_rule() {
                    Rule::name => Variable::Named(inner.as_str().to_string()),
                    _ => Variable::Named(inner.as_str().to_string()),
                }
            }
        }
        _ => unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operation() {
        let result = HExprParser::parse_expr("add").unwrap();
        assert_eq!(result, Expr::Operation("add".to_string()));
    }

    #[test]
    fn test_frobenius_identity() {
        let result = HExprParser::parse_expr("[x]").unwrap();
        assert_eq!(
            result,
            Expr::Frobenius {
                inputs: vec![Variable::Named("x".to_string())],
                outputs: vec![Variable::Named("x".to_string())],
            }
        );
    }

    #[test]
    fn test_frobenius_full() {
        let result = HExprParser::parse_expr("[x x . x]").unwrap();
        assert_eq!(
            result,
            Expr::Frobenius {
                inputs: vec![Variable::Named("x".to_string()), Variable::Named("x".to_string())],
                outputs: vec![Variable::Named("x".to_string())],
            }
        );
    }

    #[test]
    fn test_composition() {
        let result = HExprParser::parse_expr("(add sub)").unwrap();
        assert_eq!(
            result,
            Expr::Composition(vec![
                Expr::Operation("add".to_string()),
                Expr::Operation("sub".to_string()),
            ])
        );
    }

    #[test]
    fn test_tensor() {
        let result = HExprParser::parse_expr("{add sub}").unwrap();
        assert_eq!(
            result,
            Expr::Tensor(vec![
                Expr::Operation("add".to_string()),
                Expr::Operation("sub".to_string()),
            ])
        );
    }
}