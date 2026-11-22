// Generated from docs/specs/stdlib/modules/posix.md.
// Regenerate with tools/native_bindings/generate.py; do not hand-edit.

// The declarations for `OS == .MACOS && CPU == .X64`, loaded by declarations.jai.

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

__darwin_fp_control :: struct {
    __bitfield: u16;

}

__darwin_fp_status :: struct {
    __bitfield: u16;

}

__darwin_mcontext64 :: struct {
    __es: __darwin_x86_exception_state64;
    __ss: __darwin_x86_thread_state64;
    __fs: __darwin_x86_float_state64;
}

__darwin_mmst_reg :: struct {
    __mmst_reg: [10] u8;
    __mmst_rsrv: [6] u8;
}

__darwin_pthread_handler_rec :: struct {
    __routine: #type (unknown0: *void) -> void #c_call;
    __arg: *void;
    __next: *__darwin_pthread_handler_rec;
}

__darwin_x86_exception_state64 :: struct {
    __trapno: u16;
    __cpu: u16;
    __err: u32;
    __faultvaddr: u64;
}

__darwin_x86_float_state64 :: struct {
    __fpu_reserved: [2] s32;
    __fpu_fcw: __darwin_fp_control;
    __fpu_fsw: __darwin_fp_status;
    __fpu_ftw: u8;
    __fpu_rsrv1: u8;
    __fpu_fop: u16;

    __fpu_ip: u32;
    __fpu_cs: u16;

    __fpu_rsrv2: u16;

    __fpu_dp: u32;
    __fpu_ds: u16;

    __fpu_rsrv3: u16;
    __fpu_mxcsr: u32;
    __fpu_mxcsrmask: u32;
    __fpu_stmm0: __darwin_mmst_reg;
    __fpu_stmm1: __darwin_mmst_reg;
    __fpu_stmm2: __darwin_mmst_reg;
    __fpu_stmm3: __darwin_mmst_reg;
    __fpu_stmm4: __darwin_mmst_reg;
    __fpu_stmm5: __darwin_mmst_reg;
    __fpu_stmm6: __darwin_mmst_reg;
    __fpu_stmm7: __darwin_mmst_reg;
    __fpu_xmm0: __darwin_xmm_reg;
    __fpu_xmm1: __darwin_xmm_reg;
    __fpu_xmm2: __darwin_xmm_reg;
    __fpu_xmm3: __darwin_xmm_reg;
    __fpu_xmm4: __darwin_xmm_reg;
    __fpu_xmm5: __darwin_xmm_reg;
    __fpu_xmm6: __darwin_xmm_reg;
    __fpu_xmm7: __darwin_xmm_reg;
    __fpu_xmm8: __darwin_xmm_reg;
    __fpu_xmm9: __darwin_xmm_reg;
    __fpu_xmm10: __darwin_xmm_reg;
    __fpu_xmm11: __darwin_xmm_reg;
    __fpu_xmm12: __darwin_xmm_reg;
    __fpu_xmm13: __darwin_xmm_reg;
    __fpu_xmm14: __darwin_xmm_reg;
    __fpu_xmm15: __darwin_xmm_reg;
    __fpu_rsrv4: [96] u8;
    __fpu_reserved1: s32;
}

__darwin_x86_thread_state64 :: struct {
    __rax: u64;
    __rbx: u64;
    __rcx: u64;
    __rdx: u64;
    __rdi: u64;
    __rsi: u64;
    __rbp: u64;
    __rsp: u64;
    __r8: u64;
    __r9: u64;
    __r10: u64;
    __r11: u64;
    __r12: u64;
    __r13: u64;
    __r14: u64;
    __r15: u64;
    __rip: u64;
    __rflags: u64;
    __cs: u64;
    __fs: u64;
    __gs: u64;
}

__darwin_xmm_reg :: struct {
    __xmm_reg: [16] u8;
}

__shmid_ds_new :: struct {
    shm_perm: ipc_perm;
    shm_segsz: u64 #align 4;
    shm_lpid: pid_t;
    shm_cpid: pid_t;
    shm_nattch: shmatt_t;
    shm_atime: time_t #align 4;
    shm_dtime: time_t #align 4;
    shm_ctime: time_t #align 4;
    shm_internal: *void #align 4;
}

__sigaction :: struct {
    using __sigaction_u_: __sigaction_u;
    sa_tramp: #type (unknown0: *void, unknown1: s32, unknown2: s32, unknown3: *siginfo_t, unknown4: *void) -> void #c_call;
    sa_mask: sigset_t;
    sa_flags: s32;
}

__sigaction_u :: struct {
    sa_handler: #type (unknown0: s32) -> void #c_call;

    #overlay (sa_handler) sa_sigaction: #type (unknown0: s32, unknown1: *__siginfo, unknown2: *void) -> void #c_call;
}

__siginfo :: struct {
    si_signo: s32;
    si_errno: s32;
    si_code: s32;
    si_pid: pid_t;
    si_uid: uid_t;
    si_status: s32;
    si_addr: *void;
    si_value: sigval;
    si_band: s64;
    __pad: [7] u64;
}

_filesec :: struct {}

_opaque_pthread_attr_t :: struct {
    __sig: s64;
    __opaque: [56] u8;
}

_opaque_pthread_cond_t :: struct {
    __sig: s64;
    __opaque: [40] u8;
}

_opaque_pthread_condattr_t :: struct {
    __sig: s64;
    __opaque: [8] u8;
}

_opaque_pthread_mutex_t :: struct {
    __sig: s64;
    __opaque: [56] u8;
}

_opaque_pthread_mutexattr_t :: struct {
    __sig: s64;
    __opaque: [8] u8;
}

_opaque_pthread_once_t :: struct {
    __sig: s64;
    __opaque: [8] u8;
}

_opaque_pthread_rwlock_t :: struct {
    __sig: s64;
    __opaque: [192] u8;
}

_opaque_pthread_rwlockattr_t :: struct {
    __sig: s64;
    __opaque: [16] u8;
}

_opaque_pthread_t :: struct {
    __sig: s64;
    __cleanup_stack: *__darwin_pthread_handler_rec;
    __opaque: [8176] u8;
}

_pcred :: struct {
    pc_lock: [72] u8;
    pc_ucred: *ucred;
    p_ruid: uid_t;
    p_svuid: uid_t;
    p_rgid: gid_t;
    p_svgid: gid_t;
    p_refcnt: s32;
}

#if (OS == .MACOS && CPU == .ARM64) || (OS == .MACOS && CPU == .X64) {
    using _SC_definitions :: enum s32 {
        _SC_ARG_MAX :: 1;
        _SC_CHILD_MAX :: 2;
        _SC_CLK_TCK :: 3;
        _SC_NGROUPS_MAX :: 4;
        _SC_OPEN_MAX :: 5;
        _SC_JOB_CONTROL :: 6;
        _SC_SAVED_IDS :: 7;
        _SC_VERSION :: 8;
        _SC_BC_BASE_MAX :: 9;
        _SC_BC_DIM_MAX :: 10;
        _SC_BC_SCALE_MAX :: 11;
        _SC_BC_STRING_MAX :: 12;
        _SC_COLL_WEIGHTS_MAX :: 13;
        _SC_EXPR_NEST_MAX :: 14;
        _SC_LINE_MAX :: 15;
        _SC_RE_DUP_MAX :: 16;
        _SC_2_VERSION :: 17;
        _SC_2_C_BIND :: 18;
        _SC_2_C_DEV :: 19;
        _SC_2_CHAR_TERM :: 20;
        _SC_2_FORT_DEV :: 21;
        _SC_2_FORT_RUN :: 22;
        _SC_2_LOCALEDEF :: 23;
        _SC_2_SW_DEV :: 24;
        _SC_2_UPE :: 25;
        _SC_STREAM_MAX :: 26;
        _SC_TZNAME_MAX :: 27;

        _SC_ASYNCHRONOUS_IO :: 28;
        _SC_PAGESIZE :: 29;
        _SC_MEMLOCK :: 30;
        _SC_MEMLOCK_RANGE :: 31;
        _SC_MEMORY_PROTECTION :: 32;
        _SC_MESSAGE_PASSING :: 33;
        _SC_PRIORITIZED_IO :: 34;
        _SC_PRIORITY_SCHEDULING :: 35;
        _SC_REALTIME_SIGNALS :: 36;
        _SC_SEMAPHORES :: 37;
        _SC_FSYNC :: 38;
        _SC_SHARED_MEMORY_OBJECTS :: 39;
        _SC_SYNCHRONIZED_IO :: 40;
        _SC_TIMERS :: 41;
        _SC_AIO_LISTIO_MAX :: 42;
        _SC_AIO_MAX :: 43;
        _SC_AIO_PRIO_DELTA_MAX :: 44;
        _SC_DELAYTIMER_MAX :: 45;
        _SC_MQ_OPEN_MAX :: 46;
        _SC_MAPPED_FILES :: 47;
        _SC_RTSIG_MAX :: 48;
        _SC_SEM_NSEMS_MAX :: 49;
        _SC_SEM_VALUE_MAX :: 50;
        _SC_SIGQUEUE_MAX :: 51;
        _SC_TIMER_MAX :: 52;

        _SC_NPROCESSORS_CONF :: 57;
        _SC_NPROCESSORS_ONLN :: 58;

        _SC_2_PBS :: 59;
        _SC_2_PBS_ACCOUNTING :: 60;
        _SC_2_PBS_CHECKPOINT :: 61;
        _SC_2_PBS_LOCATE :: 62;
        _SC_2_PBS_MESSAGE :: 63;
        _SC_2_PBS_TRACK :: 64;
        _SC_ADVISORY_INFO :: 65;
        _SC_BARRIERS :: 66;
        _SC_CLOCK_SELECTION :: 67;
        _SC_CPUTIME :: 68;
        _SC_FILE_LOCKING :: 69;
        _SC_GETGR_R_SIZE_MAX :: 70;
        _SC_GETPW_R_SIZE_MAX :: 71;
        _SC_HOST_NAME_MAX :: 72;
        _SC_LOGIN_NAME_MAX :: 73;
        _SC_MONOTONIC_CLOCK :: 74;
        _SC_MQ_PRIO_MAX :: 75;
        _SC_READER_WRITER_LOCKS :: 76;
        _SC_REGEXP :: 77;
        _SC_SHELL :: 78;
        _SC_SPAWN :: 79;
        _SC_SPIN_LOCKS :: 80;
        _SC_SPORADIC_SERVER :: 81;
        _SC_THREAD_ATTR_STACKADDR :: 82;
        _SC_THREAD_ATTR_STACKSIZE :: 83;
        _SC_THREAD_CPUTIME :: 84;
        _SC_THREAD_DESTRUCTOR_ITERATIONS :: 85;
        _SC_THREAD_KEYS_MAX :: 86;
        _SC_THREAD_PRIO_INHERIT :: 87;
        _SC_THREAD_PRIO_PROTECT :: 88;
        _SC_THREAD_PRIORITY_SCHEDULING :: 89;
        _SC_THREAD_PROCESS_SHARED :: 90;
        _SC_THREAD_SAFE_FUNCTIONS :: 91;
        _SC_THREAD_SPORADIC_SERVER :: 92;
        _SC_THREAD_STACK_MIN :: 93;
        _SC_THREAD_THREADS_MAX :: 94;
        _SC_TIMEOUTS :: 95;
        _SC_THREADS :: 96;
        _SC_TRACE :: 97;
        _SC_TRACE_EVENT_FILTER :: 98;
        _SC_TRACE_INHERIT :: 99;
        _SC_TRACE_LOG :: 100;
        _SC_TTY_NAME_MAX :: 101;
        _SC_TYPED_MEMORY_OBJECTS :: 102;
        _SC_V6_ILP32_OFF32 :: 103;
        _SC_V6_ILP32_OFFBIG :: 104;
        _SC_V6_LP64_OFF64 :: 105;
        _SC_V6_LPBIG_OFFBIG :: 106;
        _SC_IPV6 :: 118;
        _SC_RAW_SOCKETS :: 119;
        _SC_SYMLOOP_MAX :: 120;

        _SC_ATEXIT_MAX :: 107;
        _SC_IOV_MAX :: 56;
        _SC_PAGE_SIZE :: _SC_PAGESIZE;
        _SC_XOPEN_CRYPT :: 108;
        _SC_XOPEN_ENH_I18N :: 109;
        _SC_XOPEN_LEGACY :: 110;
        _SC_XOPEN_REALTIME :: 111;
        _SC_XOPEN_REALTIME_THREADS :: 112;
        _SC_XOPEN_SHM :: 113;
        _SC_XOPEN_STREAMS :: 114;
        _SC_XOPEN_UNIX :: 115;
        _SC_XOPEN_VERSION :: 116;
        _SC_XOPEN_XCU_VERSION :: 121;

        _SC_XBS5_ILP32_OFF32 :: 122;
        _SC_XBS5_ILP32_OFFBIG :: 123;
        _SC_XBS5_LP64_OFF64 :: 124;
        _SC_XBS5_LPBIG_OFFBIG :: 125;

        _SC_SS_REPL_MAX :: 126;
        _SC_TRACE_EVENT_NAME_MAX :: 127;
        _SC_TRACE_NAME_MAX :: 128;
        _SC_TRACE_SYS_MAX :: 129;
        _SC_TRACE_USER_EVENT_MAX :: 130;

        _SC_PASS_MAX :: 131;

        _SC_PHYS_PAGES :: 200;
    }
}

_telldir :: struct {}

_ucred :: struct {
    cr_ref: s32;
    cr_uid: uid_t;
    cr_ngroups: s16;
    cr_groups: [16] gid_t;
}

accessx_descriptor :: struct {
    ad_name_offset: u32;
    ad_flags: s32;
    ad_pad: [2] s32;
}

#if (OS == .MACOS && CPU == .ARM64) || (OS == .MACOS && CPU == .X64) {
    aiocb :: struct {
        aio_fildes: s32;
        aio_offset: s64;
        aio_buf: *void;
        aio_nbytes: u64;
        aio_reqprio: s32;
        aio_sigevent: sigevent;
        aio_lio_opcode: s32;
    }
}

ATTR_CMN :: enum u32 {
    NAME :: 0x00000001;
    DEVID :: 0x00000002;
    FSID :: 0x00000004;
    OBJTYPE :: 0x00000008;
    OBJTAG :: 0x00000010;
    OBJID :: 0x00000020;
    OBJPERMANENTID :: 0x00000040;
    PAROBJID :: 0x00000080;
    SCRIPT :: 0x00000100;
    CRTIME :: 0x00000200;
    MODTIME :: 0x00000400;
    CHGTIME :: 0x00000800;
    ACCTIME :: 0x00001000;
    BKUPTIME :: 0x00002000;
    FNDRINFO :: 0x00004000;
    OWNERID :: 0x00008000;
    GRPID :: 0x00010000;
    ACCESSMASK :: 0x00020000;
    FLAGS :: 0x00040000;

    GEN_COUNT :: 0x00080000;
    DOCUMENT_ID :: 0x00100000;

    USERACCESS :: 0x00200000;
    EXTENDED_SECURITY :: 0x00400000;
    UUID :: 0x00800000;
    GRPUUID :: 0x01000000;
    FILEID :: 0x02000000;
    PARENTID :: 0x04000000;
    FULLPATH :: 0x08000000;
    ADDEDTIME :: 0x10000000;
    ERROR :: 0x20000000;
    DATA_PROTECT_FLAGS :: 0x40000000;

    RETURNED_ATTRS :: 0x80000000;

    VALIDMASK :: 0xFFFFFFFF;

    SETMASK :: 0x51C7FF00;
    VOLSETMASK :: 0x00006700;

    EXT_RELPATH :: 0x00000004;
    EXT_PRIVATESIZE :: 0x00000008;
    EXT_LINKID :: 0x00000010;

    EXT_VALIDMASK :: 0x0000001c;
    EXT_SETMASK :: 0x00000000;

    NAMEDATTRCOUNT :: 0x00080000;
    NAMEDATTRLIST :: 0x00100000;

    ATTR_CMN_NAME :: NAME;
    ATTR_CMN_DEVID :: DEVID;
    ATTR_CMN_FSID :: FSID;
    ATTR_CMN_OBJTYPE :: OBJTYPE;
    ATTR_CMN_OBJTAG :: OBJTAG;
    ATTR_CMN_OBJID :: OBJID;
    ATTR_CMN_OBJPERMANENTID :: OBJPERMANENTID;
    ATTR_CMN_PAROBJID :: PAROBJID;
    ATTR_CMN_SCRIPT :: SCRIPT;
    ATTR_CMN_CRTIME :: CRTIME;
    ATTR_CMN_MODTIME :: MODTIME;
    ATTR_CMN_CHGTIME :: CHGTIME;
    ATTR_CMN_ACCTIME :: ACCTIME;
    ATTR_CMN_BKUPTIME :: BKUPTIME;
    ATTR_CMN_FNDRINFO :: FNDRINFO;
    ATTR_CMN_OWNERID :: OWNERID;
    ATTR_CMN_GRPID :: GRPID;
    ATTR_CMN_ACCESSMASK :: ACCESSMASK;
    ATTR_CMN_FLAGS :: FLAGS;

    ATTR_CMN_GEN_COUNT :: GEN_COUNT;
    ATTR_CMN_DOCUMENT_ID :: DOCUMENT_ID;

    ATTR_CMN_USERACCESS :: USERACCESS;
    ATTR_CMN_EXTENDED_SECURITY :: EXTENDED_SECURITY;
    ATTR_CMN_UUID :: UUID;
    ATTR_CMN_GRPUUID :: GRPUUID;
    ATTR_CMN_FILEID :: FILEID;
    ATTR_CMN_PARENTID :: PARENTID;
    ATTR_CMN_FULLPATH :: FULLPATH;
    ATTR_CMN_ADDEDTIME :: ADDEDTIME;
    ATTR_CMN_ERROR :: ERROR;
    ATTR_CMN_DATA_PROTECT_FLAGS :: DATA_PROTECT_FLAGS;

    ATTR_CMN_RETURNED_ATTRS :: RETURNED_ATTRS;

    ATTR_CMN_VALIDMASK :: VALIDMASK;

    ATTR_CMN_SETMASK :: SETMASK;
    ATTR_CMN_VOLSETMASK :: VOLSETMASK;

    ATTR_CMNEXT_RELPATH :: EXT_RELPATH;
    ATTR_CMNEXT_PRIVATESIZE :: EXT_PRIVATESIZE;
    ATTR_CMNEXT_LINKID :: EXT_LINKID;

    ATTR_CMNEXT_VALIDMASK :: EXT_VALIDMASK;
    ATTR_CMNEXT_SETMASK :: EXT_SETMASK;

    ATTR_CMN_NAMEDATTRCOUNT :: NAMEDATTRCOUNT;
    ATTR_CMN_NAMEDATTRLIST :: NAMEDATTRLIST;
}

ATTR_DIR :: enum {
    LINKCOUNT :: 0x00000001;
    ENTRYCOUNT :: 0x00000002;
    MOUNTSTATUS :: 0x00000004;
    ALLOCSIZE :: 0x00000008;
    IOBLOCKSIZE :: 0x00000010;
    DATALENGTH :: 0x00000020;

    VALIDMASK :: 0x0000003f;
    SETMASK :: 0x00000000;

    ATTR_DIR_LINKCOUNT :: LINKCOUNT;
    ATTR_DIR_ENTRYCOUNT :: ENTRYCOUNT;
    ATTR_DIR_MOUNTSTATUS :: MOUNTSTATUS;
    ATTR_DIR_ALLOCSIZE :: ALLOCSIZE;
    ATTR_DIR_IOBLOCKSIZE :: IOBLOCKSIZE;
    ATTR_DIR_DATALENGTH :: DATALENGTH;

    ATTR_DIR_VALIDMASK :: VALIDMASK;
    ATTR_DIR_SETMASK :: SETMASK;
}

ATTR_FILE :: enum {
    LINKCOUNT :: 0x00000001;
    TOTALSIZE :: 0x00000002;
    ALLOCSIZE :: 0x00000004;
    IOBLOCKSIZE :: 0x00000008;
    DEVTYPE :: 0x00000020;
    FORKCOUNT :: 0x00000080;
    FORKLIST :: 0x00000100;
    DATALENGTH :: 0x00000200;
    DATAALLOCSIZE :: 0x00000400;
    RSRCLENGTH :: 0x00001000;
    RSRCALLOCSIZE :: 0x00002000;

    VALIDMASK :: 0x000037FF;

    SETMASK :: 0x00000020;

    CLUMPSIZE :: 0x00000010;
    FILETYPE :: 0x00000040;
    DATAEXTENTS :: 0x00000800;
    RSRCEXTENTS :: 0x00004000;

    ATTR_FILE_LINKCOUNT :: LINKCOUNT;
    ATTR_FILE_TOTALSIZE :: TOTALSIZE;
    ATTR_FILE_ALLOCSIZE :: ALLOCSIZE;
    ATTR_FILE_IOBLOCKSIZE :: IOBLOCKSIZE;
    ATTR_FILE_DEVTYPE :: DEVTYPE;
    ATTR_FILE_FORKCOUNT :: FORKCOUNT;
    ATTR_FILE_FORKLIST :: FORKLIST;
    ATTR_FILE_DATALENGTH :: DATALENGTH;
    ATTR_FILE_DATAALLOCSIZE :: DATAALLOCSIZE;
    ATTR_FILE_RSRCLENGTH :: RSRCLENGTH;
    ATTR_FILE_RSRCALLOCSIZE :: RSRCALLOCSIZE;

    ATTR_FILE_VALIDMASK :: VALIDMASK;

    ATTR_FILE_SETMASK :: SETMASK;

    ATTR_FILE_CLUMPSIZE :: CLUMPSIZE;
    ATTR_FILE_FILETYPE :: FILETYPE;
    ATTR_FILE_DATAEXTENTS :: DATAEXTENTS;
    ATTR_FILE_RSRCEXTENTS :: RSRCEXTENTS;
}

ATTR_FORK :: enum {
    TOTALSIZE :: 0x00000001;
    ALLOCSIZE :: 0x00000002;
    RESERVED :: 0xffffffff;

    VALIDMASK :: 0x00000003;
    SETMASK :: 0x00000000;

    ATTR_FORK_TOTALSIZE :: TOTALSIZE;
    ATTR_FORK_ALLOCSIZE :: ALLOCSIZE;
    ATTR_FORK_RESERVED :: RESERVED;

    ATTR_FORK_VALIDMASK :: VALIDMASK;
    ATTR_FORK_SETMASK :: SETMASK;
}

