// Generated from docs/specs/stdlib/modules/posix.md.
// Regenerate with tools/native_bindings/generate.py; do not hand-edit.

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

// Types and layouts

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    __fsid_t :: struct {
        __val: [2] s32;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    __jmp_buf_tag :: struct {
        __jmpbuf: __jmp_buf;
        __mask_was_saved: s32;
        __saved_mask: __sigset_t;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    __locale_struct :: struct {
        __locale_data :: struct {}
        __locales: [13] *__locale_data;

        __ctype_b: *u16;
        __ctype_tolower: *s32;
        __ctype_toupper: *s32;

        __names: [13] *u8;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    __mbstate_t :: struct {
        __count: s32;
        __value: struct {
            __wch: u32;

            #overlay (__wch) __wchb: [4] u8;
        };
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    __pthread_cleanup_frame :: struct {
        __cancel_routine: #type (unknown0: *void) -> void #c_call;
        __cancel_arg: *void;
        __do_it: s32;
        __cancel_type: s32;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    __pthread_cond_s :: struct {
        struct {
            __wseq: u64;

            #overlay (__wseq) __wseq32: struct {
                __low: u32;
                __high: u32;
            };
        }

        struct {
            __g1_start: u64;

            #overlay (__g1_start) __g1_start32: struct {
                __low: u32;
                __high: u32;
            };
        }

        __g_refs: [2] u32;
        __g_size: [2] u32;
        __g1_orig_size: u32;
        __wrefs: u32;
        __g_signals: [2] u32;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    __pthread_internal_list :: struct {
        __prev: *__pthread_internal_list;
        __next: *__pthread_internal_list;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    __pthread_internal_slist :: struct {
        __next: *__pthread_internal_slist;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    __pthread_mutex_s :: struct {
        __lock: s32;
        __count: u32;
        __owner: s32;

        __nusers: u32;

        __kind: s32;

        #if CPU == .ARM64 {
            __spins: s32;
        } else {
            __spins: s16;
            __elision: s16;
        }
        __list: __pthread_internal_list;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    __pthread_rwlock_arch_t :: struct {
        __readers: u32;
        __writers: u32;
        __wrphase_futex: u32;
        __writers_futex: u32;
        __pad3: u32;
        __pad4: u32;

        __cur_writer: s32;
        __shared: s32;
        __rwelision: s8;

        __pad1: [7] u8;

        __pad2: u64;

        __flags: u32;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    __pthread_unwind_buf_t :: struct {
        __cancel_jmp_buf: [1] struct {
            __cancel_jmp_buf: __jmp_buf;
            __mask_was_saved: s32;
        };

        __pad: [4] *void;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    __sigset_t :: struct {
        __val: [16] u64;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    _fpstate :: struct {
        cwd: u16;
        swd: u16;
        ftw: u16;
        fop: u16;
        rip: u64;
        rdp: u64;
        mxcsr: u32;
        mxcr_mask: u32;
        _st: [8] _fpxreg;
        _xmm: [16] _xmmreg;
        __glibc_reserved1: [24] u32;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    _fpxreg :: struct {
        significand: [4] u16;
        exponent: u16;
        __glibc_reserved1: [3] u16;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    _G_fpos64_t :: struct {
        __pos: s64;
        __state: __mbstate_t;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    _G_fpos_t :: struct {
        __pos: s64;
        __state: __mbstate_t;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    _IO_cookie_io_functions_t :: struct {
        read: cookie_read_function_t;
        write: cookie_write_function_t;
        seek: cookie_seek_function_t;
        close: cookie_close_function_t;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    _IO_FILE :: struct {}

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    _libc_fpstate :: struct {
        cwd: u16;
        swd: u16;
        ftw: u16;
        fop: u16;
        rip: u64;
        rdp: u64;
        mxcsr: u32;
        mxcr_mask: u32;
        _st: [8] _libc_fpxreg;
        _xmm: [16] _libc_xmmreg;
        __glibc_reserved1: [24] u32;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    _libc_fpxreg :: struct {
        significand: [4] u16;
        exponent: u16;
        __glibc_reserved1: [3] u16;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    _libc_xmmreg :: struct {
        element: [4] u32;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    _pthread_cleanup_buffer :: struct {
        __routine: #type (unknown0: *void) -> void #c_call;
        __arg: *void;
        __canceltype: s32;
        __prev: *_pthread_cleanup_buffer;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    _xmmreg :: struct {
        element: [4] u32;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    aiocb :: struct {
        aio_fildes: s32;
        aio_lio_opcode: s32;
        aio_reqprio: s32;
        aio_buf: *void;
        aio_nbytes: u64;
        aio_sigevent: sigevent;

        __next_prio: *aiocb;
        __abs_prio: s32;
        __policy: s32;
        __error_code: s32;
        __return_value: s64;

        aio_offset: s64;
        __pad: [0] u8;

        __glibc_reserved: [32] u8;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    aiocb64 :: struct {
        aio_fildes: s32;
        aio_lio_opcode: s32;
        aio_reqprio: s32;
        aio_buf: *void;
        aio_nbytes: u64;
        aio_sigevent: sigevent;

        __next_prio: *aiocb;
        __abs_prio: s32;
        __policy: s32;
        __error_code: s32;
        __return_value: s64;

        aio_offset: s64;
        __glibc_reserved: [32] u8;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
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

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    cpu_set_t :: struct {
        __bits: [16] __cpu_mask;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    dirent :: struct {
        d_ino: u64;
        d_off: s64;

        d_reclen: u16;
        d_type: DT;
        d_name: [256] u8;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    dirent64 :: struct {
        d_ino: u64;
        d_off: s64;
        d_reclen: u16;
        d_type: DT;
        d_name: [256] u8;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    Dl_info :: struct {
        dli_fname: *u8;
        dli_fbase: *void;
        dli_sname: *u8;
        dli_saddr: *void;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    Dl_serinfo :: struct {
        dls_size: u64;
        dls_cnt: u32;

        struct {
            __dls_serpath_pad: [1] Dl_serpath;

            #overlay (__dls_serpath_pad) dls_serpath: [0] Dl_serpath;
        }
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    Dl_serpath :: struct {
        dls_name: *u8;
        dls_flags: u32;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    f_owner_ex :: struct {
        type: __pid_type;
        pid: s32;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    file_handle :: struct {
        handle_bytes: u32;
        handle_type: s32;

        f_handle: [0] u8;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    flock :: struct {
        l_type: s16;
        l_whence: s16;

        l_start: s64;
        l_len: s64;

        l_pid: s32;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    flock64 :: struct {
        l_type: s16;
        l_whence: s16;
        l_start: s64;
        l_len: s64;
        l_pid: s32;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    iovec :: struct {
        iov_base: *void;
        iov_len: u64;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    ipc_perm :: struct {
        __key: s32;
        uid: u32;
        gid: u32;
        cuid: u32;
        cgid: u32;
        mode: u32;
        __seq: u16;
        __pad2: u16;
        __glibc_reserved1: u64;
        __glibc_reserved2: u64;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    itimerspec :: struct {
        it_interval: timespec;
        it_value: timespec;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    itimerval :: struct {

        it_interval: timeval;

        it_value: timeval;
    }

}

#if OS == .LINUX && CPU == .ARM64 {
    mcontext_t :: struct {
        fault_address: u64;
        regs: [31] u64;
        sp: u64;
        pc: u64;
        pstate: u64;
        __reserved: [4096] u8 #align 16;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    passwd :: struct {
        pw_name: *u8;

        pw_passwd: *u8;

        pw_uid: u32;
        pw_gid: u32;
        pw_gecos: *u8;
        pw_dir: *u8;
        pw_shell: *u8;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    pollfd :: struct {
        fd: s32;
        events: s16;
        revents: s16;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    pthread_attr_t :: struct {
        #if CPU == .ARM64 {
            __size: [64] u8;
        } else {
            __size: [56] u8;
        }

        #overlay (__size) __align: s64;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    pthread_barrier_t :: struct {
        __size: [32] u8;

        #overlay (__size) __align: s64;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    pthread_barrierattr_t :: struct {
        #if CPU == .ARM64 {
            __size: [8] u8;
        } else {
            __size: [4] u8;
        }

        #overlay (__size) __align: s32;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    pthread_cond_t :: struct {
        __data: __pthread_cond_s;

        #overlay (__data) __size: [48] u8;
        #overlay (__data) __align: s64;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    pthread_condattr_t :: struct {
        #if CPU == .ARM64 {
            __size: [8] u8;
        } else {
            __size: [4] u8;
        }

        #overlay (__size) __align: s32;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    pthread_mutex_t :: struct {
        __data: __pthread_mutex_s;

        #if CPU == .ARM64 {
            #overlay (__data) __size: [48] u8;
        } else {
            #overlay (__data) __size: [40] u8;
        }
        #overlay (__data) __align: s64;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    pthread_mutexattr_t :: struct {
        #if CPU == .ARM64 __size: [8] u8;
        else __size: [4] u8;

        #overlay (__size) __align: s32;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    pthread_rwlock_t :: struct {
        __data: __pthread_rwlock_arch_t;

        #overlay (__data) __size: [56] u8;
        #overlay (__data) __align: s64;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    pthread_rwlockattr_t :: struct {
        __size: [8] u8;

        #overlay (__size) __align: s64;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    rlimit :: struct {
        rlim_cur: rlim_t;

        rlim_max: rlim_t;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    rlimit64 :: struct {
        rlim_cur: rlim64_t;

        rlim_max: rlim64_t;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    rusage :: struct {
        ru_utime: timeval;

        ru_stime: timeval;

        struct {
            ru_maxrss: s64;

            #overlay (ru_maxrss) __ru_maxrss_word: s64;
        }

        struct {
            ru_ixrss: s64;

            #overlay (ru_ixrss) __ru_ixrss_word: s64;
        }

        struct {
            ru_idrss: s64;

            #overlay (ru_idrss) __ru_idrss_word: s64;
        }

        struct {
            ru_isrss: s64;

            #overlay (ru_isrss) __ru_isrss_word: s64;
        }

        struct {
            ru_minflt: s64;

            #overlay (ru_minflt) __ru_minflt_word: s64;
        }

        struct {
            ru_majflt: s64;

            #overlay (ru_majflt) __ru_majflt_word: s64;
        }

        struct {
            ru_nswap: s64;

            #overlay (ru_nswap) __ru_nswap_word: s64;
        }

        struct {
            ru_inblock: s64;

            #overlay (ru_inblock) __ru_inblock_word: s64;
        }

        struct {
            ru_oublock: s64;

            #overlay (ru_oublock) __ru_oublock_word: s64;
        }

        struct {
            ru_msgsnd: s64;

            #overlay (ru_msgsnd) __ru_msgsnd_word: s64;
        }

        struct {
            ru_msgrcv: s64;

            #overlay (ru_msgrcv) __ru_msgrcv_word: s64;
        }

        struct {
            ru_nsignals: s64;

            #overlay (ru_nsignals) __ru_nsignals_word: s64;
        }

        struct {
            ru_nvcsw: s64;

            #overlay (ru_nvcsw) __ru_nvcsw_word: s64;
        }

        struct {
            ru_nivcsw: s64;

            #overlay (ru_nivcsw) __ru_nivcsw_word: s64;
        }
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    sched_param :: struct {
        sched_priority: s32;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    sem_t :: struct {
        __size: [32] u8;

        #overlay (__size) __align: s64;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    shm_info :: struct {
        used_ids: s32;
        shm_tot: u64;
        shm_rss: u64;
        shm_swp: u64;
        swap_attempts: u64;
        swap_successes: u64;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    shmid_ds :: struct {
        shm_perm: ipc_perm;

        shm_segsz: u64;

        shm_atime: s64;
        shm_dtime: s64;
        shm_ctime: s64;

        shm_cpid: s32;
        shm_lpid: s32;
        shm_nattch: shmatt_t;
        __glibc_reserved5: u64;
        __glibc_reserved6: u64;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    shminfo :: struct {
        shmmax: u64;
        shmmin: u64;
        shmmni: u64;
        shmseg: u64;
        shmall: u64;
        __glibc_reserved1: u64;
        __glibc_reserved2: u64;
        __glibc_reserved3: u64;
        __glibc_reserved4: u64;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    sigaction_t :: struct {
        using __sigaction_handler: struct {
            sa_handler: #type (unknown0: s32) -> void #c_call;

            #overlay (sa_handler) sa_sigaction: #type (unknown0: s32, unknown1: *siginfo_t, unknown2: *void) -> void #c_call;
        };

        sa_mask: __sigset_t;

        sa_flags: s32;

        sa_restorer: #type () -> void #c_call;
    }

}

#if OS == .LINUX && CPU == .ARM64 {
    sigcontext :: struct {
        fault_address: u64;
        regs: [31] u64;
        sp: u64;
        pc: u64;
        pstate: u64;
        __reserved: [4096] u8 #align 16;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    sigevent :: struct {
        sigev_value: sigval;
        sigev_signo: s32;
        sigev_notify: s32;

        _sigev_un: struct {
            _pad: [12] s32;

            #overlay (_pad) _tid: s32;
            #overlay (_pad) _sigev_thread: struct {
                _function: #type (unknown0: sigval) -> void #c_call;
                _attribute: *pthread_attr_t;
            };
        };
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    siginfo_t :: struct {
        si_signo: s32;

        si_errno: s32;

        si_code: s32;

        __pad0: s32;

        _sifields: struct {
            _pad: [28] s32;

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
                si_utime: s64;
                si_stime: s64;
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
                si_band: s64;
                si_fd: s32;
            };
            #overlay (_pad) _sigsys: struct {
                _call_addr: *void;
                _syscall: s32;
                _arch: u32;
            };
        };
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
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

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    sigval :: struct {
        sival_ptr: *void;

        #overlay (sival_ptr) sival_int: s32;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    stack_t :: struct {
        ss_sp: *void;
        ss_flags: s32;
        ss_size: u64;
    }

}

#if OS == .LINUX && CPU == .ARM64 {
    stat64_t :: struct {
        st_dev: u64;
        st_ino: u64;
        st_mode: u32;
        st_nlink: u32;
        st_uid: u32;
        st_gid: u32;
        st_rdev: u64;
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

}

#if OS == .LINUX && CPU == .ARM64 {
    stat_t :: struct {
        st_dev: u64;
        st_ino: u64;
        st_mode: u32;
        st_nlink: u32;
        st_uid: u32;
        st_gid: u32;
        st_rdev: u64;
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

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
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

        __spare2: [14] u64;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    statx_timestamp :: struct {
        tv_sec: s64;
        tv_nsec: u32;
        __reserved: s32;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    timespec :: struct {
        tv_sec: s64;

        tv_nsec: s64;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    timeval :: struct {
        tv_sec: s64;
        tv_usec: s64;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    timex :: struct {
        modes: u32;
        offset: s64;
        freq: s64;
        maxerror: s64;
        esterror: s64;
        status: s32;
        constant: s64;
        precision: s64;
        tolerance: s64;
        time: timeval;
        tick: s64;
        ppsfreq: s64;
        jitter: s64;
        shift: s32;
        stabil: s64;
        jitcnt: s64;
        calcnt: s64;
        errcnt: s64;
        stbcnt: s64;

        tai: s32;

        __bitfield: [44] u8;

    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    timezone :: struct {
        tz_minuteswest: s32;
        tz_dsttime: s32;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
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

#if OS == .LINUX && CPU == .ARM64 {
    ucontext_t :: struct {
        uc_flags: u64;
        uc_link: *ucontext_t;
        uc_stack: stack_t;
        uc_sigmask: sigset_t;
        uc_mcontext: mcontext_t;
    }

}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    utsname :: struct {

        sysname: [65] u8;

        nodename: [65] u8;

        release: [65] u8;

        version: [65] u8;

        machine: [65] u8;

        domainname: [65] u8;
    }

}

// Callback and opaque types

OS_Error_Code :: #type,isa s32;

// Aliases and constants

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    __cpu_mask :: u64;
}

#if OS == .LINUX && CPU == .ARM64 {
    __jmp_buf :: [22] s64;
}

#if (OS == .LINUX) && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    __NGREG :: 23;
}

#if OS == .LINUX && CPU == .ARM64 {
    __O_DIRECT :: 0x10000;
}

#if OS == .LINUX && CPU == .ARM64 {
    __O_DIRECTORY :: 0x4000;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    __O_LARGEFILE :: 0;
}

#if OS == .LINUX && CPU == .ARM64 {
    __O_NOFOLLOW :: 0x8000;
}

#if (OS == .LINUX) && ARCHITECTURE != .S390X {
    __POSIX_FADV_DONTNEED :: 4;
}

#if (OS == .LINUX) && ARCHITECTURE != .S390X {
    __POSIX_FADV_NOREUSE :: 5;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    __PTHREAD_MUTEX_HAVE_PREV :: 1;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    _DIRENT_MATCHES_DIRENT64 :: 1;
}

#if OS == .LINUX && CPU == .ARM64 {
    blksize_t :: s32;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    clock_t :: s64;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 {
    F_GETLK :: 5;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 {
    F_GETLK64 :: 5;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 {
    F_SETLK :: 6;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 {
    F_SETLK64 :: 6;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 {
    F_SETLKW :: 7;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 {
    F_SETLKW64 :: 7;
}

#if (OS == .LINUX) && ARCHITECTURE != .POWERPC64 {
    FIOASYNC :: 0x5452;
}

#if (OS == .LINUX) && ARCHITECTURE != .POWERPC64 {
    FIOCLEX :: 0x5451;
}

#if (OS == .LINUX) && ARCHITECTURE != .POWERPC64 {
    FIONBIO :: 0x5421;
}

#if (OS == .LINUX) && ARCHITECTURE != .POWERPC64 {
    FIONCLEX :: 0x5450;
}

#if (OS == .LINUX) && ARCHITECTURE != .POWERPC64 {
    FIONREAD :: 0x541B;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    FIOQSIZE :: 0x5460;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    fpos64_t :: _G_fpos64_t;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    fpos_t :: _G_fpos_t;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    fpregset_t :: *_libc_fpstate;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .S390X {
    greg_t :: s64;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    gregset_t :: [23] greg_t;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    Lmid_t :: s64;
}

#if OS == .ANDROID {
    MAP_FAILED :: cast,trunc(*void) -1;
}

#if (OS == .LINUX) && ARCHITECTURE != .POWERPC64 {
    MAP_LOCKED :: 0x02000;
}

#if (OS == .LINUX) && ARCHITECTURE != .POWERPC64 {
    MAP_NORESERVE :: 0x04000;
}

#if (OS == .LINUX) && ARCHITECTURE != .POWERPC64 {
    MCL_CURRENT :: 1;
}

#if (OS == .LINUX) && ARCHITECTURE != .POWERPC64 {
    MCL_FUTURE :: 2;
}

#if (OS == .LINUX) && ARCHITECTURE != .POWERPC64 {
    MCL_ONFAULT :: 4;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    nfds_t :: u64;
}

#if OS == .LINUX && CPU == .ARM64 {
    nlink_t :: u32;
}

off_t :: s64;

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    pthread_t :: u64;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    register_t :: s64;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    shmatt_t :: u64;
}

#if ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    size_t :: u64;
}

#if ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    ssize_t :: s64;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    suseconds_t :: s64;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_accept :: 202;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_accept4 :: 242;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_acct :: 89;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_add_key :: 217;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_adjtimex :: 171;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_arch_specific_syscall :: 244;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_bind :: 200;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_bpf :: 280;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_brk :: 214;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_capget :: 90;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_capset :: 91;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_chdir :: 49;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_chroot :: 51;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_clock_adjtime :: 266;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_clock_getres :: 114;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_clock_gettime :: 113;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_clock_nanosleep :: 115;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_clock_settime :: 112;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_clone :: 220;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_clone3 :: 435;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_close :: 57;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_close_range :: 436;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_connect :: 203;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_copy_file_range :: 285;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_delete_module :: 106;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_dup :: 23;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_dup3 :: 24;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_epoll_create1 :: 20;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_epoll_ctl :: 21;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_epoll_pwait :: 22;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_epoll_pwait2 :: 441;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_eventfd2 :: 19;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_execve :: 221;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_execveat :: 281;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_exit :: 93;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_exit_group :: 94;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_faccessat :: 48;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_faccessat2 :: 439;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_fadvise64 :: 223;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_fallocate :: 47;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_fanotify_init :: 262;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_fanotify_mark :: 263;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_fchdir :: 50;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_fchmod :: 52;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_fchmodat :: 53;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_fchown :: 55;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_fchownat :: 54;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_fcntl :: 25;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_fdatasync :: 83;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_fgetxattr :: 10;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_finit_module :: 273;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_flistxattr :: 13;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_flock :: 32;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_fremovexattr :: 16;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_fsconfig :: 431;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_fsetxattr :: 7;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_fsmount :: 432;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_fsopen :: 430;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_fspick :: 433;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_fstat :: 80;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_fstatfs :: 44;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_fsync :: 82;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_ftruncate :: 46;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_futex :: 98;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_futex_waitv :: 449;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_get_mempolicy :: 236;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_get_robust_list :: 100;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_getcpu :: 168;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_getcwd :: 17;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_getdents64 :: 61;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_getegid :: 177;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_geteuid :: 175;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_getgid :: 176;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_getgroups :: 158;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_getitimer :: 102;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_getpeername :: 205;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_getpgid :: 155;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_getpid :: 172;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_getppid :: 173;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_getpriority :: 141;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_getrandom :: 278;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_getresgid :: 150;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_getresuid :: 148;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_getrlimit :: 163;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_getrusage :: 165;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_getsid :: 156;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_getsockname :: 204;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_getsockopt :: 209;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_gettid :: 178;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_gettimeofday :: 169;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_getuid :: 174;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_getxattr :: 8;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_init_module :: 105;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_inotify_add_watch :: 27;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_inotify_init1 :: 26;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_inotify_rm_watch :: 28;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_io_cancel :: 3;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_io_destroy :: 1;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_io_getevents :: 4;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_io_pgetevents :: 292;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_io_setup :: 0;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_io_submit :: 2;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_io_uring_enter :: 426;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_io_uring_register :: 427;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_io_uring_setup :: 425;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_ioctl :: 29;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_ioprio_get :: 31;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_ioprio_set :: 30;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_kcmp :: 272;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_kexec_file_load :: 294;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_kexec_load :: 104;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_keyctl :: 219;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_kill :: 129;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_landlock_add_rule :: 445;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_landlock_create_ruleset :: 444;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_landlock_restrict_self :: 446;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_lgetxattr :: 9;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_linkat :: 37;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_listen :: 201;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_listxattr :: 11;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_llistxattr :: 12;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_lookup_dcookie :: 18;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_lremovexattr :: 15;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_lseek :: 62;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_lsetxattr :: 6;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_madvise :: 233;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_mbind :: 235;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_membarrier :: 283;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_memfd_create :: 279;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_memfd_secret :: 447;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_migrate_pages :: 238;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_mincore :: 232;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_mkdirat :: 34;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_mknodat :: 33;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_mlock :: 228;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_mlock2 :: 284;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_mlockall :: 230;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_mmap :: 222;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_mount :: 40;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_mount_setattr :: 442;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_move_mount :: 429;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_move_pages :: 239;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_mprotect :: 226;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_mq_getsetattr :: 185;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_mq_notify :: 184;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_mq_open :: 180;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_mq_timedreceive :: 183;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_mq_timedsend :: 182;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_mq_unlink :: 181;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_mremap :: 216;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_msgctl :: 187;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_msgget :: 186;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_msgrcv :: 188;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_msgsnd :: 189;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_msync :: 227;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_munlock :: 229;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_munlockall :: 231;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_munmap :: 215;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_name_to_handle_at :: 264;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_nanosleep :: 101;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_newfstatat :: 79;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_nfsservctl :: 42;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_NMLN :: 65;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_open_by_handle_at :: 265;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_open_tree :: 428;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_openat :: 56;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_openat2 :: 437;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_perf_event_open :: 241;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_personality :: 92;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_pidfd_getfd :: 438;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_pidfd_open :: 434;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_pidfd_send_signal :: 424;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_pipe2 :: 59;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_pivot_root :: 41;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_pkey_alloc :: 289;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_pkey_free :: 290;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_pkey_mprotect :: 288;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_ppoll :: 73;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_prctl :: 167;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_pread64 :: 67;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_preadv :: 69;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_preadv2 :: 286;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_prlimit64 :: 261;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_process_madvise :: 440;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_process_mrelease :: 448;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_process_vm_readv :: 270;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_process_vm_writev :: 271;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_pselect6 :: 72;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_ptrace :: 117;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_pwrite64 :: 68;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_pwritev :: 70;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_pwritev2 :: 287;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_quotactl :: 60;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_quotactl_fd :: 443;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_read :: 63;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_readahead :: 213;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_readlinkat :: 78;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_readv :: 65;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_reboot :: 142;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_recvfrom :: 207;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_recvmmsg :: 243;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_recvmsg :: 212;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_remap_file_pages :: 234;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_removexattr :: 14;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_renameat :: 38;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_renameat2 :: 276;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_request_key :: 218;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_restart_syscall :: 128;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_rseq :: 293;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_rt_sigaction :: 134;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_rt_sigpending :: 136;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_rt_sigprocmask :: 135;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_rt_sigqueueinfo :: 138;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_rt_sigreturn :: 139;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_rt_sigsuspend :: 133;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_rt_sigtimedwait :: 137;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_rt_tgsigqueueinfo :: 240;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_sched_get_priority_max :: 125;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_sched_get_priority_min :: 126;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_sched_getaffinity :: 123;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_sched_getattr :: 275;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_sched_getparam :: 121;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_sched_getscheduler :: 120;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_sched_rr_get_interval :: 127;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_sched_setaffinity :: 122;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_sched_setattr :: 274;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_sched_setparam :: 118;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_sched_setscheduler :: 119;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_sched_yield :: 124;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_SECCOMP :: 1;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_seccomp :: 277;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_semctl :: 191;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_semget :: 190;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_semop :: 193;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_semtimedop :: 192;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_sendfile :: 71;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_sendmmsg :: 269;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_sendmsg :: 211;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_sendto :: 206;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_set_mempolicy :: 237;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_set_mempolicy_home_node :: 450;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_set_robust_list :: 99;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_set_tid_address :: 96;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_setdomainname :: 162;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_setfsgid :: 152;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_setfsuid :: 151;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_setgid :: 144;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_setgroups :: 159;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_sethostname :: 161;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_setitimer :: 103;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_setns :: 268;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_setpgid :: 154;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_setpriority :: 140;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_setregid :: 143;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_setresgid :: 149;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_setresuid :: 147;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_setreuid :: 145;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_setrlimit :: 164;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_setsid :: 157;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_setsockopt :: 208;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_settimeofday :: 170;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_setuid :: 146;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_setxattr :: 5;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_shmat :: 196;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_shmctl :: 195;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_shmdt :: 197;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_shmget :: 194;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_shutdown :: 210;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_sigaltstack :: 132;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_signalfd4 :: 74;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_socket :: 198;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_socketpair :: 199;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_splice :: 76;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_statfs :: 43;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_statx :: 291;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_swapoff :: 225;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_swapon :: 224;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_symlinkat :: 36;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_sync :: 81;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_sync_file_range :: 84;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_syncfs :: 267;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_syscalls :: 451;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_sysinfo :: 179;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_syslog :: 116;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_tee :: 77;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_tgkill :: 131;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_timer_create :: 107;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_timer_delete :: 111;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_timer_getoverrun :: 109;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_timer_gettime :: 108;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_timer_settime :: 110;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_timerfd_create :: 85;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_timerfd_gettime :: 87;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_timerfd_settime :: 86;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_times :: 153;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_tkill :: 130;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_truncate :: 45;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_umask :: 166;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_umount2 :: 39;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_uname :: 160;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_unlinkat :: 35;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_unshare :: 97;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_USER_DISPATCH :: 2;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_userfaultfd :: 282;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_utimensat :: 88;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_vhangup :: 58;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_vmsplice :: 75;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_wait4 :: 260;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_waitid :: 95;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_write :: 64;
}

#if (OS == .ANDROID || OS == .LINUX) && CPU == .ARM64 {
    SYS_writev :: 66;
}

#if OS == .MACOS {
    SYSCALL_MACH_BASE :: 0x1000000;
}

#if OS == .MACOS {
    SYSCALL_UNIX_BASE :: 0x2000000;
}

#if (OS == .LINUX) && ARCHITECTURE != .POWERPC64 {
    TCFLSH :: 0x540B;
}

#if (OS == .LINUX) && ARCHITECTURE != .POWERPC64 {
    TCGETA :: 0x5405;
}

#if (OS == .LINUX) && ARCHITECTURE != .POWERPC64 {
    TCGETS :: 0x5401;
}

#if (OS == .LINUX) && ARCHITECTURE != .POWERPC64 {
    TCSBRK :: 0x5409;
}

#if (OS == .LINUX) && ARCHITECTURE != .POWERPC64 {
    TCSETA :: 0x5406;
}

#if (OS == .LINUX) && ARCHITECTURE != .POWERPC64 {
    TCSETAF :: 0x5408;
}

#if (OS == .LINUX) && ARCHITECTURE != .POWERPC64 {
    TCSETAW :: 0x5407;
}

#if (OS == .LINUX) && ARCHITECTURE != .POWERPC64 {
    TCSETS :: 0x5402;
}

#if (OS == .LINUX) && ARCHITECTURE != .POWERPC64 {
    TCSETSF :: 0x5404;
}

#if (OS == .LINUX) && ARCHITECTURE != .POWERPC64 {
    TCSETSW :: 0x5403;
}

#if (OS == .LINUX) && ARCHITECTURE != .POWERPC64 {
    TCXONC :: 0x540A;
}

#if (OS == .LINUX) && ARCHITECTURE != .POWERPC64 {
    TIOCGPGRP :: 0x540F;
}

#if (OS == .LINUX) && ARCHITECTURE != .POWERPC64 {
    TIOCGWINSZ :: 0x5413;
}

#if (OS == .LINUX) && ARCHITECTURE != .POWERPC64 {
    TIOCOUTQ :: 0x5411;
}

#if (OS == .LINUX) && ARCHITECTURE != .POWERPC64 {
    TIOCSPGRP :: 0x5410;
}

#if (OS == .LINUX) && ARCHITECTURE != .POWERPC64 {
    TIOCSWINSZ :: 0x5414;
}

// Native storage

// Native entry points

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    __asprintf :: (__ptr: **u8, __fmt: *u8, __args: ..Any) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    __errno_location :: () -> *OS_Error_Code #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    __getdelim :: (__lineptr: **u8, __n: *u64, __delimiter: s32, __stream: *FILE) -> s64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    __sched_cpualloc :: (__count: u64) -> *cpu_set_t #foreign libpthread;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    __sched_cpucount :: (__setsize: u64, __setp: *cpu_set_t) -> s32 #foreign libpthread;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    __sysv_signal :: (__sig: s32, __handler: #type (unknown0: s32) -> void #c_call) -> ((unknown0: s32) -> void #c_call) #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    adjtime :: (__delta: *timeval, __olddelta: *timeval) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    aio_cancel :: (__fildes: s32, __aiocbp: *aiocb) -> s32 #foreign librt;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    aio_error :: (__aiocbp: *aiocb) -> s32 #foreign librt;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    aio_fsync :: (__operation: s32, __aiocbp: *aiocb) -> s32 #foreign librt;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    aio_read :: (__aiocbp: *aiocb) -> s32 #foreign librt;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    aio_return :: (__aiocbp: *aiocb) -> s64 #foreign librt;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    aio_return64 :: (__aiocbp: *aiocb64) -> s64 #foreign librt;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    aio_suspend :: (__list: **aiocb, __nent: s32, __timeout: *timespec) -> s32 #foreign librt;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    aio_suspend64 :: (__list: **aiocb64, __nent: s32, __timeout: *timespec) -> s32 #foreign librt;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    aio_write :: (__aiocbp: *aiocb) -> s32 #foreign librt;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    alphasort :: (__e1: **dirent, __e2: **dirent) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    alphasort64 :: (__e1: **dirent64, __e2: **dirent64) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    asprintf :: (__ptr: **u8, __fmt: *u8, __args: ..Any) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    clock_getres :: (__clock_id: clockid_t, __res: *timespec) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    clock_gettime :: (__clock_id: clockid_t, __tp: *timespec) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    clock_nanosleep :: (__clock_id: clockid_t, __flags: s32, __req: *timespec, __rem: *timespec) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    clock_settime :: (__clock_id: clockid_t, __tp: *timespec) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    clone :: (__fn: #type (__arg: *void) -> s32 #c_call, __child_stack: *void, __flags: s32, __arg: *void, __args: ..Any) -> s32 #foreign libpthread;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    confstr :: (__name: s32, __buf: *u8, __len: u64) -> u64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    copy_file_range :: (__infd: s32, __pinoff: *s64, __outfd: s32, __poutoff: *s64, __length: u64, __flags: u32) -> s64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    creat :: (__file: *u8, __mode: mode_t) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    ctime :: (__timer: *time_t) -> *u8 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    ctime_r :: (__timer: *time_t, __buf: *u8) -> *u8 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    difftime :: (__time1: time_t, __time0: time_t) -> float64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    dladdr1 :: (__address: *void, __info: *Dl_info, __extra_info: **void, __flags: s32) -> s32 #foreign libdl;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    execv :: (__path: *u8, __argv: **u8) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    execve :: (__path: *u8, __argv: **u8, __envp: **u8) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    execvp :: (__file: *u8, __argv: **u8) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    execvpe :: (__file: *u8, __argv: **u8, __envp: **u8) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    fallocate :: (__fd: s32, __mode: s32, __offset: s64, __len: s64) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    fcntl :: (__fd: s32, __cmd: s32, __args: ..Any) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    fcntl64 :: (__fd: s32, __cmd: s32, __args: ..Any) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    fexecve :: (__fd: s32, __argv: **u8, __envp: **u8) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    fgetpos :: (__stream: *FILE, __pos: *fpos_t) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    fgetpwent_r :: (__stream: *FILE, __resultbuf: *passwd, __buffer: *u8, __buflen: u64, __result: **passwd) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    fmemopen :: (__s: *void, __len: u64, __modes: *u8) -> *FILE #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    fpathconf :: (__fd: s32, __name: PC) -> s64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    fread :: (__ptr: *void, __size: u64, __n: u64, __stream: *FILE) -> u64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    fread_unlocked :: (__ptr: *void, __size: u64, __n: u64, __stream: *FILE) -> u64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    freopen :: (__filename: *u8, __modes: *u8, __stream: *FILE) -> *FILE #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    fseek :: (__stream: *FILE, __off: s64, __whence: s32) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    fseeko :: (__stream: *FILE, __off: s64, __whence: s32) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    fsetpos :: (__stream: *FILE, __pos: *fpos_t) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    ftell :: (__stream: *FILE) -> s64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    ftello :: (__stream: *FILE) -> s64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    ftruncate :: (__fd: s32, __length: s64) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    futimens :: (__fd: s32, __times: *[2] timespec) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    futimes :: (__fd: s32, __tvp: *[2] timeval) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    futimesat :: (__fd: s32, __file: *u8, __tvp: *[2] timeval) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    fwrite :: (__ptr: *void, __size: u64, __n: u64, __s: *FILE) -> u64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    fwrite_unlocked :: (__ptr: *void, __size: u64, __n: u64, __stream: *FILE) -> u64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    getcwd :: (__buf: *u8, __size: u64) -> *u8 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    getdelim :: (__lineptr: **u8, __n: *u64, __delimiter: s32, __stream: *FILE) -> s64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    getdirentries :: (__fd: s32, __buf: *u8, __nbytes: u64, __basep: *s64) -> s64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    getdirentries64 :: (__fd: s32, __buf: *u8, __nbytes: u64, __basep: *s64) -> s64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    getdomainname :: (__name: *u8, __len: u64) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    getentropy :: (__buffer: *void, __length: u64) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    gethostid :: () -> s64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    gethostname :: (__name: *u8, __len: u64) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    getitimer :: (__which: __itimer_which, __value: *itimerval) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    getline :: (__lineptr: **u8, __n: *u64, __stream: *FILE) -> s64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    getlogin_r :: (__name: *u8, __name_len: u64) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    getpwent_r :: (__resultbuf: *passwd, __buffer: *u8, __buflen: u64, __result: **passwd) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    getpwnam_r :: (__name: *u8, __resultbuf: *passwd, __buffer: *u8, __buflen: u64, __result: **passwd) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    getpwuid_r :: (__uid: u32, __resultbuf: *passwd, __buffer: *u8, __buflen: u64, __result: **passwd) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    getrlimit :: (__resource: RLIMIT_RESOURCE, __rlimits: *rlimit) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    gettimeofday :: (__tv: *timeval, __tz: *void) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    gmtime :: (__timer: *time_t) -> *tm #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    gmtime_r :: (__timer: *time_t, __tp: *tm) -> *tm #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    ioctl :: (__fd: s32, __request: u64, __args: ..Any) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    lio_listio :: (__mode: s32, __list: **aiocb, __nent: s32, __sig: *sigevent) -> s32 #foreign librt;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    lio_listio64 :: (__mode: s32, __list: **aiocb64, __nent: s32, __sig: *sigevent) -> s32 #foreign librt;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    localtime :: (__timer: *time_t) -> *tm #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    localtime_r :: (__timer: *time_t, __tp: *tm) -> *tm #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    lockf :: (__fd: s32, __cmd: s32, __len: s64) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    lseek :: (__fd: s32, __offset: s64, __whence: s32) -> s64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    lutimes :: (__file: *u8, __tvp: *[2] timeval) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    madvise :: (__addr: *void, __len: u64, __advice: s32) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    mincore :: (__start: *void, __len: u64, __vec: *u8) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    mkostemp :: (__template: *u8, __flags: s32) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    mkostemps :: (__template: *u8, __suffixlen: s32, __flags: s32) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    mkstemp :: (__template: *u8) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    mkstemps :: (__template: *u8, __suffixlen: s32) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    mktime :: (__tp: *tm) -> time_t #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    mlock :: (__addr: *void, __len: u64) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    mmap :: (__addr: *void, __len: u64, __prot: s32, __flags: s32, __fd: s32, __offset: s64) -> *void #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    mmap64 :: (__addr: *void, __len: u64, __prot: s32, __flags: s32, __fd: s32, __offset: s64) -> *void #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    mprotect :: (__addr: *void, __len: u64, __prot: s32) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    mremap :: (__addr: *void, __old_len: u64, __new_len: u64, __flags: s32, __args: ..Any) -> *void #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    msync :: (__addr: *void, __len: u64, __flags: s32) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    munlock :: (__addr: *void, __len: u64) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    munmap :: (__addr: *void, __len: u64) -> s32 #foreign libc; @MaybeFreesArgument
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    nanosleep :: (__requested_time: *timespec, __remaining: *timespec) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    open :: (__file: *u8, __oflag: s32, __args: ..Any) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    open_memstream :: (__bufloc: **u8, __sizeloc: *u64) -> *FILE #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    openat :: (__fd: s32, __file: *u8, __oflag: s32, __args: ..Any) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    pathconf :: (__path: *u8, __name: PC) -> s64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    posix_fadvise :: (__fd: s32, __offset: s64, __len: s64, __advise: s32) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    posix_fallocate :: (__fd: s32, __offset: s64, __len: s64) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    posix_madvise :: (__addr: *void, __len: u64, __advice: s32) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    ppoll :: (__fds: *pollfd, __nfds: nfds_t, __timeout: *timespec, __ss: *sigset_t) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    pread :: (__fd: s32, __buf: *void, __nbytes: u64, __offset: s64) -> s64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    pread64 :: (__fd: s32, __buf: *void, __nbytes: u64, __offset: s64) -> s64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    preadv :: (__fd: s32, __iovec: *iovec, __count: s32, __offset: s64) -> s64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    preadv2 :: (__fp: s32, __iovec: *iovec, __count: s32, __offset: s64, ___flags: s32) -> s64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    preadv64 :: (__fd: s32, __iovec: *iovec, __count: s32, __offset: s64) -> s64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    preadv64v2 :: (__fp: s32, __iovec: *iovec, __count: s32, __offset: s64, ___flags: s32) -> s64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    prlimit :: (__pid: s32, __resource: RLIMIT_RESOURCE, __new_limit: *rlimit, __old_limit: *rlimit) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    profil :: (__sample_buffer: *u16, __size: u64, __offset: u64, __scale: u32) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    pthread_attr_getaffinity_np :: (__attr: *pthread_attr_t, __cpusetsize: u64, __cpuset: *cpu_set_t) -> s32 #foreign libpthread;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    pthread_attr_getguardsize :: (__attr: *pthread_attr_t, __guardsize: *u64) -> s32 #foreign libpthread;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    pthread_attr_getstack :: (__attr: *pthread_attr_t, __stackaddr: **void, __stacksize: *u64) -> s32 #foreign libpthread;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    pthread_attr_getstackaddr :: (__attr: *pthread_attr_t, __stackaddr: **void) -> s32 #foreign libpthread;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    pthread_attr_getstacksize :: (__attr: *pthread_attr_t, __stacksize: *u64) -> s32 #foreign libpthread;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    pthread_attr_setaffinity_np :: (__attr: *pthread_attr_t, __cpusetsize: u64, __cpuset: *cpu_set_t) -> s32 #foreign libpthread;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    pthread_attr_setguardsize :: (__attr: *pthread_attr_t, __guardsize: u64) -> s32 #foreign libpthread;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    pthread_attr_setstack :: (__attr: *pthread_attr_t, __stackaddr: *void, __stacksize: u64) -> s32 #foreign libpthread;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    pthread_attr_setstacksize :: (__attr: *pthread_attr_t, __stacksize: u64) -> s32 #foreign libpthread;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    pthread_clockjoin_np :: (__th: pthread_t, __thread_return: **void, __clockid: clockid_t, __abstime: *timespec) -> s32 #foreign libpthread;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    pthread_cond_clockwait :: (__cond: *pthread_cond_t, __mutex: *pthread_mutex_t, __clock_id: s32, __abstime: *timespec) -> s32 #foreign libpthread;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    pthread_cond_timedwait :: (__cond: *pthread_cond_t, __mutex: *pthread_mutex_t, __abstime: *timespec) -> s32 #foreign libpthread;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    pthread_getaffinity_np :: (__th: pthread_t, __cpusetsize: u64, __cpuset: *cpu_set_t) -> s32 #foreign libpthread;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    pthread_getname_np :: (__target_thread: pthread_t, __buf: *u8, __buflen: u64) -> s32 #foreign libpthread;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    pthread_join :: (__th: pthread_t, __thread_return: **void) -> s32 #foreign libpthread;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    pthread_mutex_clocklock :: (__mutex: *pthread_mutex_t, __clockid: clockid_t, __abstime: *timespec) -> s32 #foreign libpthread;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    pthread_mutex_timedlock :: (__mutex: *pthread_mutex_t, __abstime: *timespec) -> s32 #foreign libpthread;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    pthread_rwlock_clockrdlock :: (__rwlock: *pthread_rwlock_t, __clockid: clockid_t, __abstime: *timespec) -> s32 #foreign libpthread;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    pthread_rwlock_clockwrlock :: (__rwlock: *pthread_rwlock_t, __clockid: clockid_t, __abstime: *timespec) -> s32 #foreign libpthread;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    pthread_rwlock_timedrdlock :: (__rwlock: *pthread_rwlock_t, __abstime: *timespec) -> s32 #foreign libpthread;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    pthread_rwlock_timedwrlock :: (__rwlock: *pthread_rwlock_t, __abstime: *timespec) -> s32 #foreign libpthread;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    pthread_setaffinity_np :: (__th: pthread_t, __cpusetsize: u64, __cpuset: *cpu_set_t) -> s32 #foreign libpthread;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    pthread_timedjoin_np :: (__th: pthread_t, __thread_return: **void, __abstime: *timespec) -> s32 #foreign libpthread;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    pthread_tryjoin_np :: (__th: pthread_t, __thread_return: **void) -> s32 #foreign libpthread;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    pwrite :: (__fd: s32, __buf: *void, __n: u64, __offset: s64) -> s64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    pwrite64 :: (__fd: s32, __buf: *void, __n: u64, __offset: s64) -> s64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    pwritev :: (__fd: s32, __iovec: *iovec, __count: s32, __offset: s64) -> s64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    pwritev2 :: (__fd: s32, __iodev: *iovec, __count: s32, __offset: s64, __flags: s32) -> s64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    pwritev64 :: (__fd: s32, __iovec: *iovec, __count: s32, __offset: s64) -> s64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    pwritev64v2 :: (__fd: s32, __iodev: *iovec, __count: s32, __offset: s64, __flags: s32) -> s64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    read :: (__fd: s32, __buf: *void, __nbytes: u64) -> s64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    readahead :: (__fd: s32, __offset: s64, __count: u64) -> s64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    readdir :: (__dirp: *DIR) -> *dirent #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    readdir64_r :: (__dirp: *DIR, __entry: *dirent64, __result: **dirent64) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    readdir_r :: (__dirp: *DIR, __entry: *dirent, __result: **dirent) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    readlink :: (__path: *u8, __buf: *u8, __len: u64) -> s64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    readlinkat :: (__fd: s32, __path: *u8, __buf: *u8, __len: u64) -> s64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    readv :: (__fd: s32, __iovec: *iovec, __count: s32) -> s64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    remap_file_pages :: (__start: *void, __size: u64, __prot: s32, __pgoff: u64, __flags: s32) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    sbrk :: (__delta: s64) -> *void #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    scandir :: (__dir: *u8, __namelist: ***dirent, __selector: #type (unknown0: *dirent) -> s32 #c_call, __cmp: #type (unknown0: **dirent, unknown1: **dirent) -> s32 #c_call) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    scandir64 :: (__dir: *u8, __namelist: ***dirent64, __selector: #type (unknown0: *dirent64) -> s32 #c_call, __cmp: #type (unknown0: **dirent64, unknown1: **dirent64) -> s32 #c_call) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    scandirat :: (__dfd: s32, __dir: *u8, __namelist: ***dirent, __selector: #type (unknown0: *dirent) -> s32 #c_call, __cmp: #type (unknown0: **dirent, unknown1: **dirent) -> s32 #c_call) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    scandirat64 :: (__dfd: s32, __dir: *u8, __namelist: ***dirent64, __selector: #type (unknown0: *dirent64) -> s32 #c_call, __cmp: #type (unknown0: **dirent64, unknown1: **dirent64) -> s32 #c_call) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    sched_getaffinity :: (__pid: s32, __cpusetsize: u64, __cpuset: *cpu_set_t) -> s32 #foreign libpthread;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    sched_rr_get_interval :: (__pid: s32, __t: *timespec) -> s32 #foreign libpthread;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    sched_setaffinity :: (__pid: s32, __cpusetsize: u64, __cpuset: *cpu_set_t) -> s32 #foreign libpthread;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    seekdir :: (__dirp: *DIR, __pos: s64) -> void #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    sem_clockwait :: (__sem: *sem_t, clock: clockid_t, __abstime: *timespec) -> s32 #foreign libpthread;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    sem_timedwait :: (__sem: *sem_t, __abstime: *timespec) -> s32 #foreign libpthread;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    setbuffer :: (__stream: *FILE, __buf: *u8, __size: u64) -> void #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    setdomainname :: (__name: *u8, __len: u64) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    sethostid :: (__id: s64) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    sethostname :: (__name: *u8, __len: u64) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    setitimer :: (__which: __itimer_which, __new: *itimerval, __old: *itimerval) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    setjmp :: (__env: *jmp_buf) -> s32 #foreign libpthread;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    setrlimit :: (__resource: RLIMIT_RESOURCE, __rlimits: *rlimit) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    settimeofday :: (__tv: *timeval, __tz: *timezone) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    setvbuf :: (__stream: *FILE, __buf: *u8, __modes: s32, __n: u64) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    shmctl :: (__shmid: s32, __cmd: s32, __buf: *shmid_ds) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    shmget :: (__key: key_t, __size: u64, __shmflg: s32) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    siglongjmp :: (__env: *sigjmp_buf, __val: s32) -> void #foreign libpthread;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    signal :: (__sig: s32, __handler: #type (unknown0: s32) -> void #c_call) -> ((unknown0: s32) -> void #c_call) #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    sigpause :: (__sig: s32) -> s32 #foreign libc "__xpg_sigpause";
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    sigset :: (__sig: s32, __disp: #type (unknown0: s32) -> void #c_call) -> ((unknown0: s32) -> void #c_call) #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    sigtimedwait :: (__set: *sigset_t, __info: *siginfo_t, __timeout: *timespec) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    snprintf :: (__s: *u8, __maxlen: u64, __format: *u8, __args: ..Any) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    splice :: (__fdin: s32, __offin: *s64, __fdout: s32, __offout: *s64, __len: u64, __flags: u32) -> s64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    ssignal :: (__sig: s32, __handler: #type (unknown0: s32) -> void #c_call) -> ((unknown0: s32) -> void #c_call) #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    strftime :: (__s: *u8, __maxsize: u64, __format: *u8, __tp: *tm) -> u64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    strftime_l :: (__s: *u8, __maxsize: u64, __format: *u8, __tp: *tm, __loc: locale_t) -> u64 #foreign libc;
}

strsignal :: (sig: s32) -> *u8 #foreign libc;

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    swab :: (__from: *void, __to: *void, __n: s64) -> void #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    syscall :: (__sysno: s64, __args: ..Any) -> s64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    sysconf :: (name: _SC_definitions) -> s64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    sysv_signal :: (__sig: s32, __handler: #type (unknown0: s32) -> void #c_call) -> ((unknown0: s32) -> void #c_call) #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    tee :: (__fdin: s32, __fdout: s32, __len: u64, __flags: u32) -> s64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    telldir :: (__dirp: *DIR) -> s64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    time :: (__timer: *time_t) -> time_t #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    timegm :: (__tp: *tm) -> time_t #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    timelocal :: (__tp: *tm) -> time_t #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    timer_gettime :: (__timerid: timer_t, __value: *itimerspec) -> s32 #foreign librt;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    timer_settime :: (__timerid: timer_t, __flags: s32, __value: *itimerspec, __ovalue: *itimerspec) -> s32 #foreign librt;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    timespec_get :: (__ts: *timespec, __base: s32) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    tmpfile :: () -> *FILE #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    truncate :: (__file: *u8, __length: s64) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    ttyname_r :: (__fd: s32, __buf: *u8, __buflen: u64) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    utimensat :: (__fd: s32, __path: *u8, __times: *[2] timespec, __flags: s32) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    utimes :: (__file: *u8, __tvp: *[2] timeval) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    versionsort :: (__e1: **dirent, __e2: **dirent) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    versionsort64 :: (__e1: **dirent64, __e2: **dirent64) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    vmsplice :: (__fdout: s32, __iov: *iovec, __count: u64, __flags: u32) -> s64 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    wait3 :: (__stat_loc: *s32, __options: s32, __usage: *rusage) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 {
    wait4 :: (__pid: s32, __stat_loc: *s32, __options: s32, __usage: *rusage) -> s32 #foreign libc;
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    write :: (__fd: s32, __buf: *void, __n: u64) -> s64 #foreign libc; @WritesOutput(0)
}

#if (OS == .LINUX) && ARCHITECTURE != .ARM && ARCHITECTURE != .X86 && ARCHITECTURE != .POWERPC64 && ARCHITECTURE != .S390X {
    writev :: (__fd: s32, __iovec: *iovec, __count: s32) -> s64 #foreign libc;
}

// Convenience operations

DTTOIF :: inline (dirtype: mode_t) -> mode_t {
    return dirtype << 12;
}

errno :: inline () -> OS_Error_Code #no_context {
    return cast(OS_Error_Code) <<binding_errno_address();
}

