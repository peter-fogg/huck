# Syntax thoughts

- Support functional idioms: everything is an expression
- Familiar -- curly braces and semicolons are OK
- Clean, simple, but not Python-style whitespace

# Declarations
Locally, `let x = 1;`. Non recursive.

At top-level, global and potentially recursive declarations. Still `let global = true;` syntax.

Functions: `let f = fn (a: i64, b: bool): () { ... }`

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
  - One thought is `#{field1: "foo", field2: true}`. `#` isn't used for anything else and clearly indicates that this is a hash table-like structure under the hood.

# Function Calls
- If it ain't broke don't fix it: `let result_of_calling_f = f();`
- Uniform call syntax: if we have `f: fn(a: SomeObjectType, b: i64): i64 {...}`, then we can either call:
  - `f(some_object, 4)`, or
  - `some_object.f(4)`
