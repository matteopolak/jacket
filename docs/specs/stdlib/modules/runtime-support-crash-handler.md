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
- [OS == .LINUX || OS == .MACOS] `handle_signal :: (sig: s32, info: *siginfo_t, secret: *void) #c_call`


## Public aggregate shapes and enum values

The Windows declarations and behavior in this module are x64-specific: `CONTEXT` is the 1232-byte record shape described below, `backtrace` reads `Rsp`, `Rbp`, and `Rip`, and stack walking uses AMD64. There is no Windows ARM64 `CONTEXT` alternative in this module. The module's Linux and macOS branches are separately conditional. Enabling this module through `Runtime_Support.ENABLE_BACKTRACE_ON_CRASH` on another target is unsupported by the handler's target branches.

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

Installs platform crash/exception handlers and captures stack information for diagnostics. Initialization must occur before failures to prepare symbol/handler state. On Windows it uses DbgHelp and x64 context/frame layouts; POSIX uses signal/action APIs. macOS handling is not guaranteed to be correct and some handler state is global rather than thread-local, so crash-path safety is limited. This is best-effort diagnostic support, not recovery or a guarantee that symbols are available. Depends on runtime stack-trace records and platform debug/signal libraries.
