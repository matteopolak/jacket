# Friction list: building the editor and UI library

Modules used: Basic, Math, Hash_Table, Flat_Pool, File, File_Utilities, File_Watcher, Sort, String, Input, Window_Creation, Window_Type, Clipboard, freetype, Extensions/{WebGPU, Image, Environment, Json, Regex, Text_Buffer (new), UI (new)}.

## Language and compiler

- `-check -no_dce` does not type check the bodies of polymorphic procedures nobody instantiates, so mistakes in generic widgets only appear at the first use. The official compiler behaves the same way (a type error inside an uncalled polymorphic procedure still compiles), so this is how polymorphs work and nothing changed.
- The string escape `\u{...}` is not supported; use the literal character. The official compiler rejects it too (`\u` takes exactly four hex digits), so nothing changed.
- Fixed by "feat(sema): warn about a positional struct literal that leaves trailing defaults": a positional struct literal with fewer values than members (`.{a, b, c}` for a four-field struct) is accepted, with a warning, when the left-out members have defaults, including when passed to `array_add`. A member without a default still needs a value, as in the official compiler.
- Fixed by "feat(sema): warn when a declaration of the program is also exported by an import": a module-level name that clashes with an imported one (`is_digit` from Basic, `compare_strings` from Sort) is reported at the declaration, with an "also declared here" note pointing at the imported one.
- Fixed by "feat(diagnostics): hint #scope_module when a file-scoped name is used from a loading file": the error for a `#scope_file` name used from a loading file now suggests `#scope_module`.

## Standard library

- Math has no `round`; the UI library defines its own. The official Math module does not export `round` either, so nothing changed.
- Fixed by "fix(file): stop logging ordinary open failures": `read_entire_file` printed "file_open failed" to stderr for a missing file; `log_errors` now defaults to `false` (also for `file_open`, `map_entire_file_start`, `map_file`, `copy_file`'s `silent`).
- Fixed by "feat(file-utilities): add delete_directory_tree": `delete_directory` is not recursive; `File_Utilities.delete_directory_tree` removes a folder with its content (links are removed, not followed).
- Temp directory lookup is manual (`TMPDIR`, then a fallback); there is no helper. The official modules export no temp-directory helper under any of the usual names (`temp_directory`, `get_temp_dir`, `make_temp_directory` and similar), so nothing was added.

## Library design notes

- Fixed by "feat(ui): warn about focusable widgets laid out with no size": a `text_editor` needs every ancestor column to `grow`; a zero-size focusable now logs a warning once, naming its role, label and key.
- Fixed by "fix(ui): show queued input in the frame that applies it": scripted input needed two frames; `frame` now builds a second time after a frame with input, so one `frame` shows the result.
- Typing ahead of a focus change: the search field must be clicked before text reaches it; shortcuts that open a field do not focus it yet.

## Tooling

- An interpreted run of the smoke test takes about a second once the binary is warm: the front end is under 0.1 s and the rest is the interpreter encoding the PNG of the GPU frame. Fixed by "perf(basic): clear zero-default arrays with one memset": `array_resize` and `NewArray` called `Initialize` once per element, which was half of the interpreted instructions. A first run of a freshly built `jaic` can still wait on the system for tens of seconds before it starts working; use `-check` while editing.
- Fixed by "perf(image): pick PNG row filters from a sample": the interpreted smoke run spent most of its time in the PNG encoder (row filtering about 58%, the match finder about 25%). Rows now score Sub, Up, Average and Paeth at 256 sampled positions and filter only with the winner (Sub and Up eight bytes at a time), and long matches no longer insert every position. The interpreter executes 325 million instructions instead of 1,060 million and the run takes about 0.7 s instead of 1.15 s.

## Fixed in the combined editor

- "fix(ui): retain editor marks until frame painting": stack-local diagnostic/search mark arrays outlived the build callback. UI now snapshots them in the frame allocator; a mutation-after-declaration regression and the wave test cover the lifetime.

- "fix(ui): render diagnostic underlines as clipped waves": underlines now match their documented wave shape and clip to the viewport; headless display-list tests cover them.
- "feat(ui): cache proportional font kerning in text layout": measuring, drawing, selection and hit-testing share cached pair adjustments; real Arial and MONO tests cover the policy.
- "fix(window): use the struct return ABI for Cocoa argument messages": large return values with an argument use objc_msgSend_stret on x64, matching the existing zero-argument helper. A display-free NSValue round trip covers the helper.
- "feat(desktop): add opt-in native menus dialogs and text services": native integrations live entirely in Extensions, with direct AppKit callbacks, menu validation and event ordering tests. Input and Window_Creation retain their existing public declarations.
- "feat(ui): route desktop menus and IME composition through the app loop": menus enter the ordinary shortcut path once; preedit remains outside the document, commits form one undo group, and empty updates cancel composition. Widget and command routing tests cover these paths.
- "feat(editor): combine language services native desktop and text input": the editor retains one session and one document model for all three branches. Save As retires the old LSP URI and opens the new identity only after a successful write. The configuration path remains owned across frames.

## Open

- Accessibility export and browser target remain unimplemented.
- jailsp has no formatting method; the editor calls Extensions/Jai_Format with the nearest jacket.toml.
- Completion snippets/additionalTextEdits need an edit transaction model. Hover is plain text on F1; completion uses a filter/click list; definition has one return location.
- X11 composition is not implemented. Pair kerning is not HarfBuzz shaping, contextual forms or bidirectional layout. Some faces expose no legacy pair adjustments.
- Preedit is a single-line overlay. Surrounding-text attributed-substring queries, reconversion and replacement ranges outside the active selection are not implemented.
- Native menus have no separators, enabled predicates, checkmarks or nested submenus. Windows uses the older shell folder browser, which ignores the starting-directory hint.
- The fallback Save As prompt writes the entered path directly, without native overwrite confirmation.
- AppKit activation is denied in this sandbox. Native menu/dialog appearance, Japanese/Chinese input, candidate selection and screen geometry need a desktop session. Windows and Linux runtime behavior is unverified on this Mac.
- File-watcher directory registration is denied here; smoke explicitly invalidates the tree. Native watcher delivery remains unverified. GPU screenshot checks are optional and no successful rendered appearance is claimed.