ATTR_VOL :: enum {
    FSTYPE :: 0x00000001;
    SIGNATURE :: 0x00000002;
    SIZE :: 0x00000004;
    SPACEFREE :: 0x00000008;
    SPACEAVAIL :: 0x00000010;
    MINALLOCATION :: 0x00000020;
    ALLOCATIONCLUMP :: 0x00000040;
    IOBLOCKSIZE :: 0x00000080;
    OBJCOUNT :: 0x00000100;
    FILECOUNT :: 0x00000200;
    DIRCOUNT :: 0x00000400;
    MAXOBJCOUNT :: 0x00000800;
    MOUNTPOINT :: 0x00001000;
    NAME :: 0x00002000;
    MOUNTFLAGS :: 0x00004000;
    MOUNTEDDEVICE :: 0x00008000;
    ENCODINGSUSED :: 0x00010000;
    CAPABILITIES :: 0x00020000;
    UUID :: 0x00040000;
    QUOTA_SIZE :: 0x10000000;
    RESERVED_SIZE :: 0x20000000;
    ATTRIBUTES :: 0x40000000;
    INFO :: 0x80000000;

    VALIDMASK :: 0xF007FFFF;

    SETMASK :: 0x80002000;

    ATTR_VOL_FSTYPE :: FSTYPE;
    ATTR_VOL_SIGNATURE :: SIGNATURE;
    ATTR_VOL_SIZE :: SIZE;
    ATTR_VOL_SPACEFREE :: SPACEFREE;
    ATTR_VOL_SPACEAVAIL :: SPACEAVAIL;
    ATTR_VOL_MINALLOCATION :: MINALLOCATION;
    ATTR_VOL_ALLOCATIONCLUMP :: ALLOCATIONCLUMP;
    ATTR_VOL_IOBLOCKSIZE :: IOBLOCKSIZE;
    ATTR_VOL_OBJCOUNT :: OBJCOUNT;
    ATTR_VOL_FILECOUNT :: FILECOUNT;
    ATTR_VOL_DIRCOUNT :: DIRCOUNT;
    ATTR_VOL_MAXOBJCOUNT :: MAXOBJCOUNT;
    ATTR_VOL_MOUNTPOINT :: MOUNTPOINT;
    ATTR_VOL_NAME :: NAME;
    ATTR_VOL_MOUNTFLAGS :: MOUNTFLAGS;
    ATTR_VOL_MOUNTEDDEVICE :: MOUNTEDDEVICE;
    ATTR_VOL_ENCODINGSUSED :: ENCODINGSUSED;
    ATTR_VOL_CAPABILITIES :: CAPABILITIES;
    ATTR_VOL_UUID :: UUID;
    ATTR_VOL_QUOTA_SIZE :: QUOTA_SIZE;
    ATTR_VOL_RESERVED_SIZE :: RESERVED_SIZE;
    ATTR_VOL_ATTRIBUTES :: ATTRIBUTES;
    ATTR_VOL_INFO :: INFO;

    ATTR_VOL_VALIDMASK :: VALIDMASK;

    ATTR_VOL_SETMASK :: SETMASK;
}

attribute_set :: struct {
    commonattr: ATTR_CMN;
    volattr: attrgroup_t;
    dirattr: attrgroup_t;
    fileattr: attrgroup_t;
    forkattr: attrgroup_t;
}

attrlist :: struct {
    bitmapcount: u16;
    reserved: u16;
    commonattr: ATTR_CMN;
    volattr: attrgroup_t;
    dirattr: attrgroup_t;
    fileattr: attrgroup_t;
    forkattr: attrgroup_t;
}

attrreference :: struct {
    attr_dataoffset: s32;
    attr_length: u32;
}

#if (OS == .MACOS && CPU == .ARM64) || (OS == .MACOS && CPU == .X64) {
    clockid_t :: enum u32 {
        REALTIME :: 0;

        MONOTONIC :: 6;

        MONOTONIC_RAW :: 4;

        MONOTONIC_RAW_APPROX :: 5;

        UPTIME_RAW :: 8;

        UPTIME_RAW_APPROX :: 9;

        PROCESS_CPUTIME_ID :: 12;

        THREAD_CPUTIME_ID :: 16;

        _CLOCK_REALTIME :: REALTIME;

        _CLOCK_MONOTONIC :: MONOTONIC;

        _CLOCK_MONOTONIC_RAW :: MONOTONIC_RAW;

        _CLOCK_MONOTONIC_RAW_APPROX :: MONOTONIC_RAW_APPROX;

        _CLOCK_UPTIME_RAW :: UPTIME_RAW;

        _CLOCK_UPTIME_RAW_APPROX :: UPTIME_RAW_APPROX;

        _CLOCK_PROCESS_CPUTIME_ID :: PROCESS_CPUTIME_ID;

        _CLOCK_THREAD_CPUTIME_ID :: THREAD_CPUTIME_ID;
    }
}

clockinfo :: struct {
    hz: s32;
    tick: s32;
    tickadj: s32;
    stathz: s32;
    profhz: s32;
}

ctlname :: struct {
    ctl_name: *u8;
    ctl_type: s32;
}

#if (OS == .MACOS && CPU == .ARM64) || (OS == .MACOS && CPU == .X64) {
    DIR :: struct {
        __dd_fd: s32;
        __dd_loc: s64;
        __dd_size: s64;
        __dd_buf: *u8;
        __dd_len: s32;
        __dd_seek: s64;
        __padding: s64;
        __dd_flags: s32;
        __dd_lock: _opaque_pthread_mutex_t;
        __dd_td: *_telldir;
    }
}

#if (OS == .MACOS && CPU == .ARM64) || (OS == .MACOS && CPU == .X64) {
    dirent :: struct {
        d_ino: u64;
        d_seekoff: u64;
        d_reclen: u16;
        d_namlen: u16;
        d_type: DT;
        d_name: [1024] u8;
    }
}

diskextent :: struct {
    startblock: u32;
    blockcount: u32;
}

dl_info :: struct {
    dli_fname: *u8;
    dli_fbase: *void;
    dli_sname: *u8;
    dli_saddr: *void;
}

#if (OS == .MACOS && CPU == .ARM64) || (OS == .MACOS && CPU == .X64) {
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

extern_proc :: struct {
    p_un: struct {
        p_st1: struct {
            __p_forw: *proc;
            __p_back: *proc;
        };

        #overlay (p_st1) __p_starttime: timeval;
    };

    p_vmspace: *vmspace;
    sigacts :: struct {}
    p_sigacts: *sigacts;
    p_flag: s32;
    p_stat: u8;
    p_pid: pid_t;
    p_oppid: pid_t;
    p_dupfd: s32;

    user_stack: *u8;
    exit_thread: *void;
    p_debugger: s32;
    sigwait: u32;

    p_estcpu: u32;
    p_cpticks: s32;
    p_pctcpu: fixpt_t;
    p_wchan: *void;
    p_wmesg: *u8;
    p_swtime: u32;
    p_slptime: u32;
    p_realtimer: itimerval;
    p_rtime: timeval;
    p_uticks: u64;
    p_sticks: u64;
    p_iticks: u64;
    p_traceflag: s32;
    vnode :: struct {}
    p_tracep: *vnode;
    p_siglist: s32;
    p_textvp: *vnode;
    p_holdcnt: s32;
    p_sigmask: sigset_t;
    p_sigignore: sigset_t;
    p_sigcatch: sigset_t;
    p_priority: u8;
    p_usrpri: u8;
    p_nice: u8;
    p_comm: [17] u8;
    p_pgrp: *pgrp;
    user :: struct {}
    p_addr: *user;
    p_xstat: u16;
    p_acflag: u16;
    rusage :: struct {}
    p_ru: *rusage;
}

fbootstraptransfer :: struct {
    fbt_offset: s64;
    fbt_length: u64;
    fbt_buffer: *void;
}

fchecklv :: struct {
    lv_file_start: s64;
    lv_error_message_size: u64;
    lv_error_message: *void;
}

fcodeblobs :: struct {
    f_cd_hash: *void;
    f_hash_size: u64;
    f_cd_buffer: *void;
    f_cd_size: u64;
    f_out_size: *u32;
    f_arch: s32;
    __padding: s32;
}

FILE :: struct {}

filesec_property_t :: enum u32 {
    OWNER :: 1;
    GROUP :: 2;
    UUID :: 3;
    MODE :: 4;
    ACL :: 5;
    GRPUUID :: 6;

    ACL_RAW :: 100;
    ACL_ALLOCSIZE :: 101;

    FILESEC_OWNER :: OWNER;
    FILESEC_GROUP :: GROUP;
    FILESEC_UUID :: UUID;
    FILESEC_MODE :: MODE;
    FILESEC_ACL :: ACL;
    FILESEC_GRPUUID :: GRPUUID;

    FILESEC_ACL_RAW :: ACL_RAW;
    FILESEC_ACL_ALLOCSIZE :: ACL_ALLOCSIZE;
}

#if (OS == .MACOS && CPU == .ARM64) || (OS == .MACOS && CPU == .X64) {
    flock :: struct {
        l_start: s64;
        l_len: s64;
        l_pid: pid_t;
        l_type: s16;
        l_whence: s16;
    }
}

flocktimeout :: struct {
    fl: flock;
    timeout: timespec;
}

fpunchhole :: struct {
    fp_flags: u32;
    reserved: u32;
    fp_offset: s64;
    fp_length: s64;
}

fsignatures :: struct {
    fs_file_start: s64;
    fs_blob_start: *void;
    fs_blob_size: u64;
}

fssearchblock :: struct {
    returnattrs: *attrlist;
    returnbuffer: *void;
    returnbuffersize: u64;
    maxmatches: u64;
    timelimit: timeval;
    searchparams1: *void;
    sizeofsearchparams1: u64;
    searchparams2: *void;
    sizeofsearchparams2: u64;
    searchattrs: attrlist;
}

fstore :: struct {
    fst_flags: u32;
    fst_posmode: s32;
    fst_offset: s64;
    fst_length: s64;
    fst_bytesalloc: s64;
}

ftrimactivefile :: struct {
    fta_offset: s64;
    fta_length: s64;
}

idtype_t :: enum u32 {
    ALL :: 0;
    PID :: 1;
    PGID :: 2;

    P_ALL :: ALL;
    P_PID :: PID;
    P_PGID :: PGID;
}

#if (OS == .MACOS && CPU == .ARM64) || (OS == .MACOS && CPU == .X64) {
    iovec :: struct {
        iov_base: *void;
        iov_len: u64;
    }
}

ipc_perm :: struct {
    uid: uid_t;
    gid: gid_t;
    cuid: uid_t;
    cgid: gid_t;
    mode: mode_t;
    _seq: u16;
    _key: key_t;
}

itimerval :: struct {
    it_interval: timeval;
    it_value: timeval;
}

kinfo_proc :: struct {
    kp_proc: extern_proc;
    eproc :: struct {
        e_paddr: *proc;
        e_sess: *session;
        e_pcred: _pcred;
        e_ucred: _ucred;
        e_vm: vmspace;
        e_ppid: pid_t;
        e_pgid: pid_t;
        e_jobc: s16;
        e_tdev: dev_t;
        e_tpgid: pid_t;
        e_tsess: *session;

        e_wmesg: [8] u8;
        e_xsize: segsz_t;
        e_xrssize: s16;
        e_xccount: s16;
        e_xswrss: s16;
        e_flag: s32;

        e_login: [12] u8;
        e_spare: [4] s32;
    }

    kp_eproc: eproc;
}

loadavg :: struct {
    ldavg: [3] fixpt_t;
    fscale: s64;
}

log2phys :: struct {
    l2p_flags: u32;
    l2p_contigbytes: s64 #align 4;

    l2p_devoffset: s64 #align 4;
}

ostat :: struct {
    st_dev: u16;
    st_ino: ino_t;
    st_mode: mode_t;
    st_nlink: nlink_t;
    st_uid: u16;
    st_gid: u16;
    st_rdev: u16;
    st_size: s32;
    st_atimespec: timespec;
    st_mtimespec: timespec;
    st_ctimespec: timespec;
    st_blksize: s32;
    st_blocks: s32;
    st_flags: u32;
    st_gen: u32;
}

#if (OS == .MACOS && CPU == .ARM64) || (OS == .MACOS && CPU == .X64) {
    passwd :: struct {
        pw_name: *u8;
        pw_passwd: *u8;
        pw_uid: uid_t;
        pw_gid: gid_t;
        pw_change: s64;
        pw_class: *u8;
        pw_gecos: *u8;
        pw_dir: *u8;
        pw_shell: *u8;
        pw_expire: s64;
    }
}

#if (OS == .MACOS && CPU == .ARM64) || (OS == .MACOS && CPU == .X64) {
    PC :: enum u32 {
        LINK_MAX :: 1;
        MAX_CANON :: 2;
        MAX_INPUT :: 3;
        NAME_MAX :: 4;
        PATH_MAX :: 5;
        PIPE_BUF :: 6;
        CHOWN_RESTRICTED :: 7;
        NO_TRUNC :: 8;
        VDISABLE :: 9;

        NAME_CHARS_MAX :: 10;
        CASE_SENSITIVE :: 11;
        CASE_PRESERVING :: 12;
        EXTENDED_SECURITY_NP :: 13;
        AUTH_OPAQUE_NP :: 14;

        _2_SYMLINKS :: 15;
        ALLOC_SIZE_MIN :: 16;
        ASYNC_IO :: 17;
        FILESIZEBITS :: 18;
        PRIO_IO :: 19;
        REC_INCR_XFER_SIZE :: 20;
        REC_MAX_XFER_SIZE :: 21;
        REC_MIN_XFER_SIZE :: 22;
        REC_XFER_ALIGN :: 23;
        SYMLINK_MAX :: 24;
        SYNC_IO :: 25;
        XATTR_SIZE_BITS :: 26;
        MIN_HOLE_SIZE :: 27;
    }
}

pgrp :: struct {}

#if (OS == .MACOS && CPU == .ARM64) || (OS == .MACOS && CPU == .X64) {
    pollfd :: struct {
        fd: s32;
        events: s16;
        revents: s16;
    }
}

#if (OS == .MACOS && CPU == .ARM64) || (OS == .MACOS && CPU == .X64) {
    PRIO :: enum s32 {
        PROCESS :: 0;
        PGRP :: 1;
        USER :: 2;

        DARWIN_THREAD :: 3;
        DARWIN_PROCESS :: 4;

        DARWIN_BG :: 0x1000;

        DARWIN_NONUI :: 0x1001;
    }
}

proc :: struct {}

proc_rlimit_control_wakeupmon :: struct {
    wm_flags: u32;
    wm_rate: s32;
}

pthread_override_s :: struct {}

QOS_CLASS :: enum u32 {
    USER_INTERACTIVE :: 33;
    USER_INITIATED :: 25;
    DEFAULT :: 21;
    UTILITY :: 17;
    BACKGROUND :: 9;
    UNSPECIFIED :: 0;
    QOS_CLASS_USER_INTERACTIVE :: USER_INTERACTIVE;
    QOS_CLASS_USER_INITIATED :: USER_INITIATED;
    QOS_CLASS_DEFAULT :: DEFAULT;
    QOS_CLASS_UTILITY :: UTILITY;
    QOS_CLASS_BACKGROUND :: BACKGROUND;
    QOS_CLASS_UNSPECIFIED :: UNSPECIFIED;
}

radvisory :: struct {
    ra_offset: s64;
    ra_count: s32;
}

#if (OS == .MACOS && CPU == .ARM64) || (OS == .MACOS && CPU == .X64) {
    rlimit :: struct {
        rlim_cur: rlim_t;
        rlim_max: rlim_t;
    }
}

#if (OS == .MACOS && CPU == .ARM64) || (OS == .MACOS && CPU == .X64) {
    RLIMIT_RESOURCE :: enum s32 {
        CPU :: 0;
        FSIZE :: 1;
        DATA :: 2;
        STACK :: 3;
        CORE :: 4;
        AS :: 5;

        RSS :: AS;
        MEMLOCK :: 6;
        NPROC :: 7;

        NOFILE :: 8;
    }
}

#if (OS == .MACOS && CPU == .ARM64) || (OS == .MACOS && CPU == .X64) {
    RUSAGE :: enum s32 {
        SELF :: 0;
        CHILDREN :: -1;
    }
}

#if (OS == .MACOS && CPU == .ARM64) || (OS == .MACOS && CPU == .X64) {
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

RUSAGE_INFO :: enum {
    V0 :: 0;
    V1 :: 1;
    V2 :: 2;
    V3 :: 3;
    V4 :: 4;
    CURRENT :: V4;
}

rusage_info_v0 :: struct {
    ri_uuid: [16] u8;
    ri_user_time: u64;
    ri_system_time: u64;
    ri_pkg_idle_wkups: u64;
    ri_interrupt_wkups: u64;
    ri_pageins: u64;
    ri_wired_size: u64;
    ri_resident_size: u64;
    ri_phys_footprint: u64;
    ri_proc_start_abstime: u64;
    ri_proc_exit_abstime: u64;
}

rusage_info_v1 :: struct {
    ri_uuid: [16] u8;
    ri_user_time: u64;
    ri_system_time: u64;
    ri_pkg_idle_wkups: u64;
    ri_interrupt_wkups: u64;
    ri_pageins: u64;
    ri_wired_size: u64;
    ri_resident_size: u64;
    ri_phys_footprint: u64;
    ri_proc_start_abstime: u64;
    ri_proc_exit_abstime: u64;
    ri_child_user_time: u64;
    ri_child_system_time: u64;
    ri_child_pkg_idle_wkups: u64;
    ri_child_interrupt_wkups: u64;
    ri_child_pageins: u64;
    ri_child_elapsed_abstime: u64;
}

rusage_info_v2 :: struct {
    ri_uuid: [16] u8;
    ri_user_time: u64;
    ri_system_time: u64;
    ri_pkg_idle_wkups: u64;
    ri_interrupt_wkups: u64;
    ri_pageins: u64;
    ri_wired_size: u64;
    ri_resident_size: u64;
    ri_phys_footprint: u64;
    ri_proc_start_abstime: u64;
    ri_proc_exit_abstime: u64;
    ri_child_user_time: u64;
    ri_child_system_time: u64;
    ri_child_pkg_idle_wkups: u64;
    ri_child_interrupt_wkups: u64;
    ri_child_pageins: u64;
    ri_child_elapsed_abstime: u64;
    ri_diskio_bytesread: u64;
    ri_diskio_byteswritten: u64;
}

rusage_info_v3 :: struct {
    ri_uuid: [16] u8;
    ri_user_time: u64;
    ri_system_time: u64;
    ri_pkg_idle_wkups: u64;
    ri_interrupt_wkups: u64;
    ri_pageins: u64;
    ri_wired_size: u64;
    ri_resident_size: u64;
    ri_phys_footprint: u64;
    ri_proc_start_abstime: u64;
    ri_proc_exit_abstime: u64;
    ri_child_user_time: u64;
    ri_child_system_time: u64;
    ri_child_pkg_idle_wkups: u64;
    ri_child_interrupt_wkups: u64;
    ri_child_pageins: u64;
    ri_child_elapsed_abstime: u64;
    ri_diskio_bytesread: u64;
    ri_diskio_byteswritten: u64;
    ri_cpu_time_qos_default: u64;
    ri_cpu_time_qos_maintenance: u64;
    ri_cpu_time_qos_background: u64;
    ri_cpu_time_qos_utility: u64;
    ri_cpu_time_qos_legacy: u64;
    ri_cpu_time_qos_user_initiated: u64;
    ri_cpu_time_qos_user_interactive: u64;
    ri_billed_system_time: u64;
    ri_serviced_system_time: u64;
}

rusage_info_v4 :: struct {
    ri_uuid: [16] u8;
    ri_user_time: u64;
    ri_system_time: u64;
    ri_pkg_idle_wkups: u64;
    ri_interrupt_wkups: u64;
    ri_pageins: u64;
    ri_wired_size: u64;
    ri_resident_size: u64;
    ri_phys_footprint: u64;
    ri_proc_start_abstime: u64;
    ri_proc_exit_abstime: u64;
    ri_child_user_time: u64;
    ri_child_system_time: u64;
    ri_child_pkg_idle_wkups: u64;
    ri_child_interrupt_wkups: u64;
    ri_child_pageins: u64;
    ri_child_elapsed_abstime: u64;
    ri_diskio_bytesread: u64;
    ri_diskio_byteswritten: u64;
    ri_cpu_time_qos_default: u64;
    ri_cpu_time_qos_maintenance: u64;
    ri_cpu_time_qos_background: u64;
    ri_cpu_time_qos_utility: u64;
    ri_cpu_time_qos_legacy: u64;
    ri_cpu_time_qos_user_initiated: u64;
    ri_cpu_time_qos_user_interactive: u64;
    ri_billed_system_time: u64;
    ri_serviced_system_time: u64;
    ri_logical_writes: u64;
    ri_lifetime_max_phys_footprint: u64;
    ri_instructions: u64;
    ri_cycles: u64;
    ri_billed_energy: u64;
    ri_serviced_energy: u64;

    ri_unused: [2] u64;
}

#if (OS == .MACOS && CPU == .ARM64) || (OS == .MACOS && CPU == .X64) {
    sched_param :: struct {
        sched_priority: s32;
        __opaque: [4] u8;
    }
}

searchstate :: struct {
    ss_union_flags: u32 #align 1;
    ss_union_layer: u32 #align 1;
    ss_fsstate: [548] u8;
}

session :: struct {}

#if (OS == .MACOS && CPU == .ARM64) || (OS == .MACOS && CPU == .X64) {
    sigaction_t :: struct {
        using __sigaction_u_: __sigaction_u;
        sa_mask: sigset_t;
        sa_flags: s32;
    }
}

#if (OS == .MACOS && CPU == .ARM64) || (OS == .MACOS && CPU == .X64) {
    sigevent :: struct {
        sigev_notify: s32;
        sigev_signo: s32;
        sigev_value: sigval;
        sigev_notify_function: #type (unknown0: sigval) -> void #c_call;
        sigev_notify_attributes: *pthread_attr_t;
    }
}

sigval :: struct {
    sival_ptr: *void;

    #overlay (sival_ptr) sival_int: s32;
}

sigvec :: struct {
    sv_handler: #type (unknown0: s32) -> void #c_call;
    sv_mask: s32;
    sv_flags: s32;
}

#if (OS == .MACOS && CPU == .ARM64) || (OS == .MACOS && CPU == .X64) {
    stack_t :: struct {
        ss_sp: *void;
        ss_size: u64;
        ss_flags: s32;
    }
}

