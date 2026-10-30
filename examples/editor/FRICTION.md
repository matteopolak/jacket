# Friction list: building the editor and UI library

Modules used: Basic, Math, Hash_Table, Flat_Pool, File, File_Utilities, File_Watcher, Sort, String, Input, Window_Creation, Window_Type, Clipboard, freetype, Extensions/{WebGPU, Image, Environment, Json, Regex, Text_Buffer (new), UI (new)}.

## Language and compiler

- `-check -no_dce` does not type check the bodies of polymorphic procedures nobody instantiates, so mistakes in generic widgets only appear at the first use.
- The string escape `\u{...}` is not supported; use the literal character.
- A positional struct literal with fewer values than members (`.{a, b, c}` for a four-field struct) failed when passed to `array_add`; named fields work.
- A module-level name that clashes with an imported one (`is_digit` from Basic, `compare_strings` from Sort) is reported only as a warning at the use site, which made the cause hard to see in a long file. Naming the other declaration in the message would help.
- Declarations after `#scope_file` are invisible to a file that `#load`s the module; `#scope_module` is what an app split over several files needs, and the error does not hint at it.

## Standard library

- Math has no `round`; the UI library defines its own.
- Fixed by "fix(file): stop logging ordinary open failures": `read_entire_file` printed "file_open failed" to stderr for a missing file; `log_errors` now defaults to `false` (also for `file_open`, `map_entire_file_start`, `map_file`, `copy_file`'s `silent`).
- Fixed by "feat(file-utilities): add delete_directory_tree": `delete_directory` is not recursive; `File_Utilities.delete_directory_tree` removes a folder with its content (links are removed, not followed).
- Temp directory lookup is manual (`TMPDIR`, then a fallback); there is no helper.

## Library design notes

- Fixed by "feat(ui): warn about focusable widgets laid out with no size": a `text_editor` needs every ancestor column to `grow`; a zero-size focusable now logs a warning once, naming its role, label and key.
- Fixed by "fix(ui): show queued input in the frame that applies it": scripted input needed two frames; `frame` now builds a second time after a frame with input, so one `frame` shows the result.
- Typing ahead of a focus change: the search field must be clicked before text reaches it; shortcuts that open a field do not focus it yet.

## Tooling

- A cold interpreted run of the smoke test takes tens of seconds, mostly compile; use `-check` while editing.
- The pre-commit lint rejects magic numbers in stdlib; every tuning constant needs a name.

## Not done

LSP client for jailsp, native menus and file dialogs (the open prompt stands in), IME and text shaping/kerning, accessibility export, a browser target.
