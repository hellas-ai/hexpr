use crate::ast::{Expr, Variable};
use open_hypergraphs::lax::{Hyperedge, NodeId, OpenHypergraph};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct HObject(pub String);

impl std::fmt::Display for HObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for HObject {
    fn from(s: String) -> Self {
        HObject(s)
    }
}

impl From<&str> for HObject {
    fn from(s: &str) -> Self {
        HObject(s.to_string())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HOperation(pub String);

impl std::fmt::Display for HOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for HOperation {
    fn from(s: String) -> Self {
        HOperation(s)
    }
}

impl From<&str> for HOperation {
    fn from(s: &str) -> Self {
        HOperation(s.to_string())
    }
}

#[derive(Debug)]
pub struct TranslationError {
    pub message: String,
}

impl std::fmt::Display for TranslationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Translation error: {}", self.message)
    }
}

impl std::error::Error for TranslationError {}

#[derive(Debug, Clone)]
pub struct OperationSignature<O> {
    pub inputs: Vec<O>,
    pub outputs: Vec<O>,
}

impl<O> OperationSignature<O> {
    pub fn new(inputs: Vec<O>, outputs: Vec<O>) -> Self {
        Self { inputs, outputs }
    }
}

pub struct Translator {
    variables: HashMap<String, NodeId>,
    anonymous_counter: usize,
    operation_signatures: HashMap<String, OperationSignature<HObject>>,
}

impl Translator {
    pub fn new(signatures: HashMap<String, OperationSignature<HObject>>) -> Self {
        Self {
            variables: HashMap::new(),
            anonymous_counter: 0,
            operation_signatures: signatures,
        }
    }

    pub fn add_operation(&mut self, name: String, signature: OperationSignature<HObject>) {
        self.operation_signatures.insert(name, signature);
    }

    pub fn translate(
        &mut self,
        expr: &Expr,
    ) -> Result<OpenHypergraph<HObject, HOperation>, TranslationError> {
        let mut graph = OpenHypergraph::empty();
        let (sources, targets) = self.translate_expr(expr, &mut graph)?;
        graph.sources = sources;
        graph.targets = targets;
        Ok(graph)
    }

    fn translate_expr(
        &mut self,
        expr: &Expr,
        graph: &mut OpenHypergraph<HObject, HOperation>,
    ) -> Result<(Vec<NodeId>, Vec<NodeId>), TranslationError> {
        match expr {
            Expr::Operation(name) => self.translate_operation(name, graph),
            Expr::Frobenius { inputs, outputs } => self.translate_frobenius(inputs, outputs, graph),
            Expr::Composition(exprs) => self.translate_composition(exprs, graph),
            Expr::Tensor(exprs) => self.translate_tensor(exprs, graph),
        }
    }

    fn translate_operation(
        &mut self,
        name: &str,
        graph: &mut OpenHypergraph<HObject, HOperation>,
    ) -> Result<(Vec<NodeId>, Vec<NodeId>), TranslationError> {
        // Look up the operation signature
        let signature = self
            .operation_signatures
            .get(name)
            .cloned()
            .ok_or_else(|| TranslationError {
                message: format!("Unknown operation: '{}'", name),
            })?;

        // Create input nodes
        let input_nodes: Vec<NodeId> = signature
            .inputs
            .iter()
            .map(|obj| graph.new_node(obj.clone()))
            .collect();

        // Create output nodes
        let output_nodes: Vec<NodeId> = signature
            .outputs
            .iter()
            .map(|obj| graph.new_node(obj.clone()))
            .collect();

        let interface = Hyperedge {
            sources: input_nodes.clone(),
            targets: output_nodes.clone(),
        };

        graph.new_edge(HOperation::from(name), interface);

        Ok((input_nodes, output_nodes))
    }

    fn translate_frobenius(
        &mut self,
        inputs: &[Variable],
        outputs: &[Variable],
        graph: &mut OpenHypergraph<HObject, HOperation>,
    ) -> Result<(Vec<NodeId>, Vec<NodeId>), TranslationError> {
        // Create nodes for inputs and outputs
        let input_nodes: Vec<NodeId> = inputs
            .iter()
            .map(|_| graph.new_node(HObject::from("obj")))
            .collect();
        let output_nodes: Vec<NodeId> = outputs
            .iter()
            .map(|_| graph.new_node(HObject::from("obj")))
            .collect();

        // Create a frobenius relation edge
        let relation_name = format!("frobenius_{}_{}", inputs.len(), outputs.len());
        let interface = Hyperedge {
            sources: input_nodes.clone(),
            targets: output_nodes.clone(),
        };

        graph.new_edge(HOperation::from(relation_name), interface);

        // Handle variable unification through the quotient map
        self.unify_variables(inputs, &input_nodes, graph)?;
        self.unify_variables(outputs, &output_nodes, graph)?;

        Ok((input_nodes, output_nodes))
    }

