# Doc comments

## What it is

Comments directly above a declaration are its documentation, written in Markdown, in the Rust style. The language server shows them in hover, completion and signature help, resolves `[name]` links in them, and the VS Code extension highlights their Markdown. The compiler ignores comments; everything here is editor tooling.

## Convention

A declaration's docs are the run of `//` lines (`///` and `//!` work too) directly above its line, with no blank line between, or a `/* */` block directly above it. One space after the marker is removed from each line; the rest is Markdown, verbatim. A line of only `-----`, `=====` or `/////` (four or more) ends the run: it is a rule, not text.

```jai
// Scales a [Point] by `factor`, starting from [`Mode.IDLE`].
//
// - p: the point to scale
// - `factor` — how much to scale by
scale :: (p: Point, factor: int) -> Point { ... }
```

Fields, enum members and parameters can also have a trailing comment on their line, or comment lines directly above them:

```jai
Point :: struct {
    x: int; // Horizontal position.
    // Vertical position, like [Point.x].
    y: int;
}
```

A trailing comment counts only when what precedes it is a whole declaration (ends in `;` or `,`, or has no `::`) and holds no `{`, so `f :: () { // note` does not document `f`.

### Parameters

A parameter is documented by, in this order:

1. comment lines directly above it, or a trailing comment on its line, in the header;
2. an item of a Markdown list in the procedure's docs that starts with its name: `- name: text`, ``- `name`: text`` or `- name — text` (an en dash or ` - ` also works). Following indented lines continue the item.

```jai
// Copies bytes.
// - count: how many to copy
copy :: (
    // Where the bytes go.
    to: *u8,
    from: *u8, // Where they come from.
    count: s64,
) {}
```

Signature help shows the active parameter's doc, and hover on a parameter, at its declaration or a use in the body, shows it too.

### Links

Rustdoc-style intra-doc links name a declaration from the scope of the documented declaration:

| Written | Names |
|---|---|
| `[print]`, ``[`print`]``, `[print()]` | `print` |
| `[Basic.print]` | `print` in the module imported as `Basic` |
| `[Point.x]`, `[Mode.IDLE]` | a struct field or enum member |
| `[Point.LIMIT]`, `[Outer.Inner]`, `[Point.helper]` | a constant, type or procedure declared inside a struct |
| `[Ring.items]`, `[Ring.CAPACITY]` | a field or parameter of a polymorphic struct, found in its source (it need not be instantiated) |
| `[the text](print)` | `print`, shown as "the text" |

A link that does not resolve stays plain text (the `broken_doc_link` rule of [jailint](../tools/jailint.md) reports it), and so do `a[i]`, `[0]`, `[x][y]` and `[text](https://...)`. Names inside inline code and code fences are never links. Untagged code fences in docs are rendered as `jai`.

## How it works

`crates/jaic/src/docs.rs` (shared with jailint, which reports links that resolve to nothing) does the text work, with no parser: it finds the comment lines above (or after) a declaration given the offset of its name, splits a parameter list, reads list items, and scans lines for links. It reads a file on demand and only for the declaration asked about, so hover and completion cost what they did before (the largest scan is 80 lines back, to tell whether a name is a parameter).

`doc_view.rs` is the glue:

