use hexpr::*;
use open_hypergraphs::lax::OpenHypergraph;

/// Operations in polynomial circuits
#[derive(Debug, Clone)]
enum ArithOp {
    Add,
    Neg,
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
struct ParseError(String);

// the signature of polynomial circuits
struct PolyCirc;

impl Signature for PolyCirc {
    type Arr = ArithOp;
    type Obj = ();
    type Error = ParseError;

    fn try_parse_op(&self, op: &Operation) -> Result<Self::Arr, Self::Error> {
        match op.as_str() {
            "add" => Ok(ArithOp::Add),
            "neg" => Ok(ArithOp::Neg),
            op => Err(ParseError(format!("invalid op: {}", op))),
        }
    }

    fn try_parse_object(&self, _object: &Hexpr) -> Result<Self::Obj, Self::Error> {
        Ok(())
    }

    fn profile(&self, op: &Self::Arr) -> (Vec<Option<Self::Obj>>, Vec<Option<Self::Obj>>) {
        let ob = Some(());
        match op {
            ArithOp::Add => (vec![ob, ob], vec![ob]),
            ArithOp::Neg => (vec![ob], vec![ob]),
        }
    }
}

struct LabelSig;

impl Signature for LabelSig {
    type Arr = ();
    type Obj = String;
    type Error = ParseError;

    fn try_parse_op(&self, op: &Operation) -> Result<Self::Arr, Self::Error> {
        Err(ParseError(format!("invalid op: {}", op)))
    }

    fn try_parse_object(&self, object: &Hexpr) -> Result<Self::Obj, Self::Error> {
        match object {
            Hexpr::Operation(op) => Ok(op.as_str().to_string()),
            object => Err(ParseError(format!("invalid object: {}", object))),
        }
    }

