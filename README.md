# HExprs: A compact notation for (open) hypergraphs

H-expressions are a notation for *open hypergraphs* inspired by S-expressions.

# HExprs

There are three kinds of expression:

Sequential composition of operations with `(...)` brackets, e.g. `(add neg copy)`:

![Sequential Composition](propaganda/sequential_composition.svg)

Parallel composition of operations with `{...}` brackets, e.g. `{add copy}`:

![Parallel Composition](propaganda/parallel_composition.svg)

Binding of names to wires with `[...]` brackets. We can write identities (wires
with no operations):

`[x y . x y]`: ![Identity with Binding](propaganda/identity_binding.svg)

You can also write this as shorthand:

`[x y]`: ![Identity Shorthand](propaganda/identity_shorthand.svg)

Joining and splitting wires:

`[x x . x]`: ![Joining Wires](propaganda/joining_wires.svg)
`[x . x x]`: ![Splitting Wires](propaganda/splitting_wires.svg)

Dispelling and summoning wires:

`[x.]`: ![Dispelling Wires](propaganda/dispelling_wires.svg)
`[.x]`: ![Summoning Wires](propaganda/summoning_wires.svg)

Lastly, `[x y z]` is shorthand for `[x y z . x y z]`.

Note that name bindings are *global*- they persist *outside* the `[..]` brackets.
This allows you to construct hypergraphs in "imperative style" using square brackets.

    ([a b.] {                    // [a b] are like "function arguments"
        ([.a b] add [acc.])     // acc = a + b
        ([.a acc] mul [result.] // result = a * acc
    })

This expression produces the following diagram:

![Imperative Example](propaganda/imperative_example.svg)

Each of these diagrams can be generated using `cargo run -- '<expr>' -qv > image.svg`--
(see also `generate_readme_images.sh`).

# Signatures

How do hexprs know that `add` has two inputs and one output? Via a **signature**:
the user must tell the hexpr library the *arity* and *type* of each operation.
For example, `add : ℝ × ℝ → ℝ`.

TODO: for src/main.rs, you can supply a signature as json using ... TODO

# Category Theory

TODO
