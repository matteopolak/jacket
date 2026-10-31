# Lexicon

`jaic::lexicon` is the one table of the language's words as tools see them: keywords by class, built-in types, directives by role, and here-string language tags. The language server, linter and editor grammars read it instead of keeping copies.

## How it works

- The lexer treats keywords as identifiers and the parser gives them meaning, so the lexicon describes and doesn't decide.
- `WORDS` gives each word a class and a `reserved` flag (rename refusal, keyword completion). `DIRECTIVES` gives each directive a role and a one-line description for completion and hover; no description means it isn't offered.
- `lexicon::directive::*` (bare name) and `lexicon::spelled::*` (with `#`) name directives tools match on. Use them instead of string literals.
- `lexer::LANGUAGE_TAGS` lives in the lexer, which owns the here-string header (`#string,cr TAG`).
- `editors/lexicon.json` is `lexicon::to_json()`, checked in. The VS Code TextMate grammar and the word lists in `editors/nvim/syntax/jai.vim` are generated from it.

## How to change it

1. Edit `crates/jaic-syntax/src/lexicon.rs` (or `LANGUAGE_TAGS` in `lexer/here.rs`).
2. `INSTA_UPDATE=always cargo test -p jaic --lib lexicon` rewrites `editors/lexicon.json`.
3. `node editors/vscode/scripts/build-grammar.ts` (Node 24) regenerates the grammar and Neovim lists.

Keep `DIRECTIVES` sorted by name. A new here-string language also needs an `EMBEDDED_LANGUAGES` entry in the grammar script.
