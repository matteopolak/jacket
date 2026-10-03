# Check

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

### Windows-specific declarations


- `DEBUG_BINDINGS :: false`
- `verify_elsewhere_binding :: (header: *Code_Procedure_Header, os: Operating_System_Tag, cpu: CPU_Tag) -> bool`
- `print_binding_check_memory_stats :: ()`
- `C_Porting_Check_Settings :: struct`
- `check_c_porting :: (message: *Message, settings: C_Porting_Check_Settings)`
- `check_c_porting :: (tc: Typechecked(Code_Procedure_Body), settings: C_Porting_Check_Settings)`
- `should_skip :: (node: *Code_Node, settings: C_Porting_Check_Settings) -> bool`
- `is_of_fixed_array_type :: (node: *Code_Node) -> bool`
- `do_error_checking :: (p: *Metaprogram_Plugin, message: *Message)`
- `finish_error_checking :: ()`

## Public type members

### Windows-specific type declarations

#### `C_Porting_Check_Settings` (struct)

- `skip_modules  := false`
- `check_treatment_of_arrays_as_pointers_in_procedure_calls  := true`
- `check_treatment_of_arrays_as_pointers_in_assignments      := true`
- `check_intermediate_cast_to_int_when_adding_small_integers := true`

## Behavioral contract and constraints

The module-scope entry point `get_plugin :: () -> *Metaprogram_Plugin` is the factory used by the generic metaprogram plugin loader when it imports `Check` by name. It returns the module's persistent plugin record, configures `message` to call `do_error_checking`, and clears callbacks the checker does not use. This qualified module entry point is required for Default_Metaprogram's built-in `Check` plugin; there is no separate `Check` factory type or `create_plugin` API. `CHECK_BINDINGS` is the module parameter that determines whether the message callback also verifies foreign bindings.

The plugin provides compile-time checks for common porting mistakes, including print-format argument mismatches, and exposes check routines for manual invocation as well as plugin use. Findings are reported through compiler diagnostics. Which checks run depends on plugin configuration and enabled routines; the API does not promise a complete C/C++ semantic compatibility audit.

`verify_elsewhere_binding` checks a header only when it is marked `ELSEWHERE` and its Jai name differs from its foreign name. Same-name bindings pass without parsing. For other eligible declarations, it parses the foreign name's supported platform mangling (Windows MSVC form or Unix Itanium form on Linux/macOS/iOS/Android; ARM64 Nintendo Switch 2 uses the Unix parser). It compares parsed argument and, where encoded, return types against the Jai declaration; a `CPP_METHOD` declaration omits its receiver from the compared native arguments. Mismatches and parse failures are continuable compiler diagnostics and produce `false`; a caller can therefore use the Boolean result without treating it as a lookup into a loaded library. This routine does not resolve `library_identifier` or verify that the named external symbol exists. Unsupported targets are logged once and skip mangled checking, returning success for that skipped check.

`print_binding_check_memory_stats` reports peak and cumulative in-band bytes used by the temporary mangled-type parsing pool, not process-wide or total heap memory. With `DEBUG_BINDINGS` enabled, each checked mangled binding resets/reuses that pool, measures its in-band usage, accumulates the measurement, and updates the peak. The default `DEBUG_BINDINGS` value is false, so this diagnostic accounting is inactive by default. `finish_error_checking` only prints these statistics when that debug option is enabled; it does not release or finalize a library cache or other binding resource.
