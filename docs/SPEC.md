# Hexpr V2 Spec

This is the proposed specification for **version 2** of the hexpr syntax.
This is NOT the current released specification.

A `hexpr` (or H-expression) is a syntax for defining arrows of symmetric monoidal categories.
In short:

- `(f g h)` is sequential composition `f ; g ; h`
- `{f g h}` is tensor product `f × g × h`
- `[x y . y z]` is a *spider*

For example, `({neg _} mul)` defines the arrow below:

![](./example.svg)

# Grammar

The `hexpr` grammar is:

    hexpr := atom                        operation
           | "^" hexpr                   labeled identity wire
           | "!" hexpr                   quote
           | "(" hexpr* ")"              composition, left to right
           | "{" hexpr* "}"              tensor
           | "[" lvars ("." lvars)? "]"  Frobenius spider

    lvars := lvar*
    lvar  := var ("^" hexpr)?            spider node, optionally labeled

    atom  := op-char+                    op-char = alnum | - _ . * + / | > < : = ?
    var   := (alpha | "-") (alnum | "-" | "_")*

Note:

- Reserved tokens are `( ) { } [ ] ! ^ #`
- `#` comments to the end of the line
- The "_" atom is always interpreted as the unlabeled identity wire

The following syntax interprets to the same data:

