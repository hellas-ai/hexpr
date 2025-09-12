# HExprs: A compact notation for (open) hypergraphs

H-expressions are a notation for *open hypergraphs* inspired by S-expressions.

# HExprs

There are three kinds of expression:

Sequential composition of operations with `(...)` brackets, e.g. `(add neg copy)`:

![Sequential Composition](propaganda/sequential_composition.svg)

Parallel composition of operations with `{...}` brackets, e.g. `{add copy}`:

![Parallel Composition](propaganda/parallel_composition.svg)

Binding of names to wires with `[...]` brackets. We can write identities (wires
with no operations) as `[x y . x y]`:

![Identity with Binding](propaganda/identity_binding.svg)

You can also write this as shorthand `[x y]`:

![Identity Shorthand](propaganda/identity_shorthand.svg)

Joining `[x x . x]` and splitting `[x . x x]` wires:

![Joining Wires](propaganda/joining_wires.svg)
![Splitting Wires](propaganda/splitting_wires.svg)

Dispelling `[x.]` and summoning `[.x]` wires:

![Dispelling Wires](propaganda/dispelling_wires.svg)
![Summoning Wires](propaganda/summoning_wires.svg)

Note that name bindings are *global*- names are still bound to a given wire
*outside* the `[..]` brackets expression.
This allows you to construct hypergraphs in "imperative style" using square brackets.

    ([a b.] {                    // [a b] are like "function arguments"
        ([.a b] add [acc.])      // acc = a + b
        ([.a acc] mul [result.]) // result = a * acc
    } [.result])                 // [.result] says there is one output wire - result.

This expression produces the following diagram:

![Imperative Example](propaganda/imperative_example.svg)

Each of these diagrams can be generated using `cargo run -- '<expr>' -qv > image.svg`--
(see also `generate_readme_images.sh`).

# Signatures

How do hexprs know that `add` has two inputs and one output? Via a **signature**:
the user must tell the hexpr library the *arity* and *type* of each operation.
For example, `add : ℝ × ℝ → ℝ`.

You can supply operation signature as JSON using the `--signature` or `-s` flag:

```bash
cargo run -- '(add mul neg)' -qv --signature my_signature.json
```

The JSON format specifies inputs and outputs as arrays of type names:

```json
{
  "add": {
    "inputs": ["ℝ", "ℝ"],
    "outputs": ["ℝ"]
  },
  "mul": {
    "inputs": ["ℝ", "ℝ"],
    "outputs": ["ℝ"]
  }
}
```

If no signature file is explicitly provided via `--signature`, an empty signature will be used. With an empty signature, no operations are available - only Frobenius structures (variable binding) can be used. Operations must be explicitly defined in a signature file.

# Category Theory

A HExpr is syntax for defining an "open hypergraph".
Jargonically: open hypergraphs form a category isomorphic to the free symmetric monoidal category on a given signature.

Pragmatically, you can think of open hypergraphs as representing the kinds of
diagram we see above in a precise mathematical sense.
These diagrams have an *algebraic* description in terms of compositions `(f g)`
and tensor products `{f g}`: we'll explore that in detail here.

A *signature* `(Σ₀, Σ₁)` is pair of sets: objects `Σ₀` label wires, and `Σ₁` label boxes (operations).
One intuition is "types" and "primitive functions", but diagrams need not always represent functions.

Each operation `f ∈ Σ₁` has a *type*: a list of source objects `X₀...Xm` and a list of target objects `Y₀..Yn`.
We write this as `f : X₀...Xm → Y₀...Yn`.

**Mapping hexprs to categories**

Now let `f : A → B` and `g : B → C` be operations. Then:

- The hexpr `(f g)` represents the *composition* of maps `f ; g : A → C`
- The hexpr `{f g}` represents the *composition* of maps `f ; g : A B → B C`
- The hexpr `[x₀ ... x_{n-1}]` represents the identity map on `n` wires

In addition, we have *special frobenius structure*.
This arises naturally as a result of the hypergraph representation, and amounts to adding four extra operations:

- comonoid: `[x . x x]`
- counit: `[x.]`
- monoid: `[x . x x]`
- unit: `[.x]`
