# Runtime_Support_Crash_Handler


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- [OS == .WINDOWS] `windows_context: CONTEXT #align 32`
- [OS == .WINDOWS] `symbols_initialized := false`
- [OS == .WINDOWS] `CurrentProcess := cast(*void) -1`
- [OS == .WINDOWS] `CONTEXT :: struct`
- [OS == .WINDOWS] `ADDRESS64 :: struct`
- [OS == .WINDOWS] `STACKFRAME64 :: struct`
- [OS == .WINDOWS] `EXCEPTION_RECORD :: struct`
- [OS == .WINDOWS] `EXCEPTION_POINTERS :: struct`
- [OS == .WINDOWS] `init :: ()`
- [OS == .WINDOWS] `attach_thread :: () #c_call`
- [OS == .WINDOWS] `backtrace :: (ctx: *CONTEXT, skips: s32) #no_context`
- [OS == .WINDOWS] `is_ptr_in_program_section :: (ptr: *void) -> in_program_section: bool, read_only: bool, section_name: string #no_context`
- [OS == .WINDOWS] `handle_exception :: (using pExceptionInfo: *EXCEPTION_POINTERS) -> s32 #c_call`
- [OS == .LINUX] `SA_RESTART_ONSTACK_SIGINFO :: 0x18000004`
- [OS == .LINUX] `SIGBUS :: 7`
- [OS == .LINUX] `sigset_t :: struct`
- [OS == .LINUX] `ucontext_t :: struct`
- [OS == .LINUX] `siginfo_t :: struct`
- [OS == .MACOS] `SA_RESTART_ONSTACK_SIGINFO :: 0x0043`
- [OS == .MACOS] `SIGBUS :: 10`
- [OS == .MACOS] `sigset_t :: u32`
- [OS == .MACOS] `stack_t :: struct`
- [OS == .MACOS] `x64_exception_state64_t_STUB :: [16] u8`
- [OS == .MACOS] `x64_thread_state64_t_STUB :: struct`
- [OS == .MACOS] `x64_float_state64_t_STUB :: [524] u8`
- [OS == .MACOS] `mcontext_t :: struct`
- [OS == .MACOS] `ucontext_t :: struct`
- [OS == .MACOS] `siginfo_t :: struct`
- [OS == .LINUX || OS == .MACOS] `size_t :: u64`
- [OS == .LINUX || OS == .MACOS] `libexecinfo :: libc`
- [OS == .LINUX || OS == .MACOS] `sigval_t :: union`
- [OS == .LINUX || OS == .MACOS] `SIGQUIT :: 3`
- [OS == .LINUX || OS == .MACOS] `SIGILL :: 4`
- [OS == .LINUX || OS == .MACOS] `SIGTRAP :: 5`
- [OS == .LINUX || OS == .MACOS] `SIGABRT :: 6`
- [OS == .LINUX || OS == .MACOS] `SIGFPE :: 8`
- [OS == .LINUX || OS == .MACOS] `SIGSEGV :: 11`
- [OS == .LINUX || OS == .MACOS] `clock_t :: s64`
- [OS == .LINUX || OS == .MACOS] `sigaction_t :: struct`
- [OS == .LINUX || OS == .MACOS] `init :: ()`
- [OS == .LINUX || OS == .MACOS] `attach_thread :: () #c_call`
- [OS == .LINUX || OS == .MACOS] `handle_signal :: (sig: s32, info: *siginfo_t, secret: *void) #c_call`


## Stack overflow reporting

A stack overflow prints `stack overflow` in place of the usual reason line (`crash signal N` on Linux and macOS, `Windows exception 0x... at 0x...` on Windows), followed by the same `native stack:` trace. Exit status is unchanged.

- Linux and macOS: signal handlers run on a per-thread alternate stack (`sigaltstack`, `SA_ONSTACK`). `init` sets it up for the calling thread and `Thread` calls `attach_thread` for each thread it creates. A `SIGSEGV`/`SIGBUS` whose fault address lies within 256 KiB below, or 64 KiB above, the thread's recorded stack limit counts as an overflow; the extent is read when the thread is attached, not in the handler. Change `OVERFLOW_WINDOW` in `crash-native/runtime.c` to widen it.
- Windows: `EXCEPTION_STACK_OVERFLOW` is reported the same way. `init` and `attach_thread` call `SetThreadStackGuarantee` for 128 KiB so the filter has room to walk the stack.

`attach_thread` does nothing until `init` has run, so programs built with `ENABLE_BACKTRACE_ON_CRASH` off are unaffected. On Windows the reserve itself does not depend on the handler: the system entry point and every `Thread` reserve 128 KiB (`internal/runtime/windows-stack-guarantee.jai`) even with the handler off, because the default room is too small for the exception dispatcher on processors with large extended register state (AVX-512, AMX), where an overflow would otherwise end the program with an access violation (`0xC0000005`) instead of a stack overflow (`0xC00000FD`). Fibers of `Extensions/Io` run on their own stacks and fault on their guard pages by design. Tests: `tests/driven/faults/stack-overflow*.jai`, run by `tools/portable/check_portable_stdlib.py`.

## Public aggregate shapes and enum values

