# Print_Color on Windows

`Print_Color` colors console output. On Windows it drives the console's attribute word (`SetConsoleTextAttribute`) by default; `USE_ANSI_CODES_ON_WINDOWS = true` switches to escape sequences, which behave exactly like the Unix path.

## How it works

The engine lives in `stdlib/internal/output/print-color.jai`. `set_console_color` pushes onto a per-stream stack and `reset_console_color` pops it. Streams are stdout and stderr and are tracked separately.

- **Push** reads the stream's current attributes with `GetConsoleScreenBufferInfo`, remembers them as the enclosing level, and applies the new word.
- **Pop** restores the remembered word.
- **Mapping** (`console_attributes_for`): a foreground color replaces only the foreground bits and a background color only the background bits, so `.BG_BLUE` inside a `.RED` block keeps red text. ANSI red/green/blue (bits 1/2/4) map to Windows red/green/blue (4/2/1). `HI_*` and `BG_HI_*` add the intensity bit. `.BOLD` brightens the foreground, `.UNDERLINE` sets `COMMON_LVB_UNDERSCORE`, and `.NORMAL` clears an enclosing underline.
- `with_console_color` and `print_color` use `defer` for the pop, so `return`, `break` and `continue` out of the block, and error paths that unwind normally, all restore the console.

## Behaviour we define

| Case | Behaviour |
| --- | --- |
| Nested blocks | The inner pop restores the enclosing block's attributes, not the default. |
| `SAVE_RESTORE_ON_WINDOWS = false` (default) | The outermost pop returns to the attributes the stream had before the first color was ever pushed, even if the program recolored the console between blocks. |
| `SAVE_RESTORE_ON_WINDOWS = true` | The outermost pop returns to whatever was set just before that outermost push. |
| Redirected or closed handle (file, pipe) | `GetConsoleScreenBufferInfo` fails, so no attribute call is made and no escape codes are written. The push is still counted so pops stay balanced. |
| Pop without a push | Restores the baseline if one is known and the stream is a console; otherwise nothing. |
| More than 32 nested levels | Levels beyond 32 are counted but not recorded; unwinding stays balanced and the outer levels restore correctly. |
| `USE_ANSI_CODES_ON_WINDOWS = true` | `\e[<style>;<color>m` and `\e[0m`. Nested blocks reset to the default, as on Unix. |

No ANSI fallback is used in native mode: with a non-console handle you get plain text.

Limits: a process that dies abnormally inside a block (assert, abort, crash, kill) cannot restore the console. The colors are process-wide console state, so threads that print colored text at the same time share one stack per stream (the stack itself is guarded by a spin lock; interleaving of the text is not).

## How to change it

- The console is reached through `Console_Backend { query, apply }`. `set_console_backend` swaps it and `reset_console_backend` restores the system one. Tests install a fake that holds an attribute word per stream.
- To add a style bit, extend `console_attributes_for` and the fake test.
- The `#module_parameters` live in `stdlib/Print_Color/module.jai`. `SAVE_RESTORE_ON_WINDOWS` is an ordinary (per-import) parameter, so two imports can use different restore policies. `USE_ANSI_CODES_ON_WINDOWS` is a program parameter (the second list: `#import "Print_Color"()(USE_ANSI_CODES_ON_WINDOWS = true)`), shared by every import because one process writes one console.

## Configuration

- `USE_ANSI_CODES_ON_WINDOWS := false`
- `SAVE_RESTORE_ON_WINDOWS := false`

## Tests

- `stdlib/Print_Color/tests/native-console.jai` runs on every host against a fake backend: attribute mapping, nesting, both restore policies, independent streams, early return, redirected stream (zero attribute calls), unbalanced pop and deep nesting.
- `stdlib/Print_Color/tests/windows-console.jai` needs a Windows host. It redirects stdout to a file and checks that the bytes are untouched, then attaches a real console screen buffer (`AllocConsole` plus `CONOUT$`) and reads the attributes back through the Windows API. The real-console half needs a console; a headless runner that cannot allocate one fails with "no console screen buffer is available" rather than skipping.
- `tests/portable-stdlib/print-color.jai` checks the ANSI byte stream.

## Dependencies

`Basic` output, and on Windows the `Windows` module's kernel32 console declarations.
