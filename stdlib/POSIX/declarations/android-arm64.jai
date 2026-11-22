// Generated from docs/specs/stdlib/modules/posix.md.
// Regenerate with tools/native_bindings/generate.py; do not hand-edit.

// The declarations for `OS == .ANDROID && CPU == .ARM64`, loaded by declarations.jai.

#scope_file
U128 :: [2] u64;

#if OS == .MACOS || OS == .IOS {
    binding_errno_address :: () -> *s32 #foreign libc "__error";
} else #if OS == .ANDROID {
    binding_errno_address :: () -> *s32 #foreign libc "__errno";
} else {
    binding_errno_address :: () -> *s32 #foreign libc "__errno_location";
}

libc :: #library,system "c"; @native(external)
libdl :: #library,system "dl"; @native(external)
libpthread :: #library,system "pthread"; @native(external)
librt :: #library,system "librt"; @native(external)
#scope_export

__kernel_fd_set :: struct {
    fds_bits: [16] u64;
}

__kernel_fsid_t :: struct {
    val: [2] s32;
}

__kernel_sigaction :: struct {
    sa_handler: #type (unknown0: s32) -> void #c_call;
    sa_flags: u64;

    sa_restorer: #type () -> void #c_call;

    sa_mask: sigset_t;
}

__locale_t :: struct {}

__pthread_cleanup_t :: struct {
    __cleanup_prev: *__pthread_cleanup_t;
    __cleanup_routine: #type () -> void #c_call;
    __cleanup_arg: *void;
}

__sFILE :: struct {}

__sifields :: struct {
    _sigchld: struct {
        _pid: s32;
        _uid: u32;
        _status: s32;
        _utime: s64;
        _stime: s64;
    };

    #overlay (_sigchld) _kill: struct {
        _pid: s32;
        _uid: u32;
    };
    #overlay (_sigchld) _timer: struct {
        _tid: s32;
        _overrun: s32;
        _sigval: sigval_t;
        _sys_private: s32;
    };
    #overlay (_sigchld) _rt: struct {
        _pid: s32;
        _uid: u32;
        _sigval: sigval_t;
    };
    #overlay (_sigchld) _sigfault: struct {
        _addr: *void;

        struct {
            _addr_bnd: struct {
                _dummy_bnd: [8] u8;
                _lower: *void;
                _upper: *void;
            };

            #overlay (_addr_bnd) _trapno: s32;
            #overlay (_addr_bnd) _addr_lsb: s16;
            #overlay (_addr_bnd) _addr_pkey: struct {
                _dummy_pkey: [8] u8;
                _pkey: u32;
            };
            #overlay (_addr_bnd) _perf: struct {
                _data: u64;
                _type: u32;
                _flags: u32;
            };
        }
    };
    #overlay (_sigchld) _sigpoll: struct {
        _band: s64;
        _fd: s32;
    };
    #overlay (_sigchld) _sigsys: struct {
        _call_addr: *void;
        _syscall: s32;
        _arch: u32;
    };
}

#if (OS == .ANDROID && CPU == .ARM64) || (OS == .ANDROID && CPU == .X64) {
    using _SC_definitions :: enum s32 {
        _SC_ARG_MAX :: 0x0000;
        _SC_BC_BASE_MAX :: 0x0001;
        _SC_BC_DIM_MAX :: 0x0002;
        _SC_BC_SCALE_MAX :: 0x0003;
        _SC_BC_STRING_MAX :: 0x0004;
        _SC_CHILD_MAX :: 0x0005;
        _SC_CLK_TCK :: 0x0006;
        _SC_COLL_WEIGHTS_MAX :: 0x0007;
        _SC_EXPR_NEST_MAX :: 0x0008;
        _SC_LINE_MAX :: 0x0009;
        _SC_NGROUPS_MAX :: 0x000a;
        _SC_OPEN_MAX :: 0x000b;
        _SC_PASS_MAX :: 0x000c;
        _SC_2_C_BIND :: 0x000d;
        _SC_2_C_DEV :: 0x000e;
        _SC_2_C_VERSION :: 0x000f;
        _SC_2_CHAR_TERM :: 0x0010;
        _SC_2_FORT_DEV :: 0x0011;
        _SC_2_FORT_RUN :: 0x0012;
        _SC_2_LOCALEDEF :: 0x0013;
        _SC_2_SW_DEV :: 0x0014;
        _SC_2_UPE :: 0x0015;
        _SC_2_VERSION :: 0x0016;
        _SC_JOB_CONTROL :: 0x0017;
        _SC_SAVED_IDS :: 0x0018;
        _SC_VERSION :: 0x0019;
        _SC_RE_DUP_MAX :: 0x001a;
        _SC_STREAM_MAX :: 0x001b;
        _SC_TZNAME_MAX :: 0x001c;
        _SC_XOPEN_CRYPT :: 0x001d;
        _SC_XOPEN_ENH_I18N :: 0x001e;
        _SC_XOPEN_SHM :: 0x001f;
        _SC_XOPEN_VERSION :: 0x0020;
        _SC_XOPEN_XCU_VERSION :: 0x0021;
        _SC_XOPEN_REALTIME :: 0x0022;
        _SC_XOPEN_REALTIME_THREADS :: 0x0023;
        _SC_XOPEN_LEGACY :: 0x0024;
        _SC_ATEXIT_MAX :: 0x0025;
        _SC_IOV_MAX :: 0x0026;
        _SC_UIO_MAXIOV :: _SC_IOV_MAX;
        _SC_PAGESIZE :: 0x0027;
        _SC_PAGE_SIZE :: 0x0028;
        _SC_XOPEN_UNIX :: 0x0029;
        _SC_XBS5_ILP32_OFF32 :: 0x002a;
        _SC_XBS5_ILP32_OFFBIG :: 0x002b;
        _SC_XBS5_LP64_OFF64 :: 0x002c;
        _SC_XBS5_LPBIG_OFFBIG :: 0x002d;
        _SC_AIO_LISTIO_MAX :: 0x002e;
        _SC_AIO_MAX :: 0x002f;
        _SC_AIO_PRIO_DELTA_MAX :: 0x0030;
        _SC_DELAYTIMER_MAX :: 0x0031;
        _SC_MQ_OPEN_MAX :: 0x0032;
        _SC_MQ_PRIO_MAX :: 0x0033;
        _SC_RTSIG_MAX :: 0x0034;
        _SC_SEM_NSEMS_MAX :: 0x0035;
        _SC_SEM_VALUE_MAX :: 0x0036;
        _SC_SIGQUEUE_MAX :: 0x0037;
        _SC_TIMER_MAX :: 0x0038;
        _SC_ASYNCHRONOUS_IO :: 0x0039;
        _SC_FSYNC :: 0x003a;
        _SC_MAPPED_FILES :: 0x003b;
        _SC_MEMLOCK :: 0x003c;
        _SC_MEMLOCK_RANGE :: 0x003d;
        _SC_MEMORY_PROTECTION :: 0x003e;
        _SC_MESSAGE_PASSING :: 0x003f;
        _SC_PRIORITIZED_IO :: 0x0040;
        _SC_PRIORITY_SCHEDULING :: 0x0041;
        _SC_REALTIME_SIGNALS :: 0x0042;
        _SC_SEMAPHORES :: 0x0043;
        _SC_SHARED_MEMORY_OBJECTS :: 0x0044;
        _SC_SYNCHRONIZED_IO :: 0x0045;
        _SC_TIMERS :: 0x0046;
        _SC_GETGR_R_SIZE_MAX :: 0x0047;
        _SC_GETPW_R_SIZE_MAX :: 0x0048;
        _SC_LOGIN_NAME_MAX :: 0x0049;
        _SC_THREAD_DESTRUCTOR_ITERATIONS :: 0x004a;
        _SC_THREAD_KEYS_MAX :: 0x004b;
        _SC_THREAD_STACK_MIN :: 0x004c;
        _SC_THREAD_THREADS_MAX :: 0x004d;
        _SC_TTY_NAME_MAX :: 0x004e;

        _SC_THREADS :: 0x004f;
        _SC_THREAD_ATTR_STACKADDR :: 0x0050;
        _SC_THREAD_ATTR_STACKSIZE :: 0x0051;
        _SC_THREAD_PRIORITY_SCHEDULING :: 0x0052;
        _SC_THREAD_PRIO_INHERIT :: 0x0053;
        _SC_THREAD_PRIO_PROTECT :: 0x0054;
        _SC_THREAD_SAFE_FUNCTIONS :: 0x0055;

        _SC_NPROCESSORS_CONF :: 0x0060;
        _SC_NPROCESSORS_ONLN :: 0x0061;
        _SC_PHYS_PAGES :: 0x0062;
        _SC_AVPHYS_PAGES :: 0x0063;
        _SC_MONOTONIC_CLOCK :: 0x0064;

        _SC_2_PBS :: 0x0065;
        _SC_2_PBS_ACCOUNTING :: 0x0066;
        _SC_2_PBS_CHECKPOINT :: 0x0067;
        _SC_2_PBS_LOCATE :: 0x0068;
        _SC_2_PBS_MESSAGE :: 0x0069;
        _SC_2_PBS_TRACK :: 0x006a;
        _SC_ADVISORY_INFO :: 0x006b;
        _SC_BARRIERS :: 0x006c;
        _SC_CLOCK_SELECTION :: 0x006d;
        _SC_CPUTIME :: 0x006e;
        _SC_HOST_NAME_MAX :: 0x006f;
        _SC_IPV6 :: 0x0070;
        _SC_RAW_SOCKETS :: 0x0071;
        _SC_READER_WRITER_LOCKS :: 0x0072;
        _SC_REGEXP :: 0x0073;
        _SC_SHELL :: 0x0074;
        _SC_SPAWN :: 0x0075;
        _SC_SPIN_LOCKS :: 0x0076;
        _SC_SPORADIC_SERVER :: 0x0077;
        _SC_SS_REPL_MAX :: 0x0078;
        _SC_SYMLOOP_MAX :: 0x0079;
        _SC_THREAD_CPUTIME :: 0x007a;
        _SC_THREAD_PROCESS_SHARED :: 0x007b;
        _SC_THREAD_ROBUST_PRIO_INHERIT :: 0x007c;
        _SC_THREAD_ROBUST_PRIO_PROTECT :: 0x007d;
        _SC_THREAD_SPORADIC_SERVER :: 0x007e;
        _SC_TIMEOUTS :: 0x007f;
        _SC_TRACE :: 0x0080;
        _SC_TRACE_EVENT_FILTER :: 0x0081;
        _SC_TRACE_EVENT_NAME_MAX :: 0x0082;
        _SC_TRACE_INHERIT :: 0x0083;
        _SC_TRACE_LOG :: 0x0084;
        _SC_TRACE_NAME_MAX :: 0x0085;
        _SC_TRACE_SYS_MAX :: 0x0086;
        _SC_TRACE_USER_EVENT_MAX :: 0x0087;
        _SC_TYPED_MEMORY_OBJECTS :: 0x0088;
        _SC_V7_ILP32_OFF32 :: 0x0089;
        _SC_V7_ILP32_OFFBIG :: 0x008a;
        _SC_V7_LP64_OFF64 :: 0x008b;
        _SC_V7_LPBIG_OFFBIG :: 0x008c;
        _SC_XOPEN_STREAMS :: 0x008d;
        _SC_XOPEN_UUCP :: 0x008e;

        _SC_LEVEL1_ICACHE_SIZE :: 0x008f;
        _SC_LEVEL1_ICACHE_ASSOC :: 0x0090;
        _SC_LEVEL1_ICACHE_LINESIZE :: 0x0091;
        _SC_LEVEL1_DCACHE_SIZE :: 0x0092;
        _SC_LEVEL1_DCACHE_ASSOC :: 0x0093;
        _SC_LEVEL1_DCACHE_LINESIZE :: 0x0094;
        _SC_LEVEL2_CACHE_SIZE :: 0x0095;
        _SC_LEVEL2_CACHE_ASSOC :: 0x0096;
        _SC_LEVEL2_CACHE_LINESIZE :: 0x0097;
        _SC_LEVEL3_CACHE_SIZE :: 0x0098;
        _SC_LEVEL3_CACHE_ASSOC :: 0x0099;
        _SC_LEVEL3_CACHE_LINESIZE :: 0x009a;
        _SC_LEVEL4_CACHE_SIZE :: 0x009b;
        _SC_LEVEL4_CACHE_ASSOC :: 0x009c;
        _SC_LEVEL4_CACHE_LINESIZE :: 0x009d;
    }
}

#if (OS == .ANDROID && CPU == .ARM64) || (OS == .ANDROID && CPU == .X64) {
    clockid_t :: enum u32 {
        REALTIME :: 0;
        MONOTONIC :: 1;
        PROCESS_CPUTIME_ID :: 2;
        THREAD_CPUTIME_ID :: 3;
        MONOTONIC_RAW :: 4;
        REALTIME_COARSE :: 5;
        MONOTONIC_COARSE :: 6;
        BOOTTIME :: 7;
        REALTIME_ALARM :: 8;
        BOOTTIME_ALARM :: 9;
        SGI_CYCLE :: 10;
        TAI :: 11;

        CLOCK_REALTIME :: REALTIME;
        CLOCK_MONOTONIC :: MONOTONIC;
        CLOCK_PROCESS_CPUTIME_ID :: PROCESS_CPUTIME_ID;
        CLOCK_THREAD_CPUTIME_ID :: THREAD_CPUTIME_ID;
        CLOCK_MONOTONIC_RAW :: MONOTONIC_RAW;
        CLOCK_REALTIME_COARSE :: REALTIME_COARSE;
        CLOCK_MONOTONIC_COARSE :: MONOTONIC_COARSE;
        CLOCK_BOOTTIME :: BOOTTIME;
        CLOCK_REALTIME_ALARM :: REALTIME_ALARM;
        CLOCK_BOOTTIME_ALARM :: BOOTTIME_ALARM;
        CLOCK_SGI_CYCLE :: SGI_CYCLE;
        CLOCK_TAI :: TAI;
    }
}

clone_args :: struct {
    flags: u64;
    pidfd: u64;
    child_tid: u64;
    parent_tid: u64;
    exit_signal: u64;
    stack: u64;
    stack_size: u64;
    tls: u64;
    set_tid: u64;
    set_tid_size: u64;
    cgroup: u64;
}

#if (OS == .ANDROID && CPU == .ARM64) || (OS == .ANDROID && CPU == .X64) {
    cpu_set_t :: struct {
        __bits: [16] u64;
    }
}

#if (OS == .ANDROID && CPU == .ARM64) || (OS == .ANDROID && CPU == .X64) {
    DIR :: struct {}
}

#if (OS == .ANDROID && CPU == .ARM64) || (OS == .ANDROID && CPU == .X64) {
    dirent :: struct {
        d_ino: ino_t;
        d_off: s64;
        d_reclen: u16;
        d_type: DT;
        d_name: [256] u8;
    }
}

