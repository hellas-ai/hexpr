use clap::{Arg, Command};
use hexpr::{
    propagate_object_labels, to_svg,
    translate::{HObject, HOperation},
    translate_expr_with_signature, HExprParser, OperationType,
};
use open_hypergraphs::lax::OpenHypergraph;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read, Write};

fn apply_quotient_if_needed(
    mut hypergraph: OpenHypergraph<HObject, HOperation>,
    quotient: bool,
) -> Result<OpenHypergraph<HObject, HOperation>, hexpr::translate::TranslationError> {
    if quotient {
        // First propagate object labels to resolve Unknown/Known conflicts
        propagate_object_labels(&mut hypergraph)?;
        // Then apply quotient
        hypergraph.quotient();
    }
    Ok(hypergraph)
}

fn load_signature_from_file(
    path: &str,
) -> Result<HashMap<String, OperationType<HObject>>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let json_signature: HashMap<String, serde_json::Value> = serde_json::from_str(&content)?;

    let mut signature = HashMap::new();

    for (name, sig_json) in json_signature {
        let inputs: Vec<String> = serde_json::from_value(sig_json["inputs"].clone())?;
        let outputs: Vec<String> = serde_json::from_value(sig_json["outputs"].clone())?;

        let input_objects: Vec<HObject> = inputs.into_iter().map(HObject::from).collect();
        let output_objects: Vec<HObject> = outputs.into_iter().map(HObject::from).collect();

        signature.insert(name, OperationType::new(input_objects, output_objects));
    }

    Ok(signature)
}

fn create_default_signature() -> HashMap<String, OperationType<HObject>> {
    // Return empty signature - no built-in operations
    HashMap::new()
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
        .arg(
            Arg::new("signature")
                .short('s')
                .long("signature")
                .value_name("FILE")
                .help("JSON file containing operation signature (if not provided, uses empty signature)"),
        )
        .get_matches();

    let input = matches.get_one::<String>("INPUT").unwrap();
    let pretty = matches.get_flag("pretty");
    let debug = matches.get_flag("debug");
    let translate = matches.get_flag("translate");
    let visualize = matches.get_flag("visualize");
    let quotient = matches.get_flag("quotient");
    let signature_file = matches.get_one::<String>("signature");

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
                let signature = if let Some(file_path) = signature_file {
                    load_signature_from_file(file_path).unwrap_or_else(|e| {
                        eprintln!(
                            "Warning: Could not load signature from {}: {}",
                            file_path, e
                        );
                        eprintln!("Using empty signature instead.");
                        create_default_signature()
                    })
                } else {
                    eprintln!("Warning: No signature file provided, using empty signature.");
                    create_default_signature()
                };
                match translate_expr_with_signature(&expr, signature) {
                    Ok(hypergraph) => match apply_quotient_if_needed(hypergraph, quotient) {
                        Ok(processed_hypergraph) => match to_svg(&processed_hypergraph) {
                            Ok(svg_bytes) => {
                                io::stdout().write_all(&svg_bytes).unwrap();
                            }
                            Err(e) => {
                                eprintln!("SVG generation error: {}", e);
                                std::process::exit(1);
                            }
                        },
                        Err(e) => {
                            eprintln!("Type inference error: {}", e);
                            std::process::exit(1);
                        }
                    },
                    Err(e) => {
                        eprintln!("Translation error: {}", e);
                        std::process::exit(1);
                    }
                }
            } else if translate {
                let signature = if let Some(file_path) = signature_file {
                    load_signature_from_file(file_path).unwrap_or_else(|e| {
                        eprintln!(
                            "Warning: Could not load signature from {}: {}",
                            file_path, e
                        );
                        eprintln!("Using empty signature instead.");
                        create_default_signature()
                    })
                } else {
                    eprintln!("Warning: No signature file provided, using empty signature.");
                    create_default_signature()
                };
                match translate_expr_with_signature(&expr, signature) {
                    Ok(hypergraph) => match apply_quotient_if_needed(hypergraph, quotient) {
                        Ok(processed_hypergraph) => {
                            println!("Open Hypergraph: {:#?}", processed_hypergraph);
                        }
                        Err(e) => {
                            eprintln!("Type inference error: {}", e);
                            std::process::exit(1);
                        }
                    },
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

    use hexpr::Expr;

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
