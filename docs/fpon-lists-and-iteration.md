# Lists and Iteration in FPON

This document extends the FPON language design with a primitive **list** type and a set of built-in functions for iteration. It follows the language's core idea: **collections are functions**. A map is a function from keys to values, and a list is a function from indices to values.

FPON has no `for` or `while` loops. All iteration is done with higher-order built-ins (`map`, `filter`, `fold`, and so on) combined with the pipe operator `|>`.

---

## 1. List Literals

Lists are written with square brackets and comma-separated elements.

```fpon
[1, 2, 3]
["a", "b", "c"]
[]                      # the empty list
[1, "two", true, null]  # lists may be heterogeneous
[[1, 2], [3, 4]]        # lists may be nested
[{ .id -> 1 }, { .id -> 2 }]  # lists may contain maps
```

Trailing commas are allowed, matching map literals:

```fpon
[
  "alpha",
  "beta",
  "gamma",
]
```

As with the rest of FPON, whitespace and line breaks inside a list literal are insignificant. Lists are immutable, and every operation returns a new list.

### Type summary

`List` joins the primitive types from section 4.1 of the language design. Equality (`==`, `!=`) on lists is **structural**: two lists are equal if they have the same length and equal elements in the same order.

```fpon
[1, 2] == [1, 2]   # true
[1, 2] == [2, 1]   # false
```

---

## 2. Accessing Elements

A list is a function from a zero-based integer index to a value. Access uses ordinary function application, exactly like map lookup.

```fpon
let xs = [10, 20, 30] in
xs 0    # 10
xs 2    # 30
```

Because application is left-associative, lists and maps can be navigated in a single chain:

```fpon
let data = {
  .users -> [
    { .name -> "Ada",  .roles -> ["admin", "editor"] },
    { .name -> "Alan", .roles -> ["viewer"] },
  ],
} in
data .users 0 .name        # "Ada"
data .users 0 .roles 1     # "editor"
```

### Out-of-range and invalid indices

An index that does not exist follows the same rules as a missing map key. It raises a "Pattern not found" error that names the attempted index and the valid range.

```fpon
let xs = [10, 20, 30] in
xs 5
# Error: Pattern not found: index 5 (valid indices: 0..2)

[] 0
# Error: Pattern not found: index 0 (list is empty)
```

An index that is not an integer is a type error, since implicit coercion is not allowed:

```fpon
[10, 20, 30] "a"
# Type error: list index must be an integer, got String

[10, 20, 30] 1.5
# Type error: list index must be an integer, got Number (float)
```

Negative indices are not supported. `xs -1` raises "Pattern not found". Use `length` for end-relative access:

```fpon
let last = xs -> xs (length xs - 1) in
last [10, 20, 30]
# 30
```

### Safe access with `?`

The safeguard operator works unchanged. It returns `null` instead of raising an error when the index is out of range.

```fpon
let xs = [10, 20, 30] in
xs? 1    # 20
xs? 5    # null
```

To supply a default value, match on the result:

```fpon
let xs = [10, 20, 30] in
xs? 5 |> {
  null -> 0,
  v    -> v,
}
# 0
```

---

## 3. Spread and Concatenation

### Spread: `..`

`..` inside a list literal inserts the elements of another list at that position.

```fpon
let xs = [2, 3] in
[1, ..xs, 4]
# [1, 2, 3, 4]

[..xs, ..xs]
# [2, 3, 2, 3]
```

Spreading anything other than a list is a type error:

```fpon
[1, ..5]
# Type error: cannot spread Number into a list
```

> **Lexing note:** `..` is a single token and is distinct from the word-string prefix `.`. Because a word string cannot be empty, a `.` immediately followed by another `.` can never start a valid word string, so there is no ambiguity.

### Concatenation: `+`

`+` on two lists concatenates them. Both operands must be lists. Mixing a list with any other type is a type error, consistent with the rule against implicit coercion.

```fpon
[1, 2] + [3]
# [1, 2, 3]

[1, 2] + 3
# Type error: cannot add List and Number
```

To append a single element, wrap it in a list: `xs + [x]`.

---

## 4. List Patterns

Map keys in FPON are patterns, and lists extend the pattern language. List patterns work anywhere a pattern is allowed. This includes map entries used with `|>`, which gives FPON its destructuring and structural recursion.

| Pattern    | Matches                                    | Binds                      |
| ---------- | ------------------------------------------ | -------------------------- |
| `[]`       | the empty list only                        | nothing                    |
| `[x]`      | a list of exactly one element              | `x`                        |
| `[a, b]`   | a list of exactly two elements             | `a`, `b`                   |
| `[h, ..t]` | a list of one or more elements             | `h` (head), `t` (the rest) |
| `[a, b, ..rest]` | a list of two or more elements       | `a`, `b`, `rest`           |
| `[_, _]`   | any list of exactly two elements           | nothing                    |
| `[1, x]`   | a two-element list whose first element is `1` | `x`                     |
| `[..all]`  | any list                                   | `all`                      |