#if (OS == .ANDROID && CPU == .ARM64) || (OS == .ANDROID && CPU == .X64) {
    dirent64 :: struct {
        d_ino: ino_t;
        d_off: s64;
        d_reclen: u16;
        d_type: DT;
        d_name: [256] u8;
    }
}

#if (OS == .ANDROID && CPU == .ARM64) || (OS == .ANDROID && CPU == .X64) {
    Dl_info :: struct {

        dli_fname: *u8;

        dli_fbase: *void;

        dli_sname: *u8;

        dli_saddr: *void;
    }
}

#if (OS == .ANDROID && CPU == .ARM64) || (OS == .ANDROID && CPU == .X64) {
    DT :: enum u8 {
        UNKNOWN :: 0;

        FIFO :: 1;

        CHR :: 2;

        DIR :: 4;

        BLK :: 6;

        REG :: 8;

        LNK :: 10;

        SOCK :: 12;
        WHT :: 14;
    }
}

#if (OS == .ANDROID && CPU == .ARM64) || (OS == .ANDROID && CPU == .X64) {
    f_owner_ex :: struct {
        type: s32;
        pid: s32;
    }
}

iovec :: struct {
    iov_base: *void;
    iov_len: u64;
}

ipc_kludge :: struct {
    msgbuf :: struct {}
    msgp: *msgbuf;
    msgtyp: s64;
}

itimerspec :: struct {
    it_interval: timespec;
    it_value: timespec;
}

itimerval :: struct {
    it_interval: timeval;
    it_value: timeval;
}

#if (OS == .ANDROID && CPU == .ARM64) || (OS == .ANDROID && CPU == .X64) {
    passwd :: struct {
        pw_name: *u8;
        pw_passwd: *u8;
        pw_uid: uid_t;
        pw_gid: gid_t;

        pw_gecos: *u8;

        pw_dir: *u8;
        pw_shell: *u8;
    }
}

#if (OS == .ANDROID && CPU == .ARM64) || (OS == .ANDROID && CPU == .X64) {
    PC :: enum u32 {
        FILESIZEBITS :: 0;
        LINK_MAX :: 1;
        MAX_CANON :: 2;
        MAX_INPUT :: 3;
        NAME_MAX :: 4;
        PATH_MAX :: 5;
        PIPE_BUF :: 6;
        _2_SYMLINKS :: 7;
        ALLOC_SIZE_MIN :: 8;
        REC_INCR_XFER_SIZE :: 9;
        REC_MAX_XFER_SIZE :: 10;
        REC_MIN_XFER_SIZE :: 11;
        REC_XFER_ALIGN :: 12;
        SYMLINK_MAX :: 13;
        CHOWN_RESTRICTED :: 14;
        NO_TRUNC :: 15;
        VDISABLE :: 16;
        ASYNC_IO :: 17;
        PRIO_IO :: 18;
        SYNC_IO :: 19;
    }
}

pollfd :: struct {
    fd: s32;
    events: s16;
    revents: s16;
}

#if (OS == .ANDROID && CPU == .ARM64) || (OS == .ANDROID && CPU == .X64) {
    PRIO :: enum s32 {
        PROCESS :: 0;
        PGRP :: 1;
        USER :: 2;
    }
}

#if (OS == .ANDROID && CPU == .ARM64) || (OS == .ANDROID && CPU == .X64) {
    pthread_attr_t :: struct {
        flags: u32;
        stack_base: *void;
        stack_size: u64;
        guard_size: u64;
        sched_policy: s32;
        sched_priority: s32;

        __reserved: [16] u8;
    }
}

#if (OS == .ANDROID && CPU == .ARM64) || (OS == .ANDROID && CPU == .X64) {
    pthread_barrier_t :: struct {
        __private: [4] s64;
    }
}

#if (OS == .ANDROID && CPU == .ARM64) || (OS == .ANDROID && CPU == .X64) {
    pthread_cond_t :: struct {
        __private: [12] s32;
    }
}

#if (OS == .ANDROID && CPU == .ARM64) || (OS == .ANDROID && CPU == .X64) {
    PTHREAD_MUTEX :: enum u32 {
        NORMAL :: 0;
        RECURSIVE :: 1;
        ERRORCHECK :: 2;

        ERRORCHECK_NP :: 2;
        RECURSIVE_NP :: 1;

        DEFAULT :: 0;

        PTHREAD_MUTEX_NORMAL :: NORMAL;
        PTHREAD_MUTEX_RECURSIVE :: RECURSIVE;
        PTHREAD_MUTEX_ERRORCHECK :: ERRORCHECK;

        PTHREAD_MUTEX_ERRORCHECK_NP :: ERRORCHECK_NP;
        PTHREAD_MUTEX_RECURSIVE_NP :: RECURSIVE_NP;

        PTHREAD_MUTEX_DEFAULT :: DEFAULT;
    }
}

#if (OS == .ANDROID && CPU == .ARM64) || (OS == .ANDROID && CPU == .X64) {
    pthread_mutex_t :: struct {
        __private: [10] s32;
    }
}

PTHREAD_RWLOCK_PREFER :: enum u32 {
    READER_NP :: 0;
    WRITER_NONRECURSIVE_NP :: 1;

    PTHREAD_RWLOCK_PREFER_READER_NP :: READER_NP;
    PTHREAD_RWLOCK_PREFER_WRITER_NONRECURSIVE_NP :: WRITER_NONRECURSIVE_NP;
}

#if (OS == .ANDROID && CPU == .ARM64) || (OS == .ANDROID && CPU == .X64) {
    pthread_rwlock_t :: struct {
        __private: [14] s32;
    }
}

pthread_spinlock_t :: struct {
    __private: s64;
}

#if (OS == .ANDROID && CPU == .ARM64) || (OS == .ANDROID && CPU == .X64) {
    rlimit :: struct {
        rlim_cur: u64;
        rlim_max: u64;
    }
}

#if (OS == .ANDROID && CPU == .ARM64) || (OS == .ANDROID && CPU == .X64) {
    rlimit64 :: struct {
        rlim_cur: u64;
        rlim_max: u64;
    }
}

#if (OS == .ANDROID && CPU == .ARM64) || (OS == .ANDROID && CPU == .X64) {
    RLIMIT_RESOURCE :: enum s32 {
        CPU :: 0;
        FSIZE :: 1;
        DATA :: 2;
        STACK :: 3;
        CORE :: 4;

        RSS :: 5;

        NPROC :: 6;

        NOFILE :: 7;

        MEMLOCK :: 8;

        AS :: 9;

        LOCKS :: 10;
        SIGPENDING :: 11;
        MSGQUEUE :: 12;
        NICE :: 13;
        RTPRIO :: 14;
        RTTIME :: 15;
    }
}

#if (OS == .ANDROID && CPU == .ARM64) || (OS == .ANDROID && CPU == .X64) {
    RUSAGE :: enum s32 {
        SELF :: 0;
        CHILDREN :: -1;
        BOTH :: -2;
        THREAD :: 1;
    }
}

#if (OS == .ANDROID && CPU == .ARM64) || (OS == .ANDROID && CPU == .X64) {
    rusage :: struct {
        ru_utime: timeval;
        ru_stime: timeval;
        ru_maxrss: s64;
        ru_ixrss: s64;
        ru_idrss: s64;
        ru_isrss: s64;
        ru_minflt: s64;
        ru_majflt: s64;
        ru_nswap: s64;
        ru_inblock: s64;
        ru_oublock: s64;
        ru_msgsnd: s64;
        ru_msgrcv: s64;
        ru_nsignals: s64;
        ru_nvcsw: s64;
        ru_nivcsw: s64;
    }
}

sched_param :: struct {
    sched_priority: s32;
}

#if (OS == .ANDROID && CPU == .ARM64) || (OS == .ANDROID && CPU == .X64) {
    sem_t :: struct {
        count: u32;

        __reserved: [3] s32;
    }
}

shm_info :: struct {
    used_ids: s32;
    shm_tot: u64;
    shm_rss: u64;
    shm_swp: u64;
    swap_attempts: u64;
    swap_successes: u64;
}

#if (OS == .ANDROID && CPU == .ARM64) || (OS == .ANDROID && CPU == .X64) {
    shminfo :: struct {
        shmmax: s32;
        shmmin: s32;
        shmmni: s32;
        shmseg: s32;
        shmall: s32;
    }
}

sigaction64_t :: struct {
    sa_flags: s32;
    struct {
        sa_handler: sighandler_t;

        #overlay (sa_handler) sa_sigaction: #type (unknown0: s32, unknown1: *siginfo, unknown2: *void) -> void #c_call;
    }
    sa_mask: sigset_t;
    sa_restorer: #type () -> void #c_call;
}

#if (OS == .ANDROID && CPU == .ARM64) || (OS == .ANDROID && CPU == .X64) {
    sigaction_t :: struct {
        sa_flags: s32;
        struct {
            sa_handler: sighandler_t;

            #overlay (sa_handler) sa_sigaction: #type (unknown0: s32, unknown1: *siginfo, unknown2: *void) -> void #c_call;
        }
        sa_mask: sigset_t;
        sa_restorer: #type () -> void #c_call;
    }
}

sigaltstack_t :: struct {
    ss_sp: *void;
    ss_flags: s32;
    ss_size: u64;
}

sigcontext :: struct {
    fault_address: u64;
    regs: [31] u64;
    sp: u64;
    pc: u64;
    pstate: u64;
    __reserved: [4096] u8 #align 16;
}

#if (OS == .ANDROID && CPU == .ARM64) || (OS == .ANDROID && CPU == .X64) {
    sigevent :: struct {
        sigev_value: sigval_t;
        sigev_signo: s32;
        sigev_notify: s32;
        _sigev_un: struct {
            _pad: [12] s32;

            #overlay (_pad) _tid: s32;
            #overlay (_pad) _sigev_thread: struct {
                _function: #type (unknown0: sigval_t) -> void #c_call;
                _attribute: *void;
            };
        };
    }
}

siginfo :: struct {
    struct {
        _si_pad: [32] s32;

        #overlay (_si_pad) using anon_struct: struct {
            si_signo: s32;
            si_errno: s32;
            si_code: s32;
            _sifields: __sifields;
        };
    }
}

signalfd_siginfo :: struct {
    ssi_signo: u32;
    ssi_errno: s32;
    ssi_code: s32;
    ssi_pid: u32;
    ssi_uid: u32;
    ssi_fd: s32;
    ssi_tid: u32;
    ssi_band: u32;
    ssi_overrun: u32;
    ssi_trapno: u32;
    ssi_status: s32;
    ssi_int: s32;
    ssi_ptr: u64;
    ssi_utime: u64;
    ssi_stime: u64;
    ssi_addr: u64;
    ssi_addr_lsb: u16;
    __pad2: u16;
    ssi_syscall: s32;
    ssi_call_addr: u64;
    ssi_arch: u32;
    __pad: [28] u8;
}

sigset_t :: struct {
    sig: [1] u64;
}

sigval :: struct {
    sival_ptr: *void;

    #overlay (sival_ptr) sival_int: s32;
}

stat64_t :: struct {
    st_dev: dev_t;
    st_ino: ino_t;
    st_mode: mode_t;
    st_nlink: nlink_t;
    st_uid: uid_t;
    st_gid: gid_t;
    st_rdev: dev_t;
    __pad1: u64;
    st_size: s64;
    st_blksize: s32;
    __pad2: s32;
    st_blocks: s64;
    st_atime: timespec;
    st_mtime: timespec;
    st_ctime: timespec;
    __unused4: u32;
    __unused5: u32;
}

stat_t :: struct {
    st_dev: dev_t;
    st_ino: ino_t;
    st_mode: mode_t;
    st_nlink: nlink_t;
    st_uid: uid_t;
    st_gid: gid_t;
    st_rdev: dev_t;
    __pad1: u64;
    st_size: s64;
    st_blksize: s32;
    __pad2: s32;
    st_blocks: s64;
    st_atime: timespec;
    st_mtime: timespec;
    st_ctime: timespec;
    __unused4: u32;
    __unused5: u32;
}

#if (OS == .ANDROID && CPU == .ARM64) || (OS == .ANDROID && CPU == .X64) {
    statx_t :: struct {
        stx_mask: u32;
        stx_blksize: u32;
        stx_attributes: u64;
        stx_nlink: u32;
        stx_uid: u32;
        stx_gid: u32;
        stx_mode: u16;
        __spare0: [1] u16;
        stx_ino: u64;
        stx_size: u64;
        stx_blocks: u64;
        stx_attributes_mask: u64;
        stx_atime: statx_timestamp;
        stx_btime: statx_timestamp;
        stx_ctime: statx_timestamp;
        stx_mtime: statx_timestamp;
        stx_rdev_major: u32;
        stx_rdev_minor: u32;
        stx_dev_major: u32;
        stx_dev_minor: u32;
        stx_mnt_id: u64;
        stx_dio_mem_align: u32;
        stx_dio_offset_align: u32;
        __spare3: [12] u64;
    }
}

statx_timestamp :: struct {
    tv_sec: s64;
    tv_nsec: u32;
    __reserved: s32;
}

#if (OS == .ANDROID && CPU == .ARM64) || (OS == .ANDROID && CPU == .X64) {
    timespec :: struct {
        tv_sec: time_t;

        tv_nsec: s64;
    }
}

timeval :: struct {
    tv_sec: s64;
    tv_usec: s64;
}

#if (OS == .ANDROID && CPU == .ARM64) || (OS == .ANDROID && CPU == .X64) {
    timezone :: struct {
        tz_minuteswest: s32;
        tz_dsttime: s32;
    }
}

#if (OS == .ANDROID && CPU == .ARM64) || (OS == .ANDROID && CPU == .X64) {
    tm :: struct {
        tm_sec: s32;

        tm_min: s32;

        tm_hour: s32;

        tm_mday: s32;

        tm_mon: s32;

        tm_year: s32;

        tm_wday: s32;

        tm_yday: s32;

        tm_isdst: s32;

        tm_gmtoff: s64;

        tm_zone: *u8;
    }
}

ucontext :: struct {
    uc_flags: u64;
    uc_link: *ucontext;
    uc_stack: stack_t;
    struct {
        uc_sigmask: sigset_t;

        #overlay (uc_sigmask) uc_sigmask64: sigset64_t;
    }

    __padding: [120] u8;
    uc_mcontext: mcontext_t;
}

user_fpsimd_struct :: struct {
    vregs: [32] U128 #align 16;
    fpsr: u32;
    fpcr: u32;
}

user_regs_struct :: struct {
    regs: [31] u64;
    sp: u64;
    pc: u64;
    pstate: u64;
}

#if (OS == .ANDROID && CPU == .ARM64) || (OS == .ANDROID && CPU == .X64) {
    utsname :: struct {

        sysname: [65] u8;

        nodename: [65] u8;

        release: [65] u8;

        version: [65] u8;

        machine: [65] u8;

        domainname: [65] u8;
    }
}

sig_t :: #type (unknown0: s32) -> void #c_call;

sighandler_t :: #type (unknown0: s32) -> void #c_call;

__ARCH_SI_CLOCK_T :: s64;

__DIRENT64_INO_T :: ino_t;

__FD_SETSIZE :: 1024;