The Windows facade supports x64 and arm64. x64 retains its 1232-byte context; arm64 wraps the SDK-shaped `Windows.CONTEXT` binding (912 bytes, alignment 16), including `Fp`, `Sp`, and `Pc`. Both use the same initialization, exception, backtrace, and executable-section API. The Linux and macOS branches remain separately conditional.

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `ADDRESS64`

```jai
ADDRESS64 :: struct {
        Offset: u64;
        Segment: u16;
        Mode: s32;
    }
```
### `CONTEXT`

x64:

```jai
CONTEXT :: struct {
        using members: struct {
            before_rax: [152] u8 = ---;
            Rsp: u64;
            Rbp: u64;
            other: [10] u64;

            Rip: u64;
        }

        padding: [1232 - size_of(type_of(members))] u8 = ---;
    }
```
arm64:

```jai
CONTEXT :: struct #align 16 {
    using members: Windows.CONTEXT;
}
```

### `EXCEPTION_POINTERS`

```jai
EXCEPTION_POINTERS :: struct {
        ExceptionRecord: *EXCEPTION_RECORD;
        ContextRecord:   *CONTEXT;
    }
```
### `EXCEPTION_RECORD`

```jai
EXCEPTION_RECORD :: struct {
        ExceptionCode: u32;
        ExceptionFlags: u32;
        ExceptionRecord: *EXCEPTION_RECORD;
        ExceptionAddress: *void;
        NumberParameters: u32;
        ExceptionInformation: [15]u64;
    }
```
### `STACKFRAME64`

```jai
STACKFRAME64 :: struct {
        using members: struct {
            AddrPC:     ADDRESS64;
            AddrReturn: ADDRESS64;
            AddrFrame:  ADDRESS64;
            AddrStack:  ADDRESS64;
        }

        padding: [264 - size_of(type_of(members))] u8;
    }
```
### `mcontext_t`

macOS:

```jai
mcontext_t :: struct {
            __es: x64_exception_state64_t_STUB;
            __ss: x64_thread_state64_t_STUB;
            __fs: x64_float_state64_t_STUB;
            padding: [4] u8;
        }
```
### `sigaction_t`

```jai
sigaction_t :: struct {
        sa_sigaction: (sig: s32, info: *siginfo_t, p: *void) #c_call;
        sa_mask:  sigset_t;
        sa_flags: s32;

        #if OS == .LINUX then sa_restorer: () #c_call;
    }
```
### `siginfo_t`

Variant 1 (Linux):

```jai
siginfo_t :: struct {
            si_signo: s32;
            si_errno: s32;
            si_code: s32;
            __pad0: s32;
            si_addr: *void;
        }
```
Variant 2 (macOS):

```jai
siginfo_t :: struct {
            si_signo:  s32;
            si_errno:  s32;
            si_code:   s32;
            si_pid:    s32;
            si_uid:    u32;
            si_status: s32;
            si_addr:   *void;
        }
```
### `sigset_t`

Linux:

```jai
sigset_t :: struct { __val: [16] u64; }
```
macOS:

```jai
sigset_t :: u32
```
### `sigval_t`

```jai
sigval_t :: union {
        sival_int: s32;
        sival_ptr: *void;
    }
```
### `stack_t`

macOS:

```jai
stack_t :: struct {
            ss_sp:   *void;
            ss_size:  u64;
            ss_flags: s32;
        }
```
### `ucontext_t`

Variant 1 (Linux):

```jai
ucontext_t :: struct {
            uc_flags: u64;
            uc_link: *ucontext_t;
            ss_sp:   *void;
            ss_flags: s32;
            ss_size:  size_t;
            gregs: [23] s64;
        }
```
Variant 2 (macOS):

```jai
ucontext_t :: struct {
            uc_onstack: s32;
            uc_sigmask: sigset_t;
            uc_stack: stack_t;
            uc_link: *ucontext_t;
            uc_mcsize: u64;
            uc_mcontext: *mcontext_t;
        }
```
### `x64_thread_state64_t_STUB`

macOS:

```jai
x64_thread_state64_t_STUB :: struct {
            gregs: [21] s64;
        }
```
## Behavioral contract

Installs native crash/exception handlers and captures stack information for diagnostics. Initialization prepares a private DbgHelp symbol session so it does not collide with another component's session. Windows x64 walks with `StackWalk64`; arm64 uses `RtlVirtualUnwind2`, resolved from `ntdll` (Windows 11/Server 2022 or later). The arm64 walker validates native stack bounds; both walkers cap traversal and detect repeated frames. Public context bytes are copied into aligned SDK storage before walking. Symbol lookup is serialized, and an exception uses a nonblocking lock attempt to avoid deadlocking on an interrupted symbol operation. Diagnostic support is best effort, not recovery.

Both Windows CPUs share `stdlib/internal/runtime/crash-native/runtime-windows.c`. Edit CPU-specific traversal there and public records in `windows-crash-types.jai`; preserve the existing x64 layout. `tools/stdlib/build_platform_io_runtime.py --stdlib stdlib --cpu x64|arm64` builds the matching DLL and optionally its static archive. The native SDK fixture checks context sizes/offsets independently, and the platform-I/O suite exercises initialization, filter restoration, executable sections, and actual faults at O0/O2 on Windows. Dependencies are the Windows SDK, DbgHelp, kernel32 and (arm64) the native unwind API. `Runtime_Support.ENABLE_BACKTRACE_ON_CRASH` controls startup installation; normal shutdown uses the private cleanup module to restore prior process state.
