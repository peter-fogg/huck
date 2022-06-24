# Syntax thoughts

- Support functional idioms: everything is an expression
- Familiar -- curly braces and semicolons are OK
- Clean, simple, but not Python-style whitespace

# Declarations
Locally, `let x = 1;`. Non recursive.

At top-level, global and potentially recursive declarations. Still `let global = true;` syntax.

Functions: `let f = fn (a: i64, b: bool): () { ... }

Function syntax is

  `"fn" "(" arg-list ")" ":" return-type expr`

`expr` can be a block or just a simple expression

TODO: Immutable v mutable vars?

# Types
Probably Rust-like syntax. `identifier: type` seems simple and familiar.

For row-polymorphic object types: `{foo: i64, bar: bool, ...}` where `...` signifies the row.

# Values

- Lists: `[1, 2, 3]`
- Tuples: `(1, true, "foo")`
- Objects: `{foo: true, bar: "baz"}`. How to distinguish this delimiter from blocks? Worth it? Not sure. Object access is nothing new: `let o = {foo: bar}; o.foo`