__FPE_DECDIV :: 10;

__FPE_DECERR :: 11;

__FPE_DECOVF :: 9;

__FPE_INVASC :: 12;

__FPE_INVDEC :: 13;

__ILL_BNDMOD :: 11;

__ILL_BREAK :: 10;

__O_SYNC :: 0x100000;

__O_TMPFILE :: 0x400000;

__SIGRTMAX :: _KERNEL__NSIG;

__SIGRTMIN :: 32;

__WALL :: 0x40000000;

__WCLONE :: 0x80000000;

__WNOTHREAD :: 0x20000000;

_GNU_SOURCE :: 1;

_IOFBF :: 0;

_IOLBF :: 1;

_IONBF :: 2;

_JBLEN :: 32;

_KERNEL__NSIG :: 64;

_NSIG :: _KERNEL__NSIG + 1;

_STK_LIM :: 8 * 1024 * 1024;

ACCESSPERMS :: S_IRWXU | S_IRWXG | S_IRWXO;

ALLPERMS :: S_ISUID | S_ISGID | S_ISVTX | S_IRWXU | S_IRWXG | S_IRWXO;

AT_EACCESS :: 0x200;

AT_EMPTY_PATH :: 0x1000;

AT_FDCWD :: -100;

AT_NO_AUTOMOUNT :: 0x800;

AT_RECURSIVE :: 0x8000;

AT_REMOVEDIR :: 0x200;

AT_STATX_DONT_SYNC :: 0x4000;

AT_STATX_FORCE_SYNC :: 0x2000;

AT_STATX_SYNC_AS_STAT :: 0x0000;

AT_STATX_SYNC_TYPE :: 0x6000;

AT_SYMLINK_FOLLOW :: 0x400;

AT_SYMLINK_NOFOLLOW :: 0x100;

blkcnt_t :: u64;

blksize_t :: u64;

BUFSIZ :: 1024;

BUS_ADRALN :: 1;

BUS_ADRERR :: 2;

BUS_MCEERR_AO :: 5;

BUS_MCEERR_AR :: 4;

BUS_OBJERR :: 3;

CLD_CONTINUED :: 6;

CLD_DUMPED :: 3;

CLD_EXITED :: 1;

CLD_KILLED :: 2;

CLD_STOPPED :: 5;

CLD_TRAPPED :: 4;

clock_t :: s64;

CLOCKS_MASK :: xx clockid_t.REALTIME | xx clockid_t.MONOTONIC;

CLOCKS_MONO :: xx clockid_t.MONOTONIC;

CLOCKS_PER_SEC :: 1000000;

CLONE_ARGS_SIZE_VER0 :: 64;

CLONE_ARGS_SIZE_VER1 :: 80;

CLONE_ARGS_SIZE_VER2 :: 88;

CLONE_CHILD_CLEARTID :: 0x00200000;

CLONE_CHILD_SETTID :: 0x01000000;

CLONE_CLEAR_SIGHAND :: 0x100000000;

CLONE_DETACHED :: 0x00400000;

CLONE_FILES :: 0x00000400;

CLONE_FS :: 0x00000200;

CLONE_INTO_CGROUP :: 0x200000000;

CLONE_IO :: 0x80000000;

CLONE_NEWCGROUP :: 0x02000000;

CLONE_NEWIPC :: 0x08000000;

CLONE_NEWNET :: 0x40000000;

CLONE_NEWNS :: 0x00020000;

CLONE_NEWPID :: 0x20000000;

CLONE_NEWTIME :: 0x00000080;

CLONE_NEWUSER :: 0x10000000;

CLONE_NEWUTS :: 0x04000000;

CLONE_PARENT :: 0x00008000;

CLONE_PARENT_SETTID :: 0x00100000;

CLONE_PIDFD :: 0x00001000;

CLONE_PTRACE :: 0x00002000;

CLONE_SETTLS :: 0x00080000;

CLONE_SIGHAND :: 0x00000800;

CLONE_SYSVSEM :: 0x00040000;

CLONE_THREAD :: 0x00010000;

CLONE_UNTRACED :: 0x00800000;

CLONE_VFORK :: 0x00004000;

CLONE_VM :: 0x00000100;

CPU_SETSIZE :: 1024;

CSIGNAL :: 0x000000ff;

DEFFILEMODE :: S_IRUSR | S_IWUSR | S_IRGRP | S_IWGRP | S_IROTH | S_IWOTH;

dev_t :: u64;

DIPC :: 25;

DN_ACCESS :: 0x00000001;

DN_ATTRIB :: 0x00000020;

DN_CREATE :: 0x00000004;

DN_DELETE :: 0x00000008;

DN_MODIFY :: 0x00000002;

DN_MULTISHOT :: 0x80000000;

DN_RENAME :: 0x00000010;

E2BIG :: 7;

EACCES :: 13;

EADDRINUSE :: 98;

EADDRNOTAVAIL :: 99;

EADV :: 68;

EAFNOSUPPORT :: 97;

EAGAIN :: 11;

EALREADY :: 114;

EBADE :: 52;

EBADF :: 9;

EBADFD :: 77;

EBADMSG :: 74;

EBADR :: 53;

EBADRQC :: 56;

EBADSLT :: 57;

EBFONT :: 59;

EBUSY :: 16;

ECANCELED :: 125;

ECHILD :: 10;

ECHRNG :: 44;

ECOMM :: 70;

ECONNABORTED :: 103;

ECONNREFUSED :: 111;

ECONNRESET :: 104;

EDEADLK :: 35;

EDEADLOCK :: EDEADLK;

EDESTADDRREQ :: 89;

EDOM :: 33;

EDOTDOT :: 73;

EDQUOT :: 122;

EEXIST :: 17;

EFAULT :: 14;

EFBIG :: 27;

EFD_CLOEXEC :: O_CLOEXEC;

EFD_NONBLOCK :: O_NONBLOCK;

EFD_SEMAPHORE :: 1 << 0;

EHOSTDOWN :: 112;

EHOSTUNREACH :: 113;

EHWPOISON :: 133;

EIDRM :: 43;

EILSEQ :: 84;

EINPROGRESS :: 115;

EINTR :: 4;

EINVAL :: 22;

EIO :: 5;

EISCONN :: 106;

EISDIR :: 21;

EISNAM :: 120;

EKEYEXPIRED :: 127;

EKEYREJECTED :: 129;

EKEYREVOKED :: 128;

EL2HLT :: 51;

EL2NSYNC :: 45;

EL3HLT :: 46;

EL3RST :: 47;

ELIBACC :: 79;

ELIBBAD :: 80;

ELIBEXEC :: 83;

ELIBMAX :: 82;

ELIBSCN :: 81;

ELNRNG :: 48;

ELOOP :: 40;

EMEDIUMTYPE :: 124;

EMFILE :: 24;

EMLINK :: 31;

EMSGSIZE :: 90;

EMT_TAGOVF :: 1;

EMULTIHOP :: 72;

ENAMETOOLONG :: 36;

ENAVAIL :: 119;

ENETDOWN :: 100;

ENETRESET :: 102;

ENETUNREACH :: 101;

ENFILE :: 23;

ENOANO :: 55;

ENOBUFS :: 105;

ENOCSI :: 50;

ENODATA :: 61;

ENODEV :: 19;

ENOENT :: 2;

ENOEXEC :: 8;

ENOKEY :: 126;

ENOLCK :: 37;

ENOLINK :: 67;

ENOMEDIUM :: 123;

ENOMEM :: 12;

ENOMSG :: 42;

ENONET :: 64;

ENOPKG :: 65;

ENOPROTOOPT :: 92;

ENOSPC :: 28;

ENOSR :: 63;

ENOSTR :: 60;

ENOSYS :: 38;

ENOTBLK :: 15;

ENOTCONN :: 107;

ENOTDIR :: 20;

ENOTEMPTY :: 39;

ENOTNAM :: 118;

ENOTRECOVERABLE :: 131;

ENOTSOCK :: 88;

ENOTSUP :: EOPNOTSUPP;

ENOTTY :: 25;

ENOTUNIQ :: 76;

ENXIO :: 6;

EOF :: -1;

EOPNOTSUPP :: 95;

EOVERFLOW :: 75;

EOWNERDEAD :: 130;

EPERM :: 1;

EPFNOSUPPORT :: 96;

EPIPE :: 32;

EPROTO :: 71;

EPROTONOSUPPORT :: 93;

EPROTOTYPE :: 91;

ERANGE :: 34;

EREMCHG :: 78;

EREMOTE :: 66;

EREMOTEIO :: 121;

ERESTART :: 85;

ERFKILL :: 132;

EROFS :: 30;

ESHUTDOWN :: 108;

ESOCKTNOSUPPORT :: 94;

ESPIPE :: 29;

ESRCH :: 3;

ESRMNT :: 69;

ESTALE :: 116;

ESTRPIPE :: 86;

ETIME :: 62;

ETIMEDOUT :: 110;

ETOOMANYREFS :: 109;

ETXTBSY :: 26;

EUCLEAN :: 117;

EUNATCH :: 49;

EUSERS :: 87;

eventfd_t :: u64;

EWOULDBLOCK :: EAGAIN;

EXDEV :: 18;

EXFULL :: 54;

F_ADD_SEALS :: F_LINUX_SPECIFIC_BASE + 9;

F_CANCELLK :: F_LINUX_SPECIFIC_BASE + 5;

F_DUPFD :: 0;

F_DUPFD_CLOEXEC :: F_LINUX_SPECIFIC_BASE + 6;

F_EXLCK :: 4;

F_GET_FILE_RW_HINT :: F_LINUX_SPECIFIC_BASE + 13;

F_GET_RW_HINT :: F_LINUX_SPECIFIC_BASE + 11;

F_GET_SEALS :: F_LINUX_SPECIFIC_BASE + 10;

F_GETFD :: 1;

F_GETFL :: 3;

F_GETLEASE :: F_LINUX_SPECIFIC_BASE + 1;

F_GETLK :: 5;

F_GETLK64 :: F_GETLK;

F_GETOWN :: 9;

F_GETOWN_EX :: 16;

F_GETOWNER_UIDS :: 17;

F_GETPIPE_SZ :: F_LINUX_SPECIFIC_BASE + 8;

F_GETSIG :: 11;

F_LINUX_SPECIFIC_BASE :: 1024;

F_NOTIFY :: F_LINUX_SPECIFIC_BASE + 2;

F_OFD_GETLK :: 36;

F_OFD_SETLK :: 37;

F_OFD_SETLKW :: 38;

F_OK :: 0;

F_OWNER_PGRP :: 2;

F_OWNER_PID :: 1;

F_OWNER_TID :: 0;

F_RDLCK :: 0;

F_SEAL_EXEC :: 0x0020;

F_SEAL_FUTURE_WRITE :: 0x0010;

F_SEAL_GROW :: 0x0004;

F_SEAL_SEAL :: 0x0001;

F_SEAL_SHRINK :: 0x0002;

F_SEAL_WRITE :: 0x0008;

F_SET_FILE_RW_HINT :: F_LINUX_SPECIFIC_BASE + 14;

F_SET_RW_HINT :: F_LINUX_SPECIFIC_BASE + 12;

F_SETFD :: 2;

F_SETFL :: 4;

F_SETLEASE :: F_LINUX_SPECIFIC_BASE + 0;

F_SETLK :: 6;

F_SETLK64 :: F_SETLK;

F_SETLKW :: 7;

F_SETLKW64 :: F_SETLKW;

F_SETOWN :: 8;

F_SETOWN_EX :: 15;

F_SETPIPE_SZ :: F_LINUX_SPECIFIC_BASE + 7;

F_SETSIG :: 10;

F_SHLCK :: 8;

F_UNLCK :: 2;

F_WRLCK :: 1;

FASYNC :: 0x2000;

FD_CLOEXEC :: 1;

FILE :: __sFILE;

FILENAME_MAX :: 4096;

FIOASYNC :: 0x5452;

FIOCLEX :: 0x5451;

FIONBIO :: 0x5421;

FIONCLEX :: 0x5450;

FIONREAD :: 0x541B;

FIOQSIZE :: 0x5460;

FNM_CASEFOLD :: 0x10;

FNM_FILE_NAME :: FNM_PATHNAME;

FNM_IGNORECASE :: FNM_CASEFOLD;

FNM_LEADING_DIR :: 0x08;

FNM_NOESCAPE :: 0x01;

FNM_NOMATCH :: 1;

FNM_NOSYS :: 2;

FNM_PATHNAME :: 0x02;

FNM_PERIOD :: 0x04;

FOPEN_MAX :: 20;

FPE_CONDTRAP :: 15;

FPE_FLTDIV :: 3;

FPE_FLTINV :: 7;

FPE_FLTOVF :: 4;

FPE_FLTRES :: 6;

FPE_FLTSUB :: 8;

FPE_FLTUND :: 5;

FPE_FLTUNK :: 14;

FPE_INTDIV :: 1;

FPE_INTOVF :: 2;

fpos64_t :: s64;

fpos_t :: s64;

fpregset_t :: user_fpsimd_struct;

fsblkcnt_t :: u64;

fsfilcnt_t :: u64;

gid_t :: u32;

greg_t :: u64;

gregset_t :: [34] greg_t;

id_t :: u32;

idtype_t :: s32;

ILL_BADIADDR :: 9;

ILL_BADSTK :: 8;

ILL_COPROC :: 7;

ILL_ILLADR :: 3;

ILL_ILLOPC :: 1;

ILL_ILLOPN :: 2;

ILL_ILLTRP :: 4;

ILL_PRVOPC :: 5;

ILL_PRVREG :: 6;

ino64_t :: u64;

ino_t :: u64;

IPC_64 :: 0x0100;

IPC_CREAT :: 0x200;

IPC_DIPC :: 0x1000;

IPC_EXCL :: 0x400;

IPC_INFO :: 3;

IPC_NOWAIT :: 0x800;

IPC_OLD :: 0;

IPC_OWN :: 0x2000;

IPC_PRIVATE :: cast(s32) 0;

IPC_RMID :: 0;

IPC_SET :: 1;

IPC_STAT :: 2;

ITIMER_PROF :: 2;

ITIMER_REAL :: 0;

ITIMER_VIRTUAL :: 1;

jmp_buf :: [32] s64;

key_t :: s32;

L_ctermid :: 1024;

L_tmpnam :: 4096;

locale_t :: *__locale_t;

LOCK_EX :: 2;

LOCK_MAND :: 32;

LOCK_NB :: 4;

LOCK_READ :: 64;

LOCK_RW :: 192;

LOCK_SH :: 1;

LOCK_UN :: 8;

LOCK_WRITE :: 128;

MADV_COLD :: 20;

MADV_COLLAPSE :: 25;

MADV_DODUMP :: 17;

MADV_DOFORK :: 11;

MADV_DONTDUMP :: 16;

MADV_DONTFORK :: 10;

MADV_DONTNEED :: 4;

MADV_DONTNEED_LOCKED :: 24;

MADV_FREE :: 8;

MADV_HUGEPAGE :: 14;

MADV_HWPOISON :: 100;

