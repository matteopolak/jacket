// Generated from docs/specs/stdlib/modules/posix.md.
// Regenerate with tools/native_bindings/generate.py; do not hand-edit.

// The declarations for `OS == .LINUX && ARCHITECTURE == .ARM`, loaded by declarations.jai.

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

__fsid_t :: struct {
    __val: [2] s32;
}

__jmp_buf_tag :: struct {
    __jmpbuf: __jmp_buf;
    __mask_was_saved: s32;
    __saved_mask: __sigset_t;
}

__locale_struct :: struct {
    __locales: [13] *void;
    __ctype_b: *u16;
    __ctype_tolower: *s32;
    __ctype_toupper: *s32;
    __names: [13] *u8;
}

__mbstate_t :: struct {
    __count: s32;
    __value: struct {
        __wch: u32;
        #overlay (__wch) __wchb: [4] u8;
    };
}

__pthread_cleanup_frame :: struct {
    __cancel_routine: #type (unknown0: *void) -> void #c_call;
    __cancel_arg: *void;
    __do_it: s32;
    __cancel_type: s32;
}

__pthread_cond_s :: struct {
    __wseq: struct {
        __value64: u64;
        #overlay (__value64) __value32: struct {
            __low: u32;
            __high: u32;
        };
    };
    __g1_start: struct {
        __value64: u64;
        #overlay (__value64) __value32: struct {
            __low: u32;
            __high: u32;
        };
    };
    __g_refs: [2] u32;
    __g_size: [2] u32;
    __g1_orig_size: u32;
    __wrefs: u32;
    __g_signals: [2] u32;
}

__pthread_internal_list :: struct {
    __prev: *__pthread_internal_list;
    __next: *__pthread_internal_list;
}

__pthread_internal_slist :: struct {
    __next: *__pthread_internal_slist;
}

__pthread_mutex_s :: struct {
    __lock: s32;
    __count: u32;
    __owner: s32;
    __kind: s32;
    __nusers: u32;
    using __anonymous6: struct {
        __spins: s32;
        #overlay (__spins) __list: struct {
            __next: *__pthread_internal_slist;
        };
    };
}

__pthread_rwlock_arch_t :: struct {
    __readers: u32;
    __writers: u32;
    __wrphase_futex: u32;
    __writers_futex: u32;
    __pad3: u32;
    __pad4: u32;
    __flags: u8;
    __shared: u8;
    __pad1: u8;
    __pad2: u8;
    __cur_writer: s32;
}

__pthread_unwind_buf_t :: struct {
    __cancel_jmp_buf: [1] struct {
        __cancel_jmp_buf: __jmp_buf;
        __mask_was_saved: s32;
    };
    __pad: [4] *void;
}

__sigset_t :: struct {
    __val: [32] u32;
}

_G_fpos64_t :: struct {
    __pos: s64;
    __state: __mbstate_t;
}

_G_fpos_t :: struct {
    __pos: s32;
    __state: __mbstate_t;
}

_IO_cookie_io_functions_t :: struct {
    read: *cookie_read_function_t;
    write: *cookie_write_function_t;
    seek: *cookie_seek_function_t;
    close: *cookie_close_function_t;
}

_IO_FILE :: struct {
    _flags: s32;
    _IO_read_ptr: *u8;
    _IO_read_end: *u8;
    _IO_read_base: *u8;
    _IO_write_base: *u8;
    _IO_write_ptr: *u8;
    _IO_write_end: *u8;
    _IO_buf_base: *u8;
    _IO_buf_end: *u8;
    _IO_save_base: *u8;
    _IO_backup_base: *u8;
    _IO_save_end: *u8;
    _markers: *void;
    _chain: *_IO_FILE;
    _fileno: s32;
    _flags2: s32;
    _old_offset: s32;
    _cur_column: u16;
    _vtable_offset: s8;
    _shortbuf: [1] u8;
    _lock: *void;
    _offset: s64;
    _codecvt: *void;
    _wide_data: *void;
    _freeres_list: *_IO_FILE;
    _freeres_buf: *void;
    __pad5: size_t;
    _mode: s32;
    _unused2: [40] u8;
}

_pthread_cleanup_buffer :: struct {
    __routine: #type (unknown0: *void) -> void #c_call;
    __arg: *void;
    __canceltype: s32;
    __prev: *_pthread_cleanup_buffer;
}

aiocb :: struct {
    aio_fildes: s32;
    aio_lio_opcode: s32;
    aio_reqprio: s32;
    aio_buf: *void;
    aio_nbytes: size_t;
    aio_sigevent: sigevent;
    __next_prio: *aiocb;
    __abs_prio: s32;
    __policy: s32;
    __error_code: s32;
    __return_value: s32;
    aio_offset: s64;
    __glibc_reserved: [32] u8;
}

aiocb64 :: struct {
    aio_fildes: s32;
    aio_lio_opcode: s32;
    aio_reqprio: s32;
    aio_buf: *void;
    aio_nbytes: size_t;
    aio_sigevent: sigevent;
    __next_prio: *aiocb;
    __abs_prio: s32;
    __policy: s32;
    __error_code: s32;
    __return_value: s32;
    aio_offset: s64;
    __glibc_reserved: [32] u8;
}

aioinit :: struct {
    aio_threads: s32;
    aio_num: s32;
    aio_locks: s32;
    aio_usedba: s32;
    aio_debug: s32;
    aio_numusers: s32;
    aio_idle_time: s32;
    aio_reserved: s32;
}

cpu_set_t :: struct {
    __bits: [32] __cpu_mask;
}

dirent :: struct {
    d_ino: u64;
    d_off: s64;
    d_reclen: u16;
    d_type: u8;
    d_name: [256] u8;
}

dirent64 :: struct {
    d_ino: u64;
    d_off: s64;
    d_reclen: u16;
    d_type: u8;
    d_name: [256] u8;
}

Dl_info :: struct {
    dli_fname: *u8;
    dli_fbase: *void;
    dli_sname: *u8;
    dli_saddr: *void;
}

Dl_serinfo :: struct {
    dls_size: size_t;
    dls_cnt: u32;
    using __anonymous3: struct {
        dls_serpath: [0] Dl_serpath;
        #overlay (dls_serpath) __dls_serpath_pad: [1] Dl_serpath;
    };
}

Dl_serpath :: struct {
    dls_name: *u8;
    dls_flags: u32;
}

f_owner_ex :: struct {
    type: __pid_type;
    pid: s32;
}

file_handle :: struct {
    handle_bytes: u32;
    handle_type: s32;
    f_handle: [0] u8;
}

flock :: struct {
    l_type: s16;
    l_whence: s16;
    l_start: s64;
    l_len: s64;
    l_pid: s32;
}

flock64 :: struct {
    l_type: s16;
    l_whence: s16;
    l_start: s64;
    l_len: s64;
    l_pid: s32;
}

fpos64_t :: struct {
    __pos: s64;
    __state: __mbstate_t;
};

fpos_t :: struct {
    __pos: s64;
    __state: __mbstate_t;
};

iovec :: struct {
    iov_base: *void;
    iov_len: size_t;
}

ipc_perm :: struct {
    __key: s32;
    uid: u32;
    gid: u32;
    cuid: u32;
    cgid: u32;
    mode: u32;
    __seq: u16;
    __pad2: u16;
    __glibc_reserved1: u32;
    __glibc_reserved2: u32;
}

