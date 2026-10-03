# Text_File_Handler


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `Text_File_Handler :: struct`
- `deinit :: (using x: *Text_File_Handler)`
- `start_file :: (handler: *Text_File_Handler, short_name: string, full_path: string, optional := false, loc := #caller_location)`
- `start_from_memory :: (using handler: *Text_File_Handler, _short_name: string, _full_path: string, _file_data: string, loc := #caller_location)`
- `consume_next_line :: (handler: *Text_File_Handler) -> (line: string, found: bool)`
- `consume_next_line :: (sp: *string) -> string, bool`
- `break_by_spaces :: (line: string) -> (left: string, right: string)`
- `error :: (handler: Text_File_Handler, format: string, args: .. Any, flags := Log_Flags.NONE, loc := #caller_location)`
- `warn :: (handler: Text_File_Handler, format: string, args: .. Any, flags := Log_Flags.NONE, loc := #caller_location)`
- `file_to_array :: (filename: string, do_trim := true, skip_blank_lines := true, do_version_number := false, use_data_from_memory := false, data_from_memory := "", loc := #caller_location) -> (success: bool, array: [..] string)`
- `file_to_table :: (filename: string, do_trim := true, skip_blank_lines := true, do_version_number := false, use_data_from_memory := false, data_from_memory := "", loc := #caller_location) -> (success: bool, table: Table(string, void))`


## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `Text_File_Handler`

```jai
Text_File_Handler :: struct {
    short_name: string;
    full_path:  string;

    comment_character : u8 = #char "#";

    do_version_number                 := true;
    strip_comments_from_ends_of_lines := true;
    auto_skip_blank_lines             := true;

    file_data_to_free: string;
    file_data:         string;

    failed  := false;
    version := -1;

    line_number := 0;
}
```
## Behavioral contract

`Text_File_Handler` consumes a line-oriented text view with configurable comment character, comment stripping, blank-line skipping, and optional leading `[version]`. `start_file` owns/frees bytes it reads; `start_from_memory` borrows the input and the caller must keep it alive. Names are borrowed too. The handler's sticky `failed` flag is set on parse/load errors and never reset automatically. `consume_next_line` advances the stored remainder and updates line count; `file_to_array/table` can trim and skip blank lines. Parsing helpers report errors with source path and line. The comment marker is one byte; no Unicode-aware delimiter behavior is promised.
