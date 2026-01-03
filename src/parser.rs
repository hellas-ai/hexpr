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
        Ok(parse_hexpr(expr_pair))
    }
}

fn parse_hexpr(pair: pest::iterators::Pair<Rule>) -> Hexpr {
    match pair.as_rule() {
        Rule::hexpr => pair.into_inner().map(parse_hexpr).next().unwrap(),
        Rule::composition => Hexpr::Composition(pair.into_inner().map(parse_hexpr).collect()),
        Rule::tensor => Hexpr::Tensor(pair.into_inner().map(parse_hexpr).collect()),
        Rule::frobenius => parse_frobenius(pair),
        Rule::operation => Hexpr::Operation(Operation(pair.as_str().to_string())),
        _ => unreachable!(),
    }
}

fn parse_frobenius(pair: pest::iterators::Pair<Rule>) -> Hexpr {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::frobenius_full => {
            let parts: Vec<_> = inner.into_inner().collect();
            let dot_pos = parts.iter().position(|p| p.as_rule() == Rule::dot).unwrap();
            let sources = parts[..dot_pos].iter().map(|p| parse_variable(p.clone())).collect();
            let targets = parts[dot_pos + 1..].iter().map(|p| parse_variable(p.clone())).collect();
            Hexpr::Frobenius { sources, targets }
        }
        Rule::frobenius_identity => {
            let sources: Vec<Variable> = inner.into_inner().map(parse_variable).collect();
            let targets = sources.clone();
            Hexpr::Frobenius { sources, targets }
        }
        _ => unreachable!(),
    }
}

fn parse_variable(pair: pest::iterators::Pair<Rule>) -> Variable {
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
                sources: vec![Variable("x".to_string())],
                targets: vec![Variable("x".to_string())],
            }
        );
    }

    #[test]
    fn test_frobenius_full() {
        let result = HExprParser::parse_expr("[x x . x]").unwrap();
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
                sources: vec![],
                targets: vec![],
            }
        );
    }

    #[test]
    fn test_comments_in_expressions() {
        let result = HExprParser::parse_expr("(foo // this is a comment\n bar)").unwrap();
        assert_eq!(
            result,
            Hexpr::Composition(vec![
                Hexpr::Operation(Operation("foo".to_string())),
                Hexpr::Operation(Operation("bar".to_string())),
            ])
        );
    }
}