    fn translate_composition(
        &mut self,
        exprs: &[Expr],
        graph: &mut OpenHypergraph<HObject, HOperation>,
    ) -> Result<(Vec<NodeId>, Vec<NodeId>), TranslationError> {
        if exprs.is_empty() {
            return Err(TranslationError {
                message: "Empty composition".to_string(),
            });
        }

        // Fold through the expressions, connecting outputs to inputs
        let (current_inputs, mut current_outputs) = self.translate_expr(&exprs[0], graph)?;

        for expr in &exprs[1..] {
            let (next_inputs, next_outputs) = self.translate_expr(expr, graph)?;

            // Connect current outputs to next inputs via quotient
            if current_outputs.len() != next_inputs.len() {
                return Err(TranslationError {
                    message: format!(
                        "Composition mismatch: {} outputs to {} inputs",
                        current_outputs.len(),
                        next_inputs.len()
                    ),
                });
            }

            for (&out_node, &in_node) in current_outputs.iter().zip(next_inputs.iter()) {
                graph.unify(out_node, in_node);
            }

            current_outputs = next_outputs;
        }

        Ok((current_inputs, current_outputs))
    }

    fn translate_tensor(
        &mut self,
        exprs: &[Expr],
        graph: &mut OpenHypergraph<HObject, HOperation>,
    ) -> Result<(Vec<NodeId>, Vec<NodeId>), TranslationError> {
        let mut all_inputs = Vec::new();
        let mut all_outputs = Vec::new();

        for expr in exprs {
            let (inputs, outputs) = self.translate_expr(expr, graph)?;
            all_inputs.extend(inputs);
            all_outputs.extend(outputs);
        }

        Ok((all_inputs, all_outputs))
    }

    fn unify_variables(
        &mut self,
        variables: &[Variable],
        nodes: &[NodeId],
        graph: &mut OpenHypergraph<HObject, HOperation>,
    ) -> Result<(), TranslationError> {
        for (var, &node) in variables.iter().zip(nodes.iter()) {
            match var {
                Variable::Named(name) => {
                    if let Some(&existing_node) = self.variables.get(name) {
                        // Unify with existing node for this variable name
                        graph.unify(node, existing_node);
                    } else {
                        // First occurrence of this variable name
                        self.variables.insert(name.clone(), node);
                    }
                }
                Variable::Anonymous => {
                    // Anonymous variables don't get unified across expressions
                    self.anonymous_counter += 1;
                }
            }
        }
        Ok(())
    }
}

pub fn translate_expr_with_signatures(
    expr: &Expr,
    signatures: HashMap<String, OperationSignature<HObject>>,
) -> Result<OpenHypergraph<HObject, HOperation>, TranslationError> {
    let mut translator = Translator::new(signatures);
    translator.translate(expr)
}

