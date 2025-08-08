use clap::{Arg, Command};
use h_exprs::{
    propagate_object_labels, to_svg,
    translate::{HObject, HOperation},
    translate_expr_with_signatures, HExprParser, OperationSignature,
};
use open_hypergraphs::lax::OpenHypergraph;
use std::collections::HashMap;
use std::io::{self, Read, Write};

fn apply_quotient_if_needed(
    mut hypergraph: OpenHypergraph<HObject, HOperation>,
    quotient: bool,
) -> OpenHypergraph<HObject, HOperation> {
    if quotient {
        // First propagate object labels to resolve Unknown/Known conflicts
        propagate_object_labels(&mut hypergraph);
        // Then apply quotient
        hypergraph.quotient();
    }
    hypergraph
}

fn create_default_signatures() -> HashMap<String, OperationSignature<HObject>> {
    let mut signatures = HashMap::new();

    let obj = HObject::from("ℝ"); // Using ℝ (real numbers) as the default object type

    // Binary operations: 2 → 1
    signatures.insert(
        "+".to_string(),
        OperationSignature::new(vec![obj.clone(), obj.clone()], vec![obj.clone()]),
    );
    signatures.insert(
        "-".to_string(),
        OperationSignature::new(vec![obj.clone(), obj.clone()], vec![obj.clone()]),
    );
    signatures.insert(
        "*".to_string(),
        OperationSignature::new(vec![obj.clone(), obj.clone()], vec![obj.clone()]),
    );
    signatures.insert(
        "/".to_string(),
        OperationSignature::new(vec![obj.clone(), obj.clone()], vec![obj.clone()]),
    );

    // Unary operations: 1 → 1
    signatures.insert(
        "neg".to_string(),
        OperationSignature::new(vec![obj.clone()], vec![obj.clone()]),
    );

    // Structural operations
    signatures.insert(
        "copy".to_string(),
        OperationSignature::new(vec![obj.clone()], vec![obj.clone(), obj.clone()]),
    ); // 1 → 2
    signatures.insert(
        "discard".to_string(),
        OperationSignature::new(vec![obj.clone()], vec![]),
    ); // 1 → 0
    signatures.insert(
        "create".to_string(),
        OperationSignature::new(vec![], vec![obj.clone()]),
    ); // 0 → 1

    signatures
}

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
        .arg(
            Arg::new("translate")
                .short('t')
                .long("translate")
                .action(clap::ArgAction::SetTrue)
                .help("Translate to open hypergraph representation"),
        )
        .arg(
            Arg::new("visualize")
                .short('v')
                .long("visualize")
                .action(clap::ArgAction::SetTrue)
                .help("Generate DOT visualization of the open hypergraph"),
        )
        .arg(
            Arg::new("quotient")
                .short('q')
                .long("quotient")
                .action(clap::ArgAction::SetTrue)
                .help("Apply quotient operation to the output hypergraph"),
        )
        .get_matches();

    let input = matches.get_one::<String>("INPUT").unwrap();
    let pretty = matches.get_flag("pretty");
    let debug = matches.get_flag("debug");
    let translate = matches.get_flag("translate");
    let visualize = matches.get_flag("visualize");
    let quotient = matches.get_flag("quotient");

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
            } else if visualize {
                let signatures = create_default_signatures();
                match translate_expr_with_signatures(&expr, signatures) {
                    Ok(hypergraph) => {
                        let processed_hypergraph = apply_quotient_if_needed(hypergraph, quotient);
                        match to_svg(&processed_hypergraph) {
                            Ok(svg_bytes) => {
                                io::stdout().write_all(&svg_bytes).unwrap();
                            }
                            Err(e) => {
                                eprintln!("SVG generation error: {}", e);
                                std::process::exit(1);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Translation error: {}", e);
                        std::process::exit(1);
                    }
                }
            } else if translate {
                let signatures = create_default_signatures();
                match translate_expr_with_signatures(&expr, signatures) {
                    Ok(hypergraph) => {
                        let processed_hypergraph = apply_quotient_if_needed(hypergraph, quotient);
                        println!("Open Hypergraph: {:#?}", processed_hypergraph);
                    }
                    Err(e) => {
                        eprintln!("Translation error: {}", e);
                        std::process::exit(1);
                    }
                }
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
        let expr = HExprParser::parse_expr("({[a] -} +)").unwrap();
        assert!(matches!(expr, Expr::Composition(_)));
    }
}
