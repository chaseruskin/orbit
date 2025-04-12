# Glob Patterns

A glob pattern is a string used for pattern matching against the names in a filesystem directory such that a name is expanded into a list of names matching that pattern.

The following rules are used by Orbit for any string interpreted as a glob-style pattern:

- `?` matches any single character.
- `*` matches any (possibly empty) sequence of characters.
- `**` matches the current directory and arbitrary subdirectories. To match files in arbitrary subdiretories, use `**/*`.  
This sequence __must__ form a single path component, so both `**a` and `b**` are invalid and will result in an error. A sequence of more than two consecutive `*` characters is also invalid.
- `[...]` matches any character inside the brackets. Character sequences can also specify ranges of characters, as ordered by Unicode, so e.g. `[0-9]` specifies any character between 0 and 9 inclusive. An unclosed bracket is invalid.
- `[!...]` is the negation of `[...]`, i.e. it matches any characters __not__ in the brackets.
- The metacharacters `?`, `*`, `[`, `]` can be matched by using brackets (e.g. `[?]`). When a `]` occurs immediately following `[` or `[!` then it is interpreted as being part of, rather then ending, the character set, so `]` and NOT `]` can be matched by `[]]` and `[!]] `respectively. The `-` character can be specified inside a character sequence pattern by placing it at the start or the end, e.g. `[abc-]`.

These rules originate from the [`glob`](https://docs.rs/glob/latest/glob/index.html) crate.