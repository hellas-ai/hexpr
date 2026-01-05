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

    fn profile(&self, op: &Self::Arr) -> (Vec<Option<Self::Obj>>, Vec<Option<Self::Obj>>) {
        let ob = Some(());
        match op {
            ArithOp::Add => (vec![ob, ob], vec![ob]),
            ArithOp::Neg => (vec![ob], vec![ob]),
        }
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
fn test_all() -> anyhow::Result<()> {
    let hexpr = "({[x y . x] neg} add neg [y])".parse()?;
    let signature = PolyCirc;
    let result: OpenHypergraph<Option<()>, ArithOp> = try_interpret(&signature, &hexpr)?;
    let mut result = unify(result)?;

    // NOTE: this will panic if nodes cannot be quotiented!
    result.quotient();

    assert_eq!(result.sources.len(), 3);
    assert_eq!(result.targets.len(), 1);
    assert_eq!(result.hypergraph.edges.len(), 3);
    assert_eq!(result.hypergraph.nodes.len(), 5);

    Ok(())
}