Rules:

- A rest pattern (`..name`) may appear **at most once**, and only as the **last** element of the pattern.
- `..t` binds `t` to a (possibly empty) list. `_` may be used when the rest is not needed.
- Elements may themselves be patterns: literals, variables, `_`, nested lists, or `|` alternations.
- As with maps, the **first matching pattern wins**.

### Examples

```fpon
let describe = xs -> xs |> {
  []         -> "empty",
  [_]        -> "one element",
  [_, _]     -> "two elements",
  [_, _, .._] -> "three or more",
} in
describe [7, 8, 9, 10]
# "three or more"
```

```fpon
let head = xs -> xs |> {
  []       -> null,
  [h, .._] -> h,
} in
head [4, 5, 6]
# 4
```

Pattern matching a nested structure:

```fpon
let firstName = payload -> payload |> {
  [{ .name -> n }, .._] -> n,
  _                      -> "unknown",
} in
firstName [{ .name -> "Ada" }, { .name -> "Alan" }]
# "Ada"
```

If no pattern matches and there is no `_` fallback, the usual "Pattern not found" error is raised.

---

## 5. Built-in Functions

All list built-ins are **curried** and take the **list as their last argument**. This is deliberate. It makes them compose with `|>`, which passes its left-hand side as the final argument:

```fpon
[1, 2, 3] |> map (x -> x + 1)
# equivalent to:
map (x -> x + 1) [1, 2, 3]
```

### Reference

| Function     | Signature                          | Description                                                     |
| ------------ | ---------------------------------- | --------------------------------------------------------------- |
| `map`        | `(a -> b) -> List a -> List b`     | Apply a function to every element.                              |
| `filter`     | `(a -> Bool) -> List a -> List a`  | Keep elements for which the predicate returns `true`.           |
| `fold`       | `(acc -> a -> acc) -> acc -> List a -> acc` | Left fold. Combine elements into a single value.      |
| `length`     | `List a -> Number`                 | Number of elements.                                             |
| `range`      | `Number -> Number -> List Number`  | Integers from `start` up to, but not including, `end`.          |
| `concat`     | `List a -> List a -> List a`       | Join two lists (`concat xs ys` is `xs + ys`).                   |
| `zip`        | `List a -> List b -> List [a, b]`  | Pair up elements by position.                                   |
| `sort`       | `(a -> a -> Number) -> List a -> List a` | Stable sort using a comparator.                           |
| `reverse`    | `List a -> List a`                 | Reverse the order of elements.                                  |

The signatures above are documentation notation only. FPON is dynamically typed, and type errors are raised during evaluation.

### `map`

```fpon
[1, 2, 3] |> map (x -> x * x)
# [1, 4, 9]
```

### `filter`

The predicate must return a Boolean. Any other return type is a type error.

```fpon
[1, 2, 3, 4, 5, 6] |> filter (x -> x % 2 == 0)
# [2, 4, 6]
```

### `fold`

`fold f init xs` processes the list from left to right. For each element, it calls `f acc x` and uses the result as the next accumulator. The final accumulator is the result. Folding an empty list returns `init`.

```fpon
[1, 2, 3, 4] |> fold (acc -> x -> acc + x) 0
# 10

[] |> fold (acc -> x -> acc + x) 0
# 0
```

`fold` is the primitive from which most other list operations can be built. See section 8.

### `length`

```fpon
length [10, 20, 30]   # 3
length []             # 0
```

### `range`

`range start end` is **half-open**. It includes `start` and excludes `end`. If `end <= start`, the result is the empty list. Both arguments must be integers.

```fpon
range 0 5    # [0, 1, 2, 3, 4]
range 3 6    # [3, 4, 5]
range 5 5    # []
range 5 0    # []
```

Because `range` is the only way to produce a sequence of numbers, it is also how you express "repeat n times":

```fpon
range 0 3 |> map (i -> "item-{i}")
# ["item-0", "item-1", "item-2"]
```

### `concat`

```fpon
concat [1, 2] [3, 4]
# [1, 2, 3, 4]

[3, 4] |> concat [1, 2]
# [1, 2, 3, 4]   (the piped list becomes the second argument)
```

### `zip`

`zip` pairs elements by position into two-element lists. If the lists differ in length, the result is truncated to the shorter one.

```fpon
zip [1, 2, 3] ["a", "b"]
# [[1, "a"], [2, "b"]]
```

Combine with a list pattern to destructure the pairs:

```fpon
zip [1, 2, 3] [10, 20, 30]
  |> map ([a, b] -> a + b)
# [11, 22, 33]
```

