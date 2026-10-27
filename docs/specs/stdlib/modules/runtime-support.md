# Runtime_Support


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `OS_IS_UNIX :: OS == .MACOS || OS == .LINUX || OS == .PS5 || OS == .IOS || OS == .ANDROID`
- `__element_duplicate :: (start: *u8, num_elements: s64, size: s64) #c_call #no_aoc`
- `write_nonnegative_number :: (n: u64, base := 10, to_standard_error := false) #no_context #no_abc #no_aoc`
- `write_number :: (n: s64, base := 10, to_standard_error := false) #no_context #no_aoc`
- `runtime_support_assertion_failed :: (loc: Source_Code_Location, message: string) -> bool`
- `runtime_support_default_logger :: (message: string, data: *void, info: Log_Info)`
- `c_style_strlen :: (str: *u8) -> s64 #no_context`
- `to_string :: (c_string: *u8) -> string #no_context`
- `__array_bounds_check_fail :: (index: s64, limit: s64, line_number: s64, filename: *u8) #no_context`
- `__cast_bounds_check_fail :: (pre_value: s64, pre_flags: u32, post_value: s64, post_flags: u32, fatal: u8, line_number: s64, filename: *u8) #no_context`
- `__null_pointer_check_fail :: (index: s64, line_number: s64, filename: *u8) #no_context`
- `__arithmetic_overflow :: (left: s64, right: s64, type_code: u16, line_number: s64, filename: *u8) #no_context #no_aoc`
- `write_string_unsynchronized :: (s: string, to_standard_error := false) #no_context`
- `write_string :: (s: string, to_standard_error := false) #no_context #compiler`
- `write_strings :: (strings: ..string, to_standard_error := false) #no_context #compiler`
- [DEFINE_INITIALIZATION] `__jai_runtime_init :: (argc: s32, argv: **u8) -> *#Context #c_call`
- [DEFINE_INITIALIZATION] `__jai_runtime_fini :: (_context: *void) #c_call`
- `__instrumentation_first :: ()`
- `__instrumentation_second :: ()`
- `compile_time_debug_break :: () #compiler #no_context`
- `debug_break :: () #no_context`
- `one_time_init :: (synch_value: *s32, to_insert: Code) #expand`
- `Context_Base :: struct`
- `Temporary_Storage :: struct`
- [OS == .ANDROID] `android_flush_write_string_buffer :: (force := false) #no_context`
- `synch_initted: s32 = 0`


## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `Context_Base`

```jai
Context_Base :: struct {
    context_info:  *Type_Info_Struct;

    thread_index   : u32;

    allocator      := default_allocator;

    logger         := runtime_support_default_logger;
    logger_data    :  *void;
    log_source_identifier: u64;
    log_level      :  Log_Level;

    temporary_storage: *Temporary_Storage;

    stack_trace: *Stack_Trace_Node;

    assertion_failed := runtime_support_assertion_failed;
    handling_assertion_failure := false;

    default_allocator :: Allocator.{runtime_support_default_allocator_proc, null};
}
```
### `Temporary_Storage`

```jai
Temporary_Storage :: struct {
    data:     *u8;
    size:     s64;

    total_bytes_occupied: s64;
    high_water_mark: s64;
    last_set_mark_location: Source_Code_Location;

    implementation_data: [256] u8;
}
```
## Behavioral contract

`OS_IS_UNIX` is an exported compile-time boolean with the exact definition
`OS == .MACOS || OS == .LINUX || OS == .PS5 || OS == .IOS || OS == .ANDROID`.
For `Operating_System_Tag` values in Preload it is true for `LINUX`,
`ANDROID`, `IOS`, `MACOS`, and `PS5`; it is false for `NONE`, `KRAMPOS`,
`WINDOWS`, `NN_SWITCH`, `PS4`, `XBOX`, `WASM`, and `NN_SWITCH2`. This is the
module's classification used by clients that branch on `OS_IS_UNIX`; it does
not itself add filesystem procedures to Basic.