MADV_KEEPONFORK :: 19;

MADV_MERGEABLE :: 12;

MADV_NOHUGEPAGE :: 15;

MADV_NORMAL :: 0;

MADV_PAGEOUT :: 21;

MADV_POPULATE_READ :: 22;

MADV_POPULATE_WRITE :: 23;

MADV_RANDOM :: 1;

MADV_REMOVE :: 9;

MADV_SEQUENTIAL :: 2;

MADV_SOFT_OFFLINE :: 101;

MADV_UNMERGEABLE :: 13;

MADV_WILLNEED :: 3;

MADV_WIPEONFORK :: 18;

MAP_ANON :: MAP_ANONYMOUS;

MAP_ANONYMOUS :: 0x20;

MAP_DENYWRITE :: 0x0800;

MAP_EXECUTABLE :: 0x1000;

MAP_FILE :: 0;

MAP_FIXED :: 0x10;

MAP_FIXED_NOREPLACE :: 0x100000;

MAP_GROWSDOWN :: 0x0100;

MAP_HUGETLB :: 0x040000;

MAP_LOCKED :: 0x2000;

MAP_NONBLOCK :: 0x010000;

MAP_NORESERVE :: 0x4000;

MAP_POPULATE :: 0x008000;

MAP_PRIVATE :: 0x02;

MAP_SHARED :: 0x01;

MAP_SHARED_VALIDATE :: 0x03;

MAP_STACK :: 0x020000;

MAP_SYNC :: 0x080000;

MAP_TYPE :: 0x0f;

MAP_UNINITIALIZED :: 0x4000000;

MAX_CLOCKS :: 16;

MCL_CURRENT :: 1;

MCL_FUTURE :: 2;

MCL_ONFAULT :: 4;

mcontext_t :: sigcontext;

MINSIGSTKSZ :: 5120;

MLOCK_LIMIT :: 8 * 1024 * 1024;

MLOCK_ONFAULT :: 0x01;

mode_t :: u32;

MREMAP_DONTUNMAP :: 4;

MREMAP_FIXED :: 2;

MREMAP_MAYMOVE :: 1;

MS_ASYNC :: 1;

MS_INVALIDATE :: 2;

MS_SYNC :: 4;

MSGCTL :: 14;

MSGGET :: 13;

MSGRCV :: 12;

MSGSND :: 11;

nfds_t :: u32;

NGREG :: 34;

nlink_t :: u32;

NSIG :: _NSIG;

NSIGBUS :: 5;

NSIGCHLD :: 6;

NSIGEMT :: 1;

NSIGFPE :: 15;

NSIGILL :: 11;

NSIGPOLL :: 6;

NSIGSEGV :: 9;

NSIGSYS :: 2;

NSIGTRAP :: 6;

O_ACCMODE :: 0x3;

O_APPEND :: 0x400;

O_ASYNC :: FASYNC;

O_CLOEXEC :: 0x80000;

O_CREAT :: 0x40;

O_DIRECT :: 0x10000;

O_DIRECTORY :: 0x4000;

O_DSYNC :: 0x1000;

O_EXCL :: 0x80;

O_LARGEFILE :: 0x20000;

O_NDELAY :: O_NONBLOCK;

O_NOATIME :: 0x40000;

O_NOCTTY :: 0x100;

O_NOFOLLOW :: 0x8000;

O_NONBLOCK :: 0x800;

O_PATH :: 0x200000;

O_RDONLY :: 0x0;

O_RDWR :: 0x2;

O_RSYNC :: O_SYNC;

O_SYNC :: __O_SYNC | O_DSYNC;

O_TMPFILE :: __O_TMPFILE | O_DIRECTORY;

O_TMPFILE_MASK :: __O_TMPFILE | O_DIRECTORY | O_CREAT;

O_TRUNC :: 0x200;

O_WRONLY :: 0x1;

old_sigset_t :: u64;

OVERCOMMIT_ALWAYS :: 1;

OVERCOMMIT_GUESS :: 0;

OVERCOMMIT_NEVER :: 2;

P_ALL :: 0;

P_PGID :: 2;

P_PID :: 1;

P_PIDFD :: 3;

P_tmpdir :: "/tmp/";

PAGE_MASK :: ~(PAGE_SIZE - 1);

PAGE_SIZE :: 4096;

pid_t :: s32;

PKEY_ACCESS_MASK :: PKEY_DISABLE_ACCESS | PKEY_DISABLE_WRITE;

PKEY_DISABLE_ACCESS :: 0x1;

PKEY_DISABLE_WRITE :: 0x2;

POLL_BUSY_LOOP :: cast(u32) 0x8000;

POLL_ERR :: 4;

POLL_HUP :: 6;

POLL_IN :: 1;

POLL_MSG :: 3;

POLL_OUT :: 2;

POLL_PRI :: 5;

POLLERR :: 0x0008;

POLLFREE :: cast(u32) 0x4000;

POLLHUP :: 0x0010;

POLLIN :: 0x0001;

POLLMSG :: 0x0400;

POLLNVAL :: 0x0020;

POLLOUT :: 0x0004;

POLLPRI :: 0x0002;

POLLRDBAND :: 0x0080;

POLLRDHUP :: 0x2000;

POLLRDNORM :: 0x0040;

POLLREMOVE :: 0x1000;

POLLWRBAND :: 0x0200;

POLLWRNORM :: 0x0100;

PRIO_MAX :: 20;

PRIO_MIN :: -20;

PROT_BTI :: 0x10;

PROT_EXEC :: 0x4;

PROT_GROWSDOWN :: 0x01000000;

PROT_GROWSUP :: 0x02000000;

PROT_MTE :: 0x20;

PROT_NONE :: 0x0;

PROT_READ :: 0x1;

PROT_SEM :: 0x8;

PROT_WRITE :: 0x2;

PTHREAD_BARRIER_SERIAL_THREAD :: -1;

pthread_barrierattr_t :: s32;

pthread_condattr_t :: s64;

PTHREAD_CREATE_DETACHED :: 1;

PTHREAD_CREATE_JOINABLE :: 0;

PTHREAD_EXPLICIT_SCHED :: 0;

PTHREAD_INHERIT_SCHED :: 1;

pthread_key_t :: s32;

pthread_mutexattr_t :: s64;

PTHREAD_ONCE_INIT :: 0;

pthread_once_t :: s32;

PTHREAD_PRIO_INHERIT :: 1;

PTHREAD_PRIO_NONE :: 0;

PTHREAD_PROCESS_PRIVATE :: 0;

PTHREAD_PROCESS_SHARED :: 1;

pthread_rwlockattr_t :: s64;

PTHREAD_SCOPE_PROCESS :: 1;

PTHREAD_SCOPE_SYSTEM :: 0;

pthread_t :: s64;

ptrdiff_t :: s64;

R_OK :: 4;

RENAME_EXCHANGE :: 1 << 1;

RENAME_NOREPLACE :: 1 << 0;

RENAME_WHITEOUT :: 1 << 2;

RLIM64_INFINITY :: ~0;

rlim64_t :: u64;

RLIM_INFINITY :: ~0;

RLIM_NLIMITS :: 16;

RLIM_SAVED_CUR :: RLIM_INFINITY;

RLIM_SAVED_MAX :: RLIM_INFINITY;

rlim_t :: u64;

RTLD_GLOBAL :: 0x00100;

RTLD_LAZY :: 0x00001;

RTLD_LOCAL :: 0;

RTLD_NODELETE :: 0x01000;

RTLD_NOLOAD :: 0x00004;

RTLD_NOW :: 0x00002;

RWF_WRITE_LIFE_NOT_SET :: RWH_WRITE_LIFE_NOT_SET;

RWH_WRITE_LIFE_EXTREME :: 5;

RWH_WRITE_LIFE_LONG :: 4;

RWH_WRITE_LIFE_MEDIUM :: 3;

RWH_WRITE_LIFE_NONE :: 1;

RWH_WRITE_LIFE_NOT_SET :: 0;

RWH_WRITE_LIFE_SHORT :: 2;

S_IEXEC :: S_IXUSR;

S_IFBLK :: 0x6000;

S_IFCHR :: 0x2000;

S_IFDIR :: 0x4000;

S_IFIFO :: 0x1000;

S_IFLNK :: 0xa000;

S_IFMT :: 0xf000;

S_IFREG :: 0x8000;

S_IFSOCK :: 0xc000;

S_IREAD :: S_IRUSR;

S_IRGRP :: 0x20;

S_IROTH :: 0x4;

S_IRUSR :: 0x100;

S_IRWXG :: 0x38;

S_IRWXO :: 0x7;

S_IRWXU :: 0x1c0;

S_ISGID :: 0x400;

S_ISUID :: 0x800;

S_ISVTX :: 0x200;

S_IWGRP :: 0x10;

S_IWOTH :: 0x2;

S_IWRITE :: S_IWUSR;

S_IWUSR :: 0x80;

S_IXGRP :: 0x8;

S_IXOTH :: 0x1;

S_IXUSR :: 0x40;

SA_RESTORER :: 0x04000000;

SCHED_BATCH :: 3;

SCHED_DEADLINE :: 6;

SCHED_FIFO :: 1;

SCHED_FLAG_ALL :: SCHED_FLAG_RESET_ON_FORK | SCHED_FLAG_RECLAIM | SCHED_FLAG_DL_OVERRUN | SCHED_FLAG_KEEP_ALL | SCHED_FLAG_UTIL_CLAMP;

SCHED_FLAG_DL_OVERRUN :: 0x04;

SCHED_FLAG_KEEP_ALL :: SCHED_FLAG_KEEP_POLICY | SCHED_FLAG_KEEP_PARAMS;

SCHED_FLAG_KEEP_PARAMS :: 0x10;

SCHED_FLAG_KEEP_POLICY :: 0x08;

SCHED_FLAG_RECLAIM :: 0x02;

SCHED_FLAG_RESET_ON_FORK :: 0x01;

SCHED_FLAG_UTIL_CLAMP :: SCHED_FLAG_UTIL_CLAMP_MIN | SCHED_FLAG_UTIL_CLAMP_MAX;

SCHED_FLAG_UTIL_CLAMP_MAX :: 0x40;

SCHED_FLAG_UTIL_CLAMP_MIN :: 0x20;

SCHED_IDLE :: 5;

SCHED_NORMAL :: 0;

SCHED_OTHER :: SCHED_NORMAL;

SCHED_RESET_ON_FORK :: 0x40000000;

SCHED_RR :: 2;

SEEK_CUR :: 1;

SEEK_DATA :: 3;

SEEK_END :: 2;

SEEK_HOLE :: 4;

SEEK_SET :: 0;

SEGV_ACCADI :: 5;

SEGV_ACCERR :: 2;

SEGV_ADIDERR :: 6;

SEGV_ADIPERR :: 7;

SEGV_BNDERR :: 3;

SEGV_MAPERR :: 1;

SEGV_MTEAERR :: 8;

SEGV_MTESERR :: 9;

SEGV_PKUERR :: 4;

SEMCTL :: 3;

SEMGET :: 2;

SEMOP :: 1;

SEMTIMEDOP :: 4;

SFD_CLOEXEC :: O_CLOEXEC;

SFD_NONBLOCK :: O_NONBLOCK;

SHM_EXEC :: 0x8000;

SHM_HUGETLB :: 0x800;

SHM_INFO :: 14;

SHM_LOCK :: 11;

SHM_NORESERVE :: 0x1000;

SHM_R :: 0x100;

SHM_RDONLY :: 0x1000;

SHM_REMAP :: 0x4000;

SHM_RND :: 0x2000;

SHM_STAT :: 13;

SHM_STAT_ANY :: 15;

SHM_UNLOCK :: 12;

SHM_W :: 0x80;

SHMAT :: 21;

shmatt_t :: u64;

SHMCTL :: 24;

SHMDT :: 22;

SHMGET :: 23;

SHMLBA :: 4096;

SHMMIN :: 1;

SHMMNI :: 4096;

SHMSEG :: SHMMNI;

SI_ASYNCIO :: -4;

SI_ASYNCNL :: -60;

SI_DETHREAD :: -7;

SI_KERNEL :: 0x80;

SI_MAX_SIZE :: 128;

SI_MESGQ :: -3;

SI_QUEUE :: -1;

SI_SIGIO :: -5;

SI_TIMER :: -2;

SI_TKILL :: -6;

SI_USER :: 0;

sig_atomic_t :: s32;

SIGABRT :: 6;

SIGALRM :: 14;

SIGBUS :: 7;

SIGCHLD :: 17;

SIGCONT :: 18;

SIGEV_MAX_SIZE :: 64;

SIGEV_NONE :: 1;

SIGEV_SIGNAL :: 0;

SIGEV_THREAD :: 2;

SIGEV_THREAD_ID :: 4;

sigevent_t :: sigevent;

SIGFPE :: 8;

SIGHUP :: 1;

SIGILL :: 4;

siginfo_t :: siginfo;

SIGINT :: 2;

SIGIO :: 29;

SIGIOT :: 6;

sigjmp_buf :: [33] s64;

SIGKILL :: 9;

SIGPIPE :: 13;

SIGPOLL :: SIGIO;

SIGPROF :: 27;

SIGPWR :: 30;

SIGQUIT :: 3;

SIGSEGV :: 11;

sigset64_t :: sigset_t;

SIGSTKFLT :: 16;

SIGSTKSZ :: 16384;

SIGSTOP :: 19;

SIGSYS :: 31;

SIGTERM :: 15;

SIGTRAP :: 5;

SIGTSTP :: 20;

SIGTTIN :: 21;

SIGTTOU :: 22;

SIGUNUSED :: 31;

SIGURG :: 23;

SIGUSR1 :: 10;

SIGUSR2 :: 12;

sigval_t :: sigval;

SIGVTALRM :: 26;

SIGWINCH :: 28;

SIGXCPU :: 24;

SIGXFSZ :: 25;

SPLICE_F_GIFT :: 8;

SPLICE_F_MORE :: 4;

SPLICE_F_MOVE :: 1;

SPLICE_F_NONBLOCK :: 2;

SS_AUTODISARM :: 1 << 31;

SS_DISABLE :: 2;

SS_FLAG_BITS :: SS_AUTODISARM;

SS_ONSTACK :: 1;

stack_t :: sigaltstack_t;

STATX__RESERVED :: 0x80000000;

STATX_ALL :: 0x00000fff;

STATX_ATIME :: 0x00000020;

STATX_ATTR_APPEND :: 0x00000020;

STATX_ATTR_AUTOMOUNT :: 0x00001000;

STATX_ATTR_COMPRESSED :: 0x00000004;

STATX_ATTR_DAX :: 0x00200000;

STATX_ATTR_ENCRYPTED :: 0x00000800;

STATX_ATTR_IMMUTABLE :: 0x00000010;

STATX_ATTR_MOUNT_ROOT :: 0x00002000;

STATX_ATTR_NODUMP :: 0x00000040;

STATX_ATTR_VERITY :: 0x00100000;

STATX_BASIC_STATS :: 0x000007ff;

STATX_BLOCKS :: 0x00000400;

