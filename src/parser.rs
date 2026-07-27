use crate::ast::{Hexpr, LVar, Operation, Variable};
use pest::Parser;
use pest_derive::Parser;
use thiserror::Error;

#[derive(Debug, Error)]
#[error(transparent)]
pub struct ParseError(#[from] pest::error::Error<Rule>);

/// Parse multiple H-expressions from a string
pub fn parse_hexprs(input: &str) -> Result<Vec<Hexpr>, ParseError> {
    HExprParser::parse_hexprs(input)
}

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct HExprParser;

impl HExprParser {
    pub fn parse_hexpr(input: &str) -> Result<Hexpr, ParseError> {
        let pairs = HExprParser::parse(Rule::one_hexpr, input)?;
        let one_hexpr = pairs.into_iter().next().unwrap();
        let expr_pair = one_hexpr.into_inner().next().unwrap();
        Ok(parse_hexpr(expr_pair))
    }

    pub fn parse_hexprs(input: &str) -> Result<Vec<Hexpr>, ParseError> {
        let pairs = HExprParser::parse(Rule::hexprs, input)?;
        let hexprs = pairs.into_iter().next().unwrap();
        Ok(hexprs
            .into_inner()
            .filter(|p| p.as_rule() == Rule::hexpr)
            .map(parse_hexpr)
            .collect())
    }

    pub fn parse_lvar(input: &str) -> Result<LVar, ParseError> {
        let pairs = HExprParser::parse(Rule::lvar, input)?;
        let lvar = pairs.into_iter().next().unwrap();
        Ok(parse_lvar(lvar))
    }
}

fn parse_hexpr(pair: pest::iterators::Pair<Rule>) -> Hexpr {
    match pair.as_rule() {
        Rule::hexpr => pair.into_inner().map(parse_hexpr).next().unwrap(),
        Rule::composition => Hexpr::Composition(pair.into_inner().map(parse_hexpr).collect()),
        Rule::tensor => Hexpr::Tensor(pair.into_inner().map(parse_hexpr).collect()),
        Rule::frobenius => parse_frobenius(pair),
        Rule::wire => parse_wire(pair),
        Rule::hole => Hexpr::Hole,
        Rule::operation => Hexpr::Operation(Operation(pair.as_str().to_string())),
        x => panic!("unreachable: {:?}", x),
    }
}

fn parse_wire(pair: pest::iterators::Pair<Rule>) -> Hexpr {
    let label = pair.into_inner().next().unwrap();
    Hexpr::Wire(Box::new(parse_hexpr(label)))
}

fn parse_frobenius(pair: pest::iterators::Pair<Rule>) -> Hexpr {
    let mut it = pair.into_inner();
    let sources = parse_vars(it.next().unwrap());
    let targets = it.next().map(parse_vars).unwrap_or_else(|| sources.clone());
    Hexpr::Frobenius { sources, targets }
}

fn parse_vars(pair: pest::iterators::Pair<Rule>) -> Vec<LVar> {
    debug_assert_eq!(pair.as_rule(), Rule::vars);
    pair.into_inner().map(parse_lvar).collect()
}

fn parse_lvar(pair: pest::iterators::Pair<Rule>) -> LVar {
    let mut inner = pair.into_inner();
    let name = parse_variable(inner.next().unwrap());
    let label = inner.next().map(parse_hexpr);
    LVar { name, label }
}

fn parse_variable(pair: pest::iterators::Pair<Rule>) -> Variable {
    match pair.as_rule() {
        Rule::variable => Variable(pair.as_str().to_string()),
        x => panic!("unreachable: {:?}", x),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operation() -> anyhow::Result<()> {
        let result = HExprParser::parse_hexpr("add")?;
        assert_eq!(result, Hexpr::Operation(Operation("add".to_string())));
        Ok(())
    }

    #[test]
    fn test_frobenius_identity() {
        let result = HExprParser::parse_hexpr("[x]").unwrap();
        assert_eq!(
            result,
            Hexpr::Frobenius {
                sources: vec![Variable("x".to_string()).into()],
                targets: vec![Variable("x".to_string()).into()],
            }
        );
    }

    #[test]
    fn test_frobenius_full() {
        let result = HExprParser::parse_hexpr("[x x . x]").unwrap();
        assert_eq!(
            result,
            Hexpr::Frobenius {
                sources: vec![
                    Variable("x".to_string()).into(),
                    Variable("x".to_string()).into()
                ],
                targets: vec![Variable("x".to_string()).into()],
            }
        );
    }

    #[test]
    fn test_composition() {
        let result = HExprParser::parse_hexpr("(add sub)").unwrap();
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
        let result = HExprParser::parse_hexpr("{add sub}").unwrap();
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
        let result = HExprParser::parse_hexpr("[]").unwrap();
        assert_eq!(
            result,
            Hexpr::Frobenius {
                sources: vec![],
                targets: vec![],
            }
        );
    }

    #[test]
    fn test_empty_composition() {
        let result = HExprParser::parse_hexpr("()").unwrap();
        assert_eq!(result, Hexpr::Composition(vec![]));
    }

    #[test]
    fn test_empty_tensor() {
        let result = HExprParser::parse_hexpr("{}").unwrap();
        assert_eq!(result, Hexpr::Tensor(vec![]));
    }

    #[test]
    fn test_hole() {
        let result = HExprParser::parse_hexpr("_").unwrap();
        assert_eq!(result, Hexpr::Hole);
    }

    #[test]
    fn test_wire() {
        let result = HExprParser::parse_hexpr("^t").unwrap();
        assert_eq!(
            result,
            Hexpr::Wire(Box::new(Hexpr::Operation(Operation("t".to_string()))))
        );
    }

    #[test]
    fn test_labeled_frobenius_var() {
        let result = HExprParser::parse_hexpr("[x^t]").unwrap();
        assert_eq!(
            result,
            Hexpr::Frobenius {
                sources: vec![LVar {
                    name: Variable("x".to_string()),
                    label: Some(Hexpr::Operation(Operation("t".to_string()))),
                }],
                targets: vec![LVar {
                    name: Variable("x".to_string()),
                    label: Some(Hexpr::Operation(Operation("t".to_string()))),
                }],
            }
        );
    }

    #[test]
    fn test_comments_in_expressions() {
        let result = HExprParser::parse_hexpr("(foo # this is a comment\n bar)").unwrap();
        assert_eq!(
            result,
            Hexpr::Composition(vec![
                Hexpr::Operation(Operation("foo".to_string())),
                Hexpr::Operation(Operation("bar".to_string())),
            ])
        );
    }
}