itimerspec :: struct {
    it_interval: timespec;
    it_value: timespec;
}

itimerval :: struct {
    it_interval: timeval;
    it_value: timeval;
}

mcontext_t :: struct {
    trap_no: u32;
    error_code: u32;
    oldmask: u32;
    arm_r0: u32;
    arm_r1: u32;
    arm_r2: u32;
    arm_r3: u32;
    arm_r4: u32;
    arm_r5: u32;
    arm_r6: u32;
    arm_r7: u32;
    arm_r8: u32;
    arm_r9: u32;
    arm_r10: u32;
    arm_fp: u32;
    arm_ip: u32;
    arm_sp: u32;
    arm_lr: u32;
    arm_pc: u32;
    arm_cpsr: u32;
    fault_address: u32;
}

passwd :: struct {
    pw_name: *u8;
    pw_passwd: *u8;
    pw_uid: u32;
    pw_gid: u32;
    pw_gecos: *u8;
    pw_dir: *u8;
    pw_shell: *u8;
}

pollfd :: struct {
    fd: s32;
    events: s16;
    revents: s16;
}

pthread_attr_t :: struct {
    __size: [36] u8;
    #overlay (__size) __align: s32;
}

pthread_barrier_t :: struct {
    __size: [20] u8;
    #overlay (__size) __align: s32;
}

pthread_barrierattr_t :: struct {
    __size: [4] u8;
    #overlay (__size) __align: s32;
}

pthread_cond_t :: struct {
    __data: __pthread_cond_s;
    #overlay (__data) __size: [48] u8;
    #overlay (__data) __align: s64;
}

pthread_condattr_t :: struct {
    __size: [4] u8;
    #overlay (__size) __align: s32;
}

pthread_mutex_t :: struct {
    __data: __pthread_mutex_s;
    #overlay (__data) __size: [24] u8;
    #overlay (__data) __align: s32;
}

pthread_mutexattr_t :: struct {
    __size: [4] u8;
    #overlay (__size) __align: s32;
}

pthread_rwlock_t :: struct {
    __data: __pthread_rwlock_arch_t;
    #overlay (__data) __size: [32] u8;
    #overlay (__data) __align: s32;
}

pthread_rwlockattr_t :: struct {
    __size: [8] u8;
    #overlay (__size) __align: s32;
}

rlimit :: struct {
    rlim_cur: rlim_t;
    rlim_max: rlim_t;
}

rlimit64 :: struct {
    rlim_cur: rlim64_t;
    rlim_max: rlim64_t;
}

rusage :: struct {
    ru_utime: timeval;
    ru_stime: timeval;
    using __anonymous3: struct {
        ru_maxrss: s32;
        #overlay (ru_maxrss) __ru_maxrss_word: s32;
    };
    using __anonymous4: struct {
        ru_ixrss: s32;
        #overlay (ru_ixrss) __ru_ixrss_word: s32;
    };
    using __anonymous5: struct {
        ru_idrss: s32;
        #overlay (ru_idrss) __ru_idrss_word: s32;
    };
    using __anonymous6: struct {
        ru_isrss: s32;
        #overlay (ru_isrss) __ru_isrss_word: s32;
    };
    using __anonymous7: struct {
        ru_minflt: s32;
        #overlay (ru_minflt) __ru_minflt_word: s32;
    };
    using __anonymous8: struct {
        ru_majflt: s32;
        #overlay (ru_majflt) __ru_majflt_word: s32;
    };
    using __anonymous9: struct {
        ru_nswap: s32;
        #overlay (ru_nswap) __ru_nswap_word: s32;
    };
    using __anonymous10: struct {
        ru_inblock: s32;
        #overlay (ru_inblock) __ru_inblock_word: s32;
    };
    using __anonymous11: struct {
        ru_oublock: s32;
        #overlay (ru_oublock) __ru_oublock_word: s32;
    };
    using __anonymous12: struct {
        ru_msgsnd: s32;
        #overlay (ru_msgsnd) __ru_msgsnd_word: s32;
    };
    using __anonymous13: struct {
        ru_msgrcv: s32;
        #overlay (ru_msgrcv) __ru_msgrcv_word: s32;
    };
    using __anonymous14: struct {
        ru_nsignals: s32;
        #overlay (ru_nsignals) __ru_nsignals_word: s32;
    };
    using __anonymous15: struct {
        ru_nvcsw: s32;
        #overlay (ru_nvcsw) __ru_nvcsw_word: s32;
    };
    using __anonymous16: struct {
        ru_nivcsw: s32;
        #overlay (ru_nivcsw) __ru_nivcsw_word: s32;
    };
}

sched_param :: struct {
    sched_priority: s32;
}

sem_t :: struct {
    __size: [16] u8;
    #overlay (__size) __align: s32;
}

shm_info :: struct {
    used_ids: s32;
    shm_tot: u32;
    shm_rss: u32;
    shm_swp: u32;
    swap_attempts: u32;
    swap_successes: u32;
}

shmid_ds :: struct {
    shm_perm: ipc_perm;
    shm_segsz: size_t;
    shm_atime: s64;
    shm_dtime: s64;
    shm_ctime: s64;
    shm_cpid: s32;
    shm_lpid: s32;
    shm_nattch: shmatt_t;
    __glibc_reserved5: u32;
    __glibc_reserved6: u32;
}

shminfo :: struct {
    shmmax: u32;
    shmmin: u32;
    shmmni: u32;
    shmseg: u32;
    shmall: u32;
    __glibc_reserved1: u32;
    __glibc_reserved2: u32;
    __glibc_reserved3: u32;
    __glibc_reserved4: u32;
}

sigaction_t :: struct {
    using __sigaction_handler: struct {
        sa_handler: #type (unknown0: s32) -> void #c_call;
        #overlay (sa_handler) sa_sigaction: #type (unknown0: s32, unknown1: *siginfo_t, unknown2: *void) -> void #c_call;
    };
    sa_mask: __sigset_t;
    sa_flags: s32;
    sa_restorer: #type () -> void #c_call;
}

sigcontext :: struct {
    trap_no: u32;
    error_code: u32;
    oldmask: u32;
    arm_r0: u32;
    arm_r1: u32;
    arm_r2: u32;
    arm_r3: u32;
    arm_r4: u32;
    arm_r5: u32;
    arm_r6: u32;
    arm_r7: u32;
    arm_r8: u32;
    arm_r9: u32;
    arm_r10: u32;
    arm_fp: u32;
    arm_ip: u32;
    arm_sp: u32;
    arm_lr: u32;
    arm_pc: u32;
    arm_cpsr: u32;
    fault_address: u32;
}

sigevent :: struct {
    sigev_value: sigval;
    sigev_signo: s32;
    sigev_notify: s32;
    _sigev_un: struct {
        _pad: [13] s32;
        #overlay (_pad) _tid: s32;
        #overlay (_pad) _sigev_thread: struct {
            _function: #type (unknown0: sigval) -> void #c_call;
            _attribute: *pthread_attr_t;
        };
    };
}

