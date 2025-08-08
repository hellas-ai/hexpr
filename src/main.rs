use clap::{Arg, Command};
use h_exprs::HExprParser;
use std::io::{self, Read};

fn main() {
    let matches = Command::new("h-exprs")
        .version("0.1.0")
        .about("Parser and pretty-printer for H-Expressions")
        .arg(
            Arg::new("INPUT")
                .help("H-expression to parse (use '-' to read from stdin)")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("pretty")
                .short('p')
                .long("pretty")
                .action(clap::ArgAction::SetTrue)
                .help("Pretty-print the parsed expression"),
        )
        .arg(
            Arg::new("debug")
                .short('d')
                .long("debug")
                .action(clap::ArgAction::SetTrue)
                .help("Show debug AST representation"),
        )
        .get_matches();

    let input = matches.get_one::<String>("INPUT").unwrap();
    let pretty = matches.get_flag("pretty");
    let debug = matches.get_flag("debug");

    let expr_str = if input == "-" {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .expect("Failed to read from stdin");
        buffer
    } else {
        input.clone()
    };

    match HExprParser::parse_expr(&expr_str) {
        Ok(expr) => {
            if debug {
                println!("Debug AST: {:#?}", expr);
            } else if pretty {
                println!("Parsed: {}", expr);
            } else {
                println!("{}", expr);
            }
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    use h_exprs::Expr;

    #[test]
    fn test_cli_basic_parsing() {
        let expr = HExprParser::parse_expr("[x x . x]").unwrap();
        assert!(matches!(expr, Expr::Frobenius { .. }));
    }

    #[test]
    fn test_cli_complex_expression() {
        let expr = HExprParser::parse_expr("({[_] -} +)").unwrap();
        assert!(matches!(expr, Expr::Composition(_)));
    }
}