- `[xs]` ≡ `[xs . xs]`
- `^t` ≡ `[x^t]` labeled identity is the same as the labeled identity *spider*
- `_` ≡ `[x]` (unlabeled identity, the name `x` is immaterial.[^bind]

[^bind]: True for *core* (quote-free) hexprs. Macros receive syntax, and a
name-inspecting macro like `!(bind …)` distinguishes `_` from `[x]`: only
the latter is capturable.

# Interpretation

A *fully annotated* hexpr (see below) defines an *arrow* in a category `C`
which is assumed to be presented by a signature `Σ = (Σ₀, Σ₁)` consisting of:

- generating objects `Σ₀`
- generating operations `Σ₁`

The atoms (both for object labels and operations) are assumed to be elements of
`Σ₀` and `Σ₁`, respectively, but these sets are not explicitly defined by the
hexpr.
In this way, the same hexpr can be interpreted 'as needed' into a different
ambient category `C`.

By "interpretation", we more precisely mean that a hexpr is interpreted into a
[*cospan of hypergraphs*](https://docs.rs/open-hypergraphs/latest/open_hypergraphs/).

## Interpretation with Partial Object Labels

The term below is fully annotated.

    # we want to think of this as having type u64 × u64 -> u64
    ({^u64 ^u64} u64.add ^u64)

But it is also possible to use `_` to write a term with un-annotated wires:

    ({_ _} u64.add _)

In the latter case, how can we interpret it?
The arity and coarity of `u64.add` can be inferred as 2 and 1, respectively,
but the types cannot.

In this case, the arrow is interpreted into a category `U(C)` presented by
`U(Σ) = (U(Σ₀), U(Σ₁))` where

- U(Σ₀) = Σ₀ + {Unknown}
- U(Σ₁) = Σ₁

That is, the signature Σ plus an "unknown" object label--this is the type of
`_`.

Note that a *fully annotated* hexpr is guaranteed to interpret to an open
hypergraph having no `Unknown` labels.

## Expressions with ambiguous arity/coarity

It is possible to write hexprs which are not fully annotated that have
operations with ambiguous arity/coarity.
For example, consider a term

    ({_ _} f g _)

Here, the arity of `f` is `2` and coarity of `g` is `1`.
But the coarity of `f` / arity of `g` is unknown.

There are two resolutions:

1. Interpret the hexpr relative to a given signature
   (which chooses types for f and g)
2. Return an error

Both modes are possible in the library.

# Profile-Annotated and Fully Annotated Terms

Two degrees of annotation for macro-expanded terms matter:

1. Enough to determine *arity* and *coarity*
2. Enough to determine *types*

Both are checked by structural recursion computing boundaries as
lists over `Σ₀ + {Unknown}`; the latter is guaranteed to result in no `Unknown`
labels.

The recursion:

- `_`: `src = tgt = (Unknown)`.
- `^t`: `src = tgt = (t)`.
- A bare atom by itself *fails* (even its arity is nowhere written).
- `[xs . ys]`: always succeeds. A node's label is the one attached to any
  of its occurrences in the bracket (repeated labels must agree
  syntactically); unlabeled nodes get `Unknown`. `src`/`tgt` list node
  labels in occurrence order.
- `{e1 … en}`: succeeds iff every `ei` does; boundaries concatenate.
  (`{}` succeeds, with empty boundary.)
- `(e1 … en)`: call an element that is a bare atom an *operation use*.
  Every other element must succeed, and no operation use may be adjacent
  to another one, first, or last. An operation use takes its profile from
  its neighbors: `src = tgt` of the preceding element, `tgt = src` of the
  following. Adjacent boundaries must have equal width, with labels
  agreeing position-wise where both are known (`Unknown` agrees with
  anything). Then `src = src(e1)`, `tgt = tgt(en)`.
  (`()` alone fails: even its width is undetermined.)

A term is **profile-annotated** if the recursion succeeds: every operation
use has a determined arity and coarity. Profile-annotated terms are
interpretable in `U(C)` with no external signature.

A term is **fully annotated** if it is profile-annotated and no `Unknown`
appears in the boundary of any subterm — equivalently: no `_`, and every
spider node labeled. Fully annotated terms are interpretable in `C`: the
resulting open hypergraph has no `Unknown` labels, and each operation use
determines an element of `Σ₁` together with an explicit type. No external
signature is required — the term is its own presentation.

From the Interpretation section's examples: `({^u64 ^u64} u64.add ^u64)` is
fully annotated, `({_ _} u64.add _)` is profile-annotated only, and
`({_ _} f g _)` is neither.

# Macros, Quoting, and Binding Names

The quote token `!` marks its argument for *preprocessing*: a quoted hexpr
is not interpreted directly, but passed as syntax to a *macro*, whose result
is used in its place. A quote *mentions* its payload rather than using it:
names inside a quote never unify with names outside.

Macros are dispatched on the *head* of the payload:

    !(bind e1 … en)      head `bind`, args e1 … en
    !{bind e1 … en}      same macro; expansion preserves the bracket type
    !todo                an atom is its own head (nullary)

A macro may return either:

1. a hexpr, which is interpreted in the quote's place (in its own scope), or
2. an open hypergraph, spliced in directly.

The library invokes a user-supplied *macro hook* for each quote, so hosts
define their own macros. Two are provided:

- `!(bind e…)` — behaves as `(e…)`, except equal names in *different* spider
  brackets denote the same node. (Recall the default: names are local to
  their bracket, and cross-bracket wiring is positional.)
- `!(unbind e…)` — asserts no sharing: names inside stay bracket-local even
  under an enclosing `bind`. Its expander is the identity — the protection
  comes from quotation itself, since any quote is opaque to an enclosing
  `bind`. `unbind` is the "seal and nothing else" quote.

For example, the twist, wired by name:

    !(bind [x y .] [. y x])

**Macro expansion is outside-in**, as in Lisp: the outermost macro runs
first, and decides for itself whether nested quotes are code or data.

- A macro receives nested quotes *unexpanded*, as opaque leaves; in
  particular, `bind` cannot capture names inside them.
- If a macro returns a hexpr, quotes in the result then expand the same
  way, each in its own sealed scope.
- Consequently a nested `bind` or `unbind` opens a fresh region that
  forgets outer bindings.
- Porting v1: wrap a whole v1 hexpr in one top-level `!(bind …)` to restore
  v1's global binding exactly.

# Case studies

What's enabled by this new spec that wasn't possible or easy previously?

## Multiple sorts in Metacat

- Metacat currently uses `wff` and `|-` as *constructors* - actual logical symbols
- Instead, we can now use them as *sorts*!
- We defined `fol.sorts` as having constants `wff`, `|-`, `setvar`.
- Then `fol.syntax` is defined *over* `fol.sorts`...
- and axioms like `wi` become `wi : {^wff ^wff} -> ^wff` - so there are no
  constructors to unpack, only sorts to match.

Note that if we do "metacat with sorts" as described above, then some
compositions actually fail at *term construction time*.
Suppose we compose `wi : {wff wff} -> wff` with `foo : |- -> |-` - this can't
compose now, since sorts `wff` and `|-` are unequal!

## Variadic Operations (Tuples)

With the changes in V2, can we define (variable arity/coarity) tuples?
Let's say we assume a builtin 'tuple' label.

    (^3 tuple) # look, we declared an arity-3 tuple!
    ({^a ^b ^c} tuple) # more verbosely, with types