STATX_BTIME :: 0x00000800;

STATX_CTIME :: 0x00000080;

STATX_DIOALIGN :: 0x00002000;

STATX_GID :: 0x00000010;

STATX_INO :: 0x00000100;

STATX_MNT_ID :: 0x00001000;

STATX_MODE :: 0x00000002;

STATX_MTIME :: 0x00000040;

STATX_NLINK :: 0x00000004;

STATX_SIZE :: 0x00000200;

STATX_TYPE :: 0x00000001;

STATX_UID :: 0x00000008;

STDERR_FILENO :: 2;

STDIN_FILENO :: 0;

STDOUT_FILENO :: 1;

suseconds_t :: s64;

SYNC_FILE_RANGE_WAIT_AFTER :: 4;

SYNC_FILE_RANGE_WAIT_BEFORE :: 1;

SYNC_FILE_RANGE_WRITE :: 2;

TCFLSH :: 0x540B;

TCGETA :: 0x5405;

TCGETS :: 0x5401;

TCGETX :: 0x5432;

TCSBRK :: 0x5409;

TCSBRKP :: 0x5425;

TCSETA :: 0x5406;

TCSETAF :: 0x5408;

TCSETAW :: 0x5407;

TCSETS :: 0x5402;

TCSETSF :: 0x5404;

TCSETSW :: 0x5403;

TCSETX :: 0x5433;

TCSETXF :: 0x5434;

TCSETXW :: 0x5435;

TCXONC :: 0x540A;

TFD_CLOEXEC :: O_CLOEXEC;

TFD_NONBLOCK :: O_NONBLOCK;

TFD_TIMER_ABSTIME :: 1 << 0;

TFD_TIMER_CANCEL_ON_SET :: 1 << 1;

TIME_ACTIVE :: xx clockid_t.PROCESS_CPUTIME_ID + 1;

TIME_MONOTONIC :: xx clockid_t.MONOTONIC + 1;

time_t :: s64;

TIME_THREAD_ACTIVE :: xx clockid_t.THREAD_CPUTIME_ID + 1;

TIME_UTC :: xx clockid_t.REALTIME + 1;

TIMER_ABSTIME :: 0x01;

timer_t :: *void;

TIOCCBRK :: 0x5428;

TIOCCONS :: 0x541D;

TIOCEXCL :: 0x540C;

TIOCGETD :: 0x5424;

TIOCGICOUNT :: 0x545D;

TIOCGLCKTRMIOS :: 0x5456;

TIOCGPGRP :: 0x540F;

TIOCGRS485 :: 0x542E;

TIOCGSERIAL :: 0x541E;

TIOCGSID :: 0x5429;

TIOCGSOFTCAR :: 0x5419;

TIOCGWINSZ :: 0x5413;

TIOCINQ :: FIONREAD;

TIOCLINUX :: 0x541C;

TIOCMBIC :: 0x5417;

TIOCMBIS :: 0x5416;

TIOCMGET :: 0x5415;

TIOCMIWAIT :: 0x545C;

TIOCMSET :: 0x5418;

TIOCNOTTY :: 0x5422;

TIOCNXCL :: 0x540D;

TIOCOUTQ :: 0x5411;

TIOCPKT :: 0x5420;

TIOCPKT_DATA :: 0;

TIOCPKT_DOSTOP :: 32;

TIOCPKT_FLUSHREAD :: 1;

TIOCPKT_FLUSHWRITE :: 2;

TIOCPKT_IOCTL :: 64;

TIOCPKT_NOSTOP :: 16;

TIOCPKT_START :: 8;

TIOCPKT_STOP :: 4;

TIOCSBRK :: 0x5427;

TIOCSCTTY :: 0x540E;

TIOCSER_TEMT :: 0x01;

TIOCSERCONFIG :: 0x5453;

TIOCSERGETLSR :: 0x5459;

TIOCSERGETMULTI :: 0x545A;

TIOCSERGSTRUCT :: 0x5458;

TIOCSERGWILD :: 0x5454;

TIOCSERSETMULTI :: 0x545B;

TIOCSERSWILD :: 0x5455;

TIOCSETD :: 0x5423;

TIOCSLCKTRMIOS :: 0x5457;

TIOCSPGRP :: 0x5410;

TIOCSRS485 :: 0x542F;

TIOCSSERIAL :: 0x541F;

TIOCSSOFTCAR :: 0x541A;

TIOCSTI :: 0x5412;

TIOCSWINSZ :: 0x5414;

TIOCVHANGUP :: 0x5437;

TMP_MAX :: 308915776;

TRAP_BRANCH :: 3;

TRAP_BRKPT :: 1;

TRAP_HWBKPT :: 4;

TRAP_PERF :: 6;

TRAP_PERF_FLAG_ASYNC :: 1 << 0;

TRAP_TRACE :: 2;

TRAP_UNK :: 5;

ucontext_t :: ucontext;

uid_t :: u32;

uint_t :: u32;

useconds_t :: u32;

UTIME_NOW :: (1 << 30) - 1;

UTIME_OMIT :: (1 << 30) - 2;

W_OK :: 2;

WCONTINUED :: 0x00000008;

WEXITED :: 0x00000004;

WNOHANG :: 0x00000001;

WNOWAIT :: 0x01000000;

WSTOPPED :: WUNTRACED;

WUNTRACED :: 0x00000002;

X_OK :: 1;

sys_siglist: [65] *u8 #elsewhere libc;

sys_signame: [65] *u8 #elsewhere libc;

tzname: **u8 #elsewhere libc;

daylight: s32 #elsewhere libc;

environ: **u8 #elsewhere libc;

stdin: *FILE #elsewhere libc;

stdout: *FILE #elsewhere libc;

stderr: *FILE #elsewhere libc;

__errno_location :: () -> *OS_Error_Code #foreign libc "__errno";

__libc_current_sigrtmax :: () -> s32 #foreign libc;

__libc_current_sigrtmin :: () -> s32 #foreign libc;

__pthread_cleanup_pop :: (unknown0: *__pthread_cleanup_t, unknown1: s32) -> void #foreign libc;