stat64_t :: struct {

    st_dev: dev_t;

    st_mode: mode_t;

    st_nlink: nlink_t;

    st_ino: u64;

    st_uid: uid_t;

    st_gid: gid_t;

    st_rdev: dev_t;

    st_atime: timespec;

    st_mtime: timespec;

    st_ctime: timespec;

    st_birthtime: timespec;

    st_size: s64;

    st_blocks: blkcnt_t;

    st_blksize: blksize_t;

    st_flags: u32;

    st_gen: u32;

    st_lspare: s32;

    st_qspare: [2] s64;
}

#if (OS == .MACOS && CPU == .ARM64) || (OS == .MACOS && CPU == .X64) {
    stat_t :: struct {

        st_dev: dev_t;

        st_mode: mode_t;

        st_nlink: nlink_t;

        st_ino: u64;

        st_uid: uid_t;

        st_gid: gid_t;

        st_rdev: dev_t;

        st_atime: timespec;

        st_mtime: timespec;

        st_ctime: timespec;

        st_birthtime: timespec;

        st_size: s64;

        st_blocks: blkcnt_t;

        st_blksize: blksize_t;

        st_flags: u32;

        st_gen: u32;

        st_lspare: s32;

        st_qspare: [2] s64;
    }
}

#if (OS == .MACOS && CPU == .ARM64) || (OS == .MACOS && CPU == .X64) {
    timespec :: struct {
        tv_sec: s64;
        tv_nsec: s64;
    }
}

#if (OS == .MACOS && CPU == .ARM64) || (OS == .MACOS && CPU == .X64) {
    timeval :: struct {
        tv_sec: s64;
        tv_usec: s32;
    }
}

#if (OS == .MACOS && CPU == .ARM64) || (OS == .MACOS && CPU == .X64) {
    timezone :: struct {
        tz_minuteswest: s32;
        tz_dsttime: s32;
    }
}

#if (OS == .MACOS && CPU == .ARM64) || (OS == .MACOS && CPU == .X64) {
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

ttysize :: struct {
    ts_lines: u16;
    ts_cols: u16;
    ts_xxx: u16;
    ts_yyy: u16;
}

#if (OS == .MACOS && CPU == .ARM64) || (OS == .MACOS && CPU == .X64) {
    ucontext_t :: struct {
        uc_onstack: s32;
        uc_sigmask: u32;
        uc_stack: stack_t;
        uc_link: *ucontext_t;
        uc_mcsize: u64;
        uc_mcontext: *__darwin_mcontext64;
    }
}

ucred :: struct {}

#if (OS == .MACOS && CPU == .ARM64) || (OS == .MACOS && CPU == .X64) {
    utsname :: struct {
        sysname: [256] u8;
        nodename: [256] u8;
        release: [256] u8;
        version: [256] u8;
        machine: [256] u8;
    }
}

vmspace :: struct {
    dummy: s32;
    dummy2: *u8;
    dummy3: [5] s32;
    dummy4: [3] *u8;
}

vol_attributes_attr :: struct {
    validattr: attribute_set_t;
    nativeattr: attribute_set_t;
}

vol_capabilities_attr :: struct {
    capabilities: vol_capabilities_set_t;
    valid: vol_capabilities_set_t;
}

vtagtype :: enum u32 {
    NON :: 0;

    UFS :: 1;

    NFS :: 2;
    MFS :: 3;
    MSDOSFS :: 4;
    LFS :: 5;

    LOFS :: 6;
    FDESC :: 7;
    PORTAL :: 8;
    NULL :: 9;
    UMAP :: 10;

    KERNFS :: 11;
    PROCFS :: 12;
    AFS :: 13;
    ISOFS :: 14;
    MOCKFS :: 15;

    HFS :: 16;
    ZFS :: 17;
    DEVFS :: 18;
    WEBDAV :: 19;
    UDF :: 20;

    AFP :: 21;
    CDDA :: 22;
    CIFS :: 23;
    OTHER :: 24;
    APFS :: 25;

    VT_NON :: NON;

    VT_UFS :: UFS;

    VT_NFS :: NFS;
    VT_MFS :: MFS;
    VT_MSDOSFS :: MSDOSFS;
    VT_LFS :: LFS;

    VT_LOFS :: LOFS;
    VT_FDESC :: FDESC;
    VT_PORTAL :: PORTAL;
    VT_NULL :: NULL;
    VT_UMAP :: UMAP;

    VT_KERNFS :: KERNFS;
    VT_PROCFS :: PROCFS;
    VT_AFS :: AFS;
    VT_ISOFS :: ISOFS;
    VT_MOCKFS :: MOCKFS;

    VT_HFS :: HFS;
    VT_ZFS :: ZFS;
    VT_DEVFS :: DEVFS;
    VT_WEBDAV :: WEBDAV;
    VT_UDF :: UDF;

    VT_AFP :: AFP;
    VT_CDDA :: CDDA;
    VT_CIFS :: CIFS;
    VT_OTHER :: OTHER;
    VT_APFS :: APFS;
}

vtype :: enum u32 {
    NON :: 0;

    REG :: 1;
    DIR :: 2;
    BLK :: 3;
    CHR :: 4;
    LNK :: 5;

    SOCK :: 6;
    FIFO :: 7;
    BAD :: 8;
    STR :: 9;
    CPLX :: 10;

    VNON :: NON;

    VREG :: REG;
    VDIR :: DIR;
    VBLK :: BLK;
    VCHR :: CHR;
    VLNK :: LNK;

    VSOCK :: SOCK;
    VFIFO :: FIFO;
    VBAD :: BAD;
    VSTR :: STR;
    VCPLX :: CPLX;
}

wait_t :: struct {
    w_status: s32;

    #overlay (w_status) w_T: struct {
        __bitfield: u32;

    };
    #overlay (w_status) w_S: struct {
        __bitfield: u32;

    };
}

xsw_usage :: struct {
    xsu_total: u64;
    xsu_avail: u64;
    xsu_used: u64;
    xsu_pagesize: u32;
    xsu_encrypted: u32;
}

sig_t :: #type (unknown0: s32) -> void #c_call;

__DARWIN_MAXNAMLEN :: 255;

__DARWIN_MAXPATHLEN :: 1024;

__DARWIN_NSIG :: 32;

__DTF_READALL :: 0x0008;

__DTF_SKIPREAD :: 0x0010;

__ILP32_OFF32 :: -1;

__ILP32_OFFBIG :: 1;

__ipc_perm_new :: ipc_perm;

__LP64_OFF64 :: 1;

__LPBIG_OFFBIG :: 1;

__PTHREAD_ATTR_SIZE__ :: 56;

__PTHREAD_COND_SIZE__ :: 40;

__PTHREAD_CONDATTR_SIZE__ :: 8;

__PTHREAD_MUTEX_SIZE__ :: 56;

__PTHREAD_MUTEXATTR_SIZE__ :: 8;

__PTHREAD_ONCE_SIZE__ :: 8;

__PTHREAD_RWLOCK_SIZE__ :: 192;

__PTHREAD_RWLOCKATTR_SIZE__ :: 16;

__PTHREAD_SIZE__ :: 8176;

__SALC :: 0x4000;

__SAPP :: 0x0100;

__SEOF :: 0x0020;

__SERR :: 0x0040;

__SIGN :: 0x8000;

__SLBF :: 0x0001;

__SMBF :: 0x0080;

__SMOD :: 0x2000;

__SNBF :: 0x0002;

__SNPT :: 0x0800;

__SOFF :: 0x1000;

__SOPT :: 0x0400;

__SRD :: 0x0004;

__SRW :: 0x0010;

__SSTR :: 0x0200;

__SWR :: 0x0008;

_ACCESS_EXTENDED_MASK :: _READ_OK | _WRITE_OK | _EXECUTE_OK | _DELETE_OK | _APPEND_OK | _RMFILE_OK | _REXT_OK | _WEXT_OK | _RATTR_OK | _WATTR_OK | _RPERM_OK | _WPERM_OK | _CHOWN_OK;

_APPEND_OK :: 1 << 13;

_CHOWN_OK :: 1 << 21;

_CS_DARWIN_USER_CACHE_DIR :: 65538;

_CS_DARWIN_USER_DIR :: 65536;

_CS_DARWIN_USER_TEMP_DIR :: 65537;

_CS_PATH :: 1;

_CS_POSIX_V6_ILP32_OFF32_CFLAGS :: 2;

_CS_POSIX_V6_ILP32_OFF32_LDFLAGS :: 3;

_CS_POSIX_V6_ILP32_OFF32_LIBS :: 4;

_CS_POSIX_V6_ILP32_OFFBIG_CFLAGS :: 5;

_CS_POSIX_V6_ILP32_OFFBIG_LDFLAGS :: 6;

_CS_POSIX_V6_ILP32_OFFBIG_LIBS :: 7;

_CS_POSIX_V6_LP64_OFF64_CFLAGS :: 8;

_CS_POSIX_V6_LP64_OFF64_LDFLAGS :: 9;

_CS_POSIX_V6_LP64_OFF64_LIBS :: 10;

_CS_POSIX_V6_LPBIG_OFFBIG_CFLAGS :: 11;

_CS_POSIX_V6_LPBIG_OFFBIG_LDFLAGS :: 12;

_CS_POSIX_V6_LPBIG_OFFBIG_LIBS :: 13;

_CS_POSIX_V6_WIDTH_RESTRICTED_ENVS :: 14;

_CS_XBS5_ILP32_OFF32_CFLAGS :: 20;

_CS_XBS5_ILP32_OFF32_LDFLAGS :: 21;

_CS_XBS5_ILP32_OFF32_LIBS :: 22;

_CS_XBS5_ILP32_OFF32_LINTFLAGS :: 23;

_CS_XBS5_ILP32_OFFBIG_CFLAGS :: 24;

_CS_XBS5_ILP32_OFFBIG_LDFLAGS :: 25;

_CS_XBS5_ILP32_OFFBIG_LIBS :: 26;

_CS_XBS5_ILP32_OFFBIG_LINTFLAGS :: 27;

_CS_XBS5_LP64_OFF64_CFLAGS :: 28;

_CS_XBS5_LP64_OFF64_LDFLAGS :: 29;

_CS_XBS5_LP64_OFF64_LIBS :: 30;

_CS_XBS5_LP64_OFF64_LINTFLAGS :: 31;

_CS_XBS5_LPBIG_OFFBIG_CFLAGS :: 32;

_CS_XBS5_LPBIG_OFFBIG_LDFLAGS :: 33;

_CS_XBS5_LPBIG_OFFBIG_LIBS :: 34;

_CS_XBS5_LPBIG_OFFBIG_LINTFLAGS :: 35;

_DELETE_OK :: 1 << 12;

_EXECUTE_OK :: 1 << 11;

_FILESEC_REMOVE_ACL :: cast(*void) 1;

_FILESEC_UNSET_PROPERTY :: cast(*void) 0;

_IOFBF :: 0;

_IOLBF :: 1;

_IONBF :: 2;

_JBLEN :: (9 * 2) + 3 + 16;

_MASTERPASSWD :: "master.passwd";

_MP_DB :: "pwd.db";

_PASSWD :: "passwd";

_PASSWORD_CHGNOW :: -1;

_PASSWORD_LEN :: 128;

_PASSWORD_NOCHG :: 0x04;

_PASSWORD_NOEXP :: 0x08;

_PASSWORD_NOGID :: 0x02;

_PASSWORD_NOUID :: 0x01;

_PASSWORD_WARNDAYS :: 14;

_PATH_MASTERPASSWD :: "/etc/master.passwd";

_PATH_MASTERPASSWD_LOCK :: "/etc/ptmp";

_PATH_MP_DB :: "/etc/pwd.db";

_PATH_PASSWD :: "/etc/passwd";

_PATH_PWD :: "/etc";

_PATH_PWD_MKDB :: "/usr/sbin/pwd_mkdb";

_PATH_SMP_DB :: "/etc/spwd.db";

_PTHREAD_MUTEX_POLICY_FAIRSHARE :: 1;

_PTHREAD_MUTEX_POLICY_FIRSTFIT :: 2;

_PTHREAD_MUTEX_POLICY_NONE :: 0;

_RATTR_OK :: 1 << 15;

_READ_OK :: 1 << 9;

_REXT_OK :: 1 << 17;

_RLIMIT_POSIX_FLAG :: 0x1000;

_RMFILE_OK :: 1 << 14;

_RPERM_OK :: 1 << 19;

_SMP_DB :: "spwd.db";

_SYS_NAMELEN :: 256;

_V6_ILP32_OFF32 :: __ILP32_OFF32;

_V6_ILP32_OFFBIG :: __ILP32_OFFBIG;

_V6_LP64_OFF64 :: __LP64_OFF64;

_V6_LPBIG_OFFBIG :: __LPBIG_OFFBIG;

_WATTR_OK :: 1 << 16;

_WEXT_OK :: 1 << 18;

_WPERM_OK :: 1 << 20;

_WRITE_OK :: 1 << 10;

_WSTOPPED :: 0x7f;

_XBS5_ILP32_OFF32 :: __ILP32_OFF32;

_XBS5_ILP32_OFFBIG :: __ILP32_OFFBIG;

_XBS5_LP64_OFF64 :: __LP64_OFF64;

_XBS5_LPBIG_OFFBIG :: __LPBIG_OFFBIG;

_XOPEN_CRYPT :: 1;

_XOPEN_ENH_I18N :: 1;

_XOPEN_LEGACY :: -1;

_XOPEN_REALTIME :: -1;

_XOPEN_REALTIME_THREADS :: -1;

_XOPEN_SHM :: 1;

_XOPEN_STREAMS :: -1;

_XOPEN_UNIX :: 1;

_XOPEN_VERSION :: 600;

_XOPEN_XCU_VERSION :: 4;

ACCESSPERMS :: S_IRWXU | S_IRWXG | S_IRWXO;

ACCESSX_MAX_DESCRIPTORS :: 100;

ACCESSX_MAX_TABLESIZE :: 16 * 1024;

AIO_ALLDONE :: 0x1;

AIO_CANCELED :: 0x2;

AIO_LISTIO_MAX :: 16;

AIO_NOTCANCELED :: 0x4;

ALLOCATEALL :: 0x00000004;

ALLOCATECONTIG :: 0x00000002;

ALLOCATEFROMPEOF :: 0x00000010;

ALLOCATEFROMVOL :: 0x00000020;

ALLPERMS :: S_ISUID | S_ISGID | S_ISTXT | S_IRWXU | S_IRWXG | S_IRWXO;

AT_EACCESS :: 0x0010;

AT_FDCWD :: -2;

AT_REMOVEDIR :: 0x0080;

AT_SYMLINK_FOLLOW :: 0x0040;

AT_SYMLINK_NOFOLLOW :: 0x0020;

ATTR_BIT_MAP_COUNT :: 5;

ATTR_BULK_REQUIRED :: xx ATTR_CMN.NAME | xx ATTR_CMN.RETURNED_ATTRS;

ATTR_MAX_BUFFER :: 8192;

attrgroup_t :: u32;

attribute_set_t :: attribute_set;

attrreference_t :: attrreference;

BADSIG :: SIG_ERR;

blkcnt_t :: s64;

blksize_t :: s32;

BUFSIZ :: 1024;

BUS_ADRALN :: 1;

BUS_ADRERR :: 2;

BUS_NOOP :: 0;

BUS_OBJERR :: 3;

CLD_CONTINUED :: 6;

CLD_DUMPED :: 3;

CLD_EXITED :: 1;

CLD_KILLED :: 2;

CLD_NOOP :: 0;

CLD_STOPPED :: 5;

CLD_TRAPPED :: 4;

clock_t :: u64;

CLOCKS_PER_SEC :: 1000000;

COMAPT_MAXLOGNAME :: 12;

CPF_IGNORE_MODE :: 0x0002;

CPF_MASK :: CPF_OVERWRITE | CPF_IGNORE_MODE;

CPF_OVERWRITE :: 0x0001;

CPUMON_MAKE_FATAL :: 0x1000;

CTL_DEBUG :: 5;

CTL_DEBUG_MAXID :: 20;

CTL_DEBUG_NAME :: 0;

CTL_DEBUG_VALUE :: 1;

CTL_HW :: 6;

CTL_KERN :: 1;

CTL_MACHDEP :: 7;

CTL_MAXID :: 9;

CTL_MAXNAME :: 12;

CTL_NET :: 4;

CTL_UNSPEC :: 0;

CTL_USER :: 8;

CTL_VFS :: 3;

CTL_VM :: 2;

CTLFLAG_ANYBODY :: 0x10000000;

CTLFLAG_KERN :: 0x01000000;

CTLFLAG_LOCKED :: 0x00800000;

CTLFLAG_MASKED :: 0x04000000;

CTLFLAG_NOAUTO :: 0x02000000;

CTLFLAG_NOLOCK :: 0x20000000;

CTLFLAG_OID2 :: 0x00400000;

CTLFLAG_RD :: 0x80000000;

CTLFLAG_RW :: CTLFLAG_RD | CTLFLAG_WR;

CTLFLAG_SECURE :: 0x08000000;

CTLFLAG_WR :: 0x40000000;

CTLTYPE :: 0xf;

CTLTYPE_INT :: 2;

CTLTYPE_NODE :: 1;

CTLTYPE_OPAQUE :: 5;

CTLTYPE_QUAD :: 4;

CTLTYPE_STRING :: 3;

CTLTYPE_STRUCT :: CTLTYPE_OPAQUE;

DEFFILEMODE :: S_IRUSR | S_IWUSR | S_IRGRP | S_IWGRP | S_IROTH | S_IWOTH;

dev_t :: s32;

DIR_MNTSTATUS_MNTPOINT :: 0x00000001;

DIR_MNTSTATUS_TRIGGER :: 0x00000002;

DIRBLKSIZ :: 1024;

Dl_info :: dl_info;

DST_AUST :: 2;

DST_CAN :: 6;

DST_EET :: 5;

DST_MET :: 4;

DST_NONE :: 0;

DST_USA :: 1;

DST_WET :: 3;

DTF_HIDEW :: 0x0001;

DTF_NODUP :: 0x0002;

DTF_REWIND :: 0x0004;

E2BIG :: 7;

EACCES :: 13;

EADDRINUSE :: 48;

EADDRNOTAVAIL :: 49;

EAFNOSUPPORT :: 47;

EAGAIN :: 35;

EALREADY :: 37;

EAUTH :: 80;

EBADARCH :: 86;

EBADEXEC :: 85;

EBADF :: 9;

EBADMACHO :: 88;

EBADMSG :: 94;

EBADRPC :: 72;

EBUSY :: 16;

ECANCELED :: 89;

ECHILD :: 10;

ECONNABORTED :: 53;

ECONNREFUSED :: 61;

ECONNRESET :: 54;

EDEADLK :: 11;

EDESTADDRREQ :: 39;

EDEVERR :: 83;

EDOM :: 33;

EDQUOT :: 69;

EEXIST :: 17;

EFAULT :: 14;

EFBIG :: 27;

EFTYPE :: 79;

EHOSTDOWN :: 64;

EHOSTUNREACH :: 65;

EIDRM :: 90;

EILSEQ :: 92;

EINPROGRESS :: 36;

EINTR :: 4;

EINVAL :: 22;

EIO :: 5;

EISCONN :: 56;

EISDIR :: 21;

ELAST :: 106;

ELOOP :: 62;

EMFILE :: 24;

EMLINK :: 31;

EMSGSIZE :: 40;

EMULTIHOP :: 95;

ENAMETOOLONG :: 63;

ENEEDAUTH :: 81;

ENETDOWN :: 50;

ENETRESET :: 52;

ENETUNREACH :: 51;

ENFILE :: 23;

ENOATTR :: 93;

ENOBUFS :: 55;

ENODATA :: 96;

ENODEV :: 19;

ENOENT :: 2;

ENOEXEC :: 8;

ENOLCK :: 77;

ENOLINK :: 97;

ENOMEM :: 12;

ENOMSG :: 91;

ENOPOLICY :: 103;

ENOPROTOOPT :: 42;

ENOSPC :: 28;

ENOSR :: 98;

ENOSTR :: 99;

ENOSYS :: 78;

ENOTBLK :: 15;

ENOTCONN :: 57;

ENOTDIR :: 20;

ENOTEMPTY :: 66;

ENOTRECOVERABLE :: 104;

ENOTSOCK :: 38;

ENOTSUP :: 45;

ENOTTY :: 25;

ENXIO :: 6;

EOF :: -1;

EOPNOTSUPP :: 102;

EOVERFLOW :: 84;

EOWNERDEAD :: 105;

EPERM :: 1;

EPFNOSUPPORT :: 46;

EPIPE :: 32;

EPROC_CTTY :: 0x01;

EPROC_SLEADER :: 0x02;

EPROCLIM :: 67;

EPROCUNAVAIL :: 76;

EPROGMISMATCH :: 75;

EPROGUNAVAIL :: 74;

EPROTO :: 100;

EPROTONOSUPPORT :: 43;

EPROTOTYPE :: 41;

EPWROFF :: 82;

EQFULL :: 106;

ERANGE :: 34;

EREMOTE :: 71;

EROFS :: 30;

ERPCMISMATCH :: 73;

ESHLIBVERS :: 87;

ESHUTDOWN :: 58;

ESOCKTNOSUPPORT :: 44;

ESPIPE :: 29;

ESRCH :: 3;

ESTALE :: 70;

ETIME :: 101;

ETIMEDOUT :: 60;

ETOOMANYREFS :: 59;

ETXTBSY :: 26;

EUSERS :: 68;

EWOULDBLOCK :: EAGAIN;

EXDEV :: 18;

extentrecord :: [8] diskextent;

F_ADDFILESIGS :: 61;

F_ADDFILESIGS_FOR_DYLD_SIM :: 83;

F_ADDFILESIGS_RETURN :: 97;

F_ADDSIGS :: 59;

F_ALLOCATEALL :: 0x00000004;

F_ALLOCATECONTIG :: 0x00000002;

F_BARRIERFSYNC :: 85;

F_CHECK_LV :: 98;

F_CHKCLEAN :: 41;

F_DUPFD :: 0;

F_DUPFD_CLOEXEC :: 67;

F_FINDSIGS :: 78;

F_FLUSH_DATA :: 40;

F_FREEZE_FS :: 53;

F_FULLFSYNC :: 51;

F_GETCODEDIR :: 72;

F_GETFD :: 1;

F_GETFL :: 3;

F_GETLK :: 7;

F_GETLKPID :: 66;

F_GETNOSIGPIPE :: 74;

F_GETOWN :: 5;

F_GETPATH :: 50;

F_GETPATH_MTMINFO :: 71;

