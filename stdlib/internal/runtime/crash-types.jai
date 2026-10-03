size_t :: u64;
clock_t :: s64;
libexecinfo :: libc;
SIGQUIT :: 3;
SIGILL :: 4;
SIGTRAP :: 5;
SIGABRT :: 6;
SIGFPE :: 8;
SIGSEGV :: 11;

sigval_t :: union {
    sival_int: s32;
    sival_ptr: *void;
}

sigaction_t :: struct {
    sa_sigaction: (sig: s32, info: *siginfo_t, p: *void) #c_call;
    sa_mask: sigset_t;
    sa_flags: s32;
    #if OS == .LINUX sa_restorer: () #c_call;
}

#if OS == .MACOS {
    SIGBUS :: 10;
    SA_RESTART_ONSTACK_SIGINFO :: 0x0043;
    sigset_t :: u32;

    stack_t :: struct {
        ss_sp: *void;
        ss_size: u64;
        ss_flags: s32;
    }

    siginfo_t :: struct {
        si_signo: s32;
        si_errno: s32;
        si_code: s32;
        si_pid: s32;
        si_uid: u32;
        si_status: s32;
        si_addr: *void;
    }

    x64_exception_state64_t_STUB :: [16] u8;

    x64_thread_state64_t_STUB :: struct {
        gregs: [21] s64;
    }

    x64_float_state64_t_STUB :: [524] u8;

    mcontext_t :: struct {
        __es: x64_exception_state64_t_STUB;
        __ss: x64_thread_state64_t_STUB;
        __fs: x64_float_state64_t_STUB;
        padding: [4] u8;
    }

    ucontext_t :: struct {
        uc_onstack: s32;
        uc_sigmask: sigset_t;
        uc_stack: stack_t;
        uc_link: *ucontext_t;
        uc_mcsize: u64;
        uc_mcontext: *mcontext_t;
    }
} else {
    SIGBUS :: 7;
    SA_RESTART_ONSTACK_SIGINFO :: 0x18000004;

    sigset_t :: struct {
        __val: [16] u64;
    }

    siginfo_t :: struct {
        si_signo: s32;
        si_errno: s32;
        si_code: s32;
        __pad0: s32;
        si_addr: *void;
    }

    ucontext_t :: struct {
        uc_flags: u64;
        uc_link: *ucontext_t;
        ss_sp: *void;
        ss_flags: s32;
        ss_size: size_t;
        gregs: [23] s64;
    }
}

#scope_module
libc :: #system_library "libc";