- **Hover.** `Analysis::hover` also returns where what it describes is declared (`Compiler::ide_hover_decls`: one declaration per overload, in the order of the hover's lines). Each declaration's docs are read from the compiler's copy of its file, so the stdlib and modules work like the open document. With overloads, each signature is followed by its own docs. Markdown clients get the signature in a `jai` fence and then the docs; plain-text clients get the comment as written.
- **Links.** Names in the docs resolve through `Compiler::ide_resolve_path` from the scope at the documented declaration (the file's scope when the file has no recorded scopes, so stdlib docs can link to their own file's declarations). A resolved link is rewritten to `[text](file:///path#L<line>)`, which VS Code opens at the line. Unresolved links are left alone.
- **Completion.** Items carry `data` (the declaring file and offset, from `IdeName::decl`). Lists of at most `EAGER_DOCS` (24) items carry `documentation` straight away; longer ones leave it to `completionItem/resolve`, so completing at an empty prefix reads no comments. Completion answers from the last compile, which matches the document only up to the cursor; `Analysis::shift_to` moves the offsets of declarations below the edit to where they are now, or their docs would be read from the wrong line.
- **Signature help.** `SignatureInformation` has `documentation` (the procedure's) and `ParameterInformation.documentation` per parameter. The docs are looked up in the same repair as the signature (the open call's line blanked), so they appear while the call is still unclosed. Links fall back to the last compile when the document cannot be repaired.
- **In the source.** `docs::sites` finds the links of the comments above declarations. Go to definition and hover work on the label, `textDocument/documentLink` reports each (target `file:///path#L<line>`), and semantic tokens colour the label like what it names (a type, function, enum member, property, namespace or constant). Only the label spans get a semantic token, so the TextMate styling of the rest of the comment stays.
- **Code fences.** `docs::fenced_code` finds the code lines of untagged or `jai` fences in `//` comments; `fence_tokens` lexes each and sends keyword, string, number, directive and operator tokens, with a name before `(` as a function, a capitalized name as a type and other names as variables (nothing is resolved). Comments inside the code get no token. Clients without a stateful grammar (the playground) get highlighted fences from these alone.
- **Auto-import completion** uses the same extraction (`exports.rs`), limited to 40 lines.

### Highlighting

Markers stay visible; only styling changes. `editors/vscode/syntaxes/jai-doc-comments.tmLanguage.json` is an injection grammar (`injectTo: ["source.jai"]`) into `comment.line.double-slash.jai`, `comment.block.jai` and `meta.comment-run.jai`. It scopes:

| Markdown | Scope |
|---|---|
| `**bold**` | `markup.bold.jai` |
| `*italic*`, `_italic_` | `markup.italic.jai` |
| `` `code` `` | `markup.inline.raw.string.jai` |
| `# Heading` (the rest of the line) | `markup.heading.jai` |
| list bullets and `1.` | `punctuation.definition.list.begin.markdown.jai markup.list.unnumbered.jai` |
| `[name]`, `[text](target)` labels and targets | `markup.underline.link.jai` |
| a ```` ``` ```` fence with no language or `jai`, in `//` lines or a block comment | `markup.fenced_code.block.jai`, the code with `source.jai` scopes; the `// ` prefix stays `comment.line.double-slash.jai` |

A fence must span several lines, but a TextMate rule per `//` line cannot carry state. So `jai.tmLanguage.json` (generated by `scripts/build-grammar.ts`) wraps a run of whole-line comments in `meta.comment-run.jai`, a rule that adds no styling, and the injection puts the fence rule in it.

Resolved links also arrive as semantic tokens (above), which override the TextMate colour on exactly those spans.

## How to change it

- The convention (what counts as a doc, parameter items, link syntax, which unresolved links count as meant): `crates/jaic/src/docs.rs`. Its unit tests cover each rule; `tests/doc_comments.rs` covers the editor features.
- What a link can name: `Compiler::ide_resolve_path` in `crates/jaic/src/sema/ide.rs` (entities, module members, struct fields, enum members, and what a struct declares in its body).
- The colour of a link label: `target_kind` in `doc_view.rs` guesses from the target's declaration line.
- Markdown styling: edit the regexes in `jai-doc-comments.tmLanguage.json`, then add a case to `editors/vscode/test/grammar/doc-comments.jai` (`pnpm run test:grammar`). `scripts/tokenize-corpus.ts` loads the injection too, so the corpus check covers it.
- The playground (another repository) needs: the `meta.comment-run.jai` wrapper from the regenerated grammar, the injection grammar above (or equivalent rules), a mapping from `markup.bold`, `markup.italic`, `markup.inline.raw`, `markup.heading`, `markup.list` and `markup.underline.link` to styles, and the semantic token types the server sends on link labels (`type`, `function`, `enumMember`, `property`, `namespace`, `variable` with `readonly`), which it must apply on top of the comment colour. Fenced code arrives as semantic tokens too, so the fence grammar rule is optional there.

Gotchas: link names are looked up from the scope of the documented declaration, not the use site; a doc comment above a declaration inside a procedure sees that procedure's scope. Extraction starts from the name's offset, so a declaration that starts on a different line from its name (`name ::` and the literal on the next line) takes the docs above the name's line. Block comments are read only when they end directly above the declaration line.

## Configuration

None. Hover and signature help use Markdown when the client lists it in `textDocument.hover.contentFormat` or `textDocument.signatureHelp.signatureInformation.documentationFormat`; completion documentation is always Markdown.

## Dependencies

- `jaic` editor facts (`ide_hover_decls`, `ide_resolve_path`, `IdeName::decl`) for locating declarations and resolving links.
- `vscode-textmate`/Oniguruma semantics for the injection grammar, tested with `vscode-tmgrammar-test`.
- See [language server](language-server.md) for the surrounding features.
