# Print_Color on Windows

`Print_Color` colors console output. On Windows it drives the console's attribute word (`SetConsoleTextAttribute`) by default; `USE_ANSI_CODES_ON_WINDOWS = true` switches to escape sequences, as on Unix.

## How it works

The engine is in `stdlib/internal/output/print-color.jai`. `set_console_color` pushes onto a per-stream stack (stdout and stderr separately) and `reset_console_color` pops. `with_console_color` and `print_color` pop with `defer`, so `return`, `break` and `continue` restore the console.

- Push reads the stream's attributes with `GetConsoleScreenBufferInfo`, remembers them, and applies the new word. Pop restores them.
- `console_attributes_for` maps colors: a foreground color replaces only foreground bits, a background only background bits, so `.BG_BLUE` inside `.RED` keeps red text. ANSI red/green/blue swap bit order for Windows. `HI_*` adds intensity, `.BOLD` brightens, `.UNDERLINE` sets `COMMON_LVB_UNDERSCORE`.

| Case | Behaviour |
| --- | --- |
| Nested blocks | The inner pop restores the enclosing block's attributes. |
| `SAVE_RESTORE_ON_WINDOWS = false` (default) | The outermost pop returns to the attributes before the first push, even if the program recolored in between. |
| `SAVE_RESTORE_ON_WINDOWS = true` | The outermost pop returns to what was set just before that push. |
| Redirected or closed handle | No attribute call and no escape codes; pops stay balanced. |
| More than 32 levels | Counted but not recorded; unwinding stays balanced. |
| `USE_ANSI_CODES_ON_WINDOWS = true` | `\e[<style>;<color>m` and `\e[0m`; nested blocks reset to default. |

A process that dies inside a block cannot restore the console. Threads printing colors at once share one stack per stream.

## How to change it

- The console is reached through `Console_Backend { query, apply }`; `set_console_backend` swaps it, `reset_console_backend` restores it. Tests install a fake.
- A new style bit goes in `console_attributes_for` and the fake test.
- Parameters live in `stdlib/Print_Color/module.jai`. `SAVE_RESTORE_ON_WINDOWS` is per import. `USE_ANSI_CODES_ON_WINDOWS` is a program parameter (`#import "Print_Color"()(USE_ANSI_CODES_ON_WINDOWS = true)`), shared by all imports.

## Tests

`tests/native-console.jai` runs everywhere against a fake backend. `tests/windows-console.jai` needs a Windows host and a real console (`AllocConsole`); without one it fails rather than skips. `tests/basics.jai` checks the ANSI stream.

## Dependencies

`Basic` output, and on Windows the `Windows` module's kernel32 console declarations.