F_GETPROTECTIONCLASS :: 63;

F_GETPROTECTIONLEVEL :: 77;

F_GLOBAL_NOCACHE :: 55;

F_LOCK :: 1;

F_LOG2PHYS :: 49;

F_LOG2PHYS_EXT :: 65;

F_NOCACHE :: 48;

F_NODIRECT :: 62;

F_OK :: 0;

F_PATHPKG_CHECK :: 52;

F_PEOFPOSMODE :: 3;

F_PREALLOCATE :: 42;

F_PUNCHHOLE :: 99;

F_RDADVISE :: 44;

F_RDAHEAD :: 45;

F_RDLCK :: 1;

F_SETBACKINGSTORE :: 70;

F_SETFD :: 2;

F_SETFL :: 4;

F_SETLK :: 8;

F_SETLKW :: 9;

F_SETLKWTIMEOUT :: 10;

F_SETNOSIGPIPE :: 73;

F_SETOWN :: 6;

F_SETPROTECTIONCLASS :: 64;

F_SETSIZE :: 43;

F_SINGLE_WRITER :: 76;

F_TEST :: 3;

F_THAW_FS :: 54;

F_TLOCK :: 2;

F_TRANSCODEKEY :: 75;

F_TRIM_ACTIVE_FILE :: 100;

F_ULOCK :: 0;

F_UNLCK :: 2;

F_VOLPOSMODE :: 4;

F_WRLCK :: 3;

FAPPEND :: O_APPEND;

FASYNC :: O_ASYNC;

fbootstraptransfer_t :: fbootstraptransfer;

fchecklv_t :: fchecklv;

FCNTL_FS_SPECIFIC_BASE :: 0x00010000;

fcodeblobs_t :: fcodeblobs;

FD_CLOEXEC :: 1;

FFSYNC :: O_FSYNC;

FILENAME_MAX :: 1024;

filesec_t :: *_filesec;

fixpt_t :: u32;

FNDELAY :: O_NONBLOCK;

FNM_CASEFOLD :: 0x10;

FNM_FILE_NAME :: FNM_PATHNAME;

FNM_IGNORECASE :: FNM_CASEFOLD;

FNM_LEADING_DIR :: 0x08;

FNM_NOESCAPE :: 0x01;

FNM_NOMATCH :: 1;

FNM_NOSYS :: -1;

FNM_PATHNAME :: 0x02;

FNM_PERIOD :: 0x04;

FNONBLOCK :: O_NONBLOCK;

FOPEN_MAX :: 20;

FPE_FLTDIV :: 1;

FPE_FLTINV :: 5;

FPE_FLTOVF :: 2;

FPE_FLTRES :: 4;

FPE_FLTSUB :: 6;

FPE_FLTUND :: 3;

FPE_INTDIV :: 7;

FPE_INTOVF :: 8;

FPE_NOOP :: 0;

fpos_t :: s64;

fpunchhole_t :: fpunchhole;

FREAD :: 0x0001;

FREEREMAINDER :: 0x00000008;

fsfile_type_t :: u32;

fsignatures_t :: fsignatures;

fsobj_tag_t :: u32;

fsobj_type_t :: u32;

FSOPT_ATTR_CMN_EXTENDED :: 0x00000020;

FSOPT_NOFOLLOW :: 0x00000001;

FSOPT_NOINMEMUPDATE :: 0x00000002;

FSOPT_PACK_INVAL_ATTRS :: 0x00000008;

FSOPT_REPORT_FULLSIZE :: 0x00000004;

FST_EOF :: -1;

fstore_t :: fstore;

fsvolid_t :: u32;

ftello64 :: ftello;

ftrimactivefile_t :: ftrimactivefile;

FWRITE :: 0x0002;

gid_t :: u32;

HW_AVAILCPU :: 25;

HW_BUS_FREQ :: 14;

HW_BYTEORDER :: 4;

HW_CACHELINE :: 16;

HW_CPU_FREQ :: 15;

HW_DISKNAMES :: 8;

HW_DISKSTATS :: 9;

HW_EPOCH :: 10;

HW_FLOATINGPT :: 11;

HW_L1DCACHESIZE :: 18;

HW_L1ICACHESIZE :: 17;

HW_L2CACHESIZE :: 20;

HW_L2SETTINGS :: 19;

HW_L3CACHESIZE :: 22;

HW_L3SETTINGS :: 21;

HW_MACHINE :: 1;

HW_MACHINE_ARCH :: 12;

HW_MAXID :: 26;

HW_MEMSIZE :: 24;

HW_MODEL :: 2;

HW_NCPU :: 3;

HW_PAGESIZE :: 7;

HW_PHYSMEM :: 5;

HW_TB_FREQ :: 23;

HW_USERMEM :: 6;

HW_VECTORUNIT :: 13;

id_t :: u32;

ILL_BADSTK :: 8;

ILL_COPROC :: 7;

ILL_ILLADR :: 5;

ILL_ILLOPC :: 1;

ILL_ILLOPN :: 4;

ILL_ILLTRP :: 2;

ILL_NOOP :: 0;

ILL_PRVOPC :: 3;

ILL_PRVREG :: 6;

ino_t :: u64;

IOPOL_APPLICATION :: IOPOL_STANDARD;

IOPOL_DEFAULT :: 0;

IOPOL_IMPORTANT :: 1;

IOPOL_NORMAL :: IOPOL_IMPORTANT;

IOPOL_PASSIVE :: 2;

IOPOL_SCOPE_DARWIN_BG :: 2;

IOPOL_SCOPE_PROCESS :: 0;

IOPOL_SCOPE_THREAD :: 1;

IOPOL_STANDARD :: 5;

IOPOL_THROTTLE :: 3;

IOPOL_TYPE_DISK :: 0;

IOPOL_UTILITY :: 4;

IPC_CREAT :: 0x200;

IPC_EXCL :: 0x400;

IPC_M :: 0x1000;

IPC_NOWAIT :: 0x800;

IPC_PRIVATE :: cast(key_t) 0;

IPC_R :: 0x100;

IPC_RMID :: 0;

IPC_SET :: 1;

IPC_STAT :: 2;

IPC_W :: 0x80;

ITIMER_PROF :: 2;

ITIMER_REAL :: 0;

ITIMER_VIRTUAL :: 1;

jmp_buf :: [37] s32;

KERN_AFFINITY :: 43;

KERN_AIOMAX :: 46;

KERN_AIOPROCMAX :: 47;

KERN_AIOTHREADS :: 48;

KERN_ARGMAX :: 8;

KERN_BOOTFILE :: 28;

KERN_BOOTTIME :: 21;

KERN_CHECKOPENEVT :: 70;

KERN_CLASSIC :: KERN_TRANSLATE;

KERN_CLASSICHANDLER :: KERN_EXEC;

KERN_CLOCKRATE :: 12;

KERN_COREDUMP :: 51;

KERN_COREFILE :: 50;

KERN_DOMAINNAME :: KERN_NISDOMAINNAME;

KERN_DUMMY :: 33;

KERN_DUMPDEV :: 31;

KERN_EXEC :: 45;

KERN_FILE :: 15;

KERN_HOSTID :: 11;

KERN_HOSTNAME :: 10;

KERN_IPC :: 32;

KERN_JOB_CONTROL :: 19;

KERN_KDBUFWAIT :: 23;

KERN_KDCPUMAP :: 24;

KERN_KDDFLAGS :: 2;

KERN_KDEBUG :: 24;

KERN_KDEFLAGS :: 1;

KERN_KDENABLE :: 3;

KERN_KDGETBUF :: 5;

KERN_KDGETENTROPY :: 16;

KERN_KDGETREG :: 9;

KERN_KDPIDEX :: 14;

KERN_KDPIDTR :: 11;

KERN_KDREADCURTHRMAP :: 21;

KERN_KDREADTR :: 10;

KERN_KDREMOVE :: 7;

KERN_KDSET_TYPEFILTER :: 22;

KERN_KDSETBUF :: 4;

KERN_KDSETREG :: 8;

KERN_KDSETRTCDEC :: 15;

KERN_KDSETUP :: 6;

KERN_KDTEST :: 19;

KERN_KDTHRMAP :: 12;

KERN_KDWRITEMAP :: 18;

KERN_KDWRITEMAP_V3 :: 27;

KERN_KDWRITETR :: 17;

KERN_KDWRITETR_V3 :: 28;

KERN_LOGSIGEXIT :: 36;

KERN_LOW_PRI_DELAY :: 57;

KERN_LOW_PRI_WINDOW :: 56;

KERN_MAXFILES :: 7;

KERN_MAXFILESPERPROC :: 29;

KERN_MAXID :: 72;

KERN_MAXPARTITIONS :: 23;

KERN_MAXPROC :: 6;

KERN_MAXPROCPERUID :: 30;

KERN_MAXVNODES :: 5;

KERN_NETBOOT :: 40;

KERN_NGROUPS :: 18;

KERN_NISDOMAINNAME :: 22;

KERN_NTP_PLL :: 27;

KERN_NX_PROTECTION :: 60;

KERN_OPENEVT_PROC :: 1;

KERN_OSRELDATE :: 26;

KERN_OSRELEASE :: 2;

KERN_OSREV :: 3;

KERN_OSTYPE :: 1;

KERN_OSVERSION :: 65;

KERN_POSIX :: 58;

KERN_POSIX1 :: 17;

KERN_PROC :: 14;

KERN_PROC_ALL :: 0;

KERN_PROC_LCID :: 7;

KERN_PROC_PGRP :: 2;

KERN_PROC_PID :: 1;

KERN_PROC_RUID :: 6;

KERN_PROC_SESSION :: 3;

KERN_PROC_TTY :: 4;

KERN_PROC_UID :: 5;

KERN_PROCARGS :: 38;

KERN_PROCARGS2 :: 49;

KERN_PROCDELAYTERM :: 53;

KERN_PROCNAME :: 62;

KERN_PROF :: 16;

KERN_PS_STRINGS :: 34;

KERN_RAGE_PROC :: 1;

KERN_RAGE_THREAD :: 2;

KERN_RAGEVNODE :: 68;

KERN_SAFEBOOT :: 66;

KERN_SAVED_IDS :: 20;

KERN_SECURELVL :: 9;

KERN_SHREG_PRIVATIZABLE :: 54;

KERN_SPECULATIVE_READS :: 64;

KERN_SUGID_COREDUMP :: 52;

KERN_SYMFILE :: 37;

KERN_SYSV :: 42;

KERN_TFP :: 61;

KERN_TFP_POLICY :: 1;

KERN_TFP_POLICY_DEFAULT :: 2;

KERN_TFP_POLICY_DENY :: 0;

KERN_THALTSTACK :: 63;

KERN_THREADNAME :: 71;

KERN_TRANSLATE :: 44;

KERN_TTY :: 69;

KERN_UNOPENEVT_PROC :: 2;

KERN_UNRAGE_PROC :: 3;

KERN_UNRAGE_THREAD :: 4;

KERN_UPDATEINTERVAL :: 25;

KERN_USRSTACK :: KERN_USRSTACK64;

KERN_USRSTACK32 :: 35;

KERN_USRSTACK64 :: 59;

KERN_VERSION :: 4;

KERN_VNODE :: 13;

key_t :: s32;

KIPC_MAX_DATALEN :: 7;

KIPC_MAX_HDR :: 6;

KIPC_MAX_LINKHDR :: 4;

KIPC_MAX_PROTOHDR :: 5;

KIPC_MAXSOCKBUF :: 1;

KIPC_MBSTAT :: 8;

KIPC_NMBCLUSTERS :: 9;

KIPC_SOCKBUF_WASTE :: 2;

KIPC_SOMAXCONN :: 3;

KIPC_SOQLIMITCOMPAT :: 10;

L_ctermid :: 1024;

L_tmpnam :: 1024;

LIO_NOP :: 0x0;

LIO_NOWAIT :: 0x1;

LIO_READ :: 0x1;

LIO_WAIT :: 0x2;

LIO_WRITE :: 0x2;

LOCK_EX :: 0x02;

LOCK_NB :: 0x04;

LOCK_SH :: 0x01;

LOCK_UN :: 0x08;

LSCALE :: 1000;

mach_port_t :: u32;

MADV_CAN_REUSE :: 9;

MADV_DONTNEED :: 4;

MADV_FREE :: 5;

MADV_FREE_REUSABLE :: 7;

MADV_FREE_REUSE :: 8;

MADV_NORMAL :: 0;

MADV_PAGEOUT :: 10;

MADV_RANDOM :: 1;

MADV_SEQUENTIAL :: 2;

MADV_WILLNEED :: 3;

MADV_ZERO_WIRED_PAGES :: 6;

MAP_ANON :: 0x1000;

MAP_ANONYMOUS :: MAP_ANON;

MAP_COPY :: MAP_PRIVATE;

MAP_FAILED :: cast,trunc(*void) -1;

MAP_FILE :: 0x0000;

MAP_FIXED :: 0x0010;

MAP_HASSEMAPHORE :: 0x0200;

MAP_JIT :: 0x0800;

MAP_NOCACHE :: 0x0400;

MAP_NOEXTEND :: 0x0100;

MAP_NORESERVE :: 0x0040;

MAP_PRIVATE :: 0x0002;

MAP_RENAME :: 0x0020;

MAP_RESERVED0080 :: 0x0080;

MAP_RESILIENT_CODESIGN :: 0x2000;

MAP_RESILIENT_MEDIA :: 0x4000;

MAP_SHARED :: 0x0001;

MAXNAMLEN :: __DARWIN_MAXNAMLEN;

MCL_CURRENT :: 0x0001;

MCL_FUTURE :: 0x0002;

MINCORE_ANONYMOUS :: 0x80;

MINCORE_COPIED :: 0x40;

MINCORE_INCORE :: 0x1;

MINCORE_MODIFIED :: 0x4;

MINCORE_MODIFIED_OTHER :: 0x10;

MINCORE_PAGED_OUT :: 0x20;

MINCORE_REFERENCED :: 0x2;

MINCORE_REFERENCED_OTHER :: 0x8;

MINSIGSTKSZ :: 32768;

mode_t :: u16;

MS_ASYNC :: 0x0001;

MS_DEACTIVATE :: 0x0008;

MS_INVALIDATE :: 0x0002;

MS_KILLPAGES :: 0x0004;

MS_SYNC :: 0x0010;

nfds_t :: u32;

nlink_t :: u16;

NSIG :: __DARWIN_NSIG;

O_ACCMODE :: 0x0003;

O_ALERT :: 0x20000000;

O_APPEND :: 0x0008;

O_ASYNC :: 0x0040;

O_CLOEXEC :: 0x1000000;

O_CREAT :: 0x0200;

O_DIRECTORY :: 0x100000;

O_DP_GETRAWENCRYPTED :: 0x0001;

O_DP_GETRAWUNENCRYPTED :: 0x0002;

O_EVTONLY :: 0x8000;

O_EXCL :: 0x0800;

O_EXLOCK :: 0x0020;

O_FSYNC :: O_SYNC;

O_NDELAY :: O_NONBLOCK;

O_NOCTTY :: 0x20000;

O_NOFOLLOW :: 0x0100;

O_NONBLOCK :: 0x0004;

O_POPUP :: 0x80000000;

O_RDONLY :: 0x0000;

O_RDWR :: 0x0002;

O_SHLOCK :: 0x0010;

O_SYMLINK :: 0x200000;

O_SYNC :: 0x0080;

O_TRUNC :: 0x0400;

O_WRONLY :: 0x0001;

OID_AUTO :: -1;

OID_AUTO_START :: 100;

P_tmpdir :: "/var/tmp/";

P_TRACED :: 0x00000800;

pid_t :: s32;

POLL_ERR :: 4;

POLL_HUP :: 6;

POLL_IN :: 1;

POLL_MSG :: 3;

POLL_OUT :: 2;

POLL_PRI :: 5;

POLLATTRIB :: 0x0400;

POLLERR :: 0x0008;

POLLEXTEND :: 0x0200;

POLLHUP :: 0x0010;

POLLIN :: 0x0001;

POLLNLINK :: 0x0800;

POLLNVAL :: 0x0020;

POLLOUT :: 0x0004;

POLLPRI :: 0x0002;

POLLRDBAND :: 0x0080;

POLLRDNORM :: 0x0040;

POLLSTANDARD :: POLLIN | POLLPRI | POLLOUT | POLLRDNORM | POLLRDBAND | POLLWRBAND | POLLERR | POLLHUP | POLLNVAL;

POLLWRBAND :: 0x0100;

POLLWRITE :: 0x1000;

POLLWRNORM :: POLLOUT;

PREALLOCATE :: 0x00000001;

PRIO_MAX :: 20;

PRIO_MIN :: -20;

PROT_EXEC :: 0x04;

PROT_NONE :: 0x00;

PROT_READ :: 0x01;

PROT_WRITE :: 0x02;

pthread_attr_t :: _opaque_pthread_attr_t;

PTHREAD_CANCEL_ASYNCHRONOUS :: 0x00;

PTHREAD_CANCEL_DEFERRED :: 0x02;

PTHREAD_CANCEL_DISABLE :: 0x00;

PTHREAD_CANCEL_ENABLE :: 0x01;

PTHREAD_CANCELED :: cast(*void) 1;

pthread_cond_t :: _opaque_pthread_cond_t;

pthread_condattr_t :: _opaque_pthread_condattr_t;

PTHREAD_CREATE_DETACHED :: 2;

PTHREAD_CREATE_JOINABLE :: 1;

PTHREAD_EXPLICIT_SCHED :: 2;

PTHREAD_INHERIT_SCHED :: 1;

pthread_key_t :: u64;

PTHREAD_MUTEX_DEFAULT :: PTHREAD_MUTEX_NORMAL;

PTHREAD_MUTEX_ERRORCHECK :: 1;

PTHREAD_MUTEX_NORMAL :: 0;

PTHREAD_MUTEX_RECURSIVE :: 2;

pthread_mutex_t :: _opaque_pthread_mutex_t;

pthread_mutexattr_t :: _opaque_pthread_mutexattr_t;

pthread_once_t :: _opaque_pthread_once_t;

pthread_override_t :: *pthread_override_s;

PTHREAD_PRIO_INHERIT :: 1;

PTHREAD_PRIO_NONE :: 0;

PTHREAD_PRIO_PROTECT :: 2;

PTHREAD_PROCESS_PRIVATE :: 2;

PTHREAD_PROCESS_SHARED :: 1;

pthread_rwlock_t :: _opaque_pthread_rwlock_t;

pthread_rwlockattr_t :: _opaque_pthread_rwlockattr_t;

PTHREAD_SCOPE_PROCESS :: 2;

PTHREAD_SCOPE_SYSTEM :: 1;

pthread_t :: *_opaque_pthread_t;

qos_class_t :: u32;

QOS_MIN_RELATIVE_PRIORITY :: -15;

R_OK :: 1 << 2;

RLIM_INFINITY :: (cast(u64) 1 << 63) - 1;

RLIM_NLIMITS :: 9;

RLIM_SAVED_CUR :: RLIM_INFINITY;

RLIM_SAVED_MAX :: RLIM_INFINITY;

rlim_t :: u64;

RLIMIT_CPU_USAGE_MONITOR :: 0x2;

RLIMIT_THREAD_CPULIMITS :: 0x3;

RLIMIT_WAKEUPS_MONITOR :: 0x1;

RTLD_DEFAULT :: cast,trunc(*void) -2;

RTLD_FIRST :: 0x100;

RTLD_GLOBAL :: 0x8;

RTLD_LAZY :: 0x1;

RTLD_LOCAL :: 0x4;

RTLD_MAIN_ONLY :: cast,trunc(*void) -5;

RTLD_NEXT :: cast,trunc(*void) -1;

RTLD_NODELETE :: 0x80;

RTLD_NOLOAD :: 0x10;

RTLD_NOW :: 0x2;

RTLD_SELF :: cast,trunc(*void) -3;

rusage_info_current :: rusage_info_v4;

rusage_info_t :: *void;

S_BLKSIZE :: 512;

S_IEXEC :: S_IXUSR;

S_IFBLK :: 0x6000;

S_IFCHR :: 0x2000;

S_IFDIR :: 0x4000;

S_IFIFO :: 0x1000;

S_IFLNK :: 0xa000;

S_IFMT :: 0xf000;

S_IFREG :: 0x8000;

S_IFSOCK :: 0xc000;

S_IFWHT :: 0xe000;

S_IREAD :: S_IRUSR;

S_IRGRP :: 0x20;

S_IROTH :: 0x4;

S_IRUSR :: 0x100;

S_IRWXG :: 0x38;

S_IRWXO :: 0x7;

S_IRWXU :: 0x1c0;

S_ISGID :: 0x400;

S_ISTXT :: S_ISVTX;

S_ISUID :: 0x800;

S_ISVTX :: 0x200;

S_IWGRP :: 0x10;

S_IWOTH :: 0x2;

S_IWRITE :: S_IWUSR;

S_IWUSR :: 0x80;

S_IXGRP :: 0x8;

S_IXOTH :: 0x1;

S_IXUSR :: 0x40;

SA_64REGSET :: 0x0200;

SA_NOCLDSTOP :: 0x0008;

SA_NOCLDWAIT :: 0x0020;

SA_NODEFER :: 0x0010;

SA_ONSTACK :: 0x0001;

SA_RESETHAND :: 0x0004;

SA_RESTART :: 0x0002;

SA_SIGINFO :: 0x0040;

SA_USERSPACE_MASK :: SA_ONSTACK | SA_RESTART | SA_RESETHAND | SA_NOCLDSTOP | SA_NODEFER | SA_NOCLDWAIT | SA_SIGINFO;

SA_USERTRAMP :: 0x0100;

SEARCHFS_MAX_SEARCHPARMS :: 4096;

SEEK_CUR :: 1;

SEEK_END :: 2;

SEEK_SET :: 0;

segsz_t :: s32;

SEGV_ACCERR :: 2;

SEGV_MAPERR :: 1;

SEGV_NOOP :: 0;

sem_t :: s32;

SF_APPEND :: 0x00040000;

SF_ARCHIVED :: 0x00010000;

SF_IMMUTABLE :: 0x00020000;

SF_NOUNLINK :: 0x00100000;

SF_RESTRICTED :: 0x00080000;

SF_SETTABLE :: 0xffff0000;

SF_SUPPORTED :: 0x001f0000;

SHM_R :: IPC_R;

SHM_RDONLY :: 0x1000;

SHM_RND :: 0x2000;

SHM_W :: IPC_W;

shmatt_t :: u16;

shmid_ds :: __shmid_ds_new;

SHMLBA :: 4096;

SI_ASYNCIO :: 0x10004;

