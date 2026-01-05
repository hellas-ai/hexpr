use crate::ast::{Hexpr, Operation, Variable};
use pest::{error::Error, Parser};
use pest_derive::Parser;

/// Parse multiple H-expressions from a string
pub fn parse_hexprs(input: &str) -> Result<Vec<Hexpr>, Error<Rule>> {
    HExprParser::parse_hexprs(input)
}

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct HExprParser;

impl HExprParser {
    pub fn parse_hexpr(input: &str) -> Result<Hexpr, Error<Rule>> {
        let pairs = HExprParser::parse(Rule::one_hexpr, input)?;
        let one_hexpr = pairs.into_iter().next().unwrap();
        let expr_pair = one_hexpr.into_inner().next().unwrap();
        Ok(parse_hexpr(expr_pair))
    }

    pub fn parse_hexprs(input: &str) -> Result<Vec<Hexpr>, Error<Rule>> {
        let pairs = HExprParser::parse(Rule::hexprs, input)?;
        let hexprs = pairs.into_iter().next().unwrap();
        Ok(hexprs
            .into_inner()
            .filter(|p| p.as_rule() == Rule::hexpr)
            .map(parse_hexpr)
            .collect())
    }
}

fn parse_hexpr(pair: pest::iterators::Pair<Rule>) -> Hexpr {
    match pair.as_rule() {
        Rule::hexpr => pair.into_inner().map(parse_hexpr).next().unwrap(),
        Rule::composition => Hexpr::Composition(pair.into_inner().map(parse_hexpr).collect()),
        Rule::tensor => Hexpr::Tensor(pair.into_inner().map(parse_hexpr).collect()),
        Rule::frobenius => parse_frobenius(pair),
        Rule::operation => Hexpr::Operation(Operation(pair.as_str().to_string())),
        x => panic!("unreachable: {:?}", x),
    }
}

fn parse_frobenius(pair: pest::iterators::Pair<Rule>) -> Hexpr {
    let mut it = pair.into_inner();
    let sources = parse_vars(it.next().unwrap());
    let targets = it.next().map(parse_vars).unwrap_or_else(|| sources.clone());
    Hexpr::Frobenius { sources, targets }
}

fn parse_vars(pair: pest::iterators::Pair<Rule>) -> Vec<Variable> {
    debug_assert_eq!(pair.as_rule(), Rule::vars);
    pair.into_inner().map(parse_variable).collect()
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
                sources: vec![Variable("x".to_string())],
                targets: vec![Variable("x".to_string())],
            }
        );
    }

    #[test]
    fn test_frobenius_full() {
        let result = HExprParser::parse_hexpr("[x x . x]").unwrap();
        assert_eq!(
            result,
            Hexpr::Frobenius {
                sources: vec![Variable("x".to_string()), Variable("x".to_string())],
                targets: vec![Variable("x".to_string())],
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
