# FPON Language Specification Addendum: The `$` Interpolation Sigil

## Overview

In FPON, the dollar sign (`$`) serves as the universal sigil for **evaluation and interpolation**. It explicitly instructs the interpreter to evaluate an expression in the surrounding scope rather than treating an identifier as a raw string or binding a new pattern variable[cite: 3, 9, 30].

The `$` sigil operates in two contexts:
1. **String Interpolation:** Embedded inside string literals to inject evaluated values as text[cite: 30].
2. **Pattern Interpolation:** Used on the left-hand side of map entries (`->`) to match against an existing variable's value while preserving its concrete type[cite: 9, 24].

---

## 1. String Interpolation

When used inside double-quoted string literals, `$` evaluates an expression and implicitly converts the result to its string representation[cite: 30].

### Syntax

* `$identifier`: Evaluates a single variable.
* `$(expression)`: Evaluates a complex expression wrapped in parentheses.

### Examples

```fpon
let name = "Bob" in
let score = 95 in
let message = "$name scored $score points" in
message
# Result: "Bob scored 95 points"

let width = 10 in
let height = 20 in
let dimensions = "Area: $(width * height) sq units" in
dimensions
# Result: "Area: 200 sq units"

```

### Escaping `$`

To output a literal `$` character inside a string, escape it with a backslash (`\$`):

```fpon
let price = 50 in
"Total cost: \$ $price"
# Result: "Total cost: $ 50"

```

---

## 2. Pattern Interpolation

By default, an unadorned identifier on the left side of a map pattern (`key -> value`) acts as a **variable binding**, creating a dynamic function that matches any input and binds it to `key`.

Adding the `$` sigil before a pattern key (`$key -> value`) changes its behavior: it evaluates `key` in the enclosing scope at map creation time and treats the result as a static, literal pattern match.

### Syntax

* `$identifier -> value`: Evaluates the variable `identifier` and matches against its value.
* `$(expression) -> value`: Evaluates a parenthesized expression at map creation time and matches against the result.

### Type-Preserving Matching vs. String-Coerced Matching

Using `$` directly on a pattern preserves the value's native type (Number, Boolean, List, String). Wrapping `$key` in quotes (`"$key"`) coerces the value to a String before matching.

```fpon
let targetCode = 404 in
let keyName = "status" in

let response = {
  $targetCode -> "Not Found",   # Matches Number 404 (preserves type)
  "$keyName" -> "OK",           # Matches String "status" (coerced)
  _ -> "Unknown"
} in

response 404
# Result: "Not Found"

response "status"
# Result: "OK"

```

---

## 3. Entry Compatibility

In FPON, a map is considered a collection of **entries** only if all of its keys evaluate to static literals.

Because `$key` and `$(expression)` are evaluated **once when the map is constructed**, they produce fixed literal keys. As a result, maps constructed with pattern interpolation remain fully compatible with the `entries` built-in function:

```fpon
let primaryKey = "id" in
let record = {
  $primaryKey -> 101,
  .status -> "active"
} in

record |> entries
# Result: [["id", 101], [".status", "active"]]

```

---

## 4. Symbol Comparison Matrix

| Syntax | Context | Meaning | Resulting Type |
|:---|:---|:---|:---|
| `key -> v` | Map Pattern | Binds argument to a variable `key`. | Functional match (Variadic/Dynamic) |
| `.key` | Expression / Key | Word string literal `"key"`. | String |
| `"$key"` | String | String interpolation. | String (Coerced) |
| `$key` | Map Pattern | Evaluates `key` in scope for exact match. | Original Type (Preserved) |
| `$(expr)` | Map Pattern | Evaluates `expr` in scope for exact match. | Original Type (Preserved) |

---

## 5. Grammar Rules (EBNF Sketch)

```ebnf
string_literal      = '"' { string_char | interpolation } '"' ;
interpolation       = "$" identifier \vert{} "$(" expression ")" ;

map_entry           = pattern "->" expression ;
pattern             = evaluated_pattern | bind_pattern | literal_pattern ;
evaluated_pattern   = "$" identifier \vert{} "$(" expression ")" ;
bind_pattern        = identifier ;
literal_pattern     = number | string_literal | word_string | boolean | null ;
word_string         = "." identifier ;

```
