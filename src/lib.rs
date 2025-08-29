pub mod ast;
pub mod inference;
pub mod parser;
pub mod translate;

pub use ast::{Expr, Variable};
pub use inference::propagate_object_labels;
pub use parser::HExprParser;
pub use translate::{
    to_svg, translate_expr_with_signature, HObject, HOperation, OperationType, TranslationError,
    Translator,
};

use open_hypergraphs::lax::{Hypergraph, OpenHypergraph};
use std::collections::HashMap;

/// Parse an H-Expression, translate to an open hypergraph using the supplied signature, then
/// resolve unknown labels using type inference.
///
/// # Arguments
/// * `hexpr_string` - The H-expression string to parse
/// * `signature` - Operation signatures mapping operation names to their input/output types
///
/// # Returns
/// Result containing the processed open hypergraph or an error
///
/// # Example
/// ```
/// use hexpr::{parse, OperationType};
/// use std::collections::HashMap;
///
/// let mut signature = HashMap::new();
/// signature.insert("+".to_string(), OperationType::new(vec!["ℝ".to_string(), "ℝ".to_string()], vec!["ℝ".to_string()]));
///
/// let result = parse("([x . x x] +)", signature);
/// assert!(result.is_ok());
/// ```
pub fn parse(
    hexpr: &str,
    signature: HashMap<String, OperationType<String>>,
) -> Result<OpenHypergraph<String, String>, Box<dyn std::error::Error>> {
    // Step 1: Parse the H-expression string
    let expr =
        HExprParser::parse_expr(hexpr).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    // Step 2: Convert String signature to HObject signature
    let hobject_signature: HashMap<String, OperationType<HObject>> = signature
        .into_iter()
        .map(|(name, op_type)| {
            let inputs = op_type.inputs.into_iter().map(HObject::from).collect();
            let outputs = op_type.outputs.into_iter().map(HObject::from).collect();
            (name, OperationType::new(inputs, outputs))
        })
        .collect();

    // Step 3: Translate with the converted signature
    let mut open_hypergraph = translate_expr_with_signature(&expr, hobject_signature)?;

    // Step 4: Perform inference to resolve unknown labels
    propagate_object_labels(&mut open_hypergraph)?;

    // Step 5: Verify no Unknown values remain and convert to String labels
    let string_labels: Result<Vec<String>, _> = open_hypergraph
        .hypergraph
        .nodes
        .iter()
        .map(|node| match node {
            HObject::Named(name) => Ok(name.clone()),
            HObject::Unknown => Err(TranslationError {
                message: "Unknown object type remains after inference".to_string(),
            }),
        })
        .collect();

    let string_labels = string_labels?;
    let node_converted_hypergraph = with_node_labels(open_hypergraph, string_labels);

    // Step 6: Convert HOperation edge labels to String
    let string_edge_labels: Vec<String> = node_converted_hypergraph
        .hypergraph
        .edges
        .iter()
        .map(|edge| edge.0.clone())
        .collect();

    let final_hypergraph = with_edge_labels(node_converted_hypergraph, string_edge_labels);

    Ok(final_hypergraph)
}
//

fn with_node_labels<X, T, U>(
    term: OpenHypergraph<T, X>,
    new_node_labels: Vec<U>,
) -> OpenHypergraph<U, X> {
    OpenHypergraph {
        hypergraph: Hypergraph {
            nodes: new_node_labels,
            edges: term.hypergraph.edges,
            adjacency: term.hypergraph.adjacency,
            quotient: term.hypergraph.quotient,
        },
        sources: term.sources,
        targets: term.targets,
    }
}

fn with_edge_labels<T, X, Y>(
    term: OpenHypergraph<T, X>,
    new_edge_labels: Vec<Y>,
) -> OpenHypergraph<T, Y> {
    OpenHypergraph {
        hypergraph: Hypergraph {
            nodes: term.hypergraph.nodes,
            edges: new_edge_labels,
            adjacency: term.hypergraph.adjacency,
            quotient: term.hypergraph.quotient,
        },
        sources: term.sources,
        targets: term.targets,
    }
}

#[cfg(test)]
mod tests;
