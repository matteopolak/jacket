# Print_Vars


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `Print_Vars_Config :: struct`
- `append_indented :: (builder: *String_Builder, field: string, indentation: Print_Vars_Config.Indentation, gap: s64)`
- `print_helper :: (builder: *String_Builder, using config: Print_Vars_Config, name_width: s64, value_width: s64, name: string, value: string, type: Type)`
- `get_code_name :: (code: Code) -> string #compile_time`
- `print_vars_helper :: (builder: *String_Builder, config: Print_Vars_Config, $args: [$N] Code, loc: Source_Code_Location) #expand`
- `print_vars_to :: (builder: *String_Builder, $args: .. Code, loc := #caller_location) #expand`
- `print_vars :: ($args: .. Code, loc := #caller_location) #expand`
- `log_vars :: ($args: .. Code, loc := #caller_location) #expand`


## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `Print_Vars_Config`

```jai
Print_Vars_Config :: struct {
    control_flags: enum_flags u32 {
        PRINT_NAME  :: 0x1;
        PRINT_VALUE :: 0x2;
        PRINT_TYPE  :: 0x4;
    } = xx 0x7;

    before_name := "";
    after_name  := " = ";

    before_value := "";
    after_value  := " (";

    before_type := "";
    after_type  := ")";

    line_separator := "\n";

    Indentation :: enum u8 {
        NONE;
        LEFT_JUSTIFIED;
        RIGHT_JUSTIFIED;
    }

    name\_indentation := Indentation.LEFT_JUSTIFIED;
    value_indentation := Indentation.LEFT_JUSTIFIED;
}
```
## Behavioral contract

`print_vars`, `print_vars_to`, and `log_vars` are compile-time expansions over caller expressions. They report source-level names, rendered values, and type information, with `Print_Vars_Config` controlling labels, indentation, spacing, and type display. The `_to` form appends to a caller-provided builder; `print_vars` writes formatted output and `log_vars` routes it through logging. `#caller_location` supplies source location automatically. The expressions are evaluated for reporting, so side-effecting expressions should not be passed unless repeated/eager evaluation is acceptable. Depends on compiler code inspection, program printing, and Basic formatting/logging.