IFTODT :: inline (mode: mode_t) -> mode_t {
    return (mode & S_IFMT) >> 12;
}

S_ISBLK :: inline (mode: mode_t) -> bool {
    return (mode & S_IFMT) == S_IFBLK;
}

S_ISCHR :: inline (mode: mode_t) -> bool {
    return (mode & S_IFMT) == S_IFCHR;
}

S_ISDIR :: inline (mode: mode_t) -> bool {
    return (mode & S_IFMT) == S_IFDIR;
}

S_ISFIFO :: inline (mode: mode_t) -> bool {
    return (mode & S_IFMT) == S_IFIFO;
}

S_ISLNK :: inline (mode: mode_t) -> bool {
    return (mode & S_IFMT) == S_IFLNK;
}

S_ISREG :: inline (mode: mode_t) -> bool {
    return (mode & S_IFMT) == S_IFREG;
}

S_ISSOCK :: inline (mode: mode_t) -> bool {
    return (mode & S_IFMT) == S_IFSOCK;
}

set_errno :: inline (value: OS_Error_Code) #no_context {
    <<binding_errno_address() = cast(s32) value;
}

#if OS == .LINUX || OS == .ANDROID {
    WCOREDUMP :: inline (status: s32) -> s32 {
        return status & 128;
    }
}

