# Codex

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `collect_metrics :: (tc: Typechecked(Code_Procedure_Body)) -> (metrics: Metrics)`
- `get_file_and_module :: (node: *Code_Node) -> (file: *Message_File, import: *Message_Import)`
- `is_global_ident :: (ident: *Code_Ident) -> bool`
- `Math :: #import "Math"`
- `Header_Record :: struct`
- `Module_Record :: struct`
- `File_Record :: struct`
- `Procedure_Record :: struct`
- `Metrics :: struct`
- `Revision_Control_Type :: enum u64`
- `Codex_Help :: struct`
- `deinit :: (using help: *Codex_Help)`
- `make_help :: (memory: [] u8) -> (success: bool, help: Codex_Help, error_string: string)`
- `dump_file :: (name: string) -> bool`
- `put :: (builder: *String_Builder, x: $T) #modify`
- `put :: (builder: *String_Builder, x: string)`
- `get :: (s: *string, x: *$T) #modify`
- `get :: (s: *string, result: *string, error: *bool)`
- `put_short_string :: (builder: *String_Builder, s: string, $count_type: Type, $zero_terminated := false)`
- `get_short_string :: (input: *string, $count_type: Type, error: *bool, $allocate := true, $zero_terminated := false) -> string`
- `get_plugin :: () -> *Metaprogram_Plugin`
- `message :: (p: *Metaprogram_Plugin, message: *Message)`
- `finish :: (p: *Metaprogram_Plugin)`
- `shutdown :: (p: *Metaprogram_Plugin)`
- `communication_proc :: (_p: *Metaprogram_Plugin, key: string, value: string) -> bool`
- `init :: (p : *Metaprogram_Plugin, options: [] string) -> bool`
- `log_help :: (p: *Metaprogram_Plugin)`
- `HELP_STRING :: #string DONE`

## Public type members

#### `Header_Record` (struct)

- `magic:   u32`
- `version: u32`
- `time_generated: Apollo_Time                   #align 1`
- `revision_control_type: Revision_Control_Type  #align 1`
- `revision_number:       u64                    #align 1`
- `num_modules:    s32                           #align 1`
- `num_files:      s32                           #align 1`
- `num_procedures: s32                           #align 1`
- `reserved: [256] u8 #align 1`
- `OS: Operating_System_Tag`
- `optimization_type: Optimization_Type`
- `project_name:              Relative_String(s16)`
- `executable_name:           Relative_String(s16)`
- `url_or_generation_command: Relative_String(s16)`
- `comment:                   Relative_String(s16)`

#### `Module_Record` (struct)

- `module_type:           type_of(Message_Import.module_type) #align 1`
- `module_name:           Relative_String(s16)  #align 1`
- `fully_pathed_filename: Relative_String(s16)  #align 1`

#### `File_Record` (struct)

- `fully_pathed_filename: Relative_String(s16)  #align 1`
- `module_index:          s32         #align 1`
- `from_a_string:         bool        #align 1`

#### `Procedure_Record` (struct)

- `file_index:      s32         #align 1`
- `l0:              s32         #align 1`
- `c0:              s32         #align 1`
- `l1:              s32         #align 1`
- `c1:              s32         #align 1`
- `procedure_flags: type_of(Code_Procedure_Header.procedure_flags)  #align 1`
- `num_expressions: u32         #align 1`
- `metrics:         Metrics     #align 1`
- `name:            Relative_String(s16)  #align 1`

#### `Metrics` (struct)

- `if_points:         s32         #align 1`
- `maximum_if_depth:  s32         #align 1`
- `maximum_loop_depth:s32         #align 1`
- `total_calls:       s32         #align 1`
- `same_file_calls:   s32         #align 1`
- `same_module_calls: s32         #align 1`
- `nonconstant_calls: s32         #align 1`
- `allocations:       s32         #align 1`
- `frees:             s32         #align 1`
- `reassignments:     s32         #align 1`
- `global_reads:      s32         #align 1`
- `global_writes:     s32         #align 1`

#### `Revision_Control_Type` (enum)

- `UNKNOWN   :: 0`
- `SVN       :: 1`
- `GIT       :: 2`
- `KRAMPVS   :: 3`
- `MERCURIAL :: 4`
- `PERFORCE  :: 5`

#### `Codex_Help` (struct)

- `memory: [] u8`
- `header: *Header_Record`
- `modules:    [] *Module_Record`
- `files:      [] *File_Record`
- `procedures: [] *Procedure_Record`

## Behavioral contract and constraints

The module exports program metadata in a serialized format for consumption by external tools. Its records describe modules, files, procedures, source spans, and collected procedure metrics. The serializer and reader expose an interchange surface with a concrete current format version; the reader accepts only that version, so forward and backward compatibility are not promised.

## Serialized interchange contract

The file begins with a fixed header followed by module, file, and procedure record sections in that order. The four magic bytes are `03 00 04 05` and the version must equal `2`; the reader rejects either lower or higher version values. The header then stores generation time, revision-control metadata, three signed 32-bit record counts, 256 reserved bytes, target OS and optimization tags, and four `Relative_String(s16)` fields. The serialized header strings are physically laid out as project name, executable name, comment, then URL or generation command. Each record's inline strings immediately follow that record, and the next record begins after those string bytes and their trailing NUL bytes. The implementation hardcodes its little-endian assumption and copies scalar/record bytes without byte swapping. Big-endian behavior is unsupported; there is no reliable big-endian conversion path.