siginfo_t :: struct {
    si_signo: s32;
    si_errno: s32;
    si_code: s32;
    _sifields: struct {
        _pad: [29] s32;
        #overlay (_pad) _kill: struct {
            si_pid: s32;
            si_uid: u32;
        };
        #overlay (_pad) _timer: struct {
            si_tid: s32;
            si_overrun: s32;
            si_sigval: sigval;
        };
        #overlay (_pad) _rt: struct {
            si_pid: s32;
            si_uid: u32;
            si_sigval: sigval;
        };
        #overlay (_pad) _sigchld: struct {
            si_pid: s32;
            si_uid: u32;
            si_status: s32;
            si_utime: s32;
            si_stime: s32;
        };
        #overlay (_pad) _sigfault: struct {
            si_addr: *void;
            si_addr_lsb: s16;
            _bounds: struct {
                _addr_bnd: struct {
                    _lower: *void;
                    _upper: *void;
                };
                #overlay (_addr_bnd) _pkey: u32;
            };
        };
        #overlay (_pad) _sigpoll: struct {
            si_band: s32;
            si_fd: s32;
        };
        #overlay (_pad) _sigsys: struct {
            _call_addr: *void;
            _syscall: s32;
            _arch: u32;
        };
    };
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

sigval :: struct {
    sival_int: s32;
    #overlay (sival_int) sival_ptr: *void;
}

stack_t :: struct {
    ss_sp: *void;
    ss_flags: s32;
    ss_size: size_t;
}

stat64_t :: struct {
    st_dev: u64;
    st_ino: u64;
    st_mode: u32;
    st_nlink: u32;
    st_uid: u32;
    st_gid: u32;
    st_rdev: u64;
    st_size: s64;
    st_blksize: s32;
    st_blocks: s64;
    st_atime: timespec;
    st_mtime: timespec;
    st_ctime: timespec;
}

stat_t :: struct {
    st_dev: u64;
    st_ino: u64;
    st_mode: u32;
    st_nlink: u32;
    st_uid: u32;
    st_gid: u32;
    st_rdev: u64;
    st_size: s64;
    st_blksize: s32;
    st_blocks: s64;
    st_atime: timespec;
    st_mtime: timespec;
    st_ctime: timespec;
}

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

statx_timestamp :: struct {
    tv_sec: s64;
    tv_nsec: u32;
    __reserved: s32;
}

timespec :: struct {
    tv_sec: s64;
    tv_nsec: s32;
    #overlay (tv_nsec) __pad3: [8] u8;
}

timeval :: struct {
    tv_sec: s64;
    tv_usec: s64;
}

timex :: struct {
    modes: u32;
    #overlay (modes) __pad2: [8] u8;
    offset: s64;
    freq: s64;
    maxerror: s64;
    esterror: s64;
    status: s32;
    #overlay (status) __pad8: [8] u8;
    constant: s64;
    precision: s64;
    tolerance: s64;
    time: timeval;
    tick: s64;
    ppsfreq: s64;
    jitter: s64;
    shift: s32;
    #overlay (shift) __pad17: [8] u8;
    stabil: s64;
    jitcnt: s64;
    calcnt: s64;
    errcnt: s64;
    stbcnt: s64;
    tai: s32;
    #overlay (tai) __pad24: [48] u8;
}

timezone :: struct {
    tz_minuteswest: s32;
    tz_dsttime: s32;
}

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
    tm_gmtoff: s32;
    tm_zone: *u8;
}

ucontext_t :: struct {
    uc_flags: u32;
    uc_link: *ucontext_t;
    uc_stack: stack_t;
    uc_mcontext: mcontext_t;
    uc_sigmask: sigset_t;
    uc_regspace: [128] u32;
}

utsname :: struct {
    sysname: [65] u8;
    nodename: [65] u8;
    release: [65] u8;
    version: [65] u8;
    machine: [65] u8;
    domainname: [65] u8;
}

__cpu_mask :: u32;

__jmp_buf :: [64] s32;

__O_DIRECT :: 65536;

__O_DIRECTORY :: 16384;

__O_LARGEFILE :: 131072;

__O_NOFOLLOW :: 32768;

__PTHREAD_MUTEX_HAVE_PREV :: 0;

_DIRENT_MATCHES_DIRENT64 :: 0;

blksize_t :: s32;

clock_t :: s32;

F_GETLK :: 12;

F_GETLK64 :: 12;

F_SETLK :: 13;

F_SETLK64 :: 13;

F_SETLKW :: 14;

F_SETLKW64 :: 14;

FIOQSIZE :: 21598;

fpregset_t :: _libc_fpstate;

greg_t :: s32;

gregset_t :: [18] greg_t;

Lmid_t :: s32;

nfds_t :: u32;

nlink_t :: u32;

pthread_t :: u32;

register_t :: s32;

shmatt_t :: u32;

size_t :: u32;

ssize_t :: s32;

suseconds_t :: s32;

SYS__sysctl :: 149;

SYS_accept :: 285;

SYS_accept4 :: 366;

SYS_access :: 33;

SYS_acct :: 51;

SYS_add_key :: 309;

SYS_adjtimex :: 124;

SYS_bind :: 282;

SYS_bpf :: 386;

SYS_brk :: 45;

SYS_capget :: 184;

SYS_capset :: 185;

SYS_chdir :: 12;

SYS_chmod :: 15;

SYS_chown :: 182;

SYS_chroot :: 61;

SYS_clock_adjtime :: 372;

SYS_clock_getres :: 264;

SYS_clock_gettime :: 263;

SYS_clock_nanosleep :: 265;

SYS_clock_settime :: 262;

SYS_clone :: 120;

SYS_clone3 :: 435;

SYS_close :: 6;

SYS_connect :: 283;

SYS_copy_file_range :: 391;

SYS_creat :: 8;

SYS_delete_module :: 129;

SYS_dup :: 41;

SYS_dup2 :: 63;

SYS_dup3 :: 358;

SYS_epoll_create :: 250;

SYS_epoll_create1 :: 357;

SYS_epoll_ctl :: 251;

SYS_epoll_pwait :: 346;

SYS_epoll_wait :: 252;

SYS_eventfd :: 351;

SYS_eventfd2 :: 356;

SYS_execve :: 11;

SYS_execveat :: 387;

SYS_exit :: 1;

SYS_exit_group :: 248;

SYS_faccessat :: 334;

SYS_fallocate :: 352;

SYS_fanotify_init :: 367;

SYS_fanotify_mark :: 368;

SYS_fchdir :: 133;

SYS_fchmod :: 94;

SYS_fchmodat :: 333;

SYS_fchown :: 95;

SYS_fchownat :: 325;

SYS_fcntl :: 55;

SYS_fdatasync :: 148;

SYS_fgetxattr :: 231;

SYS_finit_module :: 379;

SYS_flistxattr :: 234;

SYS_flock :: 143;

SYS_fork :: 2;

SYS_fremovexattr :: 237;

SYS_fsconfig :: 431;

SYS_fsetxattr :: 228;

SYS_fsmount :: 432;

SYS_fsopen :: 430;

SYS_fspick :: 433;

SYS_fstat :: 108;

SYS_fstatfs :: 100;

SYS_fsync :: 118;

SYS_ftruncate :: 93;

SYS_futex :: 240;

SYS_futimesat :: 326;

SYS_get_mempolicy :: 320;

SYS_get_robust_list :: 339;

SYS_getcpu :: 345;

