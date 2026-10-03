# System


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `Processor_Query_Type :: enum u8`
- [OS == .WINDOWS] `get_username :: () -> string, success:bool`
- [OS == .WINDOWS] `get_home_directory :: (username := "") -> string, success: bool`
- [OS == .WINDOWS] `get_machinename :: () -> string`
- [OS == .WINDOWS] `get_path_of_running_executable :: () -> string`
- [OS == .WINDOWS] `get_number_of_processors :: ($type: Processor_Query_Type = .PERFORMANCE_LOGICAL) -> count: s32`
- [OS == .WINDOWS] `get_error_value :: Windows.GetLastError`
- [OS == .WINDOWS] `get_error_string :: (error_code: OS_Error_Code) -> string`
- [OS == .WINDOWS] `get_error_value_and_string :: () -> (error_code: OS_Error_Code, description: string)`
- [else branch && OS_IS_UNIX && OS != .PS5] `get_error_value :: errno`
- [else branch && OS_IS_UNIX && OS != .PS5] `get_error_value_and_string :: () -> (error_code: OS_Error_Code, description: string)`
- [else branch && OS != .PS5 && !(OS == .NN_SWITCH2 && CPU == .X64)] `get_error_string :: (error_code: OS_Error_Code) -> string`
- [OS == .LINUX || OS == .ANDROID] `get_passwd_for_user :: (username := "") -> (success: bool, p: passwd)`
- [OS == .LINUX || OS == .ANDROID] `get_username :: () -> string, success: bool`
- [OS == .LINUX || OS == .ANDROID] `get_home_directory :: (username := "") -> string, success: bool`
- [OS == .LINUX || OS == .ANDROID] `get_machinename :: () -> string`
- [OS == .LINUX || OS == .ANDROID] `get_path_of_running_executable :: () -> string`
- [OS == .LINUX || OS == .ANDROID] `get_number_of_processors :: ($type: Processor_Query_Type = .PERFORMANCE_LOGICAL) -> count: s32 #modify`
- [OS == .MACOS] `get_username :: () -> string, success:bool`
- [OS == .MACOS] `get_home_directory :: (username := "") -> string, success: bool`
- [OS == .MACOS] `get_machinename :: () -> string`
- [OS == .MACOS] `get_path_of_running_executable :: () -> string`
- [OS == .MACOS] `get_number_of_processors :: ($type: Processor_Query_Type = .PERFORMANCE_LOGICAL) -> count: s32`
- [CPU == .X64] `read_cpu_counter :: inline () -> u64`
- [CPU == .X64] `M64 :: #import "Machine_X64"`
- [OS == .WINDOWS] `read_cpu_counter :: inline () -> u64`
- [OS == .WINDOWS] `Windows :: #import "Windows"`
- [OS_IS_UNIX] `read_cpu_counter :: inline () -> u64`


## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `Processor_Query_Type`

```jai
Processor_Query_Type :: enum u8 {
    ALL_PHYSICAL;
    ALL_LOGICAL;
    PERFORMANCE_PHYSICAL;
    PERFORMANCE_LOGICAL;
}
```
## Behavioral contract

Provides platform-selected username, home directory, machine name, executable path, processor count, and OS error details, plus CPU-counter reads where supported. Returned strings are allocated or temporary-storage-backed as indicated by each implementation; executable path is documented as temporary storage. Processor-count query type selects logical/physical/performance-oriented counts where the platform exposes them; support varies by OS and may fall back. Windows, POSIX, and Apple implementations differ, including native Objective-C calls on Apple systems. Depends on platform modules and OS APIs; do not retain temporary results past temporary-storage reset.

`get_error_value` is a zero-argument alias of `Windows.GetLastError` on Windows and therefore returns `Windows.OS_Error_Code`, a distinct `u32`-based type. On Unix targets other than PS5 it aliases the current thread's `errno` accessor and returns the POSIX `OS_Error_Code` (`s32`). The Windows and POSIX numeric domains are OS-specific and must be interpreted using the originating API. `get_error_value_and_string` pairs that value with the platform description; the string may use temporary storage and must be consumed before its storage is reset. The POSIX `errno` value is only meaningful for the most recent failing operation on the calling thread; successful calls do not necessarily clear it.

`read_cpu_counter` is a platform-selected raw counter with no common unit or frequency contract. On x64, the CPU condition takes precedence and returns the `rdtsc` value. On Windows targets not selected by that x64 branch, it returns the native `QueryPerformanceCounter` integer unchanged (no frequency conversion). The Unix fallback returns the `tv_nsec` member from a `CLOCK_MONOTONIC_RAW` read; this is not a cross-platform cycle count.