Each relative string stores a signed 16-bit byte count that excludes its NUL terminator and a signed 16-bit relative displacement. The displacement is the canonical `Relative_Pointers.Relative_String(s16)` encoding: its base is the address of the displacement field, and the stored bits use the high-bit XOR encoding so zero denotes null. The pointed-to bytes are followed by a NUL byte. Strings longer than 32767 bytes are truncated to 32767 bytes before writing; strings are copied as supplied without a character conversion. Module records carry the import kind and module name and path. File records carry a path, module index, and whether the input originated as a string. Procedure records carry file index, start/end line and column, procedure flags, expression count, `Metrics`, and procedure name. `Metrics` contains the public counts listed above; these are the values exported by `collect_metrics` for a typed procedure body.

`make_help` requires at least a full header, matching magic, and exactly version 2. It returns `success=false` and an error string for a short header, bad magic, or unsupported version. For accepted input it allocates record-pointer arrays and points each record and relative string into the caller-provided byte buffer; that buffer must remain alive and unchanged while the returned help is used. `deinit` releases those pointer arrays, not the caller's input buffer or its record/string targets; cleanup must use the allocator that owns the arrays. The reader does not validate all count, offset, or payload bounds before forming these views, so malformed/truncated payload behavior beyond the checked header cases is not a compatibility guarantee.

`get_plugin` creates a plugin instance and registers its init, message, finish, shutdown, and communication callbacks. The message callback records file/import messages in received order and retains typed procedure bodies keyed by body identity; a repeated body identity replaces the prior value rather than adding a duplicate output record. `init` accepts only `-export_file path`; an absent path is allowed and `finish` then writes `exported.codex`. A missing path argument or unknown option makes initialization fail. `communication_proc` copies values for the keys `project_name`, `comment`, and `url_or_generation_command`, and rejects other keys. `finish` writes the assembled metadata file and logs a write failure; it has no success result. `shutdown` releases plugin-owned builder, arrays, tables, and the plugin object. The header's revision-control values and reserved bytes are part of the record shape, but the writer does not promise initialized values for every metadata field. The recorded OS describes the running host and may not describe a cross-compiled target. Do not treat the optimization tag, revision metadata, or reserved bytes as authoritative when their values were not supplied.

`collect_metrics` returns counts over the typed procedure expression tree. For each conditional node it adds the current nesting depth. A non-switch `ifx` contributes one additional point; an ordinary if contributes two; each node with an else contributes one more. A switch-like node contributes one instead of the ordinary/ifx form weight. A chained `else if` adds score at its current depth but does not increase maximum conditional depth. Maximum loop depth counts nested `for` and `while` constructs. The total case-node count adds the nearest integer to its square root after the tree walk. `total_calls` includes resolved procedure calls except type instantiations and also unresolved calls; `nonconstant_calls` counts unresolved calls. Same-file and same-module counts are subsets of resolved calls. Allocation/free counts count resolved calls to Basic declarations named `alloc`, `Dynamic_New`, `NewArray`, `alloc_string`, `copy_string`, and `New` as allocations, `free` as a free, and `realloc` as both; they are not general heap instrumentation. Reassignments count assignment operators, global reads count global identifier references, and global writes count assignments whose left side is a global identifier. These fields are heuristics over the typed tree, not runtime counts or a complete semantic complexity measure.

The public serializer helpers `put` and `get` accept integer, floating-point, boolean, and enum scalar types and copy their in-memory bytes in little-endian order. Unsupported types fail the compile-time predicate; a short scalar input asserts rather than returning an error. The string `put` overload writes a signed 64-bit byte count followed by the bytes without a terminator. Its matching string `get` consumes that count, allocates storage, and copies the requested bytes. It sets the caller's error flag to true on a negative length, allocation failure, or insufficient payload, but does not reset a preexisting flag on success. On success the source assigns the allocated pointer but does not assign `result.count`, so callers cannot rely on a valid returned string count from this overload. Malformed string failure does not roll back the already-consumed count. `put_short_string` accepts signed or unsigned 8-, 16-, or 32-bit count types, stores the byte count, then payload bytes, and optionally appends a NUL not included in the count. A value too large for the selected count type fails the checked cast; it is not truncated. `get_short_string` has the same count-type and optional-NUL parameters; its source signature names an `allocate` option defaulting to true, but the body tests a differently named compile-time option, so allocation-vs-borrowed behavior is unspecified. Its short-count and short-payload paths set the error flag but do not clear a prior error on success; no stronger malformed-input cursor or output preservation guarantee is specified. A successful zero-terminated read expects the terminator byte to be present even though the body checks the payload length separately. `dump_file` reads a file, invokes `make_help`, prints its records when parsing succeeds, and returns false for file-read or header-validation failure.

Exact source formatting is irrelevant: consumers should parse the binary contract above. The magic, version, field widths/order, endianness, relative-string base, and record field meanings are compatibility requirements.
