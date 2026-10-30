# Lexicon

## What it is

`jaic::lexicon` is the one table of the language's words as tools see them: keywords by class, built-in types, directives by role, and the here-string terminators that name a language. The language server, the linter and the editor grammars read it instead of keeping copies.

## How it works

- The lexer treats keywords as plain identifiers and the parser gives them meaning, so the lexicon describes and does not decide how a file is read.
- `WORDS` has a class per word (conditional, repeat, statement, ...) and a `reserved` flag. `is_reserved` and `reserved()` drive rename refusal and keyword completion in `jailsp`; `class_of` lets `jailint` ask what a word is.
- `DIRECTIVES` has a role (import, conditional, modifier, value, run) for highlighting and a one-line description for completion and hover. A directive without a description is not offered.
- `lexicon::directive::*` (bare name, for `Tok::directive()` and token comparisons) and `lexicon::spelled::*` (with the `#`, for text searches) name the directives the tools match on. Use them instead of string literals; a test checks each is in `DIRECTIVES`. Add one to the `known_directives!` list when a tool needs to match another.
- `lexer::LANGUAGE_TAGS` and `here_string_tag` live with the lexer, because the lexer owns the here-string header (`#string,cr TAG`): `lexer/here.rs` is the only reader of it, for the lexer and for the language server.
- `editors/lexicon.json` is `lexicon::to_json()`, checked in. A unit test fails while it is stale.
- `editors/vscode/scripts/build-grammar.ts` reads the JSON to write the TextMate grammar and the word lists between the `GENERATED` markers of `editors/nvim/syntax/jai.vim`. `--check` fails when either is stale, and when the lexicon's here-string languages differ from the grammar's `EMBEDDED_LANGUAGES`.

## How to change it

1. Edit `crates/jaic/src/lexicon.rs` (or `LANGUAGE_TAGS` in `lexer/here.rs`).
2. `JAI_BLESS=1 cargo test -p jaic --lib lexicon` rewrites `editors/lexicon.json`.
3. `node editors/vscode/scripts/build-grammar.ts` regenerates the grammar and the Neovim lists.

Keep `DIRECTIVES` sorted by name (a test checks it). A new here-string language also needs an `EMBEDDED_LANGUAGES` entry.

## Configuration

`JAI_BLESS=1` makes the lexicon test rewrite the JSON.

## Dependencies

Node 24 for the grammar script; nothing else.
