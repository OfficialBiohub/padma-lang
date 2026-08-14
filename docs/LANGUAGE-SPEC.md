# Padma Language Specification Draft

Padma has one semantic language and two keyword vocabularies. Bengali and English keywords are aliases, not separate dialects. A source file may use either vocabulary or mix them deliberately; diagnostics follow the detected source locale unless `# padma:locale=bn` or `# padma:locale=en` overrides it.

The current executable core supports UTF-8 identifiers, Bengali digits, numbers, strings with `\\n`, `\\t`, escaped quotes, booleans, variable declaration, assignment, output, arithmetic, comparisons, conditional blocks, and bounded `while` loops. Statements are separated by newlines; blocks use braces.

List literals use square brackets and comma-separated expressions, for example `[১, ২, ৩]`. Indexing and collection mutation are planned additions.

The first runtime builtins are `input(prompt)`, `file.write(relative_path, content)`, `http.get(url)`, `process.run(program, arguments...)`, and `media.download(url, relative_output_template)`. Process execution is allowlisted and arguments are passed without a shell. Output paths must be relative to the current directory and cannot contain `..`; downloader use remains subject to authorization and platform terms.

| Concept | Bengali | English |
|---|---|---|
| Declaration | `ধরি` | `let` |
| Output | `দেখাও` | `print` |
| Conditional | `যদি` / `নইলে` | `if` / `else` |
| Loop | `যতক্ষণ` | `while` |
| Function | `ফাংশন` | `function` / `fn` |
| Return | `ফেরত` | `return` |
| Boolean | `সত্য` / `মিথ্যা` | `true` / `false` |

Functions use lexical call scopes, comma-separated parameters, calls with parentheses, and an optional `ফেরত`/`return` value. The specification is versioned with the compiler. A syntax change requires a test, an example, and a documented compatibility decision before it is considered stable.
