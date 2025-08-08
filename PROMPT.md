# Implementation Prompt: H-Exprs Parser with pest

Please implement a parser for the H-Exprs language using the `pest` PEG parser generator in Rust. This is a compact notation for open hypergraphs with S-expression-like syntax.

## Language Specification

### Grammar
```
name: [a-zA-Z][a-zA-Z0-9_-]+
variable: name | _
expr: 
  | '(' expr+ ')'                    // composition of expressions
  | '{' expr+ '}'                    // tensoring of expressions  
  | '[' variable* '.' variable* ']'  // frobenius relation
  | name                             // primitive operation
```

### Key Rules
- Variables (in frobenius relations) are scoped to the entire expression
- The dot `.` in `[vars . vars]` separates input variables from output variables
- `[vars]` without dot is shorthand for identity: `[x y] ≡ [x y . x y]`
- `_` represents anonymous/unbound variables
- Names are primitive operations when not in frobenius relations

## Required Implementation

### 1. Grammar File (grammar.pest)
Create a pest grammar file defining:
```pest
// Define the complete grammar using pest syntax
// Include proper whitespace handling with WHITESPACE rule
// Handle comments if desired with COMMENT rule
// Ensure left-recursion is avoided (pest requirement)
```

Key pest rules to implement:
- `name` - Identifier pattern
- `variable` - Named variable or anonymous `_`
- `frobenius_full` - Full form `[vars . vars]`
- `frobenius_identity` - Identity shorthand `[vars]`
- `frobenius` - Either form
- `composition` - `(expr+)`
- `tensor` - `{expr+}`
- `operation` - Bare operation name
- `expr` - Main expression rule
- `program` - Top-level rule

### 2. AST Types (ast.rs)
Define AST types to represent:
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Composition(Vec<Expr>),
    Tensor(Vec<Expr>),
    Frobenius { inputs: Vec<Variable>, outputs: Vec<Variable> },
    Operation(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Variable {
    Named(String),
    Anonymous,
}
```

### 3. Parser Implementation (parser.rs)
Implement parser using pest:
```rust
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct HExprParser;

// Implement conversion from pest::Pairs to AST
impl From<pest::iterators::Pair<'_, Rule>> for Expr { ... }
```

Key functions:
- `parse_expr` - Convert pest pairs to AST
- `parse_variable` - Handle named/anonymous variables
- `parse_frobenius` - Handle both full and identity forms
- `build_ast` - Main AST construction function

### 4. Test Cases
Include comprehensive tests for these examples:

**Basic frobenius relations:**
```rust
"[x x . x]"     // 2->1 join
"[x . x x]"     // 1->2 split  
"[x y]"         // 2->2 identity (shorthand)
"[_]"           // anonymous identity
"([x.][.x])"    // identity via composition
```

**Complex expressions:**
```rust
"({[_] -} +)"                           // subtraction (pointfree)
"([x y.] ([.y] - [z.]) [.x z] +)"      // subtraction (pointed)
"[x y . y x]"                           // explicit swap relation
```

**Edge cases:**
```rust
"[.]"           // empty inputs/outputs
"[x .]"         // discard x
"[. x]"         // create x
"[_ _ . _]"      // dispell 2, summon 1
```

### 5. Error Handling
- Leverage pest's built-in error reporting
- Provide meaningful error messages for common mistakes
- Handle whitespace and comments appropriately
- Validate semantic constraints during AST construction

### 6. Project Structure
```
src/
├── main.rs          // CLI interface for testing
├── lib.rs           // Library exports
├── ast.rs           // AST type definitions
├── parser.rs        // pest parser implementation
├── tests.rs         // Comprehensive test suite
└── grammar.pest     // pest grammar definition
```

### 7. Cargo.toml
```toml
[package]
name = "h-exprs"
version = "0.1.0"
edition = "2021"

[dependencies]
pest = "2.7"
pest_derive = "2.1"
clap = "4.0"  # for CLI interface
```

## Implementation Notes

### pest-Specific Guidelines
- Use `WHITESPACE = _{ " " | "\t" | "\n" | "\r" }` for automatic whitespace handling
- Avoid left-recursion - pest is PEG-based
- Use `@` for atomic rules and `_` for silent rules where appropriate
- Consider using `$` for string capture when needed

### Grammar Design Tips
- Make sure expression lists use `+` (one or more) not `*` (zero or more)
- Handle the dot separator carefully in frobenius relations
- Use pest's built-in precedence handling if needed
- Consider making whitespace handling explicit in key places

### AST Construction
- Convert pest's tree structure to your clean AST types
- Handle the identity shorthand transformation: `[x y]` → `[x y . x y]`
- Validate variable names during AST construction
- Preserve source location information if useful for error reporting

## Deliverables

1. Complete Rust project with Cargo.toml and grammar.pest
2. Well-documented parser implementation using pest
3. Clean AST types representing the language structure
4. Comprehensive test suite covering all grammar rules
5. CLI tool to parse, validate, and pretty-print H-expressions
6. Clear error messages leveraging pest's error reporting

The goal is a clean, maintainable parser that takes advantage of pest's grammar-first approach while correctly handling the compositional structure and variable scoping semantics of H-Exprs.