SI_MESGQ :: 0x10005;

SI_QUEUE :: 0x10002;

SI_TIMER :: 0x10003;

SI_USER :: 0x10001;

sig_atomic_t :: s32;

SIG_BLOCK :: 1;

SIG_DFL :: cast(*void) 0;

SIG_ERR :: cast(*void) -1;

SIG_HOLD :: cast(*void) 5;

SIG_IGN :: cast(*void) 1;

SIG_SETMASK :: 3;

SIG_UNBLOCK :: 2;

SIGABRT :: 6;

SIGALRM :: 14;

SIGBUS :: 10;

SIGCHLD :: 20;

SIGCONT :: 19;

SIGEMT :: 7;

SIGEV_NONE :: 0;

SIGEV_SIGNAL :: 1;

SIGEV_THREAD :: 3;

SIGFPE :: 8;

SIGHUP :: 1;

SIGILL :: 4;

SIGINFO :: 29;

siginfo_t :: __siginfo;

SIGINT :: 2;

SIGIO :: 23;

SIGIOT :: SIGABRT;

sigjmp_buf :: [38] s32;

SIGKILL :: 9;

SIGPIPE :: 13;

SIGPROF :: 27;

SIGQUIT :: 3;

SIGSEGV :: 11;

sigset_t :: u32;

SIGSTKSZ :: 131072;

SIGSTOP :: 17;

SIGSYS :: 12;

SIGTERM :: 15;

SIGTRAP :: 5;

SIGTSTP :: 18;

SIGTTIN :: 21;

SIGTTOU :: 22;

SIGURG :: 16;

SIGUSR1 :: 30;

SIGUSR2 :: 31;

SIGVTALRM :: 26;

SIGWINCH :: 28;

SIGXCPU :: 24;

SIGXFSZ :: 25;

SRCHFS_MATCHDIRS :: 0x00000004;

SRCHFS_MATCHFILES :: 0x00000008;

SRCHFS_MATCHPARTIALNAMES :: 0x00000002;

SRCHFS_NEGATEPARAMS :: 0x80000000;

SRCHFS_SKIPINAPPROPRIATE :: 0x00000080;

SRCHFS_SKIPINVISIBLE :: 0x00000020;

SRCHFS_SKIPLINKS :: 0x00000010;

SRCHFS_SKIPPACKAGES :: 0x00000040;

SRCHFS_START :: 0x00000001;

SRCHFS_VALIDOPTIONSMASK :: 0x800000FF;

SS_DISABLE :: 0x0004;

SS_ONSTACK :: 0x0001;

STDERR_FILENO :: 2;

STDIN_FILENO :: 0;

STDOUT_FILENO :: 1;

suseconds_t :: s32;

SV_INTERRUPT :: SA_RESTART;

SV_NOCLDSTOP :: SA_NOCLDSTOP;

SV_NODEFER :: SA_NODEFER;

SV_ONSTACK :: SA_ONSTACK;

SV_RESETHAND :: SA_RESETHAND;

SV_SIGINFO :: SA_SIGINFO;

SYNC_VOLUME_FULLSYNC :: 0x01;

SYNC_VOLUME_WAIT :: 0x02;

SYS___channel_get_info :: 511;

SYS___channel_get_opt :: 513;

SYS___channel_open :: 510;

SYS___channel_set_opt :: 514;

SYS___channel_sync :: 512;

SYS___disable_threadsignal :: 331;

SYS___mac_execve :: 380;

SYS___mac_get_fd :: 388;

SYS___mac_get_file :: 382;

SYS___mac_get_link :: 384;

SYS___mac_get_mount :: 425;

SYS___mac_get_pid :: 390;

SYS___mac_get_proc :: 386;

SYS___mac_getfsstat :: 426;

SYS___mac_mount :: 424;

SYS___mac_set_fd :: 389;

SYS___mac_set_file :: 383;

SYS___mac_set_link :: 385;

SYS___mac_set_proc :: 387;

SYS___mac_syscall :: 381;

SYS___nexus_create :: 506;

SYS___nexus_deregister :: 505;

SYS___nexus_destroy :: 507;

SYS___nexus_get_opt :: 508;

SYS___nexus_open :: 503;

SYS___nexus_register :: 504;

SYS___nexus_set_opt :: 509;

SYS___old_semwait_signal :: 370;

SYS___old_semwait_signal_nocancel :: 371;

SYS___pthread_canceled :: 333;

SYS___pthread_chdir :: 348;

SYS___pthread_fchdir :: 349;

SYS___pthread_kill :: 328;

SYS___pthread_markcancel :: 332;

SYS___pthread_sigmask :: 329;

SYS___semwait_signal :: 334;

SYS___semwait_signal_nocancel :: 423;

SYS___sigwait :: 330;

SYS___sigwait_nocancel :: 422;

SYS_abort_with_payload :: 521;

SYS_accept :: 30;

SYS_accept_nocancel :: 404;

SYS_access :: 33;

SYS_access_extended :: 284;

SYS_acct :: 51;

SYS_adjtime :: 140;

SYS_aio_cancel :: 316;

SYS_aio_error :: 317;

SYS_aio_fsync :: 313;

SYS_aio_read :: 318;

SYS_aio_return :: 314;

SYS_aio_suspend :: 315;

SYS_aio_suspend_nocancel :: 421;

SYS_aio_write :: 319;

SYS_audit :: 350;

SYS_audit_session_join :: 429;

SYS_audit_session_port :: 432;

SYS_audit_session_self :: 428;

SYS_auditctl :: 359;

SYS_auditon :: 351;

SYS_bind :: 104;

SYS_bsdthread_create :: 360;

SYS_bsdthread_ctl :: 478;

SYS_bsdthread_register :: 366;

SYS_bsdthread_terminate :: 361;

SYS_change_fdguard_np :: 444;

SYS_chdir :: 12;

SYS_chflags :: 34;

SYS_chmod :: 15;

SYS_chmod_extended :: 282;

SYS_chown :: 16;

SYS_chroot :: 61;

SYS_clonefileat :: 462;

SYS_close :: 6;

SYS_close_nocancel :: 399;

SYS_coalition :: 458;

SYS_coalition_info :: 459;

SYS_connect :: 98;

SYS_connect_nocancel :: 409;

SYS_connectx :: 447;

SYS_copyfile :: 227;

SYS_csops :: 169;

SYS_csops_audittoken :: 170;

SYS_csrctl :: 483;

SYS_delete :: 226;

SYS_disconnectx :: 448;

SYS_dup :: 41;

SYS_dup2 :: 90;

SYS_exchangedata :: 223;

SYS_execve :: 59;

SYS_exit :: 1;

SYS_faccessat :: 466;

SYS_fchdir :: 13;

SYS_fchflags :: 35;

SYS_fchmod :: 124;

SYS_fchmod_extended :: 283;

SYS_fchmodat :: 467;

SYS_fchown :: 123;

SYS_fchownat :: 468;

SYS_fclonefileat :: 517;

SYS_fcntl :: 92;

SYS_fcntl_nocancel :: 406;

SYS_fdatasync :: 187;

SYS_ffsctl :: 245;

SYS_fgetattrlist :: 228;

SYS_fgetxattr :: 235;

SYS_fhopen :: 248;

SYS_fileport_makefd :: 431;

SYS_fileport_makeport :: 430;

SYS_flistxattr :: 241;

SYS_flock :: 131;

SYS_fmount :: 526;

SYS_fork :: 2;

SYS_fpathconf :: 192;

SYS_fremovexattr :: 239;

SYS_fs_snapshot :: 518;

SYS_fsctl :: 242;

SYS_fsetattrlist :: 229;

SYS_fsetxattr :: 237;

SYS_fsgetpath :: 427;

SYS_fstat :: 189;

SYS_fstat64 :: 339;

SYS_fstat64_extended :: 343;

SYS_fstat_extended :: 281;

SYS_fstatat :: 469;

SYS_fstatat64 :: 470;

SYS_fstatfs :: 158;

SYS_fstatfs64 :: 346;

SYS_fsync :: 95;

SYS_fsync_nocancel :: 408;

SYS_ftruncate :: 201;

SYS_futimes :: 139;

SYS_getattrlist :: 220;

SYS_getattrlistat :: 476;

SYS_getattrlistbulk :: 461;

SYS_getaudit_addr :: 357;

SYS_getauid :: 353;

SYS_getdirentries :: 196;

SYS_getdirentries64 :: 344;

SYS_getdirentriesattr :: 222;

SYS_getdtablesize :: 89;

SYS_getegid :: 43;

SYS_getentropy :: 500;

SYS_geteuid :: 25;

SYS_getfh :: 161;

SYS_getfsstat :: 18;

SYS_getfsstat64 :: 347;

SYS_getgid :: 47;

SYS_getgroups :: 79;

SYS_gethostuuid :: 142;

SYS_getitimer :: 86;

SYS_getlogin :: 49;

SYS_getpeername :: 31;

SYS_getpgid :: 151;

SYS_getpgrp :: 81;

SYS_getpid :: 20;

SYS_getppid :: 39;

SYS_getpriority :: 100;

SYS_getrlimit :: 194;

SYS_getrusage :: 117;

SYS_getsgroups :: 288;

SYS_getsid :: 310;

SYS_getsockname :: 32;

SYS_getsockopt :: 118;

SYS_gettid :: 286;

SYS_gettimeofday :: 116;

SYS_getuid :: 24;

SYS_getwgroups :: 290;

SYS_getxattr :: 234;

SYS_grab_pgo_data :: 493;

SYS_guarded_close_np :: 442;

SYS_guarded_kqueue_np :: 443;

SYS_guarded_open_dprotected_np :: 484;

SYS_guarded_open_np :: 441;

SYS_guarded_pwrite_np :: 486;

SYS_guarded_write_np :: 485;

SYS_guarded_writev_np :: 487;

SYS_identitysvc :: 293;

SYS_initgroups :: 243;

SYS_invalid :: 63;

SYS_ioctl :: 54;

SYS_iopolicysys :: 322;

SYS_issetugid :: 327;

SYS_kas_info :: 439;

SYS_kdebug_trace :: 180;

SYS_kdebug_trace64 :: 179;

SYS_kdebug_trace_string :: 178;

SYS_kdebug_typefilter :: 177;

SYS_kevent :: 363;

SYS_kevent64 :: 369;

SYS_kevent_id :: 375;

SYS_kevent_qos :: 374;

SYS_kill :: 37;

SYS_kqueue :: 362;

SYS_lchown :: 364;

SYS_ledger :: 373;

SYS_link :: 9;

SYS_linkat :: 471;

SYS_lio_listio :: 320;

SYS_listen :: 106;

SYS_listxattr :: 240;

SYS_lseek :: 199;

SYS_lstat :: 190;

SYS_lstat64 :: 340;

SYS_lstat64_extended :: 342;

SYS_lstat_extended :: 280;

SYS_madvise :: 75;

SYS_MAXSYSCALL :: 530;

SYS_memorystatus_control :: 440;

SYS_memorystatus_get_level :: 453;

SYS_microstackshot :: 492;

SYS_mincore :: 78;

SYS_minherit :: 250;

SYS_mkdir :: 136;

SYS_mkdir_extended :: 292;

SYS_mkdirat :: 475;

SYS_mkfifo :: 132;

SYS_mkfifo_extended :: 291;

SYS_mknod :: 14;

SYS_mlock :: 203;

SYS_mlockall :: 324;

SYS_mmap :: 197;

SYS_modwatch :: 233;

SYS_mount :: 167;

SYS_mprotect :: 74;

SYS_mremap_encrypted :: 489;

SYS_msgctl :: 258;

SYS_msgget :: 259;

SYS_msgrcv :: 261;

SYS_msgrcv_nocancel :: 419;

SYS_msgsnd :: 260;

SYS_msgsnd_nocancel :: 418;

SYS_msgsys :: 252;

SYS_msync :: 65;

SYS_msync_nocancel :: 405;

SYS_munlock :: 204;

SYS_munlockall :: 325;

SYS_munmap :: 73;

SYS_necp_client_action :: 502;

SYS_necp_match_policy :: 460;

SYS_necp_open :: 501;

SYS_necp_session_action :: 523;

SYS_necp_session_open :: 522;

SYS_net_qos_guideline :: 525;

SYS_netagent_trigger :: 490;

SYS_nfsclnt :: 247;

SYS_nfssvc :: 155;

SYS_ntp_adjtime :: 527;

SYS_ntp_gettime :: 528;

SYS_open :: 5;

SYS_open_dprotected_np :: 216;

SYS_open_extended :: 277;

SYS_open_nocancel :: 398;

SYS_openat :: 463;

SYS_openat_nocancel :: 464;

SYS_openbyid_np :: 479;

SYS_os_fault_with_payload :: 529;

SYS_pathconf :: 191;

SYS_peeloff :: 449;

SYS_persona :: 494;

SYS_pid_hibernate :: 435;

SYS_pid_resume :: 434;

SYS_pid_shutdown_sockets :: 436;

SYS_pid_suspend :: 433;

SYS_pipe :: 42;

SYS_poll :: 230;

SYS_poll_nocancel :: 417;

SYS_posix_spawn :: 244;

SYS_pread :: 153;

SYS_pread_nocancel :: 414;

SYS_proc_info :: 336;

SYS_proc_rlimit_control :: 446;

SYS_proc_trace_log :: 477;

SYS_proc_uuid_policy :: 452;

SYS_process_policy :: 323;

SYS_pselect :: 394;

SYS_pselect_nocancel :: 395;

SYS_psynch_cvbroad :: 303;

SYS_psynch_cvclrprepost :: 312;

SYS_psynch_cvsignal :: 304;

SYS_psynch_cvwait :: 305;

SYS_psynch_mutexdrop :: 302;

SYS_psynch_mutexwait :: 301;

SYS_psynch_rw_downgrade :: 299;

SYS_psynch_rw_longrdlock :: 297;

SYS_psynch_rw_rdlock :: 306;

SYS_psynch_rw_unlock :: 308;

SYS_psynch_rw_unlock2 :: 309;

SYS_psynch_rw_upgrade :: 300;

SYS_psynch_rw_wrlock :: 307;

SYS_psynch_rw_yieldwrlock :: 298;

SYS_ptrace :: 26;

SYS_pwrite :: 154;

SYS_pwrite_nocancel :: 415;

SYS_quotactl :: 165;

SYS_read :: 3;

SYS_read_nocancel :: 396;

SYS_readlink :: 58;

SYS_readlinkat :: 473;

SYS_readv :: 120;

SYS_readv_nocancel :: 411;

SYS_reboot :: 55;

SYS_recvfrom :: 29;

SYS_recvfrom_nocancel :: 403;

SYS_recvmsg :: 27;

SYS_recvmsg_nocancel :: 401;

SYS_recvmsg_x :: 480;

SYS_removexattr :: 238;

SYS_rename :: 128;

SYS_renameat :: 465;

SYS_renameatx_np :: 488;

SYS_revoke :: 56;

SYS_rmdir :: 137;

SYS_searchfs :: 225;

SYS_select :: 93;

SYS_select_nocancel :: 407;

SYS_sem_close :: 269;

SYS_sem_open :: 268;

SYS_sem_post :: 273;

SYS_sem_trywait :: 272;

SYS_sem_unlink :: 270;

SYS_sem_wait :: 271;

SYS_sem_wait_nocancel :: 420;

SYS_semctl :: 254;

SYS_semget :: 255;

SYS_semop :: 256;

SYS_semsys :: 251;

SYS_sendfile :: 337;

SYS_sendmsg :: 28;

SYS_sendmsg_nocancel :: 402;

SYS_sendmsg_x :: 481;

SYS_sendto :: 133;

SYS_sendto_nocancel :: 413;

SYS_setattrlist :: 221;

SYS_setattrlistat :: 524;

SYS_setaudit_addr :: 358;

SYS_setauid :: 354;

SYS_setegid :: 182;

SYS_seteuid :: 183;

SYS_setgid :: 181;

SYS_setgroups :: 80;

SYS_setitimer :: 83;

SYS_setlogin :: 50;

SYS_setpgid :: 82;

SYS_setpriority :: 96;

SYS_setprivexec :: 152;

SYS_setregid :: 127;

SYS_setreuid :: 126;

SYS_setrlimit :: 195;

SYS_setsgroups :: 287;

SYS_setsid :: 147;

SYS_setsockopt :: 105;

SYS_settid :: 285;

SYS_settid_with_pid :: 311;

SYS_settimeofday :: 122;

SYS_setuid :: 23;

SYS_setwgroups :: 289;

SYS_setxattr :: 236;

SYS_sfi_ctl :: 456;

SYS_sfi_pidctl :: 457;

SYS_shared_region_check_np :: 294;

SYS_shared_region_map_and_slide_np :: 438;

SYS_shm_open :: 266;

SYS_shm_unlink :: 267;

SYS_shmat :: 262;

SYS_shmctl :: 263;

SYS_shmdt :: 264;

SYS_shmget :: 265;

SYS_shmsys :: 253;

SYS_shutdown :: 134;

SYS_sigaction :: 46;

SYS_sigaltstack :: 53;

SYS_sigpending :: 52;

SYS_sigprocmask :: 48;

SYS_sigreturn :: 184;

SYS_sigsuspend :: 111;

SYS_sigsuspend_nocancel :: 410;

SYS_socket :: 97;

SYS_socket_delegate :: 450;

SYS_socketpair :: 135;

SYS_stack_snapshot_with_config :: 491;

SYS_stat :: 188;

SYS_stat64 :: 338;

SYS_stat64_extended :: 341;

SYS_stat_extended :: 279;

SYS_statfs :: 157;

SYS_statfs64 :: 345;

SYS_swapon :: 85;

SYS_symlink :: 57;

SYS_symlinkat :: 474;

SYS_sync :: 36;

SYS_syscall :: 0;

SYS_sysctl :: 202;

SYS_sysctlbyname :: 274;

SYS_system_override :: 454;

SYS_telemetry :: 451;

SYS_terminate_with_payload :: 520;

SYS_thread_selfcounts :: 186;

SYS_thread_selfid :: 372;

SYS_thread_selfusage :: 482;

SYS_truncate :: 200;

SYS_ulock_wait :: 515;

SYS_ulock_wake :: 516;

SYS_umask :: 60;

SYS_umask_extended :: 278;

SYS_undelete :: 205;

SYS_unlink :: 10;

SYS_unlinkat :: 472;

SYS_unmount :: 159;

SYS_usrctl :: 445;

SYS_utimes :: 138;

SYS_vfork :: 66;

SYS_vfs_purge :: 455;

SYS_vm_pressure_monitor :: 296;

SYS_wait4 :: 7;

SYS_wait4_nocancel :: 400;

SYS_waitevent :: 232;

SYS_waitid :: 173;

SYS_waitid_nocancel :: 416;

SYS_watchevent :: 231;

SYS_work_interval_ctl :: 499;

SYS_workq_kernreturn :: 368;

SYS_workq_open :: 367;

SYS_write :: 4;

SYS_write_nocancel :: 397;

SYS_writev :: 121;

SYS_writev_nocancel :: 412;

text_encoding_t :: u32;

time_t :: s64;

TMP_MAX :: 308915776;

TRAP_BRKPT :: 1;

TRAP_TRACE :: 2;

UF_APPEND :: 0x00000004;

UF_COMPRESSED :: 0x00000020;

UF_DATAVAULT :: 0x00000080;

UF_HIDDEN :: 0x00008000;

UF_IMMUTABLE :: 0x00000002;

UF_NODUMP :: 0x00000001;

UF_OPAQUE :: 0x00000008;

UF_SETTABLE :: 0x0000ffff;

UF_TRACKED :: 0x00000040;

uid_t :: u32;

useconds_t :: u32;

USER_BC_BASE_MAX :: 2;

USER_BC_DIM_MAX :: 3;

USER_BC_SCALE_MAX :: 4;

USER_BC_STRING_MAX :: 5;

USER_COLL_WEIGHTS_MAX :: 6;

USER_CS_PATH :: 1;

USER_EXPR_NEST_MAX :: 7;

USER_LINE_MAX :: 8;

USER_MAXID :: 21;

USER_POSIX2_C_BIND :: 11;

USER_POSIX2_C_DEV :: 12;

USER_POSIX2_CHAR_TERM :: 13;

USER_POSIX2_FORT_DEV :: 14;

USER_POSIX2_FORT_RUN :: 15;

USER_POSIX2_LOCALEDEF :: 16;

USER_POSIX2_SW_DEV :: 17;

USER_POSIX2_UPE :: 18;

USER_POSIX2_VERSION :: 10;

USER_RE_DUP_MAX :: 9;

USER_STREAM_MAX :: 19;

USER_TZNAME_MAX :: 20;

UTIME_NOW :: -1;

UTIME_OMIT :: -2;

uuid_t :: [16] u8;

VM_LOADAVG :: 2;

VM_MACHFACTOR :: 4;

VM_MAXID :: 6;

VM_METER :: 1;

VM_SWAPUSAGE :: 5;

VNODE_BLOCKMAP_NO_TRACK :: 0x04;

VNODE_READ :: 0x01;

VNODE_WRITE :: 0x02;

VNOVAL :: -1;

vol_attributes_attr_t :: vol_attributes_attr;

VOL_CAP_FMT_2TB_FILESIZE :: 0x00000800;

VOL_CAP_FMT_64BIT_OBJECT_IDS :: 0x00020000;

VOL_CAP_FMT_CASE_PRESERVING :: 0x00000200;

VOL_CAP_FMT_CASE_SENSITIVE :: 0x00000100;

VOL_CAP_FMT_DECMPFS_COMPRESSION :: 0x00010000;

VOL_CAP_FMT_DIR_HARDLINKS :: 0x00040000;

VOL_CAP_FMT_DOCUMENT_ID :: 0x00080000;

VOL_CAP_FMT_FAST_STATFS :: 0x00000400;

VOL_CAP_FMT_HARDLINKS :: 0x00000004;

VOL_CAP_FMT_HIDDEN_FILES :: 0x00002000;

VOL_CAP_FMT_JOURNAL :: 0x00000008;

VOL_CAP_FMT_JOURNAL_ACTIVE :: 0x00000010;

VOL_CAP_FMT_NO_IMMUTABLE_FILES :: 0x00200000;

VOL_CAP_FMT_NO_PERMISSIONS :: 0x00400000;

VOL_CAP_FMT_NO_ROOT_TIMES :: 0x00000020;

VOL_CAP_FMT_NO_VOLUME_SIZES :: 0x00008000;

VOL_CAP_FMT_OPENDENYMODES :: 0x00001000;

VOL_CAP_FMT_PATH_FROM_ID :: 0x00004000;

