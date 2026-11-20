# jacket migrate

`jacket migrate` rewrites code written for an older Jai so it compiles with the current language. By default it only prints the diff. `--apply` writes it.

```sh
jacket migrate                      # the project: show what would change
jacket migrate src/ --apply         # write the changes under src/
jacket migrate --list               # every migration, one line each
jacket migrate --only context-type-directive
```

## How it works

Each migration is named after the feature that changed (`context-type-directive`), not a language version. Migrations recognise the old form on their own, so there's no "from" version to give: every migration runs, and those that find nothing do nothing.

Migrations come in two kinds:

- **Token migrations** rewrite a form the current language rejects outright, such as a removed directive.
- **Error migrations** answer a compile error the current compiler reports on the old form. For example, `context-type-directive` rewrites `Context` only where the checker reports `unknown-identifier` for it. A program that declares or imports its own `Context` has no error there, so it is left alone.

Neither kind can touch code that already means something today. A migration whose old form could also be valid current code with a different meaning is marked ambiguous. It runs only when named with `--only`, and `--list` says so.

`jailint::migrate::plan` does the work in memory:

1. Compile the project and record its errors.
2. Run the migrations and apply the edits that don't overlap.
3. Compile again over the edited text (`driver::Options::overlay`), without writing anything.
4. Repeat steps 2 and 3 until nothing changes (at most 8 passes).
5. Compare the final errors with the original errors, matched by file, code and message.

If the migrated code has an error the original didn't, a migration was wrong for this code. The command prints those errors, writes nothing, and exits 1. Errors no migration covers are left alone and counted in a note.

A migration that finds the old form but can't tell what it should become reports a `note:` with what to do by hand, instead of guessing.

## The git guard

Before `--apply` writes anything, it checks that git could give every file back:

- Each file must be in a git work tree. Otherwise it stops; `--allow-no-vcs` skips this check.
- Each file must be tracked and have no staged or unstaged changes, so the migration's edits never mix with yours. Otherwise it lists the files and stops; `--allow-dirty` skips this check.

Each file is written through a temporary file in the same folder, and only if it still holds the text the plan started from.

If the code is in a git repository but `git` isn't installed, the command fails with an install hint. It never skips the check.

## Exit status

| Status | Meaning |
|---|---|
| 0 | Nothing to migrate, a clean dry run, or the files were written |
| 1 | The migrated code would have new errors; nothing was written |
| 2 | A usage problem, an unknown migration, or a refusal from the git guard |

## How to add a migration

1. Write the test first, in `crates/jacket/tests/migrate.rs` or the unit tests in `crates/jailint/src/migrate/tests.rs`: old code in, the expected new code out.
2. Add a `Migration` constant to `crates/jailint/src/migrate/rules.rs`. List it in `MIGRATIONS`, which runs in the order the language changed.
   - Use `Detect::Tokens` for a form the current lexer reads but the language rejects.
   - Use `Detect::Error` to answer a diagnostic code. Copy the code from the compiler's output (`error[unknown-identifier]`).
3. Make the detection decide from the code alone, and mark the migration `ambiguous` if it can't.
4. When no rewrite can be right, return `Answer::Manual` with what to do instead.
5. Add a line to the table below.

Migrations are written from what changed, not from another compiler's sources. Never name a language version in a migration's name or text.

## Migrations

| Name | Old form | New form |
|---|---|---|
| `context-type-directive` | `c: Context;` | `c: #Context;` |
| `foreign-library-directive` | `#foreign_library "x"`, `#foreign_system_library "x"` | `#library "x"`, `#system_library "x"` |
| `vector-component-constructor` | `make_vector2(x, y)` (one argument per component) | `Vector2.{x, y}`; a qualifier stays (`M.Vector3.{...}`) |
| `removed-basic-procedures` | `get_time()`, `bubble_sort(...)` | a note saying what to use; no rewrite |
| `allocator-context-argument` | `New(T, allocator = a)` | `New(T,, allocator = a)` |
| `versioned-module-import` | `#import "freetype-2.12.1";` | `#import "freetype";`, when a module named `freetype` exists |

## Configuration

There's no configuration of its own. The project's `jacket.toml` sets which files belong to the project, as for `jacket lint`. The standard library comes from `JAIC_STDLIB`, or from beside the installed tools.

## Dependencies

- `jailint::driver` compiles roots and collects compile errors, with an in-memory overlay.
- `similar` prints the diffs.
- `git` is needed for `--apply` inside a repository.
