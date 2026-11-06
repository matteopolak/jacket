# Documentation code coloring

Documentation uses a small presentation scanner driven by `crates/jacket/src/doc/coloring/spec.json`. Rust produces initial coloring spans, and a generated TypeScript interpreter colors edited examples. Compiler-resolved declaration links remain separate facts; the browser scanner never resolves a name or loads the compiler to color a keystroke.

## How it works

The spec defines ordered operations and keyword/type sets. The operations cover line comments, nested block comments, escaped quoted strings, notes, directives and here-strings, numeric/range boundaries, identifiers and a consuming plain-text fallback. The interpreter consumes scanned input forward with nesting counters and no regex backtracking. Notes reuse that interpreter for their existing plain-name, numeric and string colors with note recognition disabled inside; this adds at most one extra forward pass over note text and never exposes note identifiers to semantic linking. Unterminated constructs retain their literal source through the available input, with notes bounded to their line; this is presentation, not a replacement for compiler diagnostics.

Identifiers use the explicit ASCII/non-ASCII policy in the spec rather than each runtime's Unicode database. Rust offsets refer to UTF-8 bytes. Browser offsets refer to UTF-16 slices so editing cannot rewrite astral or partial UTF-16 text. The cross-language check normalizes browser offsets to UTF-8 and compares both lexical partitions and coloring with reviewed fixtures.

`highlight.rs` overlays admitted compiler-resolved destinations on the lexical spans for initial output. After an edit, those destinations would be stale, so the editor shows coloring without links. Reset restores the original compiler-produced markup and its links. Recoloring streams tokens and coalesces adjacent color/plain runs into spans and text nodes with fixed classes; author source is never assigned to `innerHTML`. Ordinary non-Jai fences use Astro's bundled Shiki component, while diagnostic text has its own typed presentation roles.

## How to change it

Change the spec and add a reviewed case to `coloring/fixtures.json` before changing scanner behavior. Implement a new operation in the Rust interpreter and the TypeScript runtime template together, and extend the generator's strict operation admission. Do not add an independent browser regex lexer or a second keyword table.

```sh
PYTHONPATH=tools python3 tools/generators/generate_doc_coloring.py
PYTHONPATH=tools python3 tools/generators/generate_doc_coloring.py --check
python3 tools/ci/check_doc_coloring.py --out /tmp/owned-doc-coloring.json
```

The generator's `--check` compares without writing checked-in output and requires no npm installation. The cross-language check runs the production Rust interpreter through the `doc_coloring` fixture example and the generated browser interpreter against the same independent expected segments. It requires an absent owned output path. Use the repository's heavy-work gate and owned Cargo target for compilation.

`tools/dev/doc_edit_coloring.mjs BASE OUTPUT_JSON` checks actual trusted browser edits in both themes, literal hostile text, stale-link removal and exact Reset restoration. It uses the production site and does not click Run or load a compiler for coloring.

Keep lexical classification independent of semantic resolution. The permanent site's `ResolvedText` still carries compiler-resolved targets and UTF-8 spans; the coloring spec does not admit references or infer declaration identity. Keep declaration layout in the existing `Extensions/Jai_Format` declaration profile. Astro renders its authored lines and compiler-resolved spans; this scanner supplies coloring only. Ordinary Markdown and literal contents remain unchanged.

## Configuration and dependencies

There is no runtime environment switch or network fetch for coloring. The generated browser source embeds the spec and ships with the source-only builder. Rust uses existing Serde dependencies and a once-initialized spec; browser code uses local TypeScript/DOM APIs. Generator checks and focused shared fixtures run in CI. Cross-language fixtures establish the checked behavior, not a measured browser latency or bundle-size result; the prototype still needs actual edit/browser and cost validation before the final architecture decision.
