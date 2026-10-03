# Print_Color


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `print_color :: (format_string: string, args: .. Any, color: Console_Color, style: Console_Style = .NORMAL, to_standard_error := false)`
- `Console_Color :: enum_flags u16`
- `Console_Style :: enum`
- `with_console_color :: (color: Console_Color, code: Code, style: Console_Style = .NORMAL, to_standard_error := false) #expand`
- `set_console_color :: (color: Console_Color, style: Console_Style = .NORMAL, to_standard_error := false)`
- `reset_console_color :: (to_standard_error := false)`


## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `Console_Color`

```jai
Console_Color :: enum_flags u16 {
    BLACK         :: 30;
    RED           :: 31;
    GREEN         :: 32;
    BLUE          :: 34;
    CYAN          :: 36;
    MAGENTA       :: 35;
    YELLOW        :: 33;
    WHITE         :: 37;

    HI_BLACK      :: 90;
    HI_RED        :: 91;
    HI_GREEN      :: 92;
    HI_BLUE       :: 94;
    HI_CYAN       :: 96;
    HI_MAGENTA    :: 95;
    HI_YELLOW     :: 93;
    HI_WHITE      :: 97;

    BG_BLACK      :: 40;
    BG_RED        :: 41;
    BG_GREEN      :: 42;
    BG_BLUE       :: 44;
    BG_CYAN       :: 46;
    BG_MAGENTA    :: 45;
    BG_YELLOW     :: 43;
    BG_WHITE      :: 47;

    BG_HI_BLACK   :: 100;
    BG_HI_RED     :: 101;
    BG_HI_GREEN   :: 102;
    BG_HI_BLUE    :: 104;
    BG_HI_CYAN    :: 106;
    BG_HI_MAGENTA :: 105;
    BG_HI_YELLOW  :: 103;
    BG_HI_WHITE   :: 107;
}
```
### `Console_Style`

```jai
Console_Style :: enum {
    NORMAL    :: 0;
    BOLD      :: 1;
    UNDERLINE :: 4;
}
```
## Behavioral contract

`print_color` writes formatted output using ANSI-style foreground/background color and style flags, selecting stdout or stderr. `with_console_color` is an expansion that brackets caller code with set/reset operations; explicit `set_console_color` and `reset_console_color` are also available. On Windows the default is native console attributes with a per-stream stack (see `docs/native/windows-console-colors.md`); `USE_ANSI_CODES_ON_WINDOWS` selects escape sequences. On Unix-like systems it emits escape sequences and does not detect terminal capability. Depends on `Basic` output and, on Windows, console APIs.

## Module parameters

These configure Windows console color handling; they are import-time options, not exported globals.

```jai
#module_parameters (SAVE_RESTORE_ON_WINDOWS := false)(USE_ANSI_CODES_ON_WINDOWS := false);
```