#if OS == .MACOS {
    WCOREDUMP :: inline (status: s32) -> s32 {
        return status & 128;
    }
}

#if OS == .LINUX || OS == .ANDROID {
    WEXITSTATUS :: inline (status: s32) -> s32 {
        return (status >> 8) & 255;
    }
}

#if OS == .MACOS {
    WEXITSTATUS :: inline (status: s32) -> s32 {
        return (status >> 8) & 255;
    }
}

#if OS == .LINUX || OS == .ANDROID {
    WIFCONTINUED :: inline (status: s32) -> bool {
        return status == 0xffff;
    }
}

#if OS == .MACOS {
    WIFCONTINUED :: inline (status: s32) -> bool {
        return (status & 127) == 127 && (status >> 8) == 19;
    }
}

#if OS == .LINUX || OS == .ANDROID {
    WIFEXITED :: inline (status: s32) -> bool {
        return (status & 127) == 0;
    }
}

#if OS == .MACOS {
    WIFEXITED :: inline (status: s32) -> bool {
        return (status & 127) == 0;
    }
}

#if OS == .LINUX || OS == .ANDROID {
    WIFSIGNALED :: inline (status: s32) -> bool {
        return (status & 127) != 0 && (status & 127) != 127;
    }
}

#if OS == .MACOS {
    WIFSIGNALED :: inline (status: s32) -> bool {
        return (status & 127) != 0 && (status & 127) != 127;
    }
}

