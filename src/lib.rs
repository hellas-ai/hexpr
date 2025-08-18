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
/// use hexpr::{parse, HObject, OperationType};
/// use std::collections::HashMap;
///
/// let mut signature = HashMap::new();
/// let real = HObject::from("ℝ");
/// signature.insert("+".to_string(), OperationType::new(vec![real.clone(), real.clone()], vec![real]));
///
/// let result = parse("([x . x x] +)", signature);
/// assert!(result.is_ok());
/// ```
pub fn parse(
    hexpr: &str,
    signature: HashMap<String, OperationType<HObject>>,
) -> Result<OpenHypergraph<String, HOperation>, Box<dyn std::error::Error>> {
    // Step 1: Parse the H-expression string
    let expr =
        HExprParser::parse_expr(hexpr).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    // Step 2: Translate with the given signature
    let mut open_hypergraph = translate_expr_with_signature(&expr, signature)?;

    // Step 3: Perform inference to resolve unknown labels
    propagate_object_labels(&mut open_hypergraph)?;

    // Step 4: Verify no Unknown values remain and convert to String labels
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
    let final_hypergraph = with_node_labels(open_hypergraph, string_labels);

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

#[cfg(test)]
mod tests;
