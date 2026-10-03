# Debug

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `CRASH_HANDLER_USE_SEPARATE_THREAD :: false`
- `Exit_Code :: enum s32`
- `Crash_Handler :: #type (trace: [] string)`
- `init :: ()`
- `backtrace :: (trace: []*void, skip:=0) -> int`
- `backtrace :: (skip:=0) -> [] string`
- `free_backtrace :: (trace_symbols: [] string)`
- `backtrace_symbols :: os_backtrace_symbols`
- `translate_symbols :: backtrace_symbols`
- `break_once :: inline (loc := #caller_location) -> bool`
- `breakpoint :: () -> bool #expand #no_debug`
- `is_debugger_present :: inline () -> bool`
- `attach_to_debugger :: () -> bool`
- `output :: (text: string)`
- `abort :: (exit_code := Exit_Code.Abort)`
- `get_written_minidump_filename :: () -> string`
- `Minidump_State :: enum`
- `Assert_Response :: enum`
- `set_minidump_state :: (state: Minidump_State)`
- `set_report_mode :: (interactive : bool)`
- `enable_signal_handler :: (interactive:=false)`
- `disable_signal_handler :: ()`
- `no_assert :: (loc: Source_Code_Location, message: string)`
- `print_assert :: (loc: Source_Code_Location, message: string)`
- `trace_assert :: (loc: Source_Code_Location, message: string)`
- `break_assert :: (loc: Source_Code_Location, message: string) -> bool`
- [USE_GRAPHICS] `interactive_assert :: (loc: Source_Code_Location, message: string) -> bool`
- `is_valid_pointer :: (ptr : *void) -> bool`
- `write_minidump :: (pExceptionInfo: *EXCEPTION_POINTERS, large_dump := false) -> string`
- `write_minidump :: (pExceptionInfo: *void, large_dump := false) -> string`

### Unix-specific declarations

- [OS == .MACOS] `os_is_debugger_present :: inline () -> bool`
- [OS == .LINUX] `os_is_debugger_present :: () -> bool`
- [OS == .ANDROID || OS == .WASM] `os_is_debugger_present :: () -> bool`
- `os_attach_to_debugger :: () -> bool`
- `os_abort :: (exit_code: Exit_Code)`
- `os_enable_signal_handler :: ()`
- `os_disable_signal_handler :: ()`
- `os_backtrace :: (trace: [] *void, maxcount: int, skip := 0) -> int`
- `os_backtrace_symbols :: (trace: []*void, n: int) -> [] string`
- [USE_GRAPHICS] `os_show_assert_dialog :: (loc: Source_Code_Location, message: string) -> Assert_Response`

### Windows-specific declarations

- `os_logger :: (message: string, data: *void, info: Log_Info)`
- [OS == .WINDOWS] `os_is_debugger_present :: inline () -> bool`
- `os_attach_to_debugger :: () -> bool`
- `os_abort :: (exit_code: Exit_Code)`
- `os_enable_signal_handler :: ()`
- `os_disable_signal_handler :: ()`
- `os_backtrace :: (trace: [] *void, maxcount: int, skip:=0) -> int`
- `os_backtrace_symbols :: (trace: [] *void, n: int) -> [] string`
- `init_crash_handler_thread :: ()`
- [USE_GRAPHICS] `os_show_assert_dialog :: (loc: Source_Code_Location, message: string) -> Assert_Response`

- `crash_handler: Crash_Handler`
- `ignored_asserts : Table(Source_Code_Location, bool, hash_scl, compare_scl)`

## Public type members

#### `Exit_Code` (enum)

- `Success   :: 0`
- `Failure   :: 1`
- `Abort     :: 2`
- `Assert    :: 3`
- `Exception :: 4`

#### `Minidump_State` (enum)

```jai
Minidump_State :: enum {
    None;
    Mini;
    Full;
}
```

#### `Assert_Response` (enum)

- `Ignore :: 0`
- `Debug :: 1`
- `Abort :: 2`
## Behavioral contract and constraints

The module offers debugging helpers such as debugger attachment/breakpoints, aborts, and platform diagnostic state. These calls can affect process execution (for example, break into an attached debugger or terminate). OS-specific implementations and native errors apply; callers should not assume these operations are available or harmless on every target.

On Android and WASM, `os_is_debugger_present` is defined to return `false`; on Linux it uses the non-inline exported alias form, while macOS and Windows provide inline implementations. No guarantee is given for other target variants unless they provide the declaration through their platform module.