VOL_CAP_FMT_PERSISTENTOBJECTIDS :: 0x00000001;

VOL_CAP_FMT_SPARSE_FILES :: 0x00000040;

VOL_CAP_FMT_SYMBOLICLINKS :: 0x00000002;

VOL_CAP_FMT_WRITE_GENERATION_COUNT :: 0x00100000;

VOL_CAP_FMT_ZERO_RUNS :: 0x00000080;

VOL_CAP_INT_ADVLOCK :: 0x00000100;

VOL_CAP_INT_ALLOCATE :: 0x00000040;

VOL_CAP_INT_ATTRLIST :: 0x00000002;

VOL_CAP_INT_CLONE :: 0x00010000;

VOL_CAP_INT_COPYFILE :: 0x00000020;

VOL_CAP_INT_EXCHANGEDATA :: 0x00000010;

VOL_CAP_INT_EXTENDED_ATTR :: 0x00004000;

VOL_CAP_INT_EXTENDED_SECURITY :: 0x00000400;

VOL_CAP_INT_FLOCK :: 0x00000200;

VOL_CAP_INT_MANLOCK :: 0x00001000;

VOL_CAP_INT_NAMEDSTREAMS :: 0x00002000;

VOL_CAP_INT_NFSEXPORT :: 0x00000004;

VOL_CAP_INT_READDIRATTR :: 0x00000008;

VOL_CAP_INT_RENAME_EXCL :: 0x00080000;

VOL_CAP_INT_RENAME_SWAP :: 0x00040000;

VOL_CAP_INT_SEARCHFS :: 0x00000001;

VOL_CAP_INT_SNAPSHOT :: 0x00020000;

VOL_CAP_INT_USERACCESS :: 0x00000800;

VOL_CAP_INT_VOL_RENAME :: 0x00000080;

vol_capabilities_attr_t :: vol_capabilities_attr;

VOL_CAPABILITIES_FORMAT :: 0;

VOL_CAPABILITIES_INTERFACES :: 1;

VOL_CAPABILITIES_RESERVED1 :: 2;

VOL_CAPABILITIES_RESERVED2 :: 3;

vol_capabilities_set_t :: [4] u32;

W_OK :: 1 << 1;

WAIT_ANY :: -1;

WAIT_MYPGRP :: 0;

WAKEMON_DISABLE :: 0x02;

WAKEMON_ENABLE :: 0x01;

WAKEMON_GET_PARAMS :: 0x04;

WAKEMON_MAKE_FATAL :: 0x10;

WAKEMON_SET_DEFAULTS :: 0x08;

WCONTINUED :: 0x00000010;

WCOREFLAG :: 0x80;

WEXITED :: 0x00000004;

WMESGLEN :: 7;

WNOHANG :: 0x00000001;

WNOWAIT :: 0x00000020;

WSTOPPED :: 0x00000008;

WUNTRACED :: 0x00000002;

X_OK :: 1 << 0;

tzname: **u8 #elsewhere libc;

getdate_err: s32 #elsewhere libc;

daylight: s32 #elsewhere libc;

optarg: *u8 #elsewhere libc;

optind: s32 #elsewhere libc;

opterr: s32 #elsewhere libc;

optopt: s32 #elsewhere libc;

suboptarg: *u8 #elsewhere libc;

optreset: s32 #elsewhere libc;

stdin: *FILE #elsewhere libc "__stdinp";

stdout: *FILE #elsewhere libc "__stdoutp";

stderr: *FILE #elsewhere libc "__stderrp";

sys_nerr: s32 #elsewhere libc;

sys_errlist: **u8 #elsewhere libc;

__errno_location :: () -> *OS_Error_Code #foreign libc "__error";

__opendir2 :: (unknown0: *u8, unknown1: s32) -> *DIR #foreign libc "__opendir2$INODE64";

__srget :: (unknown0: *FILE) -> s32 #foreign libc;

__swbuf :: (unknown0: s32, unknown1: *FILE) -> s32 #foreign libc;

_exit :: (unknown0: s32) -> void #foreign libc; @Exits

_Exit :: (unknown0: s32) -> void #foreign libc; @Exits

_longjmp :: (unknown0: *jmp_buf, unknown1: s32) -> void #foreign libc;

_pthread_mutex_enable_legacy_mode :: () -> void #foreign libc;

_remove :: (unknown0: *u8) -> s32 #foreign libc "remove";

_setjmp :: (unknown0: *jmp_buf) -> s32 #foreign libc;

access :: (unknown0: *u8, unknown1: s32) -> s32 #foreign libc;

acct :: (unknown0: *u8) -> s32 #foreign libc;

adjtime :: (unknown0: *timeval, unknown1: *timeval) -> s32 #foreign libc;

aio_cancel :: (fd: s32, aiocbp: *aiocb) -> s32 #foreign libc;

aio_error :: (aiocbp: *aiocb) -> s32 #foreign libc;

aio_fsync :: (op: s32, aiocbp: *aiocb) -> s32 #foreign libc;

aio_read :: (aiocbp: *aiocb) -> s32 #foreign libc;

aio_return :: (aiocbp: *aiocb) -> s64 #foreign libc;

aio_suspend :: (aiocblist: **aiocb, nent: s32, timeoutp: *timespec) -> s32 #foreign libc;

aio_write :: (aiocbp: *aiocb) -> s32 #foreign libc;

alarm :: (unknown0: u32) -> u32 #foreign libc;

alphasort :: (unknown0: **dirent, unknown1: **dirent) -> s32 #foreign libc "alphasort$INODE64";

asctime :: (unknown0: *tm) -> *u8 #foreign libc;

asctime_r :: (unknown0: *tm, unknown1: *u8) -> *u8 #foreign libc;

asprintf :: (unknown0: **u8, unknown1: *u8, __args: ..Any) -> s32 #foreign libc;

backtrace :: (unknown0: **void, unknown1: s32) -> s32 #foreign libc;

backtrace_symbols :: (unknown0: **void, unknown1: s32) -> **u8 #foreign libc;

backtrace_symbols_fd :: (unknown0: **void, unknown1: s32, unknown2: s32) -> void #foreign libc;

brk :: (unknown0: *void) -> *void #foreign libc;

