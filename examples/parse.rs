use hexpr::{parse_hexprs, Hexpr};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse a single hexpr
    let expr: Hexpr = "({copy [y]} {[x0] ([x1 y . y x1] imp)} imp)".parse()?;
    println!("{:?}", expr);

    // Parse many hexprs
    let expr: Vec<Hexpr> = parse_hexprs("(foo)(bar)")?;
    println!("{:?}", expr);

    let err: Result<Hexpr, _> = "{x (a b {c ) y}".parse();
    match err {
        Ok(x) => println!("{}", x),
        Err(e) => eprintln!("{}", e),
    }
    Ok(())
}