`Assert_Response` is a public enum defined by `Debug`, used as the result type of platform assertion dialogs. `Ignore` means dismiss and suppress that assertion location on later calls in this process; `Debug` requests debugger attachment and returns to the caller if attachment succeeds; `Abort` terminates through the assertion abort path. The Unix dialog implementation maps Retry/Ignore/Abort to those responses; the Windows implementation does the same for its native dialog. Where graphics dialogs are unavailable, the platform helper returns `Abort`.

`break_assert` prints the assertion and returns true only when debugger attachment succeeds. `interactive_assert` is available with `USE_GRAPHICS`; it reports the assertion, returns true when reporting is noninteractive, returns false for an already-ignored location, otherwise displays the platform dialog. An Ignore response is remembered by source location; Debug returns true only if attaching succeeds; Abort terminates with the assertion exit code. A default `init()` installs the module's signal/exception handling and replaces the runtime assertion callback only if that callback is still the default. On Windows it also initializes the debug-help support and installs the module logger only if the runtime logger remains default. `enable_signal_handler` and `disable_signal_handler` are idempotent state transitions; the `interactive` parameter to `enable_signal_handler` does not set report mode. Use `set_report_mode` to change the separate interactive-reporting flag. Disabling restores the prior platform handler where the implementation saves one. Unix installs handlers for SIGSEGV, SIGTRAP, SIGFPE, SIGBUS, and SIGILL and restores their saved actions on disable. Windows installs and restores the unhandled-exception filter. The module does not promise handling for other signals or exceptions, nor does `init()` establish successful minidump creation.

`set_report_mode(interactive)` is an unconditional public declaration, including on WASM; it is not inside a platform-conditional scope. It updates the module's reporting-mode flag. Its additional native error-mode side effect is Windows-specific. This declaration does not promise an interactive debugger, dialog provider, minidump support, or native ownership-output provider on WASM. A test requiring `developer_debug_output`/native ownership diagnostics must run only in modes where its real native provider is built and available; WASM should report that case unavailable, while retaining separate portable checks for explicitly supported Debug behavior such as `os_is_debugger_present() == false`.

`USE_GRAPHICS := true` is the sole Debug module parameter. It controls availability of `interactive_assert` and `os_show_assert_dialog`; on Linux it also brings in the X11 dialog dependency. On macOS, the graphics-enabled dialog helper currently returns `Abort`; it does not display a native dialog. On Linux, retry maps to `Debug`, ignore to `Ignore`, and abort to `Abort`; Windows uses the corresponding native retry/ignore/abort choices, with any unrecognized dialog result mapping to `Abort`. When graphics support is disabled, those dialog procedures are absent and `init` selects `break_assert` instead of `interactive_assert` when installing the default assertion handler.

`attach_to_debugger` returns true immediately if a debugger is already present; otherwise it delegates to the target implementation. Linux and macOS currently return false without launching a debugger. Windows reads the system postmortem debugger command, substitutes its process placeholders, and falls back to the system Visual Studio JIT debugger command when no usable registered command exists. It may ask for confirmation when graphics are enabled and the registry does not disable confirmation. It starts the debugger process, waits for that process to finish, considers exit status zero a debugger selection, and then polls for debugger attachment for up to one minute; any decline, launch failure, nonzero process result, or timeout returns false. This is a best-effort debugger launch path, not a guarantee that the requested program state can be continued at the original assertion point.

Windows minidump state defaults to `Mini`. `set_minidump_state` changes the state used by automatic unhandled-exception handling; `None` suppresses that dump, `Mini` requests a mini dump, and `Full` requests a full-memory dump. The explicit Windows `write_minidump` overloads instead map `large_dump = false` to `Mini` and `true` to `Full`, independent of the configured automatic state. They return an empty string when disabled or when file creation/writing fails; on success they return the generated dump filename in the process working directory. The filename includes the executable basename and a UTC timestamp. The `*void` overload on non-Windows targets always returns an empty string and does not create a Unix core/minidump. `get_written_minidump_filename` returns module-owned storage updated by automatic Windows exception handling with that handler's latest result (including an empty result on a failed or disabled attempt); it is initially empty, is not changed by explicit `write_minidump`, and callers must treat its string storage as borrowed and replaceable.

`init_crash_handler_thread` is declared only when the source constant `CRASH_HANDLER_USE_SEPARATE_THREAD` is true. That constant is false by default, so the function is absent from the default public surface. If a build changes the constant, the source's procedure body currently has no observable initialization effect; no separate-thread crash-handling behavior is promised.


Platform coverage note: exports for Nintendo and PS5 targets are unspecified.