SYS_getcwd :: 183;

SYS_getdents :: 141;

SYS_getdents64 :: 217;

SYS_getegid :: 50;

SYS_geteuid :: 49;

SYS_getgid :: 47;

SYS_getgroups :: 80;

SYS_getitimer :: 105;

SYS_getpeername :: 287;

SYS_getpgid :: 132;

SYS_getpgrp :: 65;

SYS_getpid :: 20;

SYS_getppid :: 64;

SYS_getpriority :: 96;

SYS_getrandom :: 384;

SYS_getresgid :: 171;

SYS_getresuid :: 165;

SYS_getrusage :: 77;

SYS_getsid :: 147;

SYS_getsockname :: 286;

SYS_getsockopt :: 295;

SYS_gettid :: 224;

SYS_gettimeofday :: 78;

SYS_getuid :: 24;

SYS_getxattr :: 229;

SYS_init_module :: 128;

SYS_inotify_add_watch :: 317;

SYS_inotify_init :: 316;

SYS_inotify_init1 :: 360;

SYS_inotify_rm_watch :: 318;

SYS_io_cancel :: 247;

SYS_io_destroy :: 244;

SYS_io_getevents :: 245;

SYS_io_pgetevents :: 399;

SYS_io_setup :: 243;

SYS_io_submit :: 246;

SYS_io_uring_enter :: 426;

SYS_io_uring_register :: 427;

SYS_io_uring_setup :: 425;

SYS_ioctl :: 54;

SYS_ioprio_get :: 315;

SYS_ioprio_set :: 314;

SYS_kcmp :: 378;

SYS_kexec_file_load :: 401;

SYS_kexec_load :: 347;

SYS_keyctl :: 311;

SYS_kill :: 37;

SYS_lchown :: 16;

SYS_lgetxattr :: 230;

SYS_link :: 9;

SYS_linkat :: 330;

SYS_listen :: 284;

SYS_listxattr :: 232;

SYS_llistxattr :: 233;

SYS_lookup_dcookie :: 249;

SYS_lremovexattr :: 236;

SYS_lseek :: 19;

SYS_lsetxattr :: 227;

SYS_lstat :: 107;

SYS_madvise :: 220;

SYS_mbind :: 319;

SYS_membarrier :: 389;

SYS_memfd_create :: 385;

SYS_migrate_pages :: 400;

SYS_mincore :: 219;

SYS_mkdir :: 39;

SYS_mkdirat :: 323;

SYS_mknod :: 14;

SYS_mknodat :: 324;

SYS_mlock :: 150;

SYS_mlock2 :: 390;

SYS_mlockall :: 152;

SYS_mount :: 21;

SYS_move_mount :: 429;

SYS_move_pages :: 344;

SYS_mprotect :: 125;

SYS_mq_getsetattr :: 279;

SYS_mq_notify :: 278;

SYS_mq_open :: 274;

SYS_mq_timedreceive :: 277;

SYS_mq_timedsend :: 276;

SYS_mq_unlink :: 275;

SYS_mremap :: 163;

SYS_msgctl :: 304;

SYS_msgget :: 303;

SYS_msgrcv :: 302;

SYS_msgsnd :: 301;

SYS_msync :: 144;

SYS_munlock :: 151;

SYS_munlockall :: 153;

SYS_munmap :: 91;

SYS_name_to_handle_at :: 370;

SYS_nanosleep :: 162;

SYS_nfsservctl :: 169;

SYS_open :: 5;

SYS_open_by_handle_at :: 371;

SYS_open_tree :: 428;

SYS_openat :: 322;

SYS_pause :: 29;

SYS_perf_event_open :: 364;

SYS_personality :: 136;

SYS_pidfd_open :: 434;

SYS_pidfd_send_signal :: 424;

SYS_pipe :: 42;

SYS_pipe2 :: 359;

SYS_pivot_root :: 218;

SYS_pkey_alloc :: 395;

SYS_pkey_free :: 396;

SYS_pkey_mprotect :: 394;

SYS_poll :: 168;

SYS_ppoll :: 336;

SYS_prctl :: 172;

SYS_pread64 :: 180;

SYS_preadv :: 361;

SYS_preadv2 :: 392;

SYS_prlimit64 :: 369;

SYS_process_vm_readv :: 376;

SYS_process_vm_writev :: 377;

SYS_pselect6 :: 335;

SYS_ptrace :: 26;

SYS_pwrite64 :: 181;

SYS_pwritev :: 362;

SYS_pwritev2 :: 393;

SYS_quotactl :: 131;

SYS_read :: 3;

SYS_readahead :: 225;

SYS_readlink :: 85;

SYS_readlinkat :: 332;

SYS_readv :: 145;

SYS_reboot :: 88;

SYS_recvfrom :: 292;

SYS_recvmmsg :: 365;

SYS_recvmsg :: 297;

SYS_remap_file_pages :: 253;

SYS_removexattr :: 235;

SYS_rename :: 38;

SYS_renameat :: 329;

SYS_renameat2 :: 382;

SYS_request_key :: 310;

SYS_restart_syscall :: 0;

SYS_rmdir :: 40;

SYS_rseq :: 398;

SYS_rt_sigaction :: 174;

SYS_rt_sigpending :: 176;

SYS_rt_sigprocmask :: 175;

SYS_rt_sigqueueinfo :: 178;

SYS_rt_sigreturn :: 173;

SYS_rt_sigsuspend :: 179;

SYS_rt_sigtimedwait :: 177;

SYS_rt_tgsigqueueinfo :: 363;

SYS_sched_get_priority_max :: 159;

SYS_sched_get_priority_min :: 160;

SYS_sched_getaffinity :: 242;

SYS_sched_getattr :: 381;

SYS_sched_getparam :: 155;

SYS_sched_getscheduler :: 157;

SYS_sched_rr_get_interval :: 161;

SYS_sched_setaffinity :: 241;

SYS_sched_setattr :: 380;

SYS_sched_setparam :: 154;

SYS_sched_setscheduler :: 156;

SYS_sched_yield :: 158;

SYS_seccomp :: 383;

SYS_semctl :: 300;

SYS_semget :: 299;

SYS_semop :: 298;

SYS_semtimedop :: 312;

SYS_sendfile :: 187;

SYS_sendmmsg :: 374;

SYS_sendmsg :: 296;

SYS_sendto :: 290;

SYS_set_mempolicy :: 321;

SYS_set_robust_list :: 338;

SYS_set_tid_address :: 256;

SYS_setdomainname :: 121;

SYS_setfsgid :: 139;

SYS_setfsuid :: 138;

SYS_setgid :: 46;

SYS_setgroups :: 81;

SYS_sethostname :: 74;

SYS_setitimer :: 104;

SYS_setns :: 375;

SYS_setpgid :: 57;

SYS_setpriority :: 97;

SYS_setregid :: 71;

SYS_setresgid :: 170;

SYS_setresuid :: 164;

SYS_setreuid :: 70;

SYS_setrlimit :: 75;

SYS_setsid :: 66;

SYS_setsockopt :: 294;

SYS_settimeofday :: 79;

SYS_setuid :: 23;

SYS_setxattr :: 226;

SYS_shmat :: 305;

SYS_shmctl :: 308;

SYS_shmdt :: 306;

SYS_shmget :: 307;

