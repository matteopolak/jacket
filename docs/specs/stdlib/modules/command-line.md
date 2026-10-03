# Command_Line

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `Is_Set :: struct (T: Type)`
- `Argument_Flags :: enum_flags u64`
- `Default_Argument_Flags :: Argument_Flags.FREE_ARGUMENTS_ALLOWED|.SORT_HELP|.ALIGN_HELP`
- `parse_arguments :: ($T: Type, flags := Default_Argument_Flags, help_triggers : [] string = .["help", "HELP", "?"], args: []string = .{count = -1}) -> (success: bool, result: T, is_set: Is_Set(T), free_arguments: [..] string)`
- `show_help :: ($T: Type, flags: Argument_Flags, help_triggers: [] string)`

## Public type members

#### `Argument_Flags` (enum_flags)

- `FREE_ARGUMENTS_ALLOWED :: 0x1`
- `DOUBLE_DASH_REQUIRED   :: 0x2`
- `SHOW_HELP_ON_ERROR     :: 0x4`
- `SORT_HELP              :: 0x8`
- `ALIGN_HELP             :: 0x10`

#### `Is_Set(T)` (generated struct)

For each eligible member of `T`, the generated struct contains a same-named `bool` field initialized to `false`. `parse_arguments` sets the field when that argument was explicitly provided.

## Behavioral contract and constraints

`parse_arguments` reflects over an argument struct and returns parsed values, an `Is_Set(T)` record, and free arguments. `Is_Set` tracks explicit assignment even when a supplied value equals its default. Member notes supply help text; notes beginning with `?` are displayed as comments.

When `args` is omitted, the procedure obtains the process argument vector, starts at index 1 (skipping the executable name), and frees that acquired vector before returning. When `args` is supplied, parsing starts at index 0 and treats every element as an argument; the caller must omit the executable name if it is not intended as an argument. Supplied storage is not freed by the parser.

By default an option begins with one `-`; with `DOUBLE_DASH_REQUIRED`, it must begin with `--`. The parser removes the required prefix and matches the remainder exactly to an argument-struct field name. Boolean fields consume no following value and are set to `true`; there is no spelling that sets a boolean field to `false`. Other supported scalar fields consume the next element as their value, and fixed-size arrays consume one element per slot. With `FREE_ARGUMENTS_ALLOWED`, non-option elements are appended to `free_arguments`; a standalone `--` ends option parsing and makes the remaining elements free arguments. A repeated option is an error. Enum flags are unsupported, and ordinary enum values are case-sensitive. Robust quoted argument handling and `#place` members are explicitly incomplete.

The default argument vector uses a sentinel count of `-1` to request process arguments. The returned free-argument array is caller-owned. A parse failure can return partially populated result and `is_set` values; callers should use them only when `success` is true.