#if OS == .LINUX || OS == .ANDROID {
    WIFSTOPPED :: inline (status: s32) -> bool {
        return (status & 255) == 127;
    }
}

#if OS == .MACOS {
    WIFSTOPPED :: inline (status: s32) -> bool {
        return (status & 127) == 127 && (status >> 8) != 19;
    }
}

#if OS == .LINUX || OS == .ANDROID {
    WSTOPSIG :: inline (status: s32) -> s32 {
        return (status >> 8) & 255;
    }
}

#if OS == .MACOS {
    WSTOPSIG :: inline (status: s32) -> s32 {
        return status >> 8;
    }
}

#if OS == .LINUX || OS == .ANDROID {
    WTERMSIG :: inline (status: s32) -> s32 {
        return status & 127;
    }
}

#if OS == .MACOS {
    WTERMSIG :: inline (status: s32) -> s32 {
        return status & 127;
    }
}

// Each platform's own declarations, parsed only when compiling for it.
#if OS == .ANDROID && CPU == .ARM64 {
    #load "declarations/android-arm64.jai";
}

#if OS == .ANDROID && CPU == .X64 {
    #load "declarations/android-x64.jai";
}

#if OS == .LINUX && ARCHITECTURE == .ARM {
    #load "declarations/linux-arm.jai";
}

#if OS == .LINUX && ARCHITECTURE == .POWERPC64 {
    #load "declarations/linux-powerpc64.jai";
}

#if OS == .LINUX && ARCHITECTURE == .S390X {
    #load "declarations/linux-s390x.jai";
}

#if OS == .LINUX && CPU == .X64 {
    #load "declarations/linux-x64.jai";
}

#if OS == .LINUX && ARCHITECTURE == .X86 {
    #load "declarations/linux-x86.jai";
}

#if OS == .LINUX {
    #load "declarations/linux.jai";
}

#if OS == .MACOS && CPU == .ARM64 {
    #load "declarations/macos-arm64.jai";
}

#if OS == .MACOS && CPU == .X64 {
    #load "declarations/macos-x64.jai";
}