### `sort`

The comparator takes two elements and returns a **number**: negative if the first should come first, zero if they are equivalent, and positive if the second should come first. The sort is **stable**.

```fpon
[3, 1, 2] |> sort (a -> b -> a - b)
# [1, 2, 3]

[3, 1, 2] |> sort (a -> b -> b - a)
# [3, 2, 1]
```

Sorting records by a field:

```fpon
let people = [
  { .name -> "Ada",  .age -> 36 },
  { .name -> "Alan", .age -> 41 },
  { .name -> "Grace", .age -> 30 },
] in
people
  |> sort (a -> b -> a.age - b.age)
  |> map (p -> p.name)
# ["Grace", "Ada", "Alan"]
```

### `reverse`

```fpon
reverse [1, 2, 3]
# [3, 2, 1]
```

---

## 6. Pipelines

The pipe operator turns nested calls into a readable left-to-right flow:

```fpon
[1, 2, 3, 4]
  |> filter (x -> x % 2 == 0)
  |> map (x -> x * 10)
  |> fold (acc -> x -> acc + x) 0
# Step 1: filter -> [2, 4]
# Step 2: map    -> [20, 40]
# Step 3: fold   -> 60
# Result: 60
```

A more realistic transformation, reshaping a list of records:

```fpon
let servers = [
  { .host -> "a.example.com", .port -> 8080, .enabled -> true  },
  { .host -> "b.example.com", .port -> 8081, .enabled -> false },
  { .host -> "c.example.com", .port -> 8082, .enabled -> true  },
] in
servers
  |> filter (s -> s.enabled)
  |> map (s -> "{s.host}:{s.port}")
# ["a.example.com:8080", "c.example.com:8082"]
```

> String interpolation converts values implicitly, so `"{s.port}"` works even though `s.host + s.port` would be a type error.

---

## 7. Enumerating Maps

A map is a function, so in general its keys cannot be listed. A map whose keys include variables, `_`, or `|` alternations matches *infinitely many* inputs. However, a map whose keys are **all literals** is effectively a record, and two built-ins convert between records and lists of entries.

| Function      | Signature                       | Description                                   |
| ------------- | ------------------------------- | --------------------------------------------- |
| `entries`     | `Map -> List [key, value]`      | List the `[key, value]` pairs of a record.    |
| `fromEntries` | `List [key, value] -> Map`      | Build a record from a list of pairs.          |

```fpon
{ .a -> 1, .b -> 2 } |> entries
# [[.a, 1], [.b, 2]]

[[.a, 1], [.b, 2]] |> fromEntries
# { .a -> 1, .b -> 2 }
```

Entries are returned in **source order**.

### Restrictions

`entries` raises an error if any key in the map is a pattern rather than a literal:

```fpon
{ .a -> 1, _ -> 0 } |> entries
# Error: cannot enumerate map: key '_' is a pattern, not a literal
```

If a literal key is duplicated, only the first entry is reachable (see section 6.3 of the language design), so only that entry is returned.

### Transforming a record

Combining `entries`, `map`, and `fromEntries` lets you transform every value of a record:

```fpon
{ .a -> 1, .b -> 2, .c -> 3 }
  |> entries
  |> map ([k, v] -> [k, v * 100])
  |> fromEntries
# { .a -> 100, .b -> 200, .c -> 300 }
```

---

## 8. Common Patterns

Everything below can be defined in FPON itself from the core built-ins.

### Sum, product, and extremes

```fpon
let sum     = fold (acc -> x -> acc + x) 0 in
let product = fold (acc -> x -> acc * x) 1 in

sum [1, 2, 3, 4]       # 10
product [1, 2, 3, 4]   # 24
```

Because functions are curried, `fold f init` is itself a function waiting for a list.

### `any` and `all`

```fpon
let any = p -> fold (acc -> x -> acc || p x) false in
let all = p -> fold (acc -> x -> acc && p x) true in

any (x -> x > 3) [1, 2, 5]   # true
all (x -> x > 3) [1, 2, 5]   # false
```

### Flatten one level

```fpon
let flatten = fold (acc -> xs -> acc + xs) [] in
flatten [[1, 2], [3], [4, 5]]
# [1, 2, 3, 4, 5]
```

### Find the first match

```fpon
let find = p -> xs -> (
  xs |> filter p |> {
    []       -> null,
    [h, .._] -> h,
  }
) in
find (x -> x > 2) [1, 2, 3, 4]
# 3
```

### Map then flatten (`flatMap`)

```fpon
let flatMap = f -> xs -> xs |> map f |> fold (acc -> ys -> acc + ys) [] in
flatMap (x -> [x, x]) [1, 2, 3]
# [1, 1, 2, 2, 3, 3]
```