bsd_signal :: (unknown0: s32, unknown1: s32, unknown2: #type (unknown0: s32) -> void #c_call) -> ((unknown0: s32, unknown1: s32, unknown2: ((unknown0: s32) -> void #c_call)) -> void #c_call) #foreign libc;

chdir :: (unknown0: *u8) -> s32 #foreign libc;

chflags :: (unknown0: *u8, unknown1: u32) -> s32 #foreign libc;

chmod :: (unknown0: *u8, unknown1: mode_t) -> s32 #foreign libc;

chmodx_np :: (unknown0: *u8, unknown1: filesec_t) -> s32 #foreign libc;

chown :: (unknown0: *u8, unknown1: uid_t, unknown2: gid_t) -> s32 #foreign libc;

chroot :: (unknown0: *u8) -> s32 #foreign libc;

clearerr :: (unknown0: *FILE) -> void #foreign libc;

clock :: () -> clock_t #foreign libc;

clock_getres :: (__clock_id: clockid_t, __res: *timespec) -> s32 #foreign libc;

clock_gettime :: (__clock_id: clockid_t, __tp: *timespec) -> s32 #foreign libc;

clock_gettime_nsec_np :: (__clock_id: clockid_t) -> u64 #foreign libc;

clock_settime :: (__clock_id: clockid_t, __tp: *timespec) -> s32 #foreign libc;

close :: (unknown0: s32) -> s32 #foreign libc; @FreesArgument

closedir :: (unknown0: *DIR) -> s32 #foreign libc;

confstr :: (unknown0: s32, unknown1: *u8, unknown2: u64) -> u64 #foreign libc;

creat :: (unknown0: *u8, unknown1: mode_t) -> s32 #foreign libc;

crypt :: (unknown0: *u8, unknown1: *u8) -> *u8 #foreign libc;

ctermid :: (unknown0: *u8) -> *u8 #foreign libc;

ctermid_r :: (unknown0: *u8) -> *u8 #foreign libc;

ctime :: (unknown0: *time_t) -> *u8 #foreign libc;

ctime_r :: (unknown0: *time_t, unknown1: *u8) -> *u8 #foreign libc;

difftime :: (unknown0: time_t, unknown1: time_t) -> float64 #foreign libc;

dirfd :: (dirp: *DIR) -> s32 #foreign libc;

dladdr :: (unknown0: *void, unknown1: *Dl_info) -> s32 #foreign libc;

dlclose :: (__handle: *void) -> s32 #foreign libc;

dlerror :: () -> *u8 #foreign libc;

dlopen :: (__path: *u8, __mode: s32) -> *void #foreign libc;

dlopen_preflight :: (__path: *u8) -> bool #foreign libc;

dlsym :: (__handle: *void, __symbol: *u8) -> *void #foreign libc;

dprintf :: (unknown0: s32, unknown1: *u8, __args: ..Any) -> s32 #foreign libc;

dup :: (unknown0: s32) -> s32 #foreign libc;

dup2 :: (unknown0: s32, unknown1: s32) -> s32 #foreign libc;

encrypt :: (unknown0: *u8, unknown1: s32) -> void #foreign libc;

endpwent :: () -> void #foreign libc;

endusershell :: () -> void #foreign libc;

exchangedata :: (unknown0: *u8, unknown1: *u8, unknown2: u32) -> s32 #foreign libc;

execl :: (__path: *u8, __arg0: *u8, __args: ..Any) -> s32 #foreign libc;

execle :: (__path: *u8, __arg0: *u8, __args: ..Any) -> s32 #foreign libc;

execlp :: (__file: *u8, __arg0: *u8, __args: ..Any) -> s32 #foreign libc;

execv :: (__path: *u8, __argv: **u8) -> s32 #foreign libc;

execve :: (__file: *u8, __argv: **u8, __envp: **u8) -> s32 #foreign libc;

execvp :: (__file: *u8, __argv: **u8) -> s32 #foreign libc;

execvP :: (__file: *u8, __searchpath: *u8, __argv: **u8) -> s32 #foreign libc;

faccessat :: (unknown0: s32, unknown1: *u8, unknown2: s32, unknown3: s32) -> s32 #foreign libc;

fchdir :: (unknown0: s32) -> s32 #foreign libc;

fchflags :: (unknown0: s32, unknown1: u32) -> s32 #foreign libc;

fchmod :: (unknown0: s32, unknown1: mode_t) -> s32 #foreign libc;

fchmodat :: (unknown0: s32, unknown1: *u8, unknown2: mode_t, unknown3: s32) -> s32 #foreign libc;

fchmodx_np :: (unknown0: s32, unknown1: filesec_t) -> s32 #foreign libc;

fchown :: (unknown0: s32, unknown1: uid_t, unknown2: gid_t) -> s32 #foreign libc;

fchownat :: (unknown0: s32, unknown1: *u8, unknown2: uid_t, unknown3: gid_t, unknown4: s32) -> s32 #foreign libc;

fclose :: (unknown0: *FILE) -> s32 #foreign libc; @FreesArgument

fcntl :: (unknown0: s32, unknown1: s32, __args: ..Any) -> s32 #foreign libc;

fdopen :: (unknown0: s32, unknown1: *u8) -> *FILE #foreign libc;

fdopendir :: (unknown0: s32) -> *DIR #foreign libc "fdopendir$INODE64";

feof :: (unknown0: *FILE) -> s32 #foreign libc;

ferror :: (unknown0: *FILE) -> s32 #foreign libc;

fflagstostr :: (unknown0: u64) -> *u8 #foreign libc;

fflush :: (unknown0: *FILE) -> s32 #foreign libc;

ffsctl :: (unknown0: s32, unknown1: u64, unknown2: *void, unknown3: u32) -> s32 #foreign libc;

fgetattrlist :: (unknown0: s32, unknown1: *void, unknown2: *void, unknown3: u64, unknown4: u32) -> s32 #foreign libc;

fgetc :: (unknown0: *FILE) -> s32 #foreign libc;

fgetln :: (unknown0: *FILE, unknown1: *u64) -> *u8 #foreign libc;

fgetpos :: (unknown0: *FILE, unknown1: *fpos_t) -> s32 #foreign libc;

fgets :: (unknown0: *u8, unknown1: s32, unknown2: *FILE) -> *u8 #foreign libc;

fileno :: (unknown0: *FILE) -> s32 #foreign libc;

filesec_dup :: (unknown0: filesec_t) -> filesec_t #foreign libc;

filesec_free :: (unknown0: filesec_t) -> void #foreign libc;

filesec_get_property :: (unknown0: filesec_t, unknown1: filesec_property_t, unknown2: *void) -> s32 #foreign libc;

filesec_init :: () -> filesec_t #foreign libc;

filesec_query_property :: (unknown0: filesec_t, unknown1: filesec_property_t, unknown2: *s32) -> s32 #foreign libc;

filesec_set_property :: (unknown0: filesec_t, unknown1: filesec_property_t, unknown2: *void) -> s32 #foreign libc;

filesec_unset_property :: (unknown0: filesec_t, unknown1: filesec_property_t) -> s32 #foreign libc;

flock_ :: (unknown0: s32, unknown1: s32) -> s32 #foreign libc "flock";

flockfile :: (unknown0: *FILE) -> void #foreign libc;

fmemopen :: (__buf: *void, __size: u64, __mode: *u8) -> *FILE #foreign libc;

fmtcheck :: (unknown0: *u8, unknown1: *u8) -> *u8 #foreign libc;

fnmatch :: (unknown0: *u8, unknown1: *u8, unknown2: s32) -> s32 #foreign libc;

fopen :: (__filename: *u8, __mode: *u8) -> *FILE #foreign libc;

fork :: () -> pid_t #foreign libc;

fpathconf :: (unknown0: s32, __name: PC) -> s64 #foreign libc;

fprintf :: (unknown0: *FILE, unknown1: *u8, __args: ..Any) -> s32 #foreign libc;

fpurge :: (unknown0: *FILE) -> s32 #foreign libc;

fputc :: (unknown0: s32, unknown1: *FILE) -> s32 #foreign libc;

fputs :: (unknown0: *u8, unknown1: *FILE) -> s32 #foreign libc;

fread :: (__ptr: *void, __size: u64, __nitems: u64, __stream: *FILE) -> u64 #foreign libc;

freopen :: (unknown0: *u8, unknown1: *u8, unknown2: *FILE) -> *FILE #foreign libc;

fscanf :: (unknown0: *FILE, unknown1: *u8, __args: ..Any) -> s32 #foreign libc;

fsctl :: (unknown0: *u8, unknown1: u64, unknown2: *void, unknown3: u32) -> s32 #foreign libc;

fseek :: (unknown0: *FILE, unknown1: s64, unknown2: s32) -> s32 #foreign libc;

fseeko :: (__stream: *FILE, __offset: s64, __whence: s32) -> s32 #foreign libc;

fsetattrlist :: (unknown0: s32, unknown1: *void, unknown2: *void, unknown3: u64, unknown4: u32) -> s32 #foreign libc;

fsetpos :: (unknown0: *FILE, unknown1: *fpos_t) -> s32 #foreign libc;

fstat :: (unknown0: s32, unknown1: *stat_t) -> s32 #foreign libc "fstat$INODE64";

fstat64 :: (unknown0: s32, unknown1: *stat64_t) -> s32 #foreign libc;

fstatat :: (unknown0: s32, unknown1: *u8, unknown2: *stat_t, unknown3: s32) -> s32 #foreign libc "fstatat$INODE64";

fstatx64_np :: (unknown0: s32, unknown1: *stat64_t, unknown2: filesec_t) -> s32 #foreign libc;

fstatx_np :: (unknown0: s32, unknown1: *stat_t, unknown2: filesec_t) -> s32 #foreign libc "fstatx_np$INODE64";

fsync :: (unknown0: s32) -> s32 #foreign libc;

fsync_volume_np :: (unknown0: s32, unknown1: s32) -> s32 #foreign libc;

ftell :: (unknown0: *FILE) -> s64 #foreign libc;

ftello :: (__stream: *FILE) -> s64 #foreign libc;

ftok :: (unknown0: *u8, unknown1: s32) -> key_t #foreign libc;

ftruncate :: (unknown0: s32, unknown1: s64) -> s32 #foreign libc;

ftrylockfile :: (unknown0: *FILE) -> s32 #foreign libc;

funlockfile :: (unknown0: *FILE) -> void #foreign libc;

funopen :: (unknown0: *void, unknown1: #type (unknown0: *void, unknown1: *u8, unknown2: s32) -> s32 #c_call, unknown2: #type (unknown0: *void, unknown1: *u8, unknown2: s32) -> s32 #c_call, unknown3: #type (unknown0: *void, unknown1: fpos_t, unknown2: s32) -> fpos_t #c_call, unknown4: #type (unknown0: *void) -> s32 #c_call) -> *FILE #foreign libc;

futimens :: (__fd: s32, __times: *[2] timespec) -> s32 #foreign libc;

futimes :: (unknown0: s32, unknown1: *timeval) -> s32 #foreign libc;

fwrite :: (__ptr: *void, __size: u64, __nitems: u64, __stream: *FILE) -> u64 #foreign libc;

getattrlist :: (unknown0: *u8, unknown1: *void, unknown2: *void, unknown3: u64, unknown4: u32) -> s32 #foreign libc;

getattrlistat :: (unknown0: s32, unknown1: *u8, unknown2: *void, unknown3: *void, unknown4: u64, unknown5: u64) -> s32 #foreign libc;

getattrlistbulk :: (unknown0: s32, unknown1: *void, unknown2: *void, unknown3: u64, unknown4: u64) -> s32 #foreign libc;

getc :: (unknown0: *FILE) -> s32 #foreign libc;

getc_unlocked :: (unknown0: *FILE) -> s32 #foreign libc;

getchar :: () -> s32 #foreign libc;

getchar_unlocked :: () -> s32 #foreign libc;

getcwd :: (unknown0: *u8, unknown1: u64) -> *u8 #foreign libc;

getdate :: (unknown0: *u8) -> *tm #foreign libc;

getdelim :: (__linep: **u8, __linecapp: *u64, __delimiter: s32, __stream: *FILE) -> s64 #foreign libc;

getdirentriesattr :: (unknown0: s32, unknown1: *void, unknown2: *void, unknown3: u64, unknown4: *u32, unknown5: *u32, unknown6: *u32, unknown7: u32) -> s32 #foreign libc;

getdomainname :: (unknown0: *u8, unknown1: s32) -> s32 #foreign libc;

getdtablesize :: () -> s32 #foreign libc;

getegid :: () -> gid_t #foreign libc;

getenv :: (unknown0: *u8) -> *u8 #foreign libc;

geteuid :: () -> uid_t #foreign libc;

getgid :: () -> gid_t #foreign libc;

getgrouplist :: (unknown0: *u8, unknown1: s32, unknown2: *s32, unknown3: *s32) -> s32 #foreign libc;

getgroups :: (unknown0: s32, unknown1: *gid_t) -> s32 #foreign libc;

gethostid :: () -> s64 #foreign libc;

gethostname :: (unknown0: *u8, unknown1: u64) -> s32 #foreign libc;

getiopolicy_np :: (unknown0: s32, unknown1: s32) -> s32 #foreign libc;

getitimer :: (unknown0: s32, unknown1: *itimerval) -> s32 #foreign libc;

getline :: (__linep: **u8, __linecapp: *u64, __stream: *FILE) -> s64 #foreign libc;

getlogin :: () -> *u8 #foreign libc;

getlogin_r :: (unknown0: *u8, unknown1: u64) -> s32 #foreign libc;

getmode :: (unknown0: *void, unknown1: mode_t) -> mode_t #foreign libc;

getopt :: (unknown0: s32, unknown1: **u8, unknown2: *u8) -> s32 #foreign libc;

getpagesize :: () -> s32 #foreign libc;

getpass :: (unknown0: *u8) -> *u8 #foreign libc;

getpeereid :: (unknown0: s32, unknown1: *uid_t, unknown2: *gid_t) -> s32 #foreign libc;

getpgid :: (unknown0: pid_t) -> pid_t #foreign libc;

getpgrp :: () -> pid_t #foreign libc;

getpid :: () -> pid_t #foreign libc;

getppid :: () -> pid_t #foreign libc;

getpriority :: (unknown0: PRIO, unknown1: id_t) -> s32 #foreign libc;

getpwent :: () -> *passwd #foreign libc;

getpwnam :: (unknown0: *u8) -> *passwd #foreign libc;

getpwnam_r :: (unknown0: *u8, unknown1: *passwd, unknown2: *u8, unknown3: u64, unknown4: **passwd) -> s32 #foreign libc;

getpwuid :: (unknown0: uid_t) -> *passwd #foreign libc;

getpwuid_r :: (unknown0: uid_t, unknown1: *passwd, unknown2: *u8, unknown3: u64, unknown4: **passwd) -> s32 #foreign libc;

getpwuuid :: (unknown0: *uuid_t) -> *passwd #foreign libc;

getpwuuid_r :: (unknown0: *uuid_t, unknown1: *passwd, unknown2: *u8, unknown3: u64, unknown4: **passwd) -> s32 #foreign libc;

getrlimit :: (unknown0: RLIMIT_RESOURCE, unknown1: *rlimit) -> s32 #foreign libc;

getrusage :: (unknown0: RUSAGE, unknown1: *rusage) -> s32 #foreign libc;

gets :: (unknown0: *u8) -> *u8 #foreign libc;

getsgroups_np :: (unknown0: *s32, unknown1: *uuid_t) -> s32 #foreign libc;

getsid :: (unknown0: pid_t) -> pid_t #foreign libc;

getsubopt :: (unknown0: **u8, unknown1: **u8, unknown2: **u8) -> s32 #foreign libc;

gettimeofday :: (unknown0: *timeval, unknown1: *void) -> s32 #foreign libc;

getuid :: () -> uid_t #foreign libc;

getusershell :: () -> *u8 #foreign libc;

getw :: (unknown0: *FILE) -> s32 #foreign libc;

getwd :: (unknown0: *u8) -> *u8 #foreign libc;

getwgroups_np :: (unknown0: *s32, unknown1: *uuid_t) -> s32 #foreign libc;

gmtime :: (unknown0: *time_t) -> *tm #foreign libc;

gmtime_r :: (unknown0: *time_t, unknown1: *tm) -> *tm #foreign libc;

initgroups :: (unknown0: *u8, unknown1: s32) -> s32 #foreign libc;

ioctl :: (unknown0: s32, unknown1: u64, __args: ..Any) -> s32 #foreign libc;

iruserok :: (unknown0: u64, unknown1: s32, unknown2: *u8, unknown3: *u8) -> s32 #foreign libc;

iruserok_sa :: (unknown0: *void, unknown1: s32, unknown2: s32, unknown3: *u8, unknown4: *u8) -> s32 #foreign libc;

isatty :: (unknown0: s32) -> s32 #foreign libc;

issetugid :: () -> s32 #foreign libc;

kill :: (unknown0: pid_t, unknown1: s32) -> s32 #foreign libc;

killpg :: (unknown0: pid_t, unknown1: s32) -> s32 #foreign libc;

lchflags :: (unknown0: *u8, unknown1: u32) -> s32 #foreign libc;

lchmod :: (unknown0: *u8, unknown1: mode_t) -> s32 #foreign libc;

lchown :: (unknown0: *u8, unknown1: uid_t, unknown2: gid_t) -> s32 #foreign libc;

link :: (unknown0: *u8, unknown1: *u8) -> s32 #foreign libc;

linkat :: (unknown0: s32, unknown1: *u8, unknown2: s32, unknown3: *u8, unknown4: s32) -> s32 #foreign libc;

lio_listio :: (mode: s32, aiocblist: **aiocb, nent: s32, sigp: *sigevent) -> s32 #foreign libc;

localtime :: (unknown0: *time_t) -> *tm #foreign libc;

localtime_r :: (unknown0: *time_t, unknown1: *tm) -> *tm #foreign libc;

lockf :: (unknown0: s32, unknown1: s32, unknown2: s64) -> s32 #foreign libc;

longjmp :: (unknown0: *jmp_buf, unknown1: s32) -> void #foreign libc;

longjmperror :: () -> void #foreign libc;

lseek :: (unknown0: s32, unknown1: s64, unknown2: s32) -> s64 #foreign libc;

lstat :: (unknown0: *u8, unknown1: *stat_t) -> s32 #foreign libc "lstat$INODE64";

lstat64 :: (unknown0: *u8, unknown1: *stat64_t) -> s32 #foreign libc;

lstatx64_np :: (unknown0: *u8, unknown1: *stat64_t, unknown2: filesec_t) -> s32 #foreign libc;

lstatx_np :: (unknown0: *u8, unknown1: *stat_t, unknown2: filesec_t) -> s32 #foreign libc "lstatx_np$INODE64";

lutimes :: (unknown0: *u8, unknown1: *timeval) -> s32 #foreign libc;

madvise :: (unknown0: *void, unknown1: u64, unknown2: s32) -> s32 #foreign libc;

mincore :: (unknown0: *void, unknown1: u64, unknown2: *u8) -> s32 #foreign libc;

minherit :: (unknown0: *void, unknown1: u64, unknown2: s32) -> s32 #foreign libc;

mkdir :: (unknown0: *u8, unknown1: mode_t) -> s32 #foreign libc;

mkdirat :: (unknown0: s32, unknown1: *u8, unknown2: mode_t) -> s32 #foreign libc;

mkdirx_np :: (unknown0: *u8, unknown1: filesec_t) -> s32 #foreign libc;

mkdtemp :: (unknown0: *u8) -> *u8 #foreign libc;

mkdtempat_np :: (dfd: s32, path: *u8) -> *u8 #foreign libc;

mkfifo :: (unknown0: *u8, unknown1: mode_t) -> s32 #foreign libc;

mkfifox_np :: (unknown0: *u8, unknown1: filesec_t) -> s32 #foreign libc;

mknod :: (unknown0: *u8, unknown1: mode_t, unknown2: dev_t) -> s32 #foreign libc;

mkostemp :: (path: *u8, oflags: s32) -> s32 #foreign libc;

mkostemps :: (path: *u8, slen: s32, oflags: s32) -> s32 #foreign libc;

mkostempsat_np :: (dfd: s32, path: *u8, slen: s32, oflags: s32) -> s32 #foreign libc;

mkpath_np :: (path: *u8, omode: mode_t) -> s32 #foreign libc;

mkpathat_np :: (dfd: s32, path: *u8, omode: mode_t) -> s32 #foreign libc;

mkstemp :: (unknown0: *u8) -> s32 #foreign libc;

mkstemp_dprotected_np :: (path: *u8, dpclass: s32, dpflags: s32) -> s32 #foreign libc;

mkstemps :: (unknown0: *u8, unknown1: s32) -> s32 #foreign libc;

mkstempsat_np :: (dfd: s32, path: *u8, slen: s32) -> s32 #foreign libc;

mktemp :: (unknown0: *u8) -> *u8 #foreign libc;

mktime :: (unknown0: *tm) -> time_t #foreign libc;

mlock :: (unknown0: *void, unknown1: u64) -> s32 #foreign libc;

mlockall :: (unknown0: s32) -> s32 #foreign libc;

mmap :: (unknown0: *void, unknown1: u64, unknown2: s32, unknown3: s32, unknown4: s32, unknown5: s64) -> *void #foreign libc;

mprotect :: (unknown0: *void, unknown1: u64, unknown2: s32) -> s32 #foreign libc;

msync :: (unknown0: *void, unknown1: u64, unknown2: s32) -> s32 #foreign libc;

munlock :: (unknown0: *void, unknown1: u64) -> s32 #foreign libc;

munlockall :: () -> s32 #foreign libc;

munmap :: (unknown0: *void, unknown1: u64) -> s32 #foreign libc; @MaybeFreesArgument

nanosleep :: (__rqtp: *timespec, __rmtp: *timespec) -> s32 #foreign libc;

nfssvc :: (unknown0: s32, unknown1: *void) -> s32 #foreign libc;

nice :: (unknown0: s32) -> s32 #foreign libc;

open :: (unknown0: *u8, unknown1: s32, __args: ..Any) -> s32 #foreign libc;

open_dprotected_np :: (unknown0: *u8, unknown1: s32, unknown2: s32, unknown3: s32, __args: ..Any) -> s32 #foreign libc;

open_memstream :: (__bufp: **u8, __sizep: *u64) -> *FILE #foreign libc;

openat :: (unknown0: s32, unknown1: *u8, unknown2: s32, __args: ..Any) -> s32 #foreign libc;

opendir :: (unknown0: *u8) -> *DIR #foreign libc "opendir$INODE64";

openx_np :: (unknown0: *u8, unknown1: s32, unknown2: filesec_t) -> s32 #foreign libc;

pathconf :: (unknown0: *u8, __name: PC) -> s64 #foreign libc;

pclose :: (unknown0: *FILE) -> s32 #foreign libc;

perror :: (unknown0: *u8) -> void #foreign libc;

pipe :: (unknown0: *[2] s32) -> s32 #foreign libc;

poll :: (unknown0: *pollfd, unknown1: nfds_t, unknown2: s32) -> s32 #foreign libc;

popen :: (unknown0: *u8, unknown1: *u8) -> *FILE #foreign libc;

posix2time :: (unknown0: time_t) -> time_t #foreign libc;

posix_madvise :: (unknown0: *void, unknown1: u64, unknown2: s32) -> s32 #foreign libc;

pread :: (__fd: s32, __buf: *void, __nbyte: u64, __offset: s64) -> s64 #foreign libc;

printf :: (unknown0: *u8, __args: ..Any) -> s32 #foreign libc; @WritesOutput

psignal :: (unknown0: u32, unknown1: *u8) -> void #foreign libc;

pthread_atfork :: (unknown0: #type () -> void #c_call, unknown1: #type () -> void #c_call, unknown2: #type () -> void #c_call) -> s32 #foreign libc;

pthread_attr_destroy :: (unknown0: *pthread_attr_t) -> s32 #foreign libc;

pthread_attr_get_qos_class_np :: (__attr: *pthread_attr_t, __qos_class: *qos_class_t, __relative_priority: *s32) -> s32 #foreign libc;

pthread_attr_getdetachstate :: (unknown0: *pthread_attr_t, unknown1: *s32) -> s32 #foreign libc;

pthread_attr_getguardsize :: (unknown0: *pthread_attr_t, unknown1: *u64) -> s32 #foreign libc;

pthread_attr_getinheritsched :: (unknown0: *pthread_attr_t, unknown1: *s32) -> s32 #foreign libc;

pthread_attr_getschedparam :: (unknown0: *pthread_attr_t, unknown1: *sched_param) -> s32 #foreign libc;

pthread_attr_getschedpolicy :: (unknown0: *pthread_attr_t, unknown1: *s32) -> s32 #foreign libc;

pthread_attr_getscope :: (unknown0: *pthread_attr_t, unknown1: *s32) -> s32 #foreign libc;

pthread_attr_getstack :: (unknown0: *pthread_attr_t, unknown1: **void, unknown2: *u64) -> s32 #foreign libc;

pthread_attr_getstackaddr :: (unknown0: *pthread_attr_t, unknown1: **void) -> s32 #foreign libc;

pthread_attr_getstacksize :: (unknown0: *pthread_attr_t, unknown1: *u64) -> s32 #foreign libc;

pthread_attr_init :: (unknown0: *pthread_attr_t) -> s32 #foreign libc;

pthread_attr_set_qos_class_np :: (__attr: *pthread_attr_t, __qos_class: qos_class_t, __relative_priority: s32) -> s32 #foreign libc;

pthread_attr_setdetachstate :: (unknown0: *pthread_attr_t, unknown1: s32) -> s32 #foreign libc;

pthread_attr_setguardsize :: (unknown0: *pthread_attr_t, unknown1: u64) -> s32 #foreign libc;

pthread_attr_setinheritsched :: (unknown0: *pthread_attr_t, unknown1: s32) -> s32 #foreign libc;

pthread_attr_setschedparam :: (unknown0: *pthread_attr_t, unknown1: *sched_param) -> s32 #foreign libc;

pthread_attr_setschedpolicy :: (unknown0: *pthread_attr_t, unknown1: s32) -> s32 #foreign libc;

pthread_attr_setscope :: (unknown0: *pthread_attr_t, unknown1: s32) -> s32 #foreign libc;

pthread_attr_setstack :: (unknown0: *pthread_attr_t, unknown1: *void, unknown2: u64) -> s32 #foreign libc;

pthread_attr_setstackaddr :: (unknown0: *pthread_attr_t, unknown1: *void) -> s32 #foreign libc;

pthread_attr_setstacksize :: (unknown0: *pthread_attr_t, unknown1: u64) -> s32 #foreign libc;

pthread_cancel :: (unknown0: pthread_t) -> s32 #foreign libc;

pthread_cond_broadcast :: (unknown0: *pthread_cond_t) -> s32 #foreign libc;

pthread_cond_destroy :: (unknown0: *pthread_cond_t) -> s32 #foreign libc;

pthread_cond_init :: (unknown0: *pthread_cond_t, unknown1: *pthread_condattr_t) -> s32 #foreign libc;

pthread_cond_signal :: (unknown0: *pthread_cond_t) -> s32 #foreign libc;

pthread_cond_signal_thread_np :: (unknown0: *pthread_cond_t, unknown1: pthread_t) -> s32 #foreign libc;

pthread_cond_timedwait :: (unknown0: *pthread_cond_t, unknown1: *pthread_mutex_t, unknown2: *timespec) -> s32 #foreign libc;

pthread_cond_timedwait_relative_np :: (unknown0: *pthread_cond_t, unknown1: *pthread_mutex_t, unknown2: *timespec) -> s32 #foreign libc;

pthread_cond_wait :: (unknown0: *pthread_cond_t, unknown1: *pthread_mutex_t) -> s32 #foreign libc;

pthread_condattr_destroy :: (unknown0: *pthread_condattr_t) -> s32 #foreign libc;

pthread_condattr_getpshared :: (unknown0: *pthread_condattr_t, unknown1: *s32) -> s32 #foreign libc;

pthread_condattr_init :: (unknown0: *pthread_condattr_t) -> s32 #foreign libc;

pthread_condattr_setpshared :: (unknown0: *pthread_condattr_t, unknown1: s32) -> s32 #foreign libc;

pthread_create :: (unknown0: *pthread_t, unknown1: *pthread_attr_t, unknown2: #type (unknown0: *void) -> *void #c_call, unknown3: *void) -> s32 #foreign libc;

pthread_create_from_mach_thread :: (unknown0: *pthread_t, unknown1: *pthread_attr_t, unknown2: #type (unknown0: *void) -> *void #c_call, unknown3: *void) -> s32 #foreign libc;

pthread_create_suspended_np :: (unknown0: *pthread_t, unknown1: *pthread_attr_t, unknown2: #type (unknown0: *void) -> *void #c_call, unknown3: *void) -> s32 #foreign libc;

pthread_detach :: (unknown0: pthread_t) -> s32 #foreign libc;

pthread_equal :: (unknown0: pthread_t, unknown1: pthread_t) -> s32 #foreign libc;

pthread_exit :: (unknown0: *void) -> void #foreign libc;

pthread_from_mach_thread_np :: (unknown0: mach_port_t) -> pthread_t #foreign libc;

pthread_get_qos_class_np :: (__pthread: pthread_t, __qos_class: *qos_class_t, __relative_priority: *s32) -> s32 #foreign libc;

pthread_get_stackaddr_np :: (unknown0: pthread_t) -> *void #foreign libc;

pthread_get_stacksize_np :: (unknown0: pthread_t) -> u64 #foreign libc;

pthread_getconcurrency :: () -> s32 #foreign libc;

pthread_getname_np :: (unknown0: pthread_t, unknown1: *u8, unknown2: u64) -> s32 #foreign libc;

pthread_getschedparam :: (unknown0: pthread_t, unknown1: *s32, unknown2: *sched_param) -> s32 #foreign libc;

pthread_getspecific :: (unknown0: pthread_key_t) -> *void #foreign libc;

pthread_is_threaded_np :: () -> s32 #foreign libc;

pthread_join :: (unknown0: pthread_t, unknown1: **void) -> s32 #foreign libc;

pthread_key_create :: (unknown0: *pthread_key_t, unknown1: #type (unknown0: *void) -> void #c_call) -> s32 #foreign libc;

pthread_key_delete :: (unknown0: pthread_key_t) -> s32 #foreign libc;

pthread_kill :: (unknown0: pthread_t, unknown1: s32) -> s32 #foreign libc;

pthread_mach_thread_np :: (unknown0: pthread_t) -> mach_port_t #foreign libc;

pthread_main_np :: () -> s32 #foreign libc;

pthread_mutex_destroy :: (unknown0: *pthread_mutex_t) -> s32 #foreign libc;

pthread_mutex_getprioceiling :: (unknown0: *pthread_mutex_t, unknown1: *s32) -> s32 #foreign libc;

pthread_mutex_init :: (unknown0: *pthread_mutex_t, unknown1: *pthread_mutexattr_t) -> s32 #foreign libc;

pthread_mutex_lock :: (unknown0: *pthread_mutex_t) -> s32 #foreign libc;

pthread_mutex_setprioceiling :: (unknown0: *pthread_mutex_t, unknown1: s32, unknown2: *s32) -> s32 #foreign libc;

pthread_mutex_trylock :: (unknown0: *pthread_mutex_t) -> s32 #foreign libc;

pthread_mutex_unlock :: (unknown0: *pthread_mutex_t) -> s32 #foreign libc;

pthread_mutexattr_destroy :: (unknown0: *pthread_mutexattr_t) -> s32 #foreign libc;

pthread_mutexattr_getpolicy_np :: (unknown0: *pthread_mutexattr_t, unknown1: *s32) -> s32 #foreign libc;

pthread_mutexattr_getprioceiling :: (unknown0: *pthread_mutexattr_t, unknown1: *s32) -> s32 #foreign libc;

pthread_mutexattr_getprotocol :: (unknown0: *pthread_mutexattr_t, unknown1: *s32) -> s32 #foreign libc;

pthread_mutexattr_getpshared :: (unknown0: *pthread_mutexattr_t, unknown1: *s32) -> s32 #foreign libc;

pthread_mutexattr_gettype :: (unknown0: *pthread_mutexattr_t, unknown1: *s32) -> s32 #foreign libc;

pthread_mutexattr_init :: (unknown0: *pthread_mutexattr_t) -> s32 #foreign libc;

pthread_mutexattr_setpolicy_np :: (unknown0: *pthread_mutexattr_t, unknown1: s32) -> s32 #foreign libc;

pthread_mutexattr_setprioceiling :: (unknown0: *pthread_mutexattr_t, unknown1: s32) -> s32 #foreign libc;

pthread_mutexattr_setprotocol :: (unknown0: *pthread_mutexattr_t, unknown1: s32) -> s32 #foreign libc;

pthread_mutexattr_setpshared :: (unknown0: *pthread_mutexattr_t, unknown1: s32) -> s32 #foreign libc;

pthread_mutexattr_settype :: (unknown0: *pthread_mutexattr_t, unknown1: s32) -> s32 #foreign libc;

pthread_once :: (unknown0: *pthread_once_t, unknown1: #type () -> void #c_call) -> s32 #foreign libc;

pthread_override_qos_class_end_np :: (__override: pthread_override_t) -> s32 #foreign libc;

pthread_override_qos_class_start_np :: (__pthread: pthread_t, __qos_class: qos_class_t, __relative_priority: s32) -> pthread_override_t #foreign libc;

pthread_rwlock_destroy :: (unknown0: *pthread_rwlock_t) -> s32 #foreign libc;

pthread_rwlock_init :: (unknown0: *pthread_rwlock_t, unknown1: *pthread_rwlockattr_t) -> s32 #foreign libc;

pthread_rwlock_rdlock :: (unknown0: *pthread_rwlock_t) -> s32 #foreign libc;

pthread_rwlock_tryrdlock :: (unknown0: *pthread_rwlock_t) -> s32 #foreign libc;

pthread_rwlock_trywrlock :: (unknown0: *pthread_rwlock_t) -> s32 #foreign libc;

pthread_rwlock_unlock :: (unknown0: *pthread_rwlock_t) -> s32 #foreign libc;

pthread_rwlock_wrlock :: (unknown0: *pthread_rwlock_t) -> s32 #foreign libc;

pthread_rwlockattr_destroy :: (unknown0: *pthread_rwlockattr_t) -> s32 #foreign libc;

pthread_rwlockattr_getpshared :: (unknown0: *pthread_rwlockattr_t, unknown1: *s32) -> s32 #foreign libc;

pthread_rwlockattr_init :: (unknown0: *pthread_rwlockattr_t) -> s32 #foreign libc;

pthread_rwlockattr_setpshared :: (unknown0: *pthread_rwlockattr_t, unknown1: s32) -> s32 #foreign libc;

pthread_self :: () -> pthread_t #foreign libc;

pthread_set_qos_class_self_np :: (__qos_class: qos_class_t, __relative_priority: s32) -> s32 #foreign libc;

pthread_setcancelstate :: (unknown0: s32, unknown1: *s32) -> s32 #foreign libc;

pthread_setcanceltype :: (unknown0: s32, unknown1: *s32) -> s32 #foreign libc;

pthread_setconcurrency :: (unknown0: s32) -> s32 #foreign libc;

pthread_setname_np :: (unknown0: *u8) -> s32 #foreign libc;

pthread_setschedparam :: (unknown0: pthread_t, unknown1: s32, unknown2: *sched_param) -> s32 #foreign libc;

pthread_setspecific :: (unknown0: pthread_key_t, unknown1: *void) -> s32 #foreign libc;

pthread_sigmask :: (unknown0: s32, unknown1: *sigset_t, unknown2: *sigset_t) -> s32 #foreign libc;

pthread_testcancel :: () -> void #foreign libc;

pthread_threadid_np :: (unknown0: pthread_t, unknown1: *u64) -> s32 #foreign libc;

pthread_yield_np :: () -> void #foreign libc;

putc :: (unknown0: s32, unknown1: *FILE) -> s32 #foreign libc;

putc_unlocked :: (unknown0: s32, unknown1: *FILE) -> s32 #foreign libc;

putchar :: (unknown0: s32) -> s32 #foreign libc; @WritesOutput

putchar_unlocked :: (unknown0: s32) -> s32 #foreign libc;

puts :: (unknown0: *u8) -> s32 #foreign libc; @WritesOutput

putw :: (unknown0: s32, unknown1: *FILE) -> s32 #foreign libc;

pwrite :: (__fd: s32, __buf: *void, __nbyte: u64, __offset: s64) -> s64 #foreign libc;

qos_class_main :: () -> qos_class_t #foreign libc;

qos_class_self :: () -> qos_class_t #foreign libc;

raise :: (unknown0: s32) -> s32 #foreign libc;

rcmd :: (unknown0: **u8, unknown1: s32, unknown2: *u8, unknown3: *u8, unknown4: *u8, unknown5: *s32) -> s32 #foreign libc;

rcmd_af :: (unknown0: **u8, unknown1: s32, unknown2: *u8, unknown3: *u8, unknown4: *u8, unknown5: *s32, unknown6: s32) -> s32 #foreign libc;

read :: (unknown0: s32, unknown1: *void, unknown2: u64) -> s64 #foreign libc;

readdir :: (unknown0: *DIR) -> *dirent #foreign libc "readdir$INODE64";

readdir_r :: (unknown0: *DIR, unknown1: *dirent, unknown2: **dirent) -> s32 #foreign libc "readdir_r$INODE64";

readlink :: (unknown0: *u8, unknown1: *u8, unknown2: u64) -> s64 #foreign libc;

readlinkat :: (unknown0: s32, unknown1: *u8, unknown2: *u8, unknown3: u64) -> s64 #foreign libc;

realpath :: (unknown0: *u8, unknown1: *u8) -> *u8 #foreign libc "realpath$DARWIN_EXTSN";

reboot :: (unknown0: s32) -> s32 #foreign libc;

rename :: (__old: *u8, __new: *u8) -> s32 #foreign libc;

revoke :: (unknown0: *u8) -> s32 #foreign libc;

rewind :: (unknown0: *FILE) -> void #foreign libc;

rewinddir :: (unknown0: *DIR) -> void #foreign libc "rewinddir$INODE64";

rmdir :: (unknown0: *u8) -> s32 #foreign libc;

rresvport :: (unknown0: *s32) -> s32 #foreign libc;

rresvport_af :: (unknown0: *s32, unknown1: s32) -> s32 #foreign libc;

ruserok :: (unknown0: *u8, unknown1: s32, unknown2: *u8, unknown3: *u8) -> s32 #foreign libc;

sbrk :: (unknown0: s32) -> *void #foreign libc;

scandir :: (unknown0: *u8, unknown1: ***dirent, unknown2: #type (unknown0: *dirent) -> s32 #c_call, unknown3: #type (unknown0: **dirent, unknown1: **dirent) -> s32 #c_call) -> s32 #foreign libc "scandir$INODE64";

scanf :: (unknown0: *u8, __args: ..Any) -> s32 #foreign libc;

sched_get_priority_max :: (unknown0: s32) -> s32 #foreign libc;

sched_get_priority_min :: (unknown0: s32) -> s32 #foreign libc;

sched_yield :: () -> s32 #foreign libc;

searchfs :: (unknown0: *u8, unknown1: *fssearchblock, unknown2: *u64, unknown3: u32, unknown4: u32, unknown5: *searchstate) -> s32 #foreign libc;

seekdir :: (unknown0: *DIR, unknown1: s64) -> void #foreign libc "seekdir$INODE64";

setattrlist :: (unknown0: *u8, unknown1: *void, unknown2: *void, unknown3: u64, unknown4: u32) -> s32 #foreign libc;

setattrlistat :: (unknown0: s32, unknown1: *u8, unknown2: *void, unknown3: *void, unknown4: u64, unknown5: u32) -> s32 #foreign libc;

setbuf :: (unknown0: *FILE, unknown1: *u8) -> void #foreign libc;

setbuffer :: (unknown0: *FILE, unknown1: *u8, unknown2: s32) -> void #foreign libc;

setdomainname :: (unknown0: *u8, unknown1: s32) -> s32 #foreign libc;

setegid :: (unknown0: gid_t) -> s32 #foreign libc;

setenv :: (__name: *u8, __value: *u8, __overwrite: s32) -> s32 #foreign libc;

seteuid :: (unknown0: uid_t) -> s32 #foreign libc;

setgid :: (unknown0: gid_t) -> s32 #foreign libc;

setgroups :: (unknown0: s32, unknown1: *gid_t) -> s32 #foreign libc;

sethostid :: (unknown0: s64) -> void #foreign libc;

sethostname :: (unknown0: *u8, unknown1: s32) -> s32 #foreign libc;

setiopolicy_np :: (unknown0: s32, unknown1: s32, unknown2: s32) -> s32 #foreign libc;

setitimer :: (unknown0: s32, unknown1: *itimerval, unknown2: *itimerval) -> s32 #foreign libc;

setjmp :: (unknown0: *jmp_buf) -> s32 #foreign libc;

setkey :: (unknown0: *u8) -> void #foreign libc;

setlinebuf :: (unknown0: *FILE) -> s32 #foreign libc;

setlogin :: (unknown0: *u8) -> s32 #foreign libc;

setmode :: (unknown0: *u8) -> *void #foreign libc;

setpassent :: (unknown0: s32) -> s32 #foreign libc;

setpgid :: (unknown0: pid_t, unknown1: pid_t) -> s32 #foreign libc;

setpgrp :: () -> pid_t #foreign libc;

setpriority :: (unknown0: PRIO, unknown1: id_t, unknown2: s32) -> s32 #foreign libc;

setpwent :: () -> void #foreign libc;

setregid :: (unknown0: gid_t, unknown1: gid_t) -> s32 #foreign libc;

setreuid :: (unknown0: uid_t, unknown1: uid_t) -> s32 #foreign libc;

setrgid :: (unknown0: gid_t) -> s32 #foreign libc;

setrlimit :: (unknown0: RLIMIT_RESOURCE, unknown1: *rlimit) -> s32 #foreign libc;

setruid :: (unknown0: uid_t) -> s32 #foreign libc;

setsgroups_np :: (unknown0: s32, unknown1: *uuid_t) -> s32 #foreign libc;

setsid :: () -> pid_t #foreign libc;

settimeofday :: (unknown0: *timeval, unknown1: *timezone) -> s32 #foreign libc;

setuid :: (unknown0: uid_t) -> s32 #foreign libc;

setusershell :: () -> void #foreign libc;

setvbuf :: (unknown0: *FILE, unknown1: *u8, unknown2: s32, unknown3: u64) -> s32 #foreign libc;

setwgroups_np :: (unknown0: s32, unknown1: *uuid_t) -> s32 #foreign libc;

shm_open :: (unknown0: *u8, unknown1: s32, __args: ..Any) -> s32 #foreign libc;

shm_unlink :: (unknown0: *u8) -> s32 #foreign libc;

shmat :: (unknown0: s32, unknown1: *void, unknown2: s32) -> *void #foreign libc;

shmctl :: (unknown0: s32, unknown1: s32, unknown2: *__shmid_ds_new) -> s32 #foreign libc;

shmdt :: (unknown0: *void) -> s32 #foreign libc;

shmget :: (unknown0: key_t, unknown1: u64, unknown2: s32) -> s32 #foreign libc;

shmsys :: (unknown0: s32, __args: ..Any) -> s32 #foreign libc;

sigaction :: (unknown0: s32, unknown1: *sigaction_t, unknown2: *sigaction_t) -> s32 #foreign libc;

sigaddset :: (unknown0: *sigset_t, unknown1: s32) -> s32 #foreign libc;

sigaltstack :: (unknown0: *stack_t, unknown1: *stack_t) -> s32 #foreign libc;

sigblock :: (unknown0: s32) -> s32 #foreign libc;

sigdelset :: (unknown0: *sigset_t, unknown1: s32) -> s32 #foreign libc;

sigemptyset :: (unknown0: *sigset_t) -> s32 #foreign libc;

sigfillset :: (unknown0: *sigset_t) -> s32 #foreign libc;

sighold :: (unknown0: s32) -> s32 #foreign libc;

sigignore :: (unknown0: s32) -> s32 #foreign libc;

siginterrupt :: (unknown0: s32, unknown1: s32) -> s32 #foreign libc;

sigismember :: (unknown0: *sigset_t, unknown1: s32) -> s32 #foreign libc;

siglongjmp :: (unknown0: *sigjmp_buf, unknown1: s32) -> void #foreign libc;

signal :: (unknown0: s32, unknown1: s32, unknown2: #type (unknown0: s32) -> void #c_call) -> ((unknown0: s32, unknown1: s32, unknown2: ((unknown0: s32) -> void #c_call)) -> void #c_call) #foreign libc;

sigpause :: (unknown0: s32) -> s32 #foreign libc;

sigpending :: (unknown0: *sigset_t) -> s32 #foreign libc;

sigprocmask :: (unknown0: s32, unknown1: *sigset_t, unknown2: *sigset_t) -> s32 #foreign libc;

sigrelse :: (unknown0: s32) -> s32 #foreign libc;

sigset :: (unknown0: s32, unknown1: s32, unknown2: #type (unknown0: s32) -> void #c_call) -> ((unknown0: s32, unknown1: s32, unknown2: ((unknown0: s32) -> void #c_call)) -> void #c_call) #foreign libc;

sigsetjmp :: (unknown0: *sigjmp_buf, unknown1: s32) -> s32 #foreign libc;

sigsetmask :: (unknown0: s32) -> s32 #foreign libc;

sigsuspend :: (unknown0: *sigset_t) -> s32 #foreign libc;

sigvec_ :: (unknown0: s32, unknown1: *sigvec, unknown2: *sigvec) -> s32 #foreign libc "sigvec";

sigwait :: (unknown0: *sigset_t, unknown1: *s32) -> s32 #foreign libc;

sleep :: (unknown0: u32) -> u32 #foreign libc;

snprintf :: (__str: *u8, __size: u64, __format: *u8, __args: ..Any) -> s32 #foreign libc;

sprintf :: (unknown0: *u8, unknown1: *u8, __args: ..Any) -> s32 #foreign libc;

sscanf :: (unknown0: *u8, unknown1: *u8, __args: ..Any) -> s32 #foreign libc;

stat :: (unknown0: *u8, unknown1: *stat_t) -> s32 #foreign libc "stat$INODE64";

stat64 :: (unknown0: *u8, unknown1: *stat64_t) -> s32 #foreign libc;

statx64_np :: (unknown0: *u8, unknown1: *stat64_t, unknown2: filesec_t) -> s32 #foreign libc;

statx_np :: (unknown0: *u8, unknown1: *stat_t, unknown2: filesec_t) -> s32 #foreign libc "statx_np$INODE64";

strftime :: (unknown0: *u8, unknown1: u64, unknown2: *u8, unknown3: *tm) -> u64 #foreign libc;

strptime :: (unknown0: *u8, unknown1: *u8, unknown2: *tm) -> *u8 #foreign libc;

strtofflags :: (unknown0: **u8, unknown1: *u64, unknown2: *u64) -> s32 #foreign libc;

swab :: (unknown0: *void, unknown1: *void, unknown2: s64) -> void #foreign libc;

swapon :: (unknown0: *u8) -> s32 #foreign libc;

symlink :: (unknown0: *u8, unknown1: *u8) -> s32 #foreign libc;

symlinkat :: (unknown0: *u8, unknown1: s32, unknown2: *u8) -> s32 #foreign libc;

sync :: () -> void #foreign libc;

sync_volume_np :: (unknown0: *u8, unknown1: s32) -> s32 #foreign libc;

syscall :: (unknown0: s32, __args: ..Any) -> s32 #foreign libc;

sysconf :: (name: _SC_definitions) -> s64 #foreign libc;

sysctl :: (unknown0: *s32, unknown1: u32, unknown2: *void, unknown3: *u64, unknown4: *void, unknown5: u64) -> s32 #foreign libc;

sysctlbyname :: (unknown0: *u8, unknown1: *void, unknown2: *u64, unknown3: *void, unknown4: u64) -> s32 #foreign libc;

sysctlnametomib :: (unknown0: *u8, unknown1: *s32, unknown2: *u64) -> s32 #foreign libc;

system :: (unknown0: *u8) -> s32 #foreign libc;

tcgetpgrp :: (unknown0: s32) -> pid_t #foreign libc;

tcsetpgrp :: (unknown0: s32, unknown1: pid_t) -> s32 #foreign libc;

telldir :: (unknown0: *DIR) -> s64 #foreign libc "telldir$INODE64";

tempnam :: (__dir: *u8, __prefix: *u8) -> *u8 #foreign libc;

time :: (unknown0: *time_t) -> time_t #foreign libc;

time2posix :: (unknown0: time_t) -> time_t #foreign libc;

timegm :: (unknown0: *tm) -> time_t #foreign libc;

timelocal :: (unknown0: *tm) -> time_t #foreign libc;

tmpfile :: () -> *FILE #foreign libc;

tmpnam :: (unknown0: *u8) -> *u8 #foreign libc;

truncate :: (unknown0: *u8, unknown1: s64) -> s32 #foreign libc;

ttyname :: (unknown0: s32) -> *u8 #foreign libc;

ttyname_r :: (unknown0: s32, unknown1: *u8, unknown2: u64) -> s32 #foreign libc;

ttyslot :: () -> s32 #foreign libc;

tzset :: () -> void #foreign libc;

tzsetwall :: () -> void #foreign libc;

ualarm :: (unknown0: useconds_t, unknown1: useconds_t) -> useconds_t #foreign libc;

umask :: (unknown0: mode_t) -> mode_t #foreign libc;

umaskx_np :: (unknown0: filesec_t) -> s32 #foreign libc;

uname :: (unknown0: *utsname) -> s32 #foreign libc;

undelete :: (unknown0: *u8) -> s32 #foreign libc;

ungetc :: (unknown0: s32, unknown1: *FILE) -> s32 #foreign libc;

unlink :: (unknown0: *u8) -> s32 #foreign libc;

unlinkat :: (unknown0: s32, unknown1: *u8, unknown2: s32) -> s32 #foreign libc;

unsetenv :: (unknown0: *u8) -> s32 #foreign libc;

user_from_uid :: (unknown0: uid_t, unknown1: s32) -> *u8 #foreign libc;

usleep :: (unknown0: useconds_t) -> s32 #foreign libc;

utimensat :: (__fd: s32, __path: *u8, __times: *[2] timespec, __flag: s32) -> s32 #foreign libc;

utimes :: (unknown0: *u8, unknown1: *timeval) -> s32 #foreign libc;

valloc :: (unknown0: u64) -> *void #foreign libc;

vfork :: () -> s32 #foreign libc;

wait :: (unknown0: *s32) -> pid_t #foreign libc;

wait3 :: (unknown0: *s32, unknown1: s32, unknown2: *rusage) -> pid_t #foreign libc;

wait4 :: (unknown0: pid_t, unknown1: *s32, unknown2: s32, unknown3: *rusage) -> pid_t #foreign libc;

waitid :: (unknown0: idtype_t, unknown1: id_t, unknown2: *siginfo_t, unknown3: s32) -> s32 #foreign libc;

waitpid :: (unknown0: pid_t, unknown1: *s32, unknown2: s32) -> pid_t #foreign libc;

write :: (__fd: s32, __buf: *void, __nbyte: u64) -> s64 #foreign libc; @WritesOutput(0)

get___busy :: (fields: __darwin_fp_status) -> bool {
    return ((fields.__bitfield >> 15) & 1) != 0;
}

get___c0 :: (fields: __darwin_fp_status) -> bool {
    return ((fields.__bitfield >> 8) & 1) != 0;
}

get___c1 :: (fields: __darwin_fp_status) -> bool {
    return ((fields.__bitfield >> 9) & 1) != 0;
}

get___c2 :: (fields: __darwin_fp_status) -> bool {
    return ((fields.__bitfield >> 10) & 1) != 0;
}

get___c3 :: (fields: __darwin_fp_status) -> bool {
    return ((fields.__bitfield >> 14) & 1) != 0;
}

get___denorm :: (fields: __darwin_fp_control) -> bool {
    return ((fields.__bitfield >> 1) & 1) != 0;
}

get___denorm :: (fields: __darwin_fp_status) -> bool {
    return ((fields.__bitfield >> 1) & 1) != 0;
}

get___errsumm :: (fields: __darwin_fp_status) -> bool {
    return ((fields.__bitfield >> 7) & 1) != 0;
}

get___invalid :: (fields: __darwin_fp_control) -> bool {
    return ((fields.__bitfield >> 0) & 1) != 0;
}

get___invalid :: (fields: __darwin_fp_status) -> bool {
    return ((fields.__bitfield >> 0) & 1) != 0;
}

get___ovrfl :: (fields: __darwin_fp_control) -> bool {
    return ((fields.__bitfield >> 3) & 1) != 0;
}

get___ovrfl :: (fields: __darwin_fp_status) -> bool {
    return ((fields.__bitfield >> 3) & 1) != 0;
}

get___pc :: (fields: __darwin_fp_control) -> u16 {
    return cast(u16) ((fields.__bitfield >> 8) & 3);
}

get___precis :: (fields: __darwin_fp_control) -> bool {
    return ((fields.__bitfield >> 5) & 1) != 0;
}

get___precis :: (fields: __darwin_fp_status) -> bool {
    return ((fields.__bitfield >> 5) & 1) != 0;
}

get___rc :: (fields: __darwin_fp_control) -> u16 {
    return cast(u16) ((fields.__bitfield >> 10) & 3);
}

get___stkflt :: (fields: __darwin_fp_status) -> bool {
    return ((fields.__bitfield >> 6) & 1) != 0;
}

get___tos :: (fields: __darwin_fp_status) -> u16 {
    return cast(u16) ((fields.__bitfield >> 11) & 7);
}

get___undfl :: (fields: __darwin_fp_control) -> bool {
    return ((fields.__bitfield >> 4) & 1) != 0;
}

get___undfl :: (fields: __darwin_fp_status) -> bool {
    return ((fields.__bitfield >> 4) & 1) != 0;
}

get___zdiv :: (fields: __darwin_fp_control) -> bool {
    return ((fields.__bitfield >> 2) & 1) != 0;
}

get___zdiv :: (fields: __darwin_fp_status) -> bool {
    return ((fields.__bitfield >> 2) & 1) != 0;
}

get_w_S_w_Filler :: (fields: wait_t) -> u32 {
    return cast(u32) ((fields.w_S.__bitfield >> 16) & 65535);
}

get_w_S_w_Stopsig :: (fields: wait_t) -> u32 {
    return cast(u32) ((fields.w_S.__bitfield >> 8) & 255);
}

get_w_S_w_Stopval :: (fields: wait_t) -> u32 {
    return cast(u32) ((fields.w_S.__bitfield >> 0) & 255);
}

get_w_T_w_Coredump :: (fields: wait_t) -> bool {
    return ((fields.w_T.__bitfield >> 7) & 1) != 0;
}

get_w_T_w_Filler :: (fields: wait_t) -> u32 {
    return cast(u32) ((fields.w_T.__bitfield >> 16) & 65535);
}

get_w_T_w_Retcode :: (fields: wait_t) -> u32 {
    return cast(u32) ((fields.w_T.__bitfield >> 8) & 255);
}

get_w_T_w_Termsig :: (fields: wait_t) -> u32 {
    return cast(u32) ((fields.w_T.__bitfield >> 0) & 127);
}

set___busy :: (fields: *__darwin_fp_status, value: bool) {
    fields.__bitfield = (fields.__bitfield & ~(cast(u16) 32768)) | (((cast(u16) value) & 1) << 15);
}

set___c0 :: (fields: *__darwin_fp_status, value: bool) {
    fields.__bitfield = (fields.__bitfield & ~(cast(u16) 256)) | (((cast(u16) value) & 1) << 8);
}

set___c1 :: (fields: *__darwin_fp_status, value: bool) {
    fields.__bitfield = (fields.__bitfield & ~(cast(u16) 512)) | (((cast(u16) value) & 1) << 9);
}

set___c2 :: (fields: *__darwin_fp_status, value: bool) {
    fields.__bitfield = (fields.__bitfield & ~(cast(u16) 1024)) | (((cast(u16) value) & 1) << 10);
}

set___c3 :: (fields: *__darwin_fp_status, value: bool) {
    fields.__bitfield = (fields.__bitfield & ~(cast(u16) 16384)) | (((cast(u16) value) & 1) << 14);
}

set___denorm :: (fields: *__darwin_fp_control, value: bool) {
    fields.__bitfield = (fields.__bitfield & ~(cast(u16) 2)) | (((cast(u16) value) & 1) << 1);
}

set___denorm :: (fields: *__darwin_fp_status, value: bool) {
    fields.__bitfield = (fields.__bitfield & ~(cast(u16) 2)) | (((cast(u16) value) & 1) << 1);
}

set___errsumm :: (fields: *__darwin_fp_status, value: bool) {
    fields.__bitfield = (fields.__bitfield & ~(cast(u16) 128)) | (((cast(u16) value) & 1) << 7);
}

set___invalid :: (fields: *__darwin_fp_control, value: bool) {
    fields.__bitfield = (fields.__bitfield & ~(cast(u16) 1)) | (((cast(u16) value) & 1) << 0);
}

set___invalid :: (fields: *__darwin_fp_status, value: bool) {
    fields.__bitfield = (fields.__bitfield & ~(cast(u16) 1)) | (((cast(u16) value) & 1) << 0);
}

set___ovrfl :: (fields: *__darwin_fp_control, value: bool) {
    fields.__bitfield = (fields.__bitfield & ~(cast(u16) 8)) | (((cast(u16) value) & 1) << 3);
}

set___ovrfl :: (fields: *__darwin_fp_status, value: bool) {
    fields.__bitfield = (fields.__bitfield & ~(cast(u16) 8)) | (((cast(u16) value) & 1) << 3);
}

set___pc :: (fields: *__darwin_fp_control, value: u16) {
    fields.__bitfield = (fields.__bitfield & ~(cast(u16) 768)) | ((value & 3) << 8);
}

set___precis :: (fields: *__darwin_fp_control, value: bool) {
    fields.__bitfield = (fields.__bitfield & ~(cast(u16) 32)) | (((cast(u16) value) & 1) << 5);
}

set___precis :: (fields: *__darwin_fp_status, value: bool) {
    fields.__bitfield = (fields.__bitfield & ~(cast(u16) 32)) | (((cast(u16) value) & 1) << 5);
}

set___rc :: (fields: *__darwin_fp_control, value: u16) {
    fields.__bitfield = (fields.__bitfield & ~(cast(u16) 3072)) | ((value & 3) << 10);
}

set___stkflt :: (fields: *__darwin_fp_status, value: bool) {
    fields.__bitfield = (fields.__bitfield & ~(cast(u16) 64)) | (((cast(u16) value) & 1) << 6);
}

set___tos :: (fields: *__darwin_fp_status, value: u16) {
    fields.__bitfield = (fields.__bitfield & ~(cast(u16) 14336)) | ((value & 7) << 11);
}

set___undfl :: (fields: *__darwin_fp_control, value: bool) {
    fields.__bitfield = (fields.__bitfield & ~(cast(u16) 16)) | (((cast(u16) value) & 1) << 4);
}

set___undfl :: (fields: *__darwin_fp_status, value: bool) {
    fields.__bitfield = (fields.__bitfield & ~(cast(u16) 16)) | (((cast(u16) value) & 1) << 4);
}

set___zdiv :: (fields: *__darwin_fp_control, value: bool) {
    fields.__bitfield = (fields.__bitfield & ~(cast(u16) 4)) | (((cast(u16) value) & 1) << 2);
}

set___zdiv :: (fields: *__darwin_fp_status, value: bool) {
    fields.__bitfield = (fields.__bitfield & ~(cast(u16) 4)) | (((cast(u16) value) & 1) << 2);
}

set_w_S_w_Filler :: (fields: *wait_t, value: u32) {
    fields.w_S.__bitfield = (fields.w_S.__bitfield & ~(cast(u32) 4294901760)) | ((value & 65535) << 16);
}

set_w_S_w_Stopsig :: (fields: *wait_t, value: u32) {
    fields.w_S.__bitfield = (fields.w_S.__bitfield & ~(cast(u32) 65280)) | ((value & 255) << 8);
}

set_w_S_w_Stopval :: (fields: *wait_t, value: u32) {
    fields.w_S.__bitfield = (fields.w_S.__bitfield & ~(cast(u32) 255)) | ((value & 255) << 0);
}

set_w_T_w_Coredump :: (fields: *wait_t, value: bool) {
    fields.w_T.__bitfield = (fields.w_T.__bitfield & ~(cast(u32) 128)) | (((cast(u32) value) & 1) << 7);
}

set_w_T_w_Filler :: (fields: *wait_t, value: u32) {
    fields.w_T.__bitfield = (fields.w_T.__bitfield & ~(cast(u32) 4294901760)) | ((value & 65535) << 16);
}

set_w_T_w_Retcode :: (fields: *wait_t, value: u32) {
    fields.w_T.__bitfield = (fields.w_T.__bitfield & ~(cast(u32) 65280)) | ((value & 255) << 8);
}

set_w_T_w_Termsig :: (fields: *wait_t, value: u32) {
    fields.w_T.__bitfield = (fields.w_T.__bitfield & ~(cast(u32) 127)) | ((value & 127) << 0);
}
