use hexpr::*;
use open_hypergraphs::lax::OpenHypergraph;

#[derive(Debug, Clone)]
enum ArithOp {
    Add,
    Neg,
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
struct Error(String);

impl TryFrom<&Operation> for ArithOp {
    type Error = Error;

    fn try_from(op: &Operation) -> Result<Self, Self::Error> {
        match op.as_str() {
            "add" => Ok(ArithOp::Add),
            "neg" => Ok(ArithOp::Neg),
            op => Err(Error(format!("invalid op: {}", op))),
        }
    }
}

impl Signature<()> for ArithOp {
    fn source(&self) -> Vec<Option<()>> {
        let ob = Some(());
        match self {
            Self::Add => vec![ob, ob],
            Self::Neg => vec![ob],
        }
    }

    fn target(&self) -> Vec<Option<()>> {
        let ob = Some(());
        match self {
            Self::Add => vec![ob],
            Self::Neg => vec![ob],
        }
    }
}

#[test]
fn test_simple_operation() -> anyhow::Result<()> {
    let hexpr = "add".parse()?;
    let result: OpenHypergraph<Option<()>, ArithOp> = try_interpret(&hexpr)?;
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
    let result: OpenHypergraph<Option<()>, ArithOp> = try_interpret(&hexpr)?;
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
    let result: OpenHypergraph<Option<()>, ArithOp> = try_interpret(&hexpr)?;

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
    let result: OpenHypergraph<Option<()>, ArithOp> = try_interpret(&hexpr)?;
    let mut result = unify(result)?;

    // NOTE: this will panic if nodes cannot be quotiented!
    result.quotient();

    assert_eq!(result.sources.len(), 3);
    assert_eq!(result.targets.len(), 1);
    assert_eq!(result.hypergraph.edges.len(), 3);
    assert_eq!(result.hypergraph.nodes.len(), 5);

    Ok(())
}