__pthread_cleanup_push :: (c: *__pthread_cleanup_t, unknown0: #type () -> void #c_call, unknown1: *void) -> void #foreign libc;

__sched_cpualloc :: (__count: u64) -> *cpu_set_t #foreign libc;

__sched_cpucount :: (__set_size: u64, __set: *cpu_set_t) -> s32 #foreign libc;

__sched_cpufree :: (__set: *cpu_set_t) -> void #foreign libc;

_exit :: (__status: s32) -> void #foreign libc; @Exits

_longjmp :: (__env: *jmp_buf, __value: s32) -> void #foreign libc;

_remove :: (__path: *u8) -> s32 #foreign libc "remove";

_setjmp :: (__env: *jmp_buf) -> s32 #foreign libc;

access :: (__path: *u8, __mode: s32) -> s32 #foreign libc;

acct :: (__path: *u8) -> s32 #foreign libc;

alarm :: (__seconds: u32) -> u32 #foreign libc;

alphasort :: (__lhs: **dirent, __rhs: **dirent) -> s32 #foreign libc;

alphasort64 :: (__lhs: **dirent64, __rhs: **dirent64) -> s32 #foreign libc;

asctime :: (__tm: *tm) -> *u8 #foreign libc;

asctime_r :: (__tm: *tm, __buf: *u8) -> *u8 #foreign libc;

asprintf :: (__s_ptr: **u8, __fmt: *u8, __args: ..Any) -> s32 #foreign libc;

backtrace :: (buffer: **void, size: s32) -> s32 #foreign libc;

backtrace_symbols :: (buffer: **void, size: s32) -> **u8 #foreign libc;

backtrace_symbols_fd :: (buffer: **void, size: s32, fd: s32) -> void #foreign libc;

brk :: (__addr: *void) -> s32 #foreign libc;

chdir :: (__path: *u8) -> s32 #foreign libc;

chmod :: (__path: *u8, __mode: mode_t) -> s32 #foreign libc;

chown :: (__path: *u8, __owner: uid_t, __group: gid_t) -> s32 #foreign libc;

chroot :: (__path: *u8) -> s32 #foreign libc;

clearerr :: (__fp: *FILE) -> void #foreign libc;

clearerr_unlocked :: (__fp: *FILE) -> void #foreign libc;

clock :: () -> clock_t #foreign libc;

clock_getcpuclockid :: (__pid: pid_t, __clock: *clockid_t) -> s32 #foreign libc;

clock_getres :: (__clock: clockid_t, __resolution: *timespec) -> s32 #foreign libc;

clock_gettime :: (__clock: clockid_t, __ts: *timespec) -> s32 #foreign libc;

clock_nanosleep :: (__clock: clockid_t, __flags: s32, __request: *timespec, __remainder: *timespec) -> s32 #foreign libc;

clock_settime :: (__clock: clockid_t, __ts: *timespec) -> s32 #foreign libc;

clone :: (__fn: #type (unknown0: *void) -> s32 #c_call, __child_stack: *void, __flags: s32, __arg: *void, __args: ..Any) -> s32 #foreign libc;

close :: (__fd: s32) -> s32 #foreign libc; @FreesArgument

close_range :: (__min_fd: u32, __max_fd: u32, __flags: s32) -> s32 #foreign libc;

closedir :: (__dir: *DIR) -> s32 #foreign libc;

copy_file_range :: (__fd_in: s32, __off_in: *s64, __fd_out: s32, __off_out: *s64, __length: u64, __flags: u32) -> s64 #foreign libc;

creat :: (__path: *u8, __mode: mode_t) -> s32 #foreign libc;

creat64 :: (__path: *u8, __mode: mode_t) -> s32 #foreign libc;

ctermid :: (__buf: *u8) -> *u8 #foreign libc;

ctime :: (__t: *time_t) -> *u8 #foreign libc;

ctime_r :: (__t: *time_t, __buf: *u8) -> *u8 #foreign libc;

daemon :: (__no_chdir: s32, __no_close: s32) -> s32 #foreign libc;

difftime :: (__lhs: time_t, __rhs: time_t) -> float64 #foreign libc;

dirfd :: (__dir: *DIR) -> s32 #foreign libc;

dladdr :: (__addr: *void, __info: *Dl_info) -> s32 #foreign libdl;

dlclose :: (__handle: *void) -> s32 #foreign libdl;

dlerror :: () -> *u8 #foreign libdl;

dlopen :: (__filename: *u8, __flag: s32) -> *void #foreign libdl;

dlsym :: (__handle: *void, __symbol: *u8) -> *void #foreign libdl;

dlvsym :: (__handle: *void, __symbol: *u8, __version: *u8) -> *void #foreign libdl;

dprintf :: (__fd: s32, __fmt: *u8, __args: ..Any) -> s32 #foreign libc;

dup :: (__old_fd: s32) -> s32 #foreign libc;

dup2 :: (__old_fd: s32, __new_fd: s32) -> s32 #foreign libc;

dup3 :: (__old_fd: s32, __new_fd: s32, __flags: s32) -> s32 #foreign libc;

endpwent :: () -> void #foreign libc;

eventfd :: (__initial_value: u32, __flags: s32) -> s32 #foreign libc;

eventfd_read :: (__fd: s32, __value: *eventfd_t) -> s32 #foreign libc;

eventfd_write :: (__fd: s32, __value: eventfd_t) -> s32 #foreign libc;

execl :: (__path: *u8, __arg0: *u8, __args: ..Any) -> s32 #foreign libc;

execle :: (__path: *u8, __arg0: *u8, __args: ..Any) -> s32 #foreign libc;

execlp :: (__file: *u8, __arg0: *u8, __args: ..Any) -> s32 #foreign libc;

execv :: (__path: *u8, __argv: **u8) -> s32 #foreign libc;

execve :: (__file: *u8, __argv: **u8, __envp: **u8) -> s32 #foreign libc;

execvp :: (__file: *u8, __argv: **u8) -> s32 #foreign libc;

execvpe :: (__file: *u8, __argv: **u8, __envp: **u8) -> s32 #foreign libc;

faccessat :: (__dirfd: s32, __path: *u8, __mode: s32, __flags: s32) -> s32 #foreign libc;

fallocate :: (__fd: s32, __mode: s32, __offset: s64, __length: s64) -> s32 #foreign libc;

fallocate64 :: (__fd: s32, __mode: s32, __offset: s64, __length: s64) -> s32 #foreign libc;

fchdir :: (__fd: s32) -> s32 #foreign libc;

fchmod :: (__fd: s32, __mode: mode_t) -> s32 #foreign libc;

fchmodat :: (__dir_fd: s32, __path: *u8, __mode: mode_t, __flags: s32) -> s32 #foreign libc;

fchown :: (__fd: s32, __owner: uid_t, __group: gid_t) -> s32 #foreign libc;

fchownat :: (__dir_fd: s32, __path: *u8, __owner: uid_t, __group: gid_t, __flags: s32) -> s32 #foreign libc;

fclose :: (__fp: *FILE) -> s32 #foreign libc; @FreesArgument

fcntl :: (__fd: s32, __cmd: s32, __args: ..Any) -> s32 #foreign libc;

fdatasync :: (__fd: s32) -> s32 #foreign libc;

fdopen :: (__fd: s32, __mode: *u8) -> *FILE #foreign libc;

fdopendir :: (__dir_fd: s32) -> *DIR #foreign libc;

feof :: (__fp: *FILE) -> s32 #foreign libc;

feof_unlocked :: (__fp: *FILE) -> s32 #foreign libc;

ferror :: (__fp: *FILE) -> s32 #foreign libc;

ferror_unlocked :: (__fp: *FILE) -> s32 #foreign libc;

fexecve :: (__fd: s32, __argv: **u8, __envp: **u8) -> s32 #foreign libc;

fflush :: (__fp: *FILE) -> s32 #foreign libc;

fflush_unlocked :: (__fp: *FILE) -> s32 #foreign libc;

fgetc :: (__fp: *FILE) -> s32 #foreign libc;

fgetc_unlocked :: (__fp: *FILE) -> s32 #foreign libc;

fgetln :: (__fp: *FILE, __length_ptr: *u64) -> *u8 #foreign libc;

fgetpos :: (__fp: *FILE, __pos: *fpos_t) -> s32 #foreign libc;

fgetpos64 :: (__fp: *FILE, __pos: *fpos64_t) -> s32 #foreign libc;

fgets :: (__buf: *u8, __size: s32, __fp: *FILE) -> *u8 #foreign libc;

fgets_unlocked :: (__buf: *u8, __size: s32, __fp: *FILE) -> *u8 #foreign libc;

fileno :: (__fp: *FILE) -> s32 #foreign libc;

fileno_unlocked :: (__fp: *FILE) -> s32 #foreign libc;

flockfile :: (__fp: *FILE) -> void #foreign libc;

fmemopen :: (__buf: *void, __size: u64, __mode: *u8) -> *FILE #foreign libc;

fnmatch :: (__pattern: *u8, __string: *u8, __flags: s32) -> s32 #foreign libc;

fopen :: (__path: *u8, __mode: *u8) -> *FILE #foreign libc;

fopen64 :: (__path: *u8, __mode: *u8) -> *FILE #foreign libc;

fork :: () -> pid_t #foreign libc;

fpathconf :: (__fd: s32, __name: PC) -> s64 #foreign libc;

fprintf :: (__fp: *FILE, __fmt: *u8, __args: ..Any) -> s32 #foreign libc;

fpurge :: (__fp: *FILE) -> s32 #foreign libc;

fputc :: (__ch: s32, __fp: *FILE) -> s32 #foreign libc;

fputc_unlocked :: (__ch: s32, __fp: *FILE) -> s32 #foreign libc;

fputs :: (__s: *u8, __fp: *FILE) -> s32 #foreign libc;

fputs_unlocked :: (__s: *u8, __fp: *FILE) -> s32 #foreign libc;

fread :: (__buf: *void, __size: u64, __count: u64, __fp: *FILE) -> u64 #foreign libc;

fread_unlocked :: (__buf: *void, __size: u64, __count: u64, __fp: *FILE) -> u64 #foreign libc;

freopen :: (__path: *u8, __mode: *u8, __fp: *FILE) -> *FILE #foreign libc;

freopen64 :: (__path: *u8, __mode: *u8, __fp: *FILE) -> *FILE #foreign libc;

fscanf :: (__fp: *FILE, __fmt: *u8, __args: ..Any) -> s32 #foreign libc;

fseek :: (__fp: *FILE, __offset: s64, __whence: s32) -> s32 #foreign libc;

fseeko :: (__fp: *FILE, __offset: s64, __whence: s32) -> s32 #foreign libc;

fseeko64 :: (__fp: *FILE, __offset: s64, __whence: s32) -> s32 #foreign libc;

fsetpos :: (__fp: *FILE, __pos: *fpos_t) -> s32 #foreign libc;

fsetpos64 :: (__fp: *FILE, __pos: *fpos64_t) -> s32 #foreign libc;

fstat :: (__fd: s32, __buf: *stat_t) -> s32 #foreign libc;

fstat64 :: (__fd: s32, __buf: *stat64_t) -> s32 #foreign libc;

fstatat :: (__dir_fd: s32, __path: *u8, __buf: *stat_t, __flags: s32) -> s32 #foreign libc;

fstatat64 :: (__dir_fd: s32, __path: *u8, __buf: *stat64_t, __flags: s32) -> s32 #foreign libc;

fsync :: (__fd: s32) -> s32 #foreign libc;

ftell :: (__fp: *FILE) -> s64 #foreign libc;

ftello :: (__fp: *FILE) -> s64 #foreign libc;

ftello64 :: (__fp: *FILE) -> s64 #foreign libc;

ftok :: (__path: *u8, __id: s32) -> key_t #foreign libc;

ftruncate :: (__fd: s32, __length: s64) -> s32 #foreign libc;

ftruncate64 :: (__fd: s32, __length: s64) -> s32 #foreign libc;

ftrylockfile :: (__fp: *FILE) -> s32 #foreign libc;

funlockfile :: (__fp: *FILE) -> void #foreign libc;

funopen :: (__cookie: *void, __read_fn: #type (unknown0: *void, unknown1: *u8, unknown2: s32) -> s32 #c_call, __write_fn: #type (unknown0: *void, unknown1: *u8, unknown2: s32) -> s32 #c_call, __seek_fn: #type (unknown0: *void, unknown1: fpos_t, unknown2: s32) -> fpos_t #c_call, __close_fn: #type (unknown0: *void) -> s32 #c_call) -> *FILE #foreign libc;

funopen64 :: (__cookie: *void, __read_fn: #type (unknown0: *void, unknown1: *u8, unknown2: s32) -> s32 #c_call, __write_fn: #type (unknown0: *void, unknown1: *u8, unknown2: s32) -> s32 #c_call, __seek_fn: #type (unknown0: *void, unknown1: fpos64_t, unknown2: s32) -> fpos64_t #c_call, __close_fn: #type (unknown0: *void) -> s32 #c_call) -> *FILE #foreign libc;

futimens :: (__fd: s32, __times: *[2] timespec) -> s32 #foreign libc;

futimes :: (__fd: s32, __times: *[2] timeval) -> s32 #foreign libc;

futimesat :: (__dir_fd: s32, __path: *u8, __times: *[2] timeval) -> s32 #foreign libc;

fwrite :: (__buf: *void, __size: u64, __count: u64, __fp: *FILE) -> u64 #foreign libc;

fwrite_unlocked :: (__buf: *void, __size: u64, __count: u64, __fp: *FILE) -> u64 #foreign libc;

getc :: (__fp: *FILE) -> s32 #foreign libc;

getc_unlocked :: (__fp: *FILE) -> s32 #foreign libc;

getchar :: () -> s32 #foreign libc;

getchar_unlocked :: () -> s32 #foreign libc;

getcwd :: (__buf: *u8, __size: u64) -> *u8 #foreign libc;

getdelim :: (__line_ptr: **u8, __line_length_ptr: *u64, __delimiter: s32, __fp: *FILE) -> s64 #foreign libc;

getdomainname :: (__buf: *u8, __buf_size: u64) -> s32 #foreign libc;

getegid :: () -> gid_t #foreign libc;

getenv :: (__name: *u8) -> *u8 #foreign libc;

geteuid :: () -> uid_t #foreign libc;

getgid :: () -> gid_t #foreign libc;

getgroups :: (__size: s32, __list: *gid_t) -> s32 #foreign libc;

gethostname :: (_buf: *u8, __buf_size: u64) -> s32 #foreign libc;

getitimer :: (__which: s32, __current_value: *itimerval) -> s32 #foreign libc;

getline :: (__line_ptr: **u8, __line_length_ptr: *u64, __fp: *FILE) -> s64 #foreign libc;

getlogin :: () -> *u8 #foreign libc;

getlogin_r :: (__buffer: *u8, __buffer_size: u64) -> s32 #foreign libc;

getpagesize :: () -> s32 #foreign libc;

getpgid :: (__pid: pid_t) -> pid_t #foreign libc;

getpgrp :: () -> pid_t #foreign libc;

getpid :: () -> pid_t #foreign libc;

getppid :: () -> pid_t #foreign libc;

getpriority :: (__which: PRIO, __who: id_t) -> s32 #foreign libc;

getpwent :: () -> *passwd #foreign libc;

getpwnam :: (__name: *u8) -> *passwd #foreign libc;

getpwnam_r :: (__name: *u8, __pwd: *passwd, __buf: *u8, __n: u64, __result: **passwd) -> s32 #foreign libc;

getpwuid :: (__uid: uid_t) -> *passwd #foreign libc;

getpwuid_r :: (__uid: uid_t, __pwd: *passwd, __buf: *u8, __n: u64, __result: **passwd) -> s32 #foreign libc;

getresgid :: (__rgid: *gid_t, __egid: *gid_t, __sgid: *gid_t) -> s32 #foreign libc;

getresuid :: (__ruid: *uid_t, __euid: *uid_t, __suid: *uid_t) -> s32 #foreign libc;

getrlimit :: (__resource: RLIMIT_RESOURCE, __limit: *rlimit) -> s32 #foreign libc;

getrlimit64 :: (__resource: RLIMIT_RESOURCE, __limit: *rlimit64) -> s32 #foreign libc;

getrusage :: (__who: RUSAGE, __usage: *rusage) -> s32 #foreign libc;

getsid :: (__pid: pid_t) -> pid_t #foreign libc;

gettid :: () -> pid_t #foreign libc;

gettimeofday :: (__tv: *timeval, __tz: *timezone) -> s32 #foreign libc;

getuid :: () -> uid_t #foreign libc;

gmtime :: (__t: *time_t) -> *tm #foreign libc;

gmtime_r :: (__t: *time_t, __tm: *tm) -> *tm #foreign libc;

isatty :: (__fd: s32) -> s32 #foreign libc;

kill :: (__pid: pid_t, __signal: s32) -> s32 #foreign libc;

killpg :: (__pgrp: s32, __signal: s32) -> s32 #foreign libc;

lchown :: (__path: *u8, __owner: uid_t, __group: gid_t) -> s32 #foreign libc;

link :: (__old_path: *u8, __new_path: *u8) -> s32 #foreign libc;

linkat :: (__old_dir_fd: s32, __old_path: *u8, __new_dir_fd: s32, __new_path: *u8, __flags: s32) -> s32 #foreign libc;

localtime :: (__t: *time_t) -> *tm #foreign libc;

localtime_r :: (__t: *time_t, __tm: *tm) -> *tm #foreign libc;

longjmp :: (__env: *jmp_buf, __value: s32) -> void #foreign libc;

lseek :: (__fd: s32, __offset: s64, __whence: s32) -> s64 #foreign libc;

lseek64 :: (__fd: s32, __offset: s64, __whence: s32) -> s64 #foreign libc;

lstat :: (__path: *u8, __buf: *stat_t) -> s32 #foreign libc;

lstat64 :: (__path: *u8, __buf: *stat64_t) -> s32 #foreign libc;

lutimes :: (__path: *u8, __times: *[2] timeval) -> s32 #foreign libc;

madvise :: (__addr: *void, __size: u64, __advice: s32) -> s32 #foreign libc;

memfd_create :: (__name: *u8, __flags: u32) -> s32 #foreign libc;

mincore :: (__addr: *void, __size: u64, __vector: *u8) -> s32 #foreign libc;

mkdir :: (__path: *u8, __mode: mode_t) -> s32 #foreign libc;

mkdirat :: (__dir_fd: s32, __path: *u8, __mode: mode_t) -> s32 #foreign libc;

mkdtemp :: (__template: *u8) -> *u8 #foreign libc;

mkfifo :: (__path: *u8, __mode: mode_t) -> s32 #foreign libc;

mkfifoat :: (__dir_fd: s32, __path: *u8, __mode: mode_t) -> s32 #foreign libc;

mknod :: (__path: *u8, __mode: mode_t, __dev: dev_t) -> s32 #foreign libc;

mknodat :: (__dir_fd: s32, __path: *u8, __mode: mode_t, __dev: dev_t) -> s32 #foreign libc;

mkostemp :: (__template: *u8, __flags: s32) -> s32 #foreign libc;

mkostemps :: (__template: *u8, __suffix_length: s32, __flags: s32) -> s32 #foreign libc;

mkstemp :: (__template: *u8) -> s32 #foreign libc;

mkstemps :: (__template: *u8, __flags: s32) -> s32 #foreign libc;

mktemp :: (__template: *u8) -> *u8 #foreign libc;

mktime :: (__tm: *tm) -> time_t #foreign libc;

mlock :: (__addr: *void, __size: u64) -> s32 #foreign libc;

mlock2 :: (__addr: *void, __size: u64, __flags: s32) -> s32 #foreign libc;

mlockall :: (__flags: s32) -> s32 #foreign libc;

mmap :: (__addr: *void, __size: u64, __prot: s32, __flags: s32, __fd: s32, __offset: s64) -> *void #foreign libc;

mmap64 :: (__addr: *void, __size: u64, __prot: s32, __flags: s32, __fd: s32, __offset: s64) -> *void #foreign libc;

mprotect :: (__addr: *void, __size: u64, __prot: s32) -> s32 #foreign libc;

mremap :: (__old_addr: *void, __old_size: u64, __new_size: u64, __flags: s32, __args: ..Any) -> *void #foreign libc;

msync :: (__addr: *void, __size: u64, __flags: s32) -> s32 #foreign libc;

munlock :: (__addr: *void, __size: u64) -> s32 #foreign libc;

munlockall :: () -> s32 #foreign libc;

munmap :: (__addr: *void, __size: u64) -> s32 #foreign libc; @MaybeFreesArgument

nanosleep :: (__request: *timespec, __remainder: *timespec) -> s32 #foreign libc;

nice :: (__incr: s32) -> s32 #foreign libc;

open :: (__path: *u8, __flags: s32, __args: ..Any) -> s32 #foreign libc;

open64 :: (__path: *u8, __flags: s32, __args: ..Any) -> s32 #foreign libc;

open_memstream :: (__ptr: **u8, __size_ptr: *u64) -> *FILE #foreign libc;

openat :: (__dir_fd: s32, __path: *u8, __flags: s32, __args: ..Any) -> s32 #foreign libc;

openat64 :: (__dir_fd: s32, __path: *u8, __flags: s32, __args: ..Any) -> s32 #foreign libc;

opendir :: (__path: *u8) -> *DIR #foreign libc;

pathconf :: (__path: *u8, __name: PC) -> s64 #foreign libc;

pause :: () -> s32 #foreign libc;

pclose :: (__fp: *FILE) -> s32 #foreign libc;

perror :: (__msg: *u8) -> void #foreign libc;

pipe :: (__fds: *[2] s32) -> s32 #foreign libc;

pipe2 :: (__fds: *[2] s32, __flags: s32) -> s32 #foreign libc;

poll :: (__fds: *pollfd, __count: nfds_t, __timeout_ms: s32) -> s32 #foreign libc;

popen :: (__command: *u8, __mode: *u8) -> *FILE #foreign libc;

posix_fadvise :: (__fd: s32, __offset: s64, __length: s64, __advice: s32) -> s32 #foreign libc;

posix_fadvise64 :: (__fd: s32, __offset: s64, __length: s64, __advice: s32) -> s32 #foreign libc;

posix_fallocate :: (__fd: s32, __offset: s64, __length: s64) -> s32 #foreign libc;

posix_fallocate64 :: (__fd: s32, __offset: s64, __length: s64) -> s32 #foreign libc;

posix_madvise :: (__addr: *void, __size: u64, __advice: s32) -> s32 #foreign libc;

ppoll :: (__fds: *pollfd, __count: nfds_t, __timeout: *timespec, __mask: *sigset_t) -> s32 #foreign libc;

ppoll64 :: (__fds: *pollfd, __count: nfds_t, __timeout: *timespec, __mask: *sigset64_t) -> s32 #foreign libc;

pread :: (__fd: s32, __buf: *void, __count: u64, __offset: s64) -> s64 #foreign libc;

pread64 :: (__fd: s32, __buf: *void, __count: u64, __offset: s64) -> s64 #foreign libc;

printf :: (__fmt: *u8, __args: ..Any) -> s32 #foreign libc; @WritesOutput

prlimit :: (__pid: pid_t, __resource: RLIMIT_RESOURCE, __new_limit: *rlimit, __old_limit: *rlimit) -> s32 #foreign libc;

prlimit64 :: (__pid: pid_t, __resource: RLIMIT_RESOURCE, __new_limit: *rlimit64, __old_limit: *rlimit64) -> s32 #foreign libc;

process_madvise :: (__pid_fd: s32, __iov: *iovec, __count: u64, __advice: s32, __flags: u32) -> s64 #foreign libc;

psiginfo :: (__info: *siginfo_t, __msg: *u8) -> void #foreign libc;

psignal :: (__signal: s32, __msg: *u8) -> void #foreign libc;

pthread_atfork :: (__prepare: #type () -> void #c_call, __parent: #type () -> void #c_call, __child: #type () -> void #c_call) -> s32 #foreign libc;

pthread_attr_destroy :: (__attr: *pthread_attr_t) -> s32 #foreign libc;

pthread_attr_getdetachstate :: (__attr: *pthread_attr_t, __state: *s32) -> s32 #foreign libc;

pthread_attr_getguardsize :: (__attr: *pthread_attr_t, __size: *u64) -> s32 #foreign libc;

pthread_attr_getinheritsched :: (__attr: *pthread_attr_t, __flag: *s32) -> s32 #foreign libc;

pthread_attr_getschedparam :: (__attr: *pthread_attr_t, __param: *sched_param) -> s32 #foreign libc;

pthread_attr_getschedpolicy :: (__attr: *pthread_attr_t, __policy: *s32) -> s32 #foreign libc;

pthread_attr_getscope :: (__attr: *pthread_attr_t, __scope: *s32) -> s32 #foreign libc;

pthread_attr_getstack :: (__attr: *pthread_attr_t, __addr: **void, __size: *u64) -> s32 #foreign libc;

pthread_attr_getstacksize :: (__attr: *pthread_attr_t, __size: *u64) -> s32 #foreign libc;

pthread_attr_init :: (__attr: *pthread_attr_t) -> s32 #foreign libc;

pthread_attr_setdetachstate :: (__attr: *pthread_attr_t, __state: s32) -> s32 #foreign libc;

pthread_attr_setguardsize :: (__attr: *pthread_attr_t, __size: u64) -> s32 #foreign libc;

pthread_attr_setinheritsched :: (__attr: *pthread_attr_t, __flag: s32) -> s32 #foreign libc;

pthread_attr_setschedparam :: (__attr: *pthread_attr_t, __param: *sched_param) -> s32 #foreign libc;

pthread_attr_setschedpolicy :: (__attr: *pthread_attr_t, __policy: s32) -> s32 #foreign libc;

pthread_attr_setscope :: (__attr: *pthread_attr_t, __scope: s32) -> s32 #foreign libc;

pthread_attr_setstack :: (__attr: *pthread_attr_t, __addr: *void, __size: u64) -> s32 #foreign libc;

pthread_attr_setstacksize :: (__addr: *pthread_attr_t, __size: u64) -> s32 #foreign libc;

pthread_barrier_destroy :: (__barrier: *pthread_barrier_t) -> s32 #foreign libc;

pthread_barrier_init :: (__barrier: *pthread_barrier_t, __attr: *pthread_barrierattr_t, __count: u32) -> s32 #foreign libc;

pthread_barrier_wait :: (__barrier: *pthread_barrier_t) -> s32 #foreign libc;

pthread_barrierattr_destroy :: (__attr: *pthread_barrierattr_t) -> s32 #foreign libc;

pthread_barrierattr_getpshared :: (__attr: *pthread_barrierattr_t, __shared: *s32) -> s32 #foreign libc;

pthread_barrierattr_init :: (__attr: *pthread_barrierattr_t) -> s32 #foreign libc;

pthread_barrierattr_setpshared :: (__attr: *pthread_barrierattr_t, __shared: s32) -> s32 #foreign libc;

pthread_cond_broadcast :: (__cond: *pthread_cond_t) -> s32 #foreign libc;

pthread_cond_clockwait :: (__cond: *pthread_cond_t, __mutex: *pthread_mutex_t, __clock: clockid_t, __timeout: *timespec) -> s32 #foreign libc;

pthread_cond_destroy :: (__cond: *pthread_cond_t) -> s32 #foreign libc;

pthread_cond_init :: (__cond: *pthread_cond_t, __attr: *pthread_condattr_t) -> s32 #foreign libc;

pthread_cond_signal :: (__cond: *pthread_cond_t) -> s32 #foreign libc;

pthread_cond_timedwait :: (__cond: *pthread_cond_t, __mutex: *pthread_mutex_t, __timeout: *timespec) -> s32 #foreign libc;

pthread_cond_timedwait_monotonic_np :: (__cond: *pthread_cond_t, __mutex: *pthread_mutex_t, __timeout: *timespec) -> s32 #foreign libc;

pthread_cond_wait :: (__cond: *pthread_cond_t, __mutex: *pthread_mutex_t) -> s32 #foreign libc;

pthread_condattr_destroy :: (__attr: *pthread_condattr_t) -> s32 #foreign libc;

pthread_condattr_getclock :: (__attr: *pthread_condattr_t, __clock: *clockid_t) -> s32 #foreign libc;

pthread_condattr_getpshared :: (__attr: *pthread_condattr_t, __shared: *s32) -> s32 #foreign libc;

pthread_condattr_init :: (__attr: *pthread_condattr_t) -> s32 #foreign libc;

pthread_condattr_setclock :: (__attr: *pthread_condattr_t, __clock: clockid_t) -> s32 #foreign libc;

pthread_condattr_setpshared :: (__attr: *pthread_condattr_t, __shared: s32) -> s32 #foreign libc;

pthread_create :: (__pthread_ptr: *pthread_t, __attr: *pthread_attr_t, __start_routine: #type (unknown0: *void) -> *void #c_call, unknown0: *void) -> s32 #foreign libc;

pthread_detach :: (__pthread: pthread_t) -> s32 #foreign libc;

pthread_equal :: (__lhs: pthread_t, __rhs: pthread_t) -> s32 #foreign libc;

pthread_exit :: (__return_value: *void) -> void #foreign libc;

pthread_getattr_np :: (__pthread: pthread_t, __attr: *pthread_attr_t) -> s32 #foreign libc;

pthread_getcpuclockid :: (__pthread: pthread_t, __clock: *clockid_t) -> s32 #foreign libc;

pthread_getname_np :: (__pthread: pthread_t, __buf: *u8, __n: u64) -> s32 #foreign libc;

pthread_getschedparam :: (__pthread: pthread_t, __policy: *s32, __param: *sched_param) -> s32 #foreign libc;

pthread_getspecific :: (__key: pthread_key_t) -> *void #foreign libc;

pthread_gettid_np :: (__pthread: pthread_t) -> pid_t #foreign libc;

pthread_join :: (__pthread: pthread_t, __return_value_ptr: **void) -> s32 #foreign libc;

pthread_key_create :: (__key_ptr: *pthread_key_t, __key_destructor: #type (unknown0: *void) -> void #c_call) -> s32 #foreign libc;

pthread_key_delete :: (__key: pthread_key_t) -> s32 #foreign libc;

pthread_mutex_clocklock :: (__mutex: *pthread_mutex_t, __clock: clockid_t, __abstime: *timespec) -> s32 #foreign libc;

pthread_mutex_destroy :: (__mutex: *pthread_mutex_t) -> s32 #foreign libc;

pthread_mutex_init :: (__mutex: *pthread_mutex_t, __attr: *pthread_mutexattr_t) -> s32 #foreign libc;

pthread_mutex_lock :: (__mutex: *pthread_mutex_t) -> s32 #foreign libc;

pthread_mutex_timedlock :: (__mutex: *pthread_mutex_t, __timeout: *timespec) -> s32 #foreign libc;

pthread_mutex_timedlock_monotonic_np :: (__mutex: *pthread_mutex_t, __timeout: *timespec) -> s32 #foreign libc;

pthread_mutex_trylock :: (__mutex: *pthread_mutex_t) -> s32 #foreign libc;

pthread_mutex_unlock :: (__mutex: *pthread_mutex_t) -> s32 #foreign libc;

pthread_mutexattr_destroy :: (__attr: *pthread_mutexattr_t) -> s32 #foreign libc;

pthread_mutexattr_getprotocol :: (__attr: *pthread_mutexattr_t, __protocol: *s32) -> s32 #foreign libc;

pthread_mutexattr_getpshared :: (__attr: *pthread_mutexattr_t, __shared: *s32) -> s32 #foreign libc;

pthread_mutexattr_gettype :: (__attr: *pthread_mutexattr_t, __type: *s32) -> s32 #foreign libc;

pthread_mutexattr_init :: (__attr: *pthread_mutexattr_t) -> s32 #foreign libc;

pthread_mutexattr_setprotocol :: (__attr: *pthread_mutexattr_t, __protocol: s32) -> s32 #foreign libc;

pthread_mutexattr_setpshared :: (__attr: *pthread_mutexattr_t, __shared: s32) -> s32 #foreign libc;

pthread_mutexattr_settype :: (__attr: *pthread_mutexattr_t, __type: s32) -> s32 #foreign libc;

pthread_once :: (__once: *pthread_once_t, __init_routine: #type () -> void #c_call) -> s32 #foreign libc;

pthread_rwlock_clockrdlock :: (__rwlock: *pthread_rwlock_t, __clock: clockid_t, __timeout: *timespec) -> s32 #foreign libc;

pthread_rwlock_clockwrlock :: (__rwlock: *pthread_rwlock_t, __clock: clockid_t, __timeout: *timespec) -> s32 #foreign libc;

pthread_rwlock_destroy :: (__rwlock: *pthread_rwlock_t) -> s32 #foreign libc;

pthread_rwlock_init :: (__rwlock: *pthread_rwlock_t, __attr: *pthread_rwlockattr_t) -> s32 #foreign libc;

pthread_rwlock_rdlock :: (__rwlock: *pthread_rwlock_t) -> s32 #foreign libc;

pthread_rwlock_timedrdlock :: (__rwlock: *pthread_rwlock_t, __timeout: *timespec) -> s32 #foreign libc;

pthread_rwlock_timedrdlock_monotonic_np :: (__rwlock: *pthread_rwlock_t, __timeout: *timespec) -> s32 #foreign libc;

pthread_rwlock_timedwrlock :: (__rwlock: *pthread_rwlock_t, __timeout: *timespec) -> s32 #foreign libc;

pthread_rwlock_timedwrlock_monotonic_np :: (__rwlock: *pthread_rwlock_t, __timeout: *timespec) -> s32 #foreign libc;

pthread_rwlock_tryrdlock :: (__rwlock: *pthread_rwlock_t) -> s32 #foreign libc;

pthread_rwlock_trywrlock :: (__rwlock: *pthread_rwlock_t) -> s32 #foreign libc;

pthread_rwlock_unlock :: (__rwlock: *pthread_rwlock_t) -> s32 #foreign libc;

pthread_rwlock_wrlock :: (__rwlock: *pthread_rwlock_t) -> s32 #foreign libc;

pthread_rwlockattr_destroy :: (__attr: *pthread_rwlockattr_t) -> s32 #foreign libc;

pthread_rwlockattr_getkind_np :: (__attr: *pthread_rwlockattr_t, __kind: *s32) -> s32 #foreign libc;

pthread_rwlockattr_getpshared :: (__attr: *pthread_rwlockattr_t, __shared: *s32) -> s32 #foreign libc;

pthread_rwlockattr_init :: (__attr: *pthread_rwlockattr_t) -> s32 #foreign libc;

pthread_rwlockattr_setkind_np :: (__attr: *pthread_rwlockattr_t, __kind: s32) -> s32 #foreign libc;

pthread_rwlockattr_setpshared :: (__attr: *pthread_rwlockattr_t, __shared: s32) -> s32 #foreign libc;

pthread_self :: () -> pthread_t #foreign libc;

pthread_setname_np :: (__pthread: pthread_t, __name: *u8) -> s32 #foreign libc;

pthread_setschedparam :: (__pthread: pthread_t, __policy: s32, __param: *sched_param) -> s32 #foreign libc;

pthread_setschedprio :: (__pthread: pthread_t, __priority: s32) -> s32 #foreign libc;

pthread_setspecific :: (__key: pthread_key_t, __value: *void) -> s32 #foreign libc;

pthread_spin_destroy :: (__spinlock: *pthread_spinlock_t) -> s32 #foreign libc;

pthread_spin_init :: (__spinlock: *pthread_spinlock_t, __shared: s32) -> s32 #foreign libc;

pthread_spin_lock :: (__spinlock: *pthread_spinlock_t) -> s32 #foreign libc;

pthread_spin_trylock :: (__spinlock: *pthread_spinlock_t) -> s32 #foreign libc;

pthread_spin_unlock :: (__spinlock: *pthread_spinlock_t) -> s32 #foreign libc;

putc :: (__ch: s32, __fp: *FILE) -> s32 #foreign libc;

putc_unlocked :: (__ch: s32, __fp: *FILE) -> s32 #foreign libc;

putchar :: (__ch: s32) -> s32 #foreign libc; @WritesOutput

putchar_unlocked :: (__ch: s32) -> s32 #foreign libc;

puts :: (__s: *u8) -> s32 #foreign libc; @WritesOutput

pwrite :: (__fd: s32, __buf: *void, __count: u64, __offset: s64) -> s64 #foreign libc;

pwrite64 :: (__fd: s32, __buf: *void, __count: u64, __offset: s64) -> s64 #foreign libc;

raise :: (__signal: s32) -> s32 #foreign libc;

read :: (__fd: s32, __buf: *void, __count: u64) -> s64 #foreign libc;

readahead :: (__fd: s32, __offset: s64, __length: u64) -> s64 #foreign libc;

readdir :: (__dir: *DIR) -> *dirent #foreign libc;

readdir64 :: (__dir: *DIR) -> *dirent64 #foreign libc;

readdir64_r :: (__dir: *DIR, __entry: *dirent64, __buffer: **dirent64) -> s32 #foreign libc;

readdir_r :: (__dir: *DIR, __entry: *dirent, __buffer: **dirent) -> s32 #foreign libc;

readlink :: (__path: *u8, __buf: *u8, __buf_size: u64) -> s64 #foreign libc;

readlinkat :: (__dir_fd: s32, __path: *u8, __buf: *u8, __buf_size: u64) -> s64 #foreign libc;

realpath :: (__path: *u8, __resolved: *u8) -> *u8 #foreign libc;

rename :: (__old_path: *u8, __new_path: *u8) -> s32 #foreign libc;

renameat :: (__old_dir_fd: s32, __old_path: *u8, __new_dir_fd: s32, __new_path: *u8) -> s32 #foreign libc;

renameat2 :: (__old_dir_fd: s32, __old_path: *u8, __new_dir_fd: s32, __new_path: *u8, __flags: u32) -> s32 #foreign libc;

rewind :: (__fp: *FILE) -> void #foreign libc;

rewinddir :: (__dir: *DIR) -> void #foreign libc;

rmdir :: (__path: *u8) -> s32 #foreign libc;

sbrk :: (__increment: ptrdiff_t) -> *void #foreign libc;

scandir :: (__path: *u8, __name_list: ***dirent, __filter: #type (unknown0: *dirent) -> s32 #c_call, __comparator: #type (unknown0: **dirent, unknown1: **dirent) -> s32 #c_call) -> s32 #foreign libc;

scandir64 :: (__path: *u8, __name_list: ***dirent64, __filter: #type (unknown0: *dirent64) -> s32 #c_call, __comparator: #type (unknown0: **dirent64, unknown1: **dirent64) -> s32 #c_call) -> s32 #foreign libc;

scandirat :: (__dir_fd: s32, __path: *u8, __name_list: ***dirent, __filter: #type (unknown0: *dirent) -> s32 #c_call, __comparator: #type (unknown0: **dirent, unknown1: **dirent) -> s32 #c_call) -> s32 #foreign libc;

scandirat64 :: (__dir_fd: s32, __path: *u8, __name_list: ***dirent64, __filter: #type (unknown0: *dirent64) -> s32 #c_call, __comparator: #type (unknown0: **dirent64, unknown1: **dirent64) -> s32 #c_call) -> s32 #foreign libc;

scanf :: (__fmt: *u8, __args: ..Any) -> s32 #foreign libc;

sched_get_priority_max :: (__policy: s32) -> s32 #foreign libc;

sched_get_priority_min :: (__policy: s32) -> s32 #foreign libc;

sched_getaffinity :: (__pid: pid_t, __set_size: u64, __set: *cpu_set_t) -> s32 #foreign libc;

sched_getcpu :: () -> s32 #foreign libc;

sched_getparam :: (__pid: pid_t, __param: *sched_param) -> s32 #foreign libc;

sched_getscheduler :: (__pid: pid_t) -> s32 #foreign libc;

sched_rr_get_interval :: (__pid: pid_t, __quantum: *timespec) -> s32 #foreign libc;

sched_setaffinity :: (__pid: pid_t, __set_size: u64, __set: *cpu_set_t) -> s32 #foreign libc;

sched_setparam :: (__pid: pid_t, __param: *sched_param) -> s32 #foreign libc;

sched_setscheduler :: (__pid: pid_t, __policy: s32, __param: *sched_param) -> s32 #foreign libc;

sched_yield :: () -> s32 #foreign libc;

seekdir :: (__dir: *DIR, __location: s64) -> void #foreign libc;

sem_clockwait :: (__sem: *sem_t, __clock: clockid_t, __ts: *timespec) -> s32 #foreign libc;

sem_close :: (__sem: *sem_t) -> s32 #foreign libc;

sem_destroy :: (__sem: *sem_t) -> s32 #foreign libc;

sem_getvalue :: (__sem: *sem_t, __value: *s32) -> s32 #foreign libc;

sem_init :: (__sem: *sem_t, __shared: s32, __value: u32) -> s32 #foreign libc;

sem_open :: (__name: *u8, _flags: s32, __args: ..Any) -> *sem_t #foreign libc;

sem_post :: (__sem: *sem_t) -> s32 #foreign libc;

sem_timedwait :: (__sem: *sem_t, __ts: *timespec) -> s32 #foreign libc;

sem_timedwait_monotonic_np :: (__sem: *sem_t, __ts: *timespec) -> s32 #foreign libc;

sem_trywait :: (__sem: *sem_t) -> s32 #foreign libc;

sem_unlink :: (__name: *u8) -> s32 #foreign libc;

sem_wait :: (__sem: *sem_t) -> s32 #foreign libc;

setbuf :: (__fp: *FILE, __buf: *u8) -> void #foreign libc;

setbuffer :: (__fp: *FILE, __buf: *u8, __size: s32) -> void #foreign libc;

setdomainname :: (__name: *u8, __n: u64) -> s32 #foreign libc;

setegid :: (__gid: gid_t) -> s32 #foreign libc;

setenv :: (__name: *u8, __value: *u8, __overwrite: s32) -> s32 #foreign libc;

seteuid :: (__uid: uid_t) -> s32 #foreign libc;

setgid :: (__gid: gid_t) -> s32 #foreign libc;

setgroups :: (__size: u64, __list: *gid_t) -> s32 #foreign libc;

sethostname :: (__name: *u8, __n: u64) -> s32 #foreign libc;

setitimer :: (__which: s32, __new_value: *itimerval, __old_value: *itimerval) -> s32 #foreign libc;

setjmp :: (__env: *jmp_buf) -> s32 #foreign libc;

setlinebuf :: (__fp: *FILE) -> s32 #foreign libc;

setns :: (__fd: s32, __ns_type: s32) -> s32 #foreign libc;

setpgid :: (__pid: pid_t, __pgid: pid_t) -> s32 #foreign libc;

setpgrp :: () -> s32 #foreign libc;

setpriority :: (__which: PRIO, __who: id_t, __priority: s32) -> s32 #foreign libc;

setpwent :: () -> void #foreign libc;

setregid :: (__rgid: gid_t, __egid: gid_t) -> s32 #foreign libc;

setresgid :: (__rgid: gid_t, __egid: gid_t, __sgid: gid_t) -> s32 #foreign libc;

setresuid :: (__ruid: uid_t, __euid: uid_t, __suid: uid_t) -> s32 #foreign libc;

setreuid :: (__ruid: uid_t, __euid: uid_t) -> s32 #foreign libc;

setrlimit :: (__resource: RLIMIT_RESOURCE, __limit: *rlimit) -> s32 #foreign libc;

setrlimit64 :: (__resource: RLIMIT_RESOURCE, __limit: *rlimit64) -> s32 #foreign libc;

setsid :: () -> pid_t #foreign libc;

settimeofday :: (__tv: *timeval, __tz: *timezone) -> s32 #foreign libc;

setuid :: (__uid: uid_t) -> s32 #foreign libc;

setvbuf :: (__fp: *FILE, __buf: *u8, __mode: s32, __size: u64) -> s32 #foreign libc;

shmat :: (__shm_id: s32, __addr: *void, __flags: s32) -> *void #foreign libc;

shmdt :: (__addr: *void) -> s32 #foreign libc;

shmget :: (key: key_t, __size: u64, __flags: s32) -> s32 #foreign libc;

sigaction :: (__signal: s32, __new_action: *sigaction_t, __old_action: *sigaction_t) -> s32 #foreign libc;

sigaction64 :: (__signal: s32, __new_action: *sigaction64_t, __old_action: *sigaction64_t) -> s32 #foreign libc;

sigaddset :: (__set: *sigset_t, __signal: s32) -> s32 #foreign libc;

sigaddset64 :: (__set: *sigset64_t, __signal: s32) -> s32 #foreign libc;

sigaltstack :: (__new_signal_stack: *stack_t, __old_signal_stack: *stack_t) -> s32 #foreign libc;

sigdelset :: (__set: *sigset_t, __signal: s32) -> s32 #foreign libc;

sigdelset64 :: (__set: *sigset64_t, __signal: s32) -> s32 #foreign libc;

sigemptyset :: (__set: *sigset_t) -> s32 #foreign libc;

sigemptyset64 :: (__set: *sigset64_t) -> s32 #foreign libc;

sigfillset :: (__set: *sigset_t) -> s32 #foreign libc;

sigfillset64 :: (__set: *sigset64_t) -> s32 #foreign libc;

sighold :: (__signal: s32) -> s32 #foreign libc;

sigignore :: (__signal: s32) -> s32 #foreign libc;

siginterrupt :: (__signal: s32, __flag: s32) -> s32 #foreign libc;

sigismember :: (__set: *sigset_t, __signal: s32) -> s32 #foreign libc;

sigismember64 :: (__set: *sigset64_t, __signal: s32) -> s32 #foreign libc;

siglongjmp :: (__env: *sigjmp_buf, __value: s32) -> void #foreign libc;

signal :: (__signal: s32, __handler: sighandler_t) -> sighandler_t #foreign libc;

signalfd :: (__fd: s32, __mask: *sigset_t, __flags: s32) -> s32 #foreign libc;

signalfd64 :: (__fd: s32, __mask: *sigset64_t, __flags: s32) -> s32 #foreign libc;

sigpause :: (__signal: s32) -> s32 #foreign libc;

sigpending :: (__set: *sigset_t) -> s32 #foreign libc;

sigpending64 :: (__set: *sigset64_t) -> s32 #foreign libc;

sigprocmask :: (__how: s32, __new_set: *sigset_t, __old_set: *sigset_t) -> s32 #foreign libc;

sigprocmask64 :: (__how: s32, __new_set: *sigset64_t, __old_set: *sigset64_t) -> s32 #foreign libc;

sigqueue :: (__pid: pid_t, __signal: s32, __value: sigval) -> s32 #foreign libc;

sigrelse :: (__signal: s32) -> s32 #foreign libc;

sigset :: (__signal: s32, __handler: sighandler_t) -> sighandler_t #foreign libc;

sigsetjmp :: (__env: *sigjmp_buf, __save_signal_mask: s32) -> s32 #foreign libc;

sigsuspend :: (__mask: *sigset_t) -> s32 #foreign libc;

sigsuspend64 :: (__mask: *sigset64_t) -> s32 #foreign libc;

sigtimedwait :: (__set: *sigset_t, __info: *siginfo_t, __timeout: *timespec) -> s32 #foreign libc;

sigtimedwait64 :: (__set: *sigset64_t, __info: *siginfo_t, __timeout: *timespec) -> s32 #foreign libc;

sigwait :: (__set: *sigset_t, __signal: *s32) -> s32 #foreign libc;

sigwait64 :: (__set: *sigset64_t, __signal: *s32) -> s32 #foreign libc;

sigwaitinfo :: (__set: *sigset_t, __info: *siginfo_t) -> s32 #foreign libc;

sigwaitinfo64 :: (__set: *sigset64_t, __info: *siginfo_t) -> s32 #foreign libc;

sleep :: (__seconds: u32) -> u32 #foreign libc;

snprintf :: (__buf: *u8, __size: u64, __fmt: *u8, __args: ..Any) -> s32 #foreign libc;

splice :: (__in_fd: s32, __in_offset: *s64, __out_fd: s32, __out_offset: *s64, __length: u64, __flags: u32) -> s64 #foreign libc;

sprintf :: (__s: *u8, __fmt: *u8, __args: ..Any) -> s32 #foreign libc;

sscanf :: (__s: *u8, __fmt: *u8, __args: ..Any) -> s32 #foreign libc;

stat :: (__path: *u8, __buf: *stat_t) -> s32 #foreign libc;

stat64 :: (__path: *u8, __buf: *stat64_t) -> s32 #foreign libc;

statx :: (__dir_fd: s32, __path: *u8, __flags: s32, __mask: u32, __buf: *statx_t) -> s32 #foreign libc;

strftime :: (__buf: *u8, __n: u64, __fmt: *u8, __tm: *tm) -> u64 #foreign libc;

strftime_l :: (__buf: *u8, __n: u64, __fmt: *u8, __tm: *tm, __l: locale_t) -> u64 #foreign libc;

strptime :: (__s: *u8, __fmt: *u8, __tm: *tm) -> *u8 #foreign libc;

strptime_l :: (__s: *u8, __fmt: *u8, __tm: *tm, __l: locale_t) -> *u8 #foreign libc;

swab :: (__src: *void, __dst: *void, __byte_count: s64) -> void #foreign libc;

symlink :: (__old_path: *u8, __new_path: *u8) -> s32 #foreign libc;

symlinkat :: (__old_path: *u8, __new_dir_fd: s32, __new_path: *u8) -> s32 #foreign libc;

sync :: () -> void #foreign libc;

sync_file_range :: (__fd: s32, __offset: s64, __length: s64, __flags: u32) -> s32 #foreign libc;

syncfs :: (__fd: s32) -> s32 #foreign libc;

syscall :: (__number: s64, __args: ..Any) -> s64 #foreign libc;

sysconf :: (name: _SC_definitions) -> s64 #foreign libc;

system :: (__command: *u8) -> s32 #foreign libc;

tcgetpgrp :: (__fd: s32) -> pid_t #foreign libc;

tcsetpgrp :: (__fd: s32, __pid: pid_t) -> s32 #foreign libc;

tee :: (__in_fd: s32, __out_fd: s32, __length: u64, __flags: u32) -> s64 #foreign libc;

telldir :: (__dir: *DIR) -> s64 #foreign libc;

tempnam :: (__dir: *u8, __prefix: *u8) -> *u8 #foreign libc;

tgkill :: (__tgid: s32, __tid: s32, __signal: s32) -> s32 #foreign libc;

time :: (__t: *time_t) -> time_t #foreign libc;

timegm :: (__tm: *tm) -> time_t #foreign libc;

timelocal :: (__tm: *tm) -> time_t #foreign libc;

timer_create :: (__clock: clockid_t, __event: *sigevent, __timer_ptr: *timer_t) -> s32 #foreign libc;

timer_delete :: (__timer: timer_t) -> s32 #foreign libc;

timer_getoverrun :: (__timer: timer_t) -> s32 #foreign libc;

timer_gettime :: (_timer: timer_t, __ts: *itimerspec) -> s32 #foreign libc;

timer_settime :: (__timer: timer_t, __flags: s32, __new_value: *itimerspec, __old_value: *itimerspec) -> s32 #foreign libc;

timerfd_create :: (__clock: clockid_t, __flags: s32) -> s32 #foreign libc;

timerfd_gettime :: (__fd: s32, __current_value: *itimerspec) -> s32 #foreign libc;

timerfd_settime :: (__fd: s32, __flags: s32, __new_value: *itimerspec, __old_value: *itimerspec) -> s32 #foreign libc;

timespec_get :: (__ts: *timespec, __base: s32) -> s32 #foreign libc;

tmpfile :: () -> *FILE #foreign libc;

tmpfile64 :: () -> *FILE #foreign libc;

tmpnam :: (__s: *u8) -> *u8 #foreign libc;

truncate :: (__path: *u8, __length: s64) -> s32 #foreign libc;

truncate64 :: (__path: *u8, __length: s64) -> s32 #foreign libc;

ttyname :: (__fd: s32) -> *u8 #foreign libc;

ttyname_r :: (__fd: s32, __buf: *u8, __buf_size: u64) -> s32 #foreign libc;

tzset :: () -> void #foreign libc;

umask :: (__mask: mode_t) -> mode_t #foreign libc;

uname :: (__buf: *utsname) -> s32 #foreign libc;

ungetc :: (__ch: s32, __fp: *FILE) -> s32 #foreign libc;

unlink :: (__path: *u8) -> s32 #foreign libc;

unlinkat :: (__dirfd: s32, __path: *u8, __flags: s32) -> s32 #foreign libc;

unsetenv :: (__name: *u8) -> s32 #foreign libc;

unshare :: (__flags: s32) -> s32 #foreign libc;

usleep :: (__microseconds: useconds_t) -> s32 #foreign libc;

utimensat :: (__dir_fd: s32, __path: *u8, __times: *[2] timespec, __flags: s32) -> s32 #foreign libc;

utimes :: (__path: *u8, __times: *[2] timeval) -> s32 #foreign libc;

vfork :: () -> s32 #foreign libc;

vmsplice :: (__fd: s32, __iov: *iovec, __count: u64, __flags: u32) -> s64 #foreign libc;

wait :: (__status: *s32) -> pid_t #foreign libc;

wait4 :: (__pid: pid_t, __status: *s32, __options: s32, __rusage: *rusage) -> pid_t #foreign libc;

waitid :: (__type: idtype_t, __id: id_t, __info: *siginfo_t, __options: s32) -> s32 #foreign libc;

waitpid :: (__pid: pid_t, __status: *s32, __options: s32) -> pid_t #foreign libc;

write :: (__fd: s32, __buf: *void, __count: u64) -> s64 #foreign libc; @WritesOutput(0)