SYS_shutdown :: 293;

SYS_sigaltstack :: 186;

SYS_signalfd :: 349;

SYS_signalfd4 :: 355;

SYS_socket :: 281;

SYS_socketpair :: 288;

SYS_splice :: 340;

SYS_stat :: 106;

SYS_statfs :: 99;

SYS_statx :: 397;

SYS_swapoff :: 115;

SYS_swapon :: 87;

SYS_symlink :: 83;

SYS_symlinkat :: 331;

SYS_sync :: 36;

SYS_syncfs :: 373;

SYS_sysfs :: 135;

SYS_sysinfo :: 116;

SYS_syslog :: 103;

SYS_tee :: 342;

SYS_tgkill :: 268;

SYS_timer_create :: 257;

SYS_timer_delete :: 261;

SYS_timer_getoverrun :: 260;

SYS_timer_gettime :: 259;

SYS_timer_settime :: 258;

SYS_timerfd_create :: 350;

SYS_timerfd_gettime :: 354;

SYS_timerfd_settime :: 353;

SYS_times :: 43;

SYS_tkill :: 238;

SYS_truncate :: 92;

SYS_umask :: 60;

SYS_umount2 :: 52;

SYS_uname :: 122;

SYS_unlink :: 10;

SYS_unlinkat :: 328;

SYS_unshare :: 337;

SYS_uselib :: 86;

SYS_userfaultfd :: 388;

SYS_ustat :: 62;

SYS_utimensat :: 348;

SYS_utimes :: 269;

SYS_vfork :: 190;

SYS_vhangup :: 111;

SYS_vmsplice :: 343;

SYS_vserver :: 313;

SYS_wait4 :: 114;

SYS_waitid :: 280;

SYS_write :: 4;

SYS_writev :: 146;

__asprintf :: (__ptr: *void, __fmt: *u8, __args: ..Any) -> s32 #foreign libc;

__errno_location :: () -> *s32 #foreign libc;

__getdelim :: (__lineptr: *void, __n: *size_t, __delimiter: s32, __stream: *FILE) -> s32 #foreign libc;

__sched_cpualloc :: (__count: size_t) -> *cpu_set_t #foreign libpthread;

__sched_cpucount :: (__setsize: size_t, __setp: *cpu_set_t) -> s32 #foreign libpthread;