pub fn to_svg(term: &OpenHypergraph<HObject, HOperation>) -> Result<Vec<u8>, std::io::Error> {
    use graphviz_rust::{
        cmd::{CommandArg, Format},
        exec,
        printer::PrinterContext,
    };
    use open_hypergraphs_dot::{generate_dot_with, Orientation};

    let opts = open_hypergraphs_dot::Options {
        node_label: Box::new(|n| format!("{}", n)),
        edge_label: Box::new(|e| format!("{}", e)),
        orientation: Orientation::LR,
        ..Default::default()
    };

    let dot_graph = generate_dot_with(term, &opts);

    exec(
        dot_graph,
        &mut PrinterContext::default(),
        vec![CommandArg::Format(Format::Svg)],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::HExprParser;

    #[test]
    fn test_translate_simple_operation() {
        use std::collections::HashMap;

        let mut signatures = HashMap::new();
        let obj = HObject::from("ℝ");
        signatures.insert(
            "add".to_string(),
            OperationSignature::new(vec![obj.clone(), obj.clone()], vec![obj.clone()]),
        );

        let expr = HExprParser::parse_expr("add").unwrap();
        let result = translate_expr_with_signatures(&expr, signatures);
        assert!(result.is_ok());
    }

    #[test]
    fn test_translate_frobenius_join() {
        let expr = HExprParser::parse_expr("[x x . x]").unwrap();
        let result = translate_expr(&expr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_translate_composition() {
        use std::collections::HashMap;

        let mut signatures = HashMap::new();
        let obj = HObject::from("ℝ");
        // copy: 1->2, add: 2->1, so they can compose
        signatures.insert(
            "copy".to_string(),
            OperationSignature::new(vec![obj.clone()], vec![obj.clone(), obj.clone()]),
        );
        signatures.insert(
            "add".to_string(),
            OperationSignature::new(vec![obj.clone(), obj.clone()], vec![obj.clone()]),
        );

        let expr = HExprParser::parse_expr("(copy add)").unwrap();
        let result = translate_expr_with_signatures(&expr, signatures);
        assert!(result.is_ok());
    }

    #[test]
    fn test_translate_tensor() {
        use std::collections::HashMap;

        let mut signatures = HashMap::new();
        let obj = HObject::from("ℝ");
        signatures.insert(
            "add".to_string(),
            OperationSignature::new(vec![obj.clone(), obj.clone()], vec![obj.clone()]),
        );
        signatures.insert(
            "sub".to_string(),
            OperationSignature::new(vec![obj.clone(), obj.clone()], vec![obj.clone()]),
        );

        let expr = HExprParser::parse_expr("{add sub}").unwrap();
        let result = translate_expr_with_signatures(&expr, signatures);
        assert!(result.is_ok());
    }

    #[test]
    fn test_translate_with_proper_signatures() {
        use std::collections::HashMap;

        let mut signatures = HashMap::new();
        let obj = HObject::from("ℝ");
        signatures.insert(
            "copy".to_string(),
            OperationSignature::new(vec![obj.clone()], vec![obj.clone(), obj.clone()]),
        ); // 1->2
        signatures.insert(
            "+".to_string(),
            OperationSignature::new(vec![obj.clone(), obj.clone()], vec![obj.clone()]),
        ); // 2->1

        let expr = HExprParser::parse_expr("(copy +)").unwrap();
        let result = translate_expr_with_signatures(&expr, signatures);
        assert!(result.is_ok());

        let hypergraph = result.unwrap();
        // copy is 1->2, + is 2->1, so they should compose properly
        assert_eq!(hypergraph.hypergraph.edges.len(), 2);
    }

    #[test]
    fn test_proper_composition_signatures() {
        use std::collections::HashMap;

        let mut signatures = HashMap::new();
        let obj = HObject::from("ℝ");
        signatures.insert(
            "copy".to_string(),
            OperationSignature::new(vec![obj.clone()], vec![obj.clone(), obj.clone()]),
        ); // 1->2
        signatures.insert(
            "+".to_string(),
            OperationSignature::new(vec![obj.clone(), obj.clone()], vec![obj.clone()]),
        ); // 2->1
        signatures.insert(
            "neg".to_string(),
            OperationSignature::new(vec![obj.clone()], vec![obj.clone()]),
        ); // 1->1

        // Test that copy (1->2) followed by + (2->1) works properly
        let expr = HExprParser::parse_expr("(copy +)").unwrap();
        let result = translate_expr_with_signatures(&expr, signatures.clone());
        assert!(result.is_ok());

        // Test a composition mismatch - this should fail
        let expr = HExprParser::parse_expr("({copy neg} +)").unwrap();
        let result = translate_expr_with_signatures(&expr, signatures);
        // copy (1->2) tensored with neg (1->1) = 3 outputs, but + expects 2 inputs
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(e.message.contains("Composition mismatch"));
        }
    }

    #[test]
    fn test_custom_signatures() {
        use std::collections::HashMap;

        let mut signatures = HashMap::new();
        let obj = HObject::from("ℝ");
        signatures.insert(
            "triple".to_string(),
            OperationSignature::new(
                vec![obj.clone()],
                vec![obj.clone(), obj.clone(), obj.clone()],
            ),
        );
        signatures.insert(
            "merge3".to_string(),
            OperationSignature::new(
                vec![obj.clone(), obj.clone(), obj.clone()],
                vec![obj.clone()],
            ),
        );

        let expr = HExprParser::parse_expr("(triple merge3)").unwrap();
        let result = translate_expr_with_signatures(&expr, signatures);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unknown_operation_error() {
        use std::collections::HashMap;

        let signatures = HashMap::new(); // Empty signatures

        let expr = HExprParser::parse_expr("unknown_op").unwrap();
        let result = translate_expr_with_signatures(&expr, signatures);
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(e.message.contains("Unknown operation: 'unknown_op'"));
        }
    }
}
