# FPON Language Specification Addendum: Pattern Matching, Entries, and Guards

## Overview

Pattern matching is the core evaluation mechanism in FPON[cite: 3, 6, 35]. In FPON, collections are functions, and maps are fundamentally pattern-matching expressions that test inputs sequentially against a series of patterns[cite: 3, 6, 16].

This document details how pattern matching operates across **Maps**, **Entries**, **Pattern Guards (`where`)**, and the **Pattern Test Operator (`is`)**.

---

## 1. Maps and Entries

### Maps as Pattern-Matching Functions
A map literal is an ordered sequence of patterns mapping to result expressions (`pattern -> result`)[cite: 3, 6]. When a map is called with an argument, it evaluates each pattern from top to bottom and returns the result of the first matching branch[cite: 3, 6, 13]:

```fpon
let statusMessage = {
  200 -> "OK",
  404 -> "Not Found",
  500 -> "Server Error",
  _   -> "Unknown Status Code" # Wildcard fallback
} in

statusMessage 404
# Result: "Not Found"

```

### Static Entries vs. Dynamic Patterns

To maintain predictable behavior between data serialization and functional logic, FPON distinguishes between concrete data entries and dynamic patterns:

* **Entry:** A single mapping whose pattern resolves strictly to a concrete literal value (a String, Word String, Number, Boolean, or Null). Maps consisting entirely of concrete entries represent static records.


* **Dynamic Pattern:** A mapping that uses variable bindings (`x -> ...`), wildcards (`_ -> ...`), evaluated patterns (`$key -> ...`), or guards (`where`).



```fpon
# A Map composed purely of Entries (Record)
let user = {
  .id -> 101,
  .role -> "admin",
  .active -> true
} in
user |> entries
# Result: [[".id", 101], [".role", "admin"], [".active", true]]

# A Map containing Dynamic Patterns (Functional Match)
let dynamicMapper = {
  0 -> "zero",
  x -> "number: $x"
} in
dynamicMapper |> entries
# Error: cannot enumerate map: key 'x' is a pattern, not a literal

```

---

## 2. Pattern Guards with `where`

When structural matching alone is insufficient, patterns can be refined using conditional guards introduced by the `where` keyword.

### Syntax

```ebnf
guarded_pattern = pattern "where" boolean_expression ;

```

A guarded branch matches only if the structural pattern matches **and** the `where` clause evaluates to `true`.

### Scope and Variable Binding

Variables bound in the left-hand pattern are immediately available inside the `where` expression as well as the right-hand result expression:

```fpon
let categorize = {
  (n where n < 0)           -> "negative",
  0                         -> "zero",
  (n where n % 2 == 0)      -> "even positive",
  n                         -> "odd positive"
} in

categorize (-5) # Returns "negative"
categorize 4    # Returns "even positive"
categorize 7    # Returns "odd positive"

```

### Complex Destructuring with Guards

Guards work seamlessly with nested list and map destructuring patterns:

```fpon
let processUser = {
  { .age -> a, .role -> "admin" } where a >= 18 -> "Adult Admin",
  { .age -> a } where a < 18                  -> "Minor",
  _                                          -> "Guest or Invalid"
} in

processUser { .age -> 22, .role -> "admin" }
# Result: "Adult Admin"

```

---

## 3. The `is` Pattern Test Operator

The `is` operator allows inline boolean evaluation of an expression against a pattern without requiring a full map abstraction.

### Syntax

```ebnf
pattern_test = expression "is" pattern ;

```

The `is` operator evaluates to `true` if `expression` matches `pattern`, and `false` otherwise.

### Conditional Expressions (`if ... is ... then ... else ...`)

The `is` operator integrates naturally into conditional flow control:

```fpon
let classify = input ->
  if input is [h, ..t] then
    "Non-empty list starting with $h"
  else if input is { .status -> 200 } then
    "Successful response object"
  else
    "Other input"
in

classify [10, 20, 30]
# Result: "Non-empty list starting with 10"

```

### Variable Scoping in `is`

When `is` binds variables inside a pattern, those bindings are scoped strictly to the `then` branch of an `if` expression:

```fpon
let payload = [100, 200] in

if payload is [x, y] where x < y then
  "Increasing pair: $x to $y"
else
  "Unmatched"
# Result: "Increasing pair: 100 to 200"

```

---

## 4. Pattern Anatomy & Precedence Reference

FPON evaluates patterns according to the following category hierarchy:

| Pattern Type | Syntax Example | Description |
|:---|:---|:---|
| **Literal** | `42`, `"ok"`, `.id`, `true`, `null` | Exact value equality match. |
| **Wildcard** | `_` | Matches any input without binding variables. |
| **Variable Binding** | `x` | Matches any input and binds it to `x`. |
| **Evaluated Pattern** | `$key`, `$(x + 1)` | Matches the evaluated value in enclosing scope. |
| **Destructuring List** | `[h, ..t]`, `[a, b]` | Structural array matching and rest-binding. |
| **Guarded Pattern** | `pat where expr` | Matches `pat` only if `expr` evaluates to `true`. |

---

## 5. Formal EBNF Grammar Extensions

```ebnf
map_literal        = "{" [ map_branch { "," map_branch } [ "," ] ] "}" ;
map_branch         = pattern_expr "->" expression ;

pattern_expr       = pattern [ "where" expression ] ;

pattern            = literal_pattern
                   | wildcard_pattern
                   | variable_pattern
                   | evaluated_pattern
                   | list_pattern ;

literal_pattern    = number | string_literal | word_string | boolean | null ;
wildcard_pattern   = "_" ;
variable_pattern   = identifier ;
evaluated_pattern  = "$" identifier \vert{} "$(" expression ")" ;
list_pattern       = "[" [ pat_item { "," pat_item } [ "," ] ] "]" ;
pat_item           = pattern | rest_pattern ;
rest_pattern       = ".." ( identifier | "_" ) ;

is_expression      = expression "is" pattern_expr ;

```