The Runtime_Support module explicitly permits a replacement implementation.
A compatible replacement must accept the public module parameters
`DEFINE_SYSTEM_ENTRY_POINT: bool`, `DEFINE_INITIALIZATION: bool`, and
`ENABLE_BACKTRACE_ON_CRASH: bool`. It must also provide any exported names
required by importing modules, including `OS_IS_UNIX` when those modules use
that condition. A missing or incompatible replacement export can change which
conditional client declarations compile. Basic's `sleep_milliseconds`,
`set_working_directory`, `get_working_directory`, and `exit` availability is
selected by Basic's platform declarations, not by `OS_IS_UNIX`: the four
procedures are present on Windows, Linux, Android, and macOS on those platforms. iOS has no corresponding Basic platform branch here; declarations
for PS5 and Nintendo Switch targets are unspecified. `OS_IS_UNIX` alone is
therefore not a compatible availability gate for these procedures.

`Runtime_Support` provides low-level output, numeric formatting, assertions, startup/initialization entry points, and platform runtime support shared by generated executables. Module parameters control whether it defines the system entry point, initialization routine, and crash backtrace support. These options must align with output type and any embedding host; duplicate entry-point definitions cause link conflicts. Assertion failure writes the source location/message to stderr and walks available stack nodes. Integer formatting clamps radix to 2–16; the signed minimum value is a documented overflow edge. Platform startup and crash behavior depends on selected OS and runtime-support build options.

`runtime_support_default_allocator_proc` is a public alias of `Default_Allocator.allocator_proc`. `Context_Base.allocator` uses that callback with null allocator data; it is not a separate allocator callback identity. Its capabilities follow the selected `Default_Allocator` target variant: optimized X64 (except iOS) and Nintendo Switch 2 support `CREATE_HEAP`; fallback variants do not support heap creation or ownership queries. On Mac ARM64 the context default does not report `Allocator_Caps.CREATE_HEAP` and does not produce a heap. This is consistent with the fallback variant and does not guarantee general assertion behavior. Clients requiring a heap must query capabilities before calling `Basic.create_heap`.

`Basic.create_heap(a)` requests `CREATE_HEAP` and returns the same callback with the new allocator-data handle. The allocator protocol defines this as a new independent heap; the input `allocator_data` is ignored for creation. `Basic.destroy_heap(b)` requests destruction of `b.data`, after which memory operations on that handle are errors. The capability set has no separate `MULTIPLE_HEAPS` bit; `CREATE_HEAP` indicates that new heap handles can be created. This does not guarantee parent/child nesting. Neither Basic nor Runtime_Support specifies cross-heap frees as valid.

`ENABLE_BACKTRACE_ON_CRASH` causes the system entry point to import and initialize `Runtime_Support_Crash_Handler` before instrumentation and the program entry point. Windows x64 and arm64 both use this public facade with their native context layouts; normal shutdown restores the previous Windows filter and symbol state through a private cleanup module. Linux and macOS use their signal implementations. To omit the handler, select `ENABLE_BACKTRACE_ON_CRASH=false` (or `.OFF` in the compiler build option). Initialization is performed only when `DEFINE_SYSTEM_ENTRY_POINT` is enabled. On Windows the system entry point also reserves 128 KiB of stack for the exception dispatcher on the main thread, whatever `ENABLE_BACKTRACE_ON_CRASH` says, so a stack overflow ends the program with `0xC00000FD` on every processor.

`__instrumentation_first()` and `__instrumentation_second()` have empty default bodies. When `DEFINE_SYSTEM_ENTRY_POINT` is true, the generated entry point calls runtime initialization, enters the first-thread context, optionally initializes the crash handler, then calls the first instrumentation hook followed by the second, and finally calls the program entry point. With no system entry point, this module does not call either hook. The bodies are places where metaprogram-added code may run before `main`; the public interface exposes no callback registration, ordering controls, or insertion API for them, so that compilation-time insertion mechanism is not specified here.

`synch_initted` is an exported mutable `s32` variable initialized to zero (`: s32 = 0`); it is runtime synchronization state, not a constant.

## Module parameters

```jai
#module_parameters(DEFINE_SYSTEM_ENTRY_POINT: bool, DEFINE_INITIALIZATION: bool, ENABLE_BACKTRACE_ON_CRASH: bool);
```