### Indexed iteration

Pair each element with its index by zipping with `range`:

```fpon
let indexed = xs -> zip (range 0 (length xs)) xs in
indexed ["a", "b", "c"]
# [[0, "a"], [1, "b"], [2, "c"]]
```

### Take and drop

```fpon
let take = n -> xs -> range 0 n |> filter (i -> i < length xs) |> map (i -> xs i) in
take 2 [10, 20, 30]
# [10, 20]
```

---

## 9. Serialization

FPON is an object notation, so lists and records need a defined mapping to JSON.

| FPON value                          | JSON result                          |
| ----------------------------------- | ------------------------------------ |
| Number, String, Boolean, `null`     | the equivalent JSON primitive        |
| Word string (`.name`)               | a JSON string (`"name"`)             |
| List                                | an array, with elements serialized in order |
| Map with only literal string keys   | an object                            |
| Map with any pattern key            | **error**                            |
| Function                            | **error**                            |

```fpon
{ .tags -> ["a", "b"], .count -> 2 }
# {"tags": ["a", "b"], "count": 2}
```

Serialization happens on the fully evaluated value. A list or map that contains a function, or a map with a pattern key, anywhere inside it is an error:

```fpon
[1, (x -> x)]
# Error: cannot serialize function at index 1
```

---

## 10. Termination and Recursion

FPON's design goal is to stay safe and predictable for configuration and data transformation. List iteration supports that goal because every built-in above **terminates on finite lists**. `map`, `filter`, `fold`, `zip`, `sort`, and `reverse` visit each element a bounded number of times, and `range` produces a finite list.

Plain `let` is **not** recursive, so a function cannot call itself by name:

```fpon
let loop = n -> loop (n + 1) in loop 0
# Error: Variable 'loop' is not defined
```

This means iteration happens only through the built-ins in section 5. Unbounded loops are therefore not expressible through ordinary code.

### Caveat: untyped lambdas

Without a type system, a self-application such as `(x -> x x) (x -> x x)` can still diverge, even without `let rec`. If FPON must guarantee termination (for example when evaluating untrusted rules), the interpreter should enforce an **evaluation step limit** and raise a clear error when it is exceeded:

```
Error: evaluation exceeded step limit (1,000,000 steps)
```

---

## 11. Error Reference

| Situation                                 | Error                                                            |
| ----------------------------------------- | ---------------------------------------------------------------- |
| Index out of range                        | `Pattern not found: index 5 (valid indices: 0..2)`               |
| Non-integer index                         | `Type error: list index must be an integer, got String`          |
| `list + non-list`                         | `Type error: cannot add List and Number`                         |
| Spreading a non-list                      | `Type error: cannot spread Number into a list`                   |
| `filter` predicate returns non-Boolean    | `Type error: filter predicate must return Boolean, got Number`   |
| `map`, `filter`, `fold` given a non-list  | `Type error: expected List, got ...`                             |
| `entries` on a map with pattern keys      | `cannot enumerate map: key '_' is a pattern, not a literal`      |
| Serializing a function                    | `cannot serialize function at ...`                               |
| Evaluation step limit exceeded            | `evaluation exceeded step limit`                                 |

---

## 12. Grammar Sketch

```ebnf
list_literal  = "[" [ list_item { "," list_item } [ "," ] ] "]" ;
list_item     = expression | spread ;
spread        = ".." expression ;

list_pattern  = "[" [ pat_item { "," pat_item } [ "," ] ] "]" ;
pat_item      = pattern | rest_pattern ;
rest_pattern  = ".." ( identifier | "_" ) ;   (* last item only, at most once *)
```

Lexing additions:

- `..` is a single token (maximal munch), distinct from the word-string prefix `.`.
- `[`, `]` and `,` become delimiter tokens.

---

## 13. Open Questions

1. **Recursion:** Should FPON add `let rec`? Today `fold` is the only looping mechanism, which keeps evaluation total on finite lists. `let rec` would allow arbitrary recursion at the cost of that guarantee unless the step limit is always enforced.
2. **Negative indices:** Should `xs -1` mean "last element"? The current design says no, to avoid confusion with unary minus and to keep the "pattern not found" rule simple.
3. **Comparator convention for `sort`:** A numeric comparator (negative, zero, positive) is specified here. A boolean "less than" function would be simpler to write but cannot express equality for stable ordering.
4. **`zip` length mismatch:** Truncation is specified here. An error on mismatched lengths is a reasonable stricter alternative.
5. **Map literals with non-string keys in serialization:** Currently an error. They could instead be stringified.
6. **Additional built-ins:** `take`, `drop`, `find`, `any`, `all`, `flatten`, `sum` and `indexed` are all definable in FPON (section 8). Whether any of them should ship in a standard library is undecided.
