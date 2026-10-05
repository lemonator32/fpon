Pipe operator `|>` example:

```fpon
let append_space = num -> str -> str + (" " * num) in
"prefix:" |> append_space 3
# Result: "prefix:   "
```

Fallback operator `<|>` example:

```fpon
let defaults = { "theme" -> "light", "lang" -> "en" } in
let overrides = { "theme" -> "dark" } in
let config = overrides <|> defaults in
{
  "theme" -> config "theme",   # "dark"  (left wins)
  "lang"  -> config "lang",    # "en"    (falls through to defaults)
  "other" -> config "other",   # no match in either, so a miss
}
```

Since maps are also functions, `|>` can be applied to a map literal to effectively get a switch/match statement.

```fpon
let fruit = "apple" in
fruit |> {
  "apple" | "banana" | "orange" -> "This is a fruit.",
  "broccoli" | "carrot" -> "This is a vegetable.",
  _ -> "Unknown food item.",
}
# Result: "This is a fruit."
```

Safeguard operator `?` inserts the previous value into a map with a catch-all case that returns `null`. For example `f? x` becomes `{ f, _ -> null } x`.

```fpon
let plants = {
  "apple" | "banana" | "orange" -> "This is a fruit.",
  "broccoli" | "carrot" -> "This is a vegetable.",
} in
plants? "lemon"
# Result: null
```