    fn profile(&self, _op: &Self::Arr) -> (Vec<Option<Self::Obj>>, Vec<Option<Self::Obj>>) {
        unreachable!("LabelSig has no operations")
    }
}

#[test]
fn test_simple_operation() -> anyhow::Result<()> {
    let hexpr = "add".parse()?;
    let signature = PolyCirc;
    let result: OpenHypergraph<Option<()>, ArithOp> = try_interpret(&signature, &hexpr)?;
    let result = unify(result)?;

    assert_eq!(result.sources.len(), 2);
    assert_eq!(result.targets.len(), 1);
    assert_eq!(result.hypergraph.edges.len(), 1);
    assert_eq!(result.hypergraph.nodes.len(), 3);

    Ok(())
}

#[test]
fn test_composition() -> anyhow::Result<()> {
    let hexpr = "(add neg)".parse()?;
    let signature = PolyCirc;
    let result: OpenHypergraph<Option<()>, ArithOp> = try_interpret(&signature, &hexpr)?;
    let result = unify(result)?;

    assert_eq!(result.sources.len(), 2);
    assert_eq!(result.targets.len(), 1);
    assert_eq!(result.hypergraph.edges.len(), 2);
    assert_eq!(result.hypergraph.nodes.len(), 4);

    Ok(())
}

#[test]
fn test_frobenius() -> anyhow::Result<()> {
    let hexpr = "[x y . x]".parse()?;
    let signature = PolyCirc;
    let result: OpenHypergraph<Option<()>, ArithOp> = try_interpret(&signature, &hexpr)?;

    assert_eq!(result.sources.len(), 2);
    assert_eq!(result.targets.len(), 1);
    assert_eq!(result.hypergraph.edges.len(), 0);
    assert_eq!(result.hypergraph.nodes.len(), 2);

    // We have no annotations, so unification should fail
    assert!(unify(result).is_err());

    Ok(())
}

#[test]
fn test_interpret_with_names() -> anyhow::Result<()> {
    let hexpr = "[x y . x]".parse()?;
    let signature = PolyCirc;
    let result = try_interpret_with_names(&signature, &hexpr)?;

    assert_eq!(result.graph.sources.len(), 2);
    assert_eq!(result.graph.targets.len(), 1);
    assert_eq!(result.names.len(), 2);
    assert_eq!(
        result.names.get(&result.graph.sources[0]),
        Some(&vec!["x".parse()?])
    );
    assert_eq!(
        result.names.get(&result.graph.sources[1]),
        Some(&vec!["y".parse()?])
    );
    assert_eq!(result.graph.targets[0], result.graph.sources[0]);

    Ok(())
}

#[test]
fn test_unify_with_names() -> anyhow::Result<()> {
    let hexpr = "([.x] [y] neg)".parse()?;
    let signature = PolyCirc;
    let result = try_interpret_with_names(&signature, &hexpr)?;
    let result = result.unify()?;

    assert_eq!(result.graph.sources.len(), 0);
    assert_eq!(result.graph.targets.len(), 1);
    assert_eq!(result.names.len(), 1);

    let variables = result.names.values().next().unwrap();
    assert_eq!(variables.len(), 2);
    assert!(variables.contains(&"x".parse()?));
    assert!(variables.contains(&"y".parse()?));

    Ok(())
}

#[test]
fn test_all() -> anyhow::Result<()> {
    let hexpr = "({[x y . x] neg} add neg [y])".parse()?;
    let signature = PolyCirc;
    let result: OpenHypergraph<Option<()>, ArithOp> = try_interpret(&signature, &hexpr)?;
    let mut result = unify(result)?;

    result
        .quotient()
        .expect("quotient should succeed after unification");

    assert_eq!(result.sources.len(), 3);
    assert_eq!(result.targets.len(), 1);
    assert_eq!(result.hypergraph.edges.len(), 3);
    assert_eq!(result.hypergraph.nodes.len(), 5);

    Ok(())
}

#[test]
fn test_hole_interprets_as_unknown_identity() -> anyhow::Result<()> {
    let hexpr = "_".parse()?;
    let signature = LabelSig;
    let result = try_interpret(&signature, &hexpr)?;

    assert_eq!(result.sources.len(), 1);
    assert_eq!(result.targets.len(), 1);
    assert_eq!(result.sources[0], result.targets[0]);
    assert_eq!(result.hypergraph.nodes[result.sources[0].0], None);
    assert!(unify(result).is_err());

    Ok(())
}

#[test]
fn test_labeled_wire_interprets_as_typed_identity() -> anyhow::Result<()> {
    let hexpr = "^num".parse()?;
    let signature = LabelSig;
    let result = try_interpret(&signature, &hexpr)?;

    assert_eq!(result.sources.len(), 1);
    assert_eq!(result.targets.len(), 1);
    assert_eq!(result.sources[0], result.targets[0]);
    assert_eq!(
        result.hypergraph.nodes[result.sources[0].0],
        Some("num".to_string())
    );

    let result = unify(result)?;
    assert_eq!(result.hypergraph.nodes[result.sources[0].0], "num");

    Ok(())
}

#[test]
fn test_labeled_spider_keeps_global_binding() -> anyhow::Result<()> {
    let hexpr = "([x^num .] [. x])".parse()?;
    let signature = LabelSig;
    let result = try_interpret_with_names(&signature, &hexpr)?;

    assert_eq!(result.graph.sources.len(), 1);
    assert_eq!(result.graph.targets.len(), 1);
    assert_eq!(result.graph.sources[0], result.graph.targets[0]);
    assert_eq!(
        result.graph.hypergraph.nodes[result.graph.sources[0].0],
        Some("num".to_string())
    );

    Ok(())
}

#[test]
fn test_conflicting_object_annotations_fail_at_unify() -> anyhow::Result<()> {
    let hexpr = "[x^num . x^bool]".parse()?;
    let signature = LabelSig;
    let result = try_interpret(&signature, &hexpr)?;

    assert!(unify(result).is_err());

    Ok(())
}

#[test]
fn test_empty_forms_interpret_as_empty_diagram() -> anyhow::Result<()> {
    let signature = LabelSig;

    for input in ["()", "[]", "{}"] {
        let hexpr = input.parse()?;
        let result = try_interpret(&signature, &hexpr)?;

        assert_eq!(result.sources.len(), 0, "sources for {input}");
        assert_eq!(result.targets.len(), 0, "targets for {input}");
        assert_eq!(result.hypergraph.nodes.len(), 0, "nodes for {input}");
        assert_eq!(result.hypergraph.edges.len(), 0, "edges for {input}");
    }

    Ok(())
}
