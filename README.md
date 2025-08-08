# H-Exprs: A compact notation for (open) hypergraphs

An S-expression-like syntax for Open Hypergraphs (aka cospans of hypergraphs).

# Grammar

    name: [a-zA-Z][a-zA-Z0-9_-]+
    variable: name | _
    (<expr>+): composition of expressions
    {<expr>+}: tensoring of expressions
    [<variable>* . <variable>*]: a 'frobenius relation', with expression-scoped variables

Examples:

    [x x . x]   // the 2 -> 1 frobenius "join", or "unify".
    [x . x x]   // the 1 -> 2 frobenius "split" or "alias".
    ([x.][.x])  // the 1 -> 1 *identity* on x: although x is discarded, the two appearances are unified.
    [ x y ]     // the 2 -> 2 identity map, bound to vars x, y.
    [_]         // anonymous (unbound) var- identity map.

Suppose we have arithmetic operators + : 2 -> 1 and negate - : 1 -> 1.
We can define subtraction:

    ({[_] -} +)

or more pointfully:

    (
        [x y.]      // bind inputs
        [.y] - [z.] // z = -y
        [.x z] +    // result
    )
