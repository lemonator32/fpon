# Strings in FPON

## Standard Strings

String literals are enclosed in double quotes. The backslash (`\`) is used for escaping.

```fpon
"Hello\nWorld"
```

Standard escape sequences are supported, including `\n`, `\t`, `\\`, `\"`, and `\'`.

String literals must be written on a single line. Literal line breaks inside a string are a syntax error; use the escape sequence `\n` instead.

```fpon
"Hello
world"
# Error: literal line break inside string
```

## Interpolation

Curly braces inside a string literal perform **string interpolation**. The expression between the braces is evaluated and implicitly converted to a string; no explicit conversion is required.

```fpon
let name = "Bob" in
let score = 95 in
"{name} scored {score} points"
# Result: "Bob scored 95 points"
```

To include a literal curly brace, escape it with a backslash (`\{` or `\}`). This disables interpolation for that brace.

```fpon
"Use \{curly braces\} for interpolation"
# Result: "Use {curly braces} for interpolation"
```

## Word Strings

In FPON, maps are applied like functions rather than with the familiar dot notation (`.`) found in many other languages. Map keys can be of any type, but strings are by far the most common.

Because the period is not used for field access, FPON repurposes it to introduce **word string** literals. A word string:

- Consists only of word characters (`A-Za-z0-9_`)
- Stops at the first non-word character
- Cannot be empty (an empty word string is a syntax error)
- Is a raw string: escape sequences and interpolation are not supported

Word strings are equivalent to ordinary double-quoted strings that contain the same characters:

```fpon
.word == "word"
```

This design enables a convenient dot-style syntax for map access. The following two expressions are equivalent:

```fpon
let userProfile = {
  "account" -> {
    "preferences" -> {
      "theme" -> "dark"
    }
  }
} in
userProfile "account" "preferences" "theme"
# Result: "dark"
```

```fpon
let userProfile = {
  .account -> {
    .preferences -> {
      .theme -> .dark
    }
  }
} in
userProfile .account .preferences .theme
# Result: .dark
```

Because whitespace is insignificant, the same access can also be written without spaces:

```fpon
userProfile.account.preferences.theme
```

Both the double-quoted and word-string forms work interchangeably for comparison and pattern matching:

```fpon
userProfile.account.preferences.theme == "dark"   # true
```

```fpon
userProfile.account.preferences.theme == .dark    # true
```

## Raw Multi-line Strings

The single quote `'` prefix introduces a **raw multi-line string** (sometimes called a doc string).

### Core Rules

- **No escape sequences:**  
  Everything is taken literally. Sequences such as `\n` or `\t` appear in the resulting string as a backslash followed by the letter `n` or `t`.
- **Implicit newlines:**  
  The compiler inserts a newline (`\n`) after every line except the final one.
- **Stripped leading whitespace:**  
  Indentation that appears before the `'` is ignored, so you can indent the block to match the surrounding code without adding unwanted spaces to the string.
- **Line termination:**  
  A multi-line string continues until a line that does not begin with `'`. An empty `'` line (nothing after the prefix) produces a blank line in the output.

### Basic Example

```fpon
# The compiler treats the following as a single string containing embedded newlines
let text =
  'Line 1: Hello World!
  'Line 2: Escape sequences like \n are treated literally.
  'Line 3: This syntax is reminiscent of line comments.
in text
```

### Typical Use: Embedding Documents in Configuration

The primary purpose of raw multi-line strings is to embed documents (HTML, shell scripts, SQL, etc.) inside configuration values.

```fpon
{
  "html_template" ->
    '<!DOCTYPE html>
    '<html>
    '  <body>
    '    <h1>Hello From FPON</h1>
    '  </body>
    '</html>
  ,  # Note: the comma must appear on the line after the final ' line
  "file_name" -> "index.html",
  "output_directory" -> "./dist/public",
  "file_size_bytes" -> 104,
  "created_at" -> "2026-10-02T18:35:00Z",
  "updated_at" -> "2026-10-02T18:35:00Z"
}
```

```fpon
{
  "name" -> "API Health Check Pipeline",
  "version" -> "1.2.0",

  "metadata" -> {
    "description" -> "Automated script to verify external service availability",
    "author" -> "DevOps Team"
  },

  "config" -> {
    "timeout_minutes" -> 5,
    "retry_attempts" -> 3,
    "allow_failure" -> false,
    "environment" -> {
      "STAGE" -> "production",
      "LOG_LEVEL" -> "DEBUG"
    }
  },

  # Embedded shell script
  "run" ->
    'CURL="/bin/curl"
    'JQ="/bin/jq"
    '
    'echo "Checking GitHub API Status..."
    '
    '# Fetch data using the tools defined above
    '$CURL -s "https://api.github.com"
    '
    'echo ""
    'echo "Script executed successfully!"
}
```
