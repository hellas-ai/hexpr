# Hexpr V2 Spec

A `hexpr` (or H-expression) is a syntax for defining arrows of symmetric monoidal categories.
Its grammar is:

(TODO: GRAMMAR)

# Interpretation

A hexpr defines an *arrow* in a category `C` which is assumed to be presented
by a signature `Σ = (Σ₀, Σ₁)` consisting of:

- generating objects `Σ₀`
- generating operations `Σ₁`

However, `Σ` (and thus `C`) are left *intentionally* undefined, so that the
same hexpr can be interpreted 'as needed' into different categories.

# Fully Annotated Terms

A term is *fully annotated* if:

- Defined so that every op is wrapped in wire types (no external sig required)
- No unlabeled wires `_`

Every *fully annotated* term is interpretable as C

# Macros, Quoting, and Binding Names

TODO