__sysv_signal :: (__sig: s32, __handler: #type (unknown0: s32) -> void #c_call) -> ((unknown0: s32) -> void #c_call) #foreign libc;

adjtime :: (__delta: *timeval, __olddelta: *timeval) -> s32 #foreign libc "__adjtime64";

aio_cancel :: (__fildes: s32, __aiocbp: *aiocb) -> s32 #foreign librt "aio_cancel64";

aio_error :: (__aiocbp: *aiocb) -> s32 #foreign librt "aio_error64";

aio_fsync :: (__operation: s32, __aiocbp: *aiocb) -> s32 #foreign librt "aio_fsync64";

aio_read :: (__aiocbp: *aiocb) -> s32 #foreign librt "aio_read64";

aio_return :: (__aiocbp: *aiocb) -> s32 #foreign librt "aio_return64";

aio_return64 :: (__aiocbp: *aiocb64) -> s32 #foreign librt;

aio_suspend :: (__list: *void, __nent: s32, __timeout: *timespec) -> s32 #foreign librt "__aio_suspend_time64";

aio_suspend64 :: (__list: *void, __nent: s32, __timeout: *timespec) -> s32 #foreign librt;

aio_write :: (__aiocbp: *aiocb) -> s32 #foreign librt "aio_write64";

alphasort :: (__e1: *void, __e2: *void) -> s32 #foreign libc "alphasort64";

alphasort64 :: (__e1: *void, __e2: *void) -> s32 #foreign libc;

asprintf :: (__ptr: *void, __fmt: *u8, __args: ..Any) -> s32 #foreign libc;

clock_getres :: (__clock_id: clockid_t, __res: *timespec) -> s32 #foreign libc "__clock_getres64";

clock_gettime :: (__clock_id: clockid_t, __tp: *timespec) -> s32 #foreign libc "__clock_gettime64";

clock_nanosleep :: (__clock_id: clockid_t, __flags: s32, __req: *timespec, __rem: *timespec) -> s32 #foreign libc "__clock_nanosleep_time64";

clock_settime :: (__clock_id: clockid_t, __tp: *timespec) -> s32 #foreign libc "__clock_settime64";

clone :: (__fn: #type (unknown0: *void) -> s32 #c_call, __child_stack: *void, __flags: s32, __arg: *void, __args: ..Any) -> s32 #foreign libpthread;

confstr :: (__name: s32, __buf: *u8, __len: size_t) -> size_t #foreign libc;

copy_file_range :: (__infd: s32, __pinoff: *s64, __outfd: s32, __poutoff: *s64, __length: size_t, __flags: u32) -> ssize_t #foreign libc;

creat :: (__file: *u8, __mode: mode_t) -> s32 #foreign libc "creat64";

ctime :: (__timer: *time_t) -> *u8 #foreign libc "__ctime64";

ctime_r :: (__timer: *time_t, __buf: *u8) -> *u8 #foreign libc "__ctime64_r";

difftime :: (__time1: time_t, __time0: time_t) -> float64 #foreign libc "__difftime64";

dladdr1 :: (__address: *void, __info: *Dl_info, __extra_info: *void, __flags: s32) -> s32 #foreign libdl;

execv :: (__path: *u8, __argv: *void) -> s32 #foreign libc;

execve :: (__path: *u8, __argv: *void, __envp: *void) -> s32 #foreign libc;

execvp :: (__file: *u8, __argv: *void) -> s32 #foreign libc;

execvpe :: (__file: *u8, __argv: *void, __envp: *void) -> s32 #foreign libc;

fallocate :: (__fd: s32, __mode: s32, __offset: s64, __len: s64) -> s32 #foreign libc "fallocate64";

fcntl :: (__fd: s32, __request: s32, __args: ..Any) -> s32 #foreign libc "__fcntl_time64";

fcntl64 :: (__fd: s32, __request: s32, __args: ..Any) -> s32 #foreign libc "__fcntl_time64";

fexecve :: (__fd: s32, __argv: *void, __envp: *void) -> s32 #foreign libc;

fgetpos :: (__stream: *FILE, __pos: *fpos_t) -> s32 #foreign libc "fgetpos64";

fgetpwent_r :: (__stream: *FILE, __resultbuf: *passwd, __buffer: *u8, __buflen: size_t, __result: *void) -> s32 #foreign libc;

fmemopen :: (__s: *void, __len: size_t, __modes: *u8) -> *FILE #foreign libc;

fpathconf :: (__fd: s32, __name: PC) -> s32 #foreign libc;

fread :: (unknown0: *void, unknown1: u32, unknown2: u32, unknown3: *FILE) -> u32 #foreign libc;

fread_unlocked :: (__ptr: *void, __size: size_t, __n: size_t, __stream: *FILE) -> size_t #foreign libc;

freopen :: (__filename: *u8, __modes: *u8, __stream: *FILE) -> *FILE #foreign libc "freopen64";

fseek :: (__stream: *FILE, __off: s32, __whence: s32) -> s32 #foreign libc;

fseeko :: (__stream: *FILE, __off: s64, __whence: s32) -> s32 #foreign libc "fseeko64";

fsetpos :: (__stream: *FILE, __pos: *fpos_t) -> s32 #foreign libc "fsetpos64";

ftell :: (__stream: *FILE) -> s32 #foreign libc;

ftello :: (__stream: *FILE) -> s64 #foreign libc "ftello64";

ftruncate :: (__fd: s32, __length: s64) -> s32 #foreign libc "ftruncate64";

futimens :: (fd: s32, __times: *[2] timespec) -> s32 #foreign libc "__futimens64";

futimes :: (__fd: s32, __tvp: *[2] timeval) -> s32 #foreign libc "__futimes64";

futimesat :: (__fd: s32, __file: *u8, __tvp: *[2] timeval) -> s32 #foreign libc "__futimesat64";

fwrite :: (unknown0: *void, unknown1: u32, unknown2: u32, unknown3: *FILE) -> u32 #foreign libc;

fwrite_unlocked :: (__ptr: *void, __size: size_t, __n: size_t, __stream: *FILE) -> size_t #foreign libc;

getcwd :: (__buf: *u8, __size: size_t) -> *u8 #foreign libc;

getdelim :: (__lineptr: *void, __n: *size_t, __delimiter: s32, __stream: *FILE) -> s32 #foreign libc;

getdirentries :: (__fd: s32, __buf: *u8, __nbytes: size_t, __basep: *s64) -> s32 #foreign libc "getdirentries64";

getdirentries64 :: (__fd: s32, __buf: *u8, __nbytes: size_t, __basep: *s64) -> s32 #foreign libc;

getdomainname :: (__name: *u8, __len: size_t) -> s32 #foreign libc;

getentropy :: (__buffer: *void, __length: size_t) -> s32 #foreign libc;

gethostid :: () -> s32 #foreign libc;

gethostname :: (__name: *u8, __len: size_t) -> s32 #foreign libc;

getitimer :: (__which: __itimer_which, __value: *itimerval) -> s32 #foreign libc "__getitimer64";

getline :: (__lineptr: *void, __n: *size_t, __stream: *FILE) -> s32 #foreign libc;

getlogin_r :: (__name: *u8, __name_len: size_t) -> s32 #foreign libc;

getpriority :: (__which: __priority_which, __who: id_t) -> s32 #foreign libc;

getpwent_r :: (__resultbuf: *passwd, __buffer: *u8, __buflen: size_t, __result: *void) -> s32 #foreign libc;

getpwnam_r :: (__name: *u8, __resultbuf: *passwd, __buffer: *u8, __buflen: size_t, __result: *void) -> s32 #foreign libc;

getpwuid_r :: (__uid: u32, __resultbuf: *passwd, __buffer: *u8, __buflen: size_t, __result: *void) -> s32 #foreign libc;

getrlimit :: (__resource: __rlimit_resource, __rlimits: *rlimit) -> s32 #foreign libc "getrlimit64";

getrlimit64 :: (__resource: __rlimit_resource, __rlimits: *rlimit64) -> s32 #foreign libc;

getrusage :: (__who: __rusage_who, __usage: *rusage) -> s32 #foreign libc;

gettimeofday :: (__tv: *timeval, __tz: *void) -> s32 #foreign libc "__gettimeofday64";

gmtime :: (__timer: *time_t) -> *tm #foreign libc "__gmtime64";

gmtime_r :: (__timer: *time_t, __tp: *tm) -> *tm #foreign libc "__gmtime64_r";

ioctl :: (__fd: s32, __request: u32, __args: ..Any) -> s32 #foreign libc "__ioctl_time64";

lio_listio :: (__mode: s32, __list: *void, __nent: s32, __sig: *sigevent) -> s32 #foreign librt "lio_listio64";

lio_listio64 :: (__mode: s32, __list: *void, __nent: s32, __sig: *sigevent) -> s32 #foreign librt;

localtime :: (__timer: *time_t) -> *tm #foreign libc "__localtime64";

localtime_r :: (__t: *time_t, __tp: *tm) -> *tm #foreign libc "__localtime64_r";

lockf :: (__fd: s32, __cmd: s32, __len: s64) -> s32 #foreign libc "lockf64";

lseek :: (__fd: s32, __offset: s64, __whence: s32) -> s64 #foreign libc "lseek64";

lutimes :: (__file: *u8, __tvp: *[2] timeval) -> s32 #foreign libc "__lutimes64";

madvise :: (__addr: *void, __len: size_t, __advice: s32) -> s32 #foreign libc;

mincore :: (__start: *void, __len: size_t, __vec: *u8) -> s32 #foreign libc;

mkostemp :: (__template: *u8, __flags: s32) -> s32 #foreign libc "mkostemp64";

mkostemps :: (__template: *u8, __suffixlen: s32, __flags: s32) -> s32 #foreign libc "mkostemps64";

mkstemp :: (__template: *u8) -> s32 #foreign libc "mkstemp64";

mkstemps :: (__template: *u8, __suffixlen: s32) -> s32 #foreign libc "mkstemps64";

mktime :: (__tp: *tm) -> time_t #foreign libc "__mktime64";

mlock :: (__addr: *void, __len: size_t) -> s32 #foreign libc;

mmap :: (__addr: *void, __len: size_t, __prot: s32, __flags: s32, __fd: s32, __offset: s64) -> *void #foreign libc "mmap64";

mmap64 :: (__addr: *void, __len: size_t, __prot: s32, __flags: s32, __fd: s32, __offset: s64) -> *void #foreign libc;

mprotect :: (__addr: *void, __len: size_t, __prot: s32) -> s32 #foreign libc;

mremap :: (__addr: *void, __old_len: size_t, __new_len: size_t, __flags: s32, __args: ..Any) -> *void #foreign libc;

msync :: (__addr: *void, __len: size_t, __flags: s32) -> s32 #foreign libc;

munlock :: (__addr: *void, __len: size_t) -> s32 #foreign libc;

munmap :: (__addr: *void, __len: size_t) -> s32 #foreign libc;

nanosleep :: (__requested_time: *timespec, __remaining: *timespec) -> s32 #foreign libc "__nanosleep64";

open :: (__file: *u8, __oflag: s32, __args: ..Any) -> s32 #foreign libc "open64";

open_memstream :: (__bufloc: *void, __sizeloc: *size_t) -> *FILE #foreign libc;

openat :: (__fd: s32, __file: *u8, __oflag: s32, __args: ..Any) -> s32 #foreign libc "openat64";

pathconf :: (__path: *u8, __name: PC) -> s32 #foreign libc;

posix_fadvise :: (__fd: s32, __offset: s64, __len: s64, __advise: s32) -> s32 #foreign libc "posix_fadvise64";

posix_fallocate :: (__fd: s32, __offset: s64, __len: s64) -> s32 #foreign libc "posix_fallocate64";

posix_madvise :: (__addr: *void, __len: size_t, __advice: s32) -> s32 #foreign libc;

ppoll :: (__fds: *pollfd, __nfds: nfds_t, __timeout: *timespec, __ss: *__sigset_t) -> s32 #foreign libc;

pread :: (__fd: s32, __buf: *void, __nbytes: size_t, __offset: s64) -> ssize_t #foreign libc "pread64";

pread64 :: (__fd: s32, __buf: *void, __nbytes: size_t, __offset: s64) -> ssize_t #foreign libc;

preadv :: (__fd: s32, __iovec: *iovec, __count: s32, __offset: s64) -> ssize_t #foreign libc "preadv64";

preadv2 :: (__fd: s32, __iovec: *iovec, __count: s32, __offset: s64, __flags: s32) -> ssize_t #foreign libc "preadv64v2";

preadv64 :: (__fd: s32, __iovec: *iovec, __count: s32, __offset: s64) -> ssize_t #foreign libc;

preadv64v2 :: (__fp: s32, __iovec: *iovec, __count: s32, __offset: s64, ___flags: s32) -> ssize_t #foreign libc;

prlimit :: (__pid: s32, __resource: __rlimit_resource, __new_limit: *rlimit, __old_limit: *rlimit) -> s32 #foreign libc "prlimit64";

prlimit64 :: (__pid: s32, __resource: __rlimit_resource, __new_limit: *rlimit64, __old_limit: *rlimit64) -> s32 #foreign libc;

profil :: (__sample_buffer: *u16, __size: size_t, __offset: size_t, __scale: u32) -> s32 #foreign libc;

pthread_attr_getaffinity_np :: (__attr: *pthread_attr_t, __cpusetsize: size_t, __cpuset: *cpu_set_t) -> s32 #foreign libpthread;

pthread_attr_getguardsize :: (__attr: *pthread_attr_t, __guardsize: *size_t) -> s32 #foreign libpthread;

pthread_attr_getstack :: (__attr: *pthread_attr_t, __stackaddr: *void, __stacksize: *size_t) -> s32 #foreign libpthread;

pthread_attr_getstackaddr :: (__attr: *pthread_attr_t, __stackaddr: *void) -> s32 #foreign libpthread;

pthread_attr_getstacksize :: (__attr: *pthread_attr_t, __stacksize: *size_t) -> s32 #foreign libpthread;

pthread_attr_setaffinity_np :: (__attr: *pthread_attr_t, __cpusetsize: size_t, __cpuset: *cpu_set_t) -> s32 #foreign libpthread;

pthread_attr_setguardsize :: (__attr: *pthread_attr_t, __guardsize: size_t) -> s32 #foreign libpthread;

pthread_attr_setstack :: (__attr: *pthread_attr_t, __stackaddr: *void, __stacksize: size_t) -> s32 #foreign libpthread;

pthread_attr_setstacksize :: (__attr: *pthread_attr_t, __stacksize: size_t) -> s32 #foreign libpthread;

pthread_clockjoin_np :: (__th: pthread_t, __thread_return: *void, __clockid: clockid_t, __abstime: *timespec) -> s32 #foreign libpthread "__pthread_clockjoin_np64";

pthread_cond_clockwait :: (__cond: *pthread_cond_t, __mutex: *pthread_mutex_t, __clock_id: s32, __abstime: *timespec) -> s32 #foreign libpthread "__pthread_cond_clockwait64";

pthread_cond_timedwait :: (__cond: *pthread_cond_t, __mutex: *pthread_mutex_t, __abstime: *timespec) -> s32 #foreign libpthread "__pthread_cond_timedwait64";

pthread_getaffinity_np :: (__th: pthread_t, __cpusetsize: size_t, __cpuset: *cpu_set_t) -> s32 #foreign libpthread;

pthread_getname_np :: (__target_thread: pthread_t, __buf: *u8, __buflen: size_t) -> s32 #foreign libpthread;

pthread_join :: (__th: pthread_t, __thread_return: *void) -> s32 #foreign libpthread;

pthread_mutex_clocklock :: (__mutex: *pthread_mutex_t, __clockid: clockid_t, __abstime: *timespec) -> s32 #foreign libpthread "__pthread_mutex_clocklock64";

pthread_mutex_timedlock :: (__mutex: *pthread_mutex_t, __abstime: *timespec) -> s32 #foreign libpthread "__pthread_mutex_timedlock64";

pthread_rwlock_clockrdlock :: (__rwlock: *pthread_rwlock_t, __clockid: clockid_t, __abstime: *timespec) -> s32 #foreign libpthread "__pthread_rwlock_clockrdlock64";

pthread_rwlock_clockwrlock :: (__rwlock: *pthread_rwlock_t, __clockid: clockid_t, __abstime: *timespec) -> s32 #foreign libpthread "__pthread_rwlock_clockwrlock64";

pthread_rwlock_timedrdlock :: (__rwlock: *pthread_rwlock_t, __abstime: *timespec) -> s32 #foreign libpthread "__pthread_rwlock_timedrdlock64";

pthread_rwlock_timedwrlock :: (__rwlock: *pthread_rwlock_t, __abstime: *timespec) -> s32 #foreign libpthread "__pthread_rwlock_timedwrlock64";

pthread_setaffinity_np :: (__th: pthread_t, __cpusetsize: size_t, __cpuset: *cpu_set_t) -> s32 #foreign libpthread;

pthread_timedjoin_np :: (__th: pthread_t, __thread_return: *void, __abstime: *timespec) -> s32 #foreign libpthread "__pthread_timedjoin_np64";

pthread_tryjoin_np :: (__th: pthread_t, __thread_return: *void) -> s32 #foreign libpthread;

pwrite :: (__fd: s32, __buf: *void, __nbytes: size_t, __offset: s64) -> ssize_t #foreign libc "pwrite64";

pwrite64 :: (__fd: s32, __buf: *void, __n: size_t, __offset: s64) -> ssize_t #foreign libc;

pwritev :: (__fd: s32, __iovec: *iovec, __count: s32, __offset: s64) -> ssize_t #foreign libc "pwritev64";

pwritev2 :: (__fd: s32, __iovec: *iovec, __count: s32, __offset: s64, __flags: s32) -> ssize_t #foreign libc "pwritev64v2";

pwritev64 :: (__fd: s32, __iovec: *iovec, __count: s32, __offset: s64) -> ssize_t #foreign libc;

pwritev64v2 :: (__fd: s32, __iodev: *iovec, __count: s32, __offset: s64, __flags: s32) -> ssize_t #foreign libc;

read :: (__fd: s32, __buf: *void, __nbytes: size_t) -> ssize_t #foreign libc;

readahead :: (__fd: s32, __offset: s64, __count: size_t) -> s32 #foreign libc;

readdir :: (__dirp: *DIR) -> *dirent #foreign libc "readdir64";

readdir64_r :: (__dirp: *DIR, __entry: *dirent64, __result: *void) -> s32 #foreign libc;

readdir_r :: (__dirp: *DIR, __entry: *dirent, __result: *void) -> s32 #foreign libc "readdir64_r";

readlink :: (__path: *u8, __buf: *u8, __len: size_t) -> ssize_t #foreign libc;

readlinkat :: (__fd: s32, __path: *u8, __buf: *u8, __len: size_t) -> ssize_t #foreign libc;

readv :: (__fd: s32, __iovec: *iovec, __count: s32) -> ssize_t #foreign libc;

remap_file_pages :: (__start: *void, __size: size_t, __prot: s32, __pgoff: size_t, __flags: s32) -> s32 #foreign libc;

sbrk :: (__delta: s32) -> *void #foreign libc;

scandir :: (__dir: *u8, __namelist: *void, __selector: #type (unknown0: *dirent) -> s32 #c_call, __cmp: #type (unknown0: *void, unknown1: *void) -> s32 #c_call) -> s32 #foreign libc "scandir64";

scandir64 :: (__dir: *u8, __namelist: *void, __selector: #type (unknown0: *dirent64) -> s32 #c_call, __cmp: #type (unknown0: *void, unknown1: *void) -> s32 #c_call) -> s32 #foreign libc;

scandirat :: (__dfd: s32, __dir: *u8, __namelist: *void, __selector: #type (unknown0: *dirent) -> s32 #c_call, __cmp: #type (unknown0: *void, unknown1: *void) -> s32 #c_call) -> s32 #foreign libc "scandirat64";

scandirat64 :: (__dfd: s32, __dir: *u8, __namelist: *void, __selector: #type (unknown0: *dirent64) -> s32 #c_call, __cmp: #type (unknown0: *void, unknown1: *void) -> s32 #c_call) -> s32 #foreign libc;

sched_getaffinity :: (__pid: s32, __cpusetsize: size_t, __cpuset: *cpu_set_t) -> s32 #foreign libpthread;

sched_rr_get_interval :: (__pid: s32, __t: *timespec) -> s32 #foreign libpthread "__sched_rr_get_interval64";

sched_setaffinity :: (__pid: s32, __cpusetsize: size_t, __cpuset: *cpu_set_t) -> s32 #foreign libpthread;

seekdir :: (__dirp: *DIR, __pos: s32) -> void #foreign libc;

sem_clockwait :: (__sem: *sem_t, clock: clockid_t, __abstime: *timespec) -> s32 #foreign libpthread "__sem_clockwait64";

sem_timedwait :: (__sem: *sem_t, __abstime: *timespec) -> s32 #foreign libpthread "__sem_timedwait64";

setbuffer :: (__stream: *FILE, __buf: *u8, __size: size_t) -> void #foreign libc;

setdomainname :: (__name: *u8, __len: size_t) -> s32 #foreign libc;

sethostid :: (__id: s32) -> s32 #foreign libc;

sethostname :: (__name: *u8, __len: size_t) -> s32 #foreign libc;

setitimer :: (__which: __itimer_which, __new: *itimerval, __old: *itimerval) -> s32 #foreign libc "__setitimer64";

setjmp :: (unknown0: *__jmp_buf_tag) -> s32 #foreign libpthread;

setpriority :: (__which: __priority_which, __who: id_t, __prio: s32) -> s32 #foreign libc;

setrlimit :: (__resource: __rlimit_resource, __rlimits: *rlimit) -> s32 #foreign libc "setrlimit64";

setrlimit64 :: (__resource: __rlimit_resource, __rlimits: *rlimit64) -> s32 #foreign libc;

settimeofday :: (__tv: *timeval, __tz: *timezone) -> s32 #foreign libc "__settimeofday64";

setvbuf :: (__stream: *FILE, __buf: *u8, __modes: s32, __n: size_t) -> s32 #foreign libc;

shmctl :: (__shmid: s32, __cmd: s32, __buf: *shmid_ds) -> s32 #foreign libc "__shmctl64";

shmget :: (__key: key_t, __size: size_t, __shmflg: s32) -> s32 #foreign libc;

siglongjmp :: (unknown0: *__jmp_buf_tag, unknown1: s32) -> void #foreign libpthread;

signal :: (__sig: s32, __handler: #type (unknown0: s32) -> void #c_call) -> ((unknown0: s32) -> void #c_call) #foreign libc;

sigpause :: (__sig: s32) -> s32 #foreign libc "__xpg_sigpause";

sigset :: (__sig: s32, __disp: #type (unknown0: s32) -> void #c_call) -> ((unknown0: s32) -> void #c_call) #foreign libc;

sigtimedwait :: (__set: *sigset_t, __info: *siginfo_t, __timeout: *timespec) -> s32 #foreign libc "__sigtimedwait64";

snprintf :: (unknown0: *u8, unknown1: u32, unknown2: *u8, __args: ..Any) -> s32 #foreign libc;

splice :: (__fdin: s32, __offin: *s64, __fdout: s32, __offout: *s64, __len: size_t, __flags: u32) -> s32 #foreign libc;

ssignal :: (__sig: s32, __handler: #type (unknown0: s32) -> void #c_call) -> ((unknown0: s32) -> void #c_call) #foreign libc;

strftime :: (__s: *u8, __maxsize: size_t, __format: *u8, __tp: *tm) -> size_t #foreign libc;

strftime_l :: (__s: *u8, __maxsize: size_t, __format: *u8, __tp: *tm, __loc: locale_t) -> size_t #foreign libc;

swab :: (__from: *void, __to: *void, __n: ssize_t) -> void #foreign libc;

syscall :: (__sysno: s32, __args: ..Any) -> s32 #foreign libc;

sysconf :: (__name: _SC_definitions) -> s32 #foreign libc;

sysv_signal :: (__sig: s32, __handler: #type (unknown0: s32) -> void #c_call) -> ((unknown0: s32) -> void #c_call) #foreign libc;

tee :: (__fdin: s32, __fdout: s32, __len: size_t, __flags: u32) -> s32 #foreign libc;

telldir :: (__dirp: *DIR) -> s32 #foreign libc;

time :: (__timer: *time_t) -> time_t #foreign libc "__time64";

timegm :: (__tp: *tm) -> time_t #foreign libc "__timegm64";

timelocal :: (__tp: *tm) -> time_t #foreign libc "__mktime64";

timer_gettime :: (__timerid: timer_t, __value: *itimerspec) -> s32 #foreign librt "__timer_gettime64";

timer_settime :: (__timerid: timer_t, __flags: s32, __value: *itimerspec, __ovalue: *itimerspec) -> s32 #foreign librt "__timer_settime64";

timespec_get :: (__ts: *timespec, __base: s32) -> s32 #foreign libc "__timespec_get64";

tmpfile :: () -> *FILE #foreign libc "tmpfile64";

truncate :: (__file: *u8, __length: s64) -> s32 #foreign libc "truncate64";

ttyname_r :: (__fd: s32, __buf: *u8, __buflen: size_t) -> s32 #foreign libc;

utimensat :: (fd: s32, __path: *u8, __times: *[2] timespec, flags: s32) -> s32 #foreign libc "__utimensat64";

utimes :: (__file: *u8, __tvp: *[2] timeval) -> s32 #foreign libc "__utimes64";

versionsort :: (__e1: *void, __e2: *void) -> s32 #foreign libc "versionsort64";

versionsort64 :: (__e1: *void, __e2: *void) -> s32 #foreign libc;

vmsplice :: (__fdout: s32, __iov: *iovec, __count: size_t, __flags: u32) -> s32 #foreign libc;

wait3 :: (__stat_loc: *s32, __options: s32, __usage: *rusage) -> s32 #foreign libc "__wait3_time64";

wait4 :: (__pid: s32, __stat_loc: *s32, __options: s32, __usage: *rusage) -> s32 #foreign libc "__wait4_time64";

write :: (__fd: s32, __buf: *void, __n: size_t) -> ssize_t #foreign libc;

writev :: (__fd: s32, __iovec: *iovec, __count: s32) -> ssize_t #foreign libc;
