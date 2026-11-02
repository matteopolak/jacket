# term

A small terminal emulator for macOS and Linux. It runs `$SHELL` on a real pseudoterminal and paints a cell grid with `Extensions/UI`. Windows type-checks and reports that ConPTY is unsupported.

```sh
jaic examples/term/main.jai -exe "$T/term"
"$T/term"                              # interactive $SHELL, /bin/sh if unset
"$T/term" -c 'printf "\033[31mred\033[0m\n"'
(cd examples/term && jacket test -j 3)
T=/path/to/external/build sh examples/term/tools/smoke.sh
```

| Flag | Meaning |
| --- | --- |
| `-c`, `--command TEXT` | Run `/bin/sh -c TEXT` instead of the interactive shell. |
| `--smoke` | Run a bounded headless session through a real PTY and check the grid, glyph display list, selection and paste. |
| `--frame PATH` | With `--smoke`, also render offscreen, check a background pixel and write a PNG if a GPU is available. |

Drag with the left mouse button to select; Cmd+C/V on macOS or Ctrl+Shift+C/V on Linux copies/pastes through `Clipboard` via UI hooks. The wheel browses up to 2,000 history rows. Typing returns to the live screen. Ctrl+A–Z sends control bytes; Enter, Backspace, Tab, Escape, arrows, Home/End, Insert/Delete and Page Up/Down send terminal sequences. Closing the window closes the PTY and reaps the shell. The window closes after the shell exits and its output has drained.

## How it works

- `vt/` owns streaming UTF-8 decoding, VT parsing, attributes, primary/alternate grids and a bounded ring of scrollback rows. It has no UI or PTY dependency. `Unicode` encodes grid scalars back to UTF-8 for display and copying.
- `terminal.jai` queues input, drains at most 128 KiB of PTY output per frame, resizes both the grid and PTY, routes mouse/keyboard events, and paints cells. UI may build twice for input, so keyboard events are consumed to prevent duplicate writes.
- `Extensions/Pty` allocates and unlocks a master, opens the slave, forks, creates a controlling terminal/session, duplicates standard descriptors and execs the shell. It prepares arguments/environment before fork and sets `TERM=xterm-256color` only in the child's environment. The nonblocking master retains partial input writes. `TIOCSWINSZ` makes the kernel signal the foreground process group with SIGWINCH.
- A custom UI leaf paints cell backgrounds, glyphs, selection, underline and cursor. Fixed cell positions bypass proportional text layout while retaining UI's FreeType atlas and glyph cache. Bold uses a second glyph draw; italic uses sheared horizontal strips of the same cached glyph. No extra public UI API is needed.

Supported controls: CR, LF/VT/FF, BS and tab; ESC index/reverse index/next line, save/restore and charset designation consumption; CSI cursor movement/position, erase display/line/characters, insert/delete characters and lines, scroll up/down and margins; SGR reset, 16/256/RGB colors, bold/italic/underline/inverse; OSC 0/2 title (BEL or ST terminator); DEC 1049 alternate screen, 2004 bracketed paste, 25 cursor visibility and 1 application cursor keys; status/cursor/device queries. Unknown CSI finals are consumed. Parameters, OSC strings and pending query replies are bounded.

## How to change it

Add sequences in `vt/parser.jai`, with byte-stream snapshots in `tests/parser.jai`. Grid operations and wide-cell invariants belong in `vt/grid.jai`. Add keys, painting and selection behavior in `terminal.jai`; `tests/interface.jai` exercises the actual custom leaf with synthetic input and clipboard hooks. Keep PTY lifecycle changes in `stdlib/Extensions/Pty`, with tests in that module. Its allocation is intended before other threads start because `ptsname` and inherited environment strings are borrowed during setup.

Resize preserves overlapping cells, clamps cursors and clears scrollback rather than reflowing old lines. The width classifier covers common CJK and emoji ranges; combining accents and complex grapheme clusters need more work (see FRICTION.md).

## Configuration and dependencies

Builds need the repository compiler and standard library, FreeType, and the UI/WebGPU native libraries. Interactive use needs a native display and GPU; headless parser/PTY checks do not. UI chooses system fonts; its `JAI_UI_FONT` and `JAI_UI_MONO_FONT` overrides also apply here. `$SHELL` must be an executable path. `T` sets the smoke build directory (a removed temporary directory when unset), `JAIC` selects its compiler, `FRAME` selects its PNG path, and `REQUIRE_GPU=1` makes an unavailable adapter fail the smoke.

The smoke script probes for a real PTY and skips when the device is unavailable. Missing GPUs skip only the GPU path; a successful adapter must pass readback and PNG checks. Tests generate their byte streams and shell output, with no external assets.
