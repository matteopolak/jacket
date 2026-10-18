# Linux

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

### Linux-specific declarations

- `Epoll_Ctl :: enum s32`
- `EPOLL_CTL_ADD :: cast(s32) Epoll_Ctl.ADD`
- `EPOLL_CTL_DEL :: cast(s32) Epoll_Ctl.DEL`
- `EPOLL_CTL_MOD :: cast(s32) Epoll_Ctl.MOD`
- `EPOLLIN: u32 : 0x1`
- `EPOLLPRI: u32 : 0x2`
- `EPOLLOUT: u32 : 0x4`
- `EPOLLERR: u32 : 0x8`
- `EPOLLHUP: u32 : 0x10`
- `EPOLLNVAL: u32 : 0x20`
- `EPOLLRDNORM: u32 : 0x40`
- `EPOLLRDBAND: u32 : 0x80`
- `EPOLLWRNORM: u32 : 0x100`
- `EPOLLWRBAND: u32 : 0x200`
- `EPOLLMSG: u32 : 0x400`
- `EPOLLRDHUP: u32 : 0x2000`
- `EPOLL_URING_WAKE: u32 : 0x8000000`
- `EPOLLEXCLUSIVE: u32 : 0x10000000`
- `EPOLLWAKEUP: u32 : 0x20000000`
- `EPOLLONESHOT: u32 : 0x40000000`
- `EPOLLET: u32 : 0x80000000`
- `EPOLL_CLOEXEC: s32 : 0x80000`
- `epoll_data :: union`
- `epoll_event :: struct`
- `epoll_create :: (size: s32) -> s32 #foreign libc`
- `epoll_create1 :: (flags: s32) -> s32 #foreign libc`
- `epoll_ctl :: (epfd: s32, op: Epoll_Ctl, fd: s32, event: *epoll_event) -> s32 #foreign libc`
- `epoll_ctl :: (epfd: s32, op: s32, fd: s32, event: *epoll_event) -> s32 #foreign libc #deprecated "Pass an Epoll_Ctl operation instead of an s32."`
- `epoll_wait :: (epfd: s32, events: *epoll_event, maxevents: s32, timeout: s32) -> s32 #foreign libc`
- `epoll_pwait :: (epfd: s32, events: *epoll_event, maxevents: s32, timeout: s32, sigmask: *sigset_t) -> s32 #foreign libc`
- `statx_timestamp_t :: struct`
- `io_uring_setup :: inline (entries: u32, p: *io_uring_params) -> s32`
- `io_uring_enter :: inline (fd: s32, to_submit: u32, min_complete: u32, flags: u32) -> s32`
- `io_uring_enter :: inline (fd: s32, to_submit: u32, min_complete: u32, flags: u32, sig: *sigset_t) -> s32`
- `io_uring_enter :: inline (fd: s32, to_submit: u32, min_complete: u32, flags: u32, arg: *io_uring_getevents_arg) -> s32`
- `io_uring_register :: inline (fd: s32, opcode: u32, arg: *void, nr_args: u32) -> s32`
- `io_uring_params :: struct`
- `io_sqring_offsets :: struct`
- `io_cqring_offsets :: struct`
- `IORING_OFF_SQ_RING : s64`
- `IORING_OFF_CQ_RING : s64`
- `IORING_OFF_SQES : s64`
- `io_uring_sqe :: struct`
- `IORING_FILE_INDEX_ALLOC : u32`
- `io_uring_cqe :: struct`
- `io_uring_getevents_arg :: struct`
- `io_uring_files_update :: struct`
- `io_uring_rsrc_register :: struct`
- `io_uring_rsrc_update :: struct`
- `io_uring_rsrc_update2 :: struct`
- `IORING_REGISTER_FILES_SKIP : s32`
- `using __kernel_rwf_t :: enum_flags s32`
- `using IORING_Setup_Flags :: enum_flags u32`
- `using IORING_FEAT_Flags :: enum_flags u32`
- `using IORING_SQ_Flags :: enum_flags u32`
- `using IORING_CQ_Flags :: enum_flags u32`
- `using IORING_OP :: enum u8`
- `using IOSQE_Flag_Bits :: enum`
- `using IOSQE_Flags :: enum_flags u8`
- `using IORING_ACCEPT_Flags :: enum_flags u16`
- `using IORING_RECVSEND_Flags :: enum_flags u64`
- `using IORING_POLL_Flags :: enum_flags u32`
- `using IORING_OP_Flags :: enum_flags u32 #specified`
- `using IOCQE_Flags :: enum_flags u32`
- `using IORING_ENTER :: enum_flags u32`
- `using IORING_REGISTER :: enum`
- `using IO_WQ_Categories :: enum`
- `using IORING_RSRC_Flags :: enum_flags u32`
- `using IO_URING_OP_Flags :: enum_flags u16`
- `using IO_URING_RESTRICTION_OP :: enum`
- `io_uring_probe :: struct`
- `io_uring_probe_op :: struct`
- `io_uring_restriction :: struct`
- `io_uring_buf_ring :: struct`
- `io_uring_buf :: struct`
- `io_uring_buf_reg :: struct`
- `KERN_SUCCESS :: 0`
- `inotify_event :: struct`
- `IN_CLOEXEC :: 0x80000`
- `IN_NONBLOCK :: 0x00800`
- `IN_ACCESS :: 0x00000001`
- `IN_MODIFY :: 0x00000002`
- `IN_ATTRIB :: 0x00000004`
- `IN_CLOSE_WRITE :: 0x00000008`
- `IN_CLOSE_NOWRITE :: 0x00000010`
- `IN_CLOSE :: (IN_CLOSE_WRITE | IN_CLOSE_NOWRITE)`
- `IN_OPEN :: 0x00000020`
- `IN_MOVED_FROM :: 0x00000040`
- `IN_MOVED_TO :: 0x00000080`
- `IN_MOVE :: (IN_MOVED_FROM | IN_MOVED_TO)`
- `IN_CREATE :: 0x00000100`
- `IN_DELETE :: 0x00000200`
- `IN_DELETE_SELF :: 0x00000400`
- `IN_MOVE_SELF :: 0x00000800`
- `IN_UNMOUNT :: 0x00002000`
- `IN_Q_OVERFLOW :: 0x00004000`
- `IN_IGNORED :: 0x00008000`
- `IN_ONLYDIR :: 0x01000000`
- `IN_DONT_FOLLOW :: 0x02000000`
- `IN_EXCL_UNLINK :: 0x04000000`
- `IN_MASK_ADD :: 0x20000000`
- `IN_ISDIR :: 0x40000000`
- `IN_ONESHOT :: 0x80000000`
- `IN_ALL_EVENTS :: (IN_ACCESS | IN_MODIFY | IN_ATTRIB | IN_CLOSE_WRITE | IN_CLOSE_NOWRITE | IN_OPEN | IN_MOVED_FROM | IN_MOVED_TO | IN_CREATE | IN_DELETE | IN_DELETE_SELF | IN_MOVE_SELF)`
- `inotify_init :: (flags: s32 = 0) -> s32 #foreign libc "inotify_init1"`
- `inotify_add_watch :: (fd: s32, pathname: *u8, mask: u32) -> s32 #foreign libc`
- `inotify_rm_watch :: (fd: s32, wd: s32) -> s32 #foreign libc`
- `sys_info :: struct`
- `sysinfo :: (info: *sys_info) -> s32 #foreign libc`
- `input_absinfo :: struct`
- `input_event :: struct`
- `EV_SYN :: 0x00`
- `EV_KEY :: 0x01`
- `EV_REL :: 0x02`
- `EV_ABS :: 0x03`
- `EV_MSC :: 0x04`
- `EV_SW :: 0x05`
- `EV_LED :: 0x11`
- `EV_SND :: 0x12`
- `EV_REP :: 0x14`
- `EV_FF :: 0x15`
- `EV_PWR :: 0x16`
- `EV_FF_STATUS :: 0x17`
- `EV_MAX :: 0x1f`
- `EV_CNT :: EV_MAX+1`
- `ABS_HAT0X :: 0x10`
- `ABS_HAT0Y :: 0x11`
- `ABS_HAT1X :: 0x12`
- `ABS_HAT1Y :: 0x13`
- `ABS_HAT2X :: 0x14`
- `ABS_HAT2Y :: 0x15`
- `ABS_HAT3X :: 0x16`
- `ABS_HAT3Y :: 0x17`
- `BTN_MISC :: 0x100`
- `BTN_JOYSTICK :: 0x120`
- `KEY_MAX :: 0x2ff`
- `ABS_MAX :: 0x3f`

## Public type members

### Linux-specific type declarations

#### `Epoll_Ctl` (enum)

- `ADD :: 1`
- `DEL :: 2`
- `MOD :: 3`

#### `epoll_data` (union)

- `ptr: *void`
- `fd: s32`
- `_u32: u32`
- `_u64: u64`

#### `epoll_event` [CPU == .X64] (struct)

- `events: u32`
- `data: epoll_data #align 4`

#### `epoll_event` [CPU == .ARM64] (struct)

The arm64 kernel does not pack the record, so `data` keeps its natural 8-byte alignment.

- `events: u32`
- `data: epoll_data`

#### `statx_timestamp_t` (struct)

- `tv_sec:     s64`
- `tv_nsec:    u32`
- `__reserved: s32`

#### `io_uring_params` (struct)

- `sq_entries:     u32`
- `cq_entries:     u32`
- `flags:          IORING_Setup_Flags`
- `sq_thread_cpu:  u32`
- `sq_thread_idle: u32`
- `features:       IORING_FEAT_Flags`
- `wq_fd:          s32`
- `resv:       [3] u32`
- `sq_off: io_sqring_offsets`
- `cq_off: io_cqring_offsets`

#### `io_sqring_offsets` (struct)

- `head:         u32`
- `tail:         u32`
- `ring_mask:    u32`
- `ring_entries: u32`
- `flags:        u32`
- `dropped:      u32`
- `array:        u32`
- `resv1:        u32`
- `resv2:        u64`

#### `io_cqring_offsets` (struct)

- `head:         u32`
- `tail:         u32`
- `ring_mask:    u32`
- `ring_entries: u32`
- `overflow:     u32`
- `cqes:         u32`
- `flags:        u32`
- `resv1:        u32`
- `resv2:        u64`

#### `io_uring_sqe` (struct)
```jai
io_uring_sqe :: struct {
    opcode: IORING_OP;
    flags: IOSQE_Flags;
    union {
        ioprio: u16;
        accept_flags: IORING_ACCEPT_Flags;
    }
    fd: s32;
    union {
        off: u64;
        addr2: u64;
        recv_send_flags: IORING_RECVSEND_Flags;
    }
    union {
        addr: u64;
        splice_off_in: u64;
    }
    union {
        len: u32;
        poll_flags: IORING_POLL_Flags;
    }
    union {
        rw_flags: __kernel_rwf_t;
        op_flags: IORING_OP_Flags;
        other_flags: u32;
        poll_events: u16;
        fadvise_advice: u32;
    }
    user_data: u64;
    union {
        buf_index: u16 #align 1;
        buf_group: u16 #align 1;
    }
    personality: u16;
    union {
        splice_fd_in: s32;
        file_index: u32;
    }
    addr3: u64;
    __pad2: [1] u64;
}
```

#### `io_uring_cqe` (struct)

- `user_data: u64`
- `res:       s32`
- `flags:     IOCQE_Flags`

#### `io_uring_getevents_arg` (struct)

- `sigmask:    u64`
- `sigmask_sz: u32`
- `pad:        u32`
- `ts:         u64`

#### `io_uring_files_update` (struct)

- `offset: u32`
- `resv:   u32`
- `fds:   *s32`

#### `io_uring_rsrc_register` (struct)

- `nr:    u32`
- `flags: IORING_RSRC_Flags`
- `resv2: u64`
- `data: *iovec`
- `tags: *u64`

#### `io_uring_rsrc_update` (struct)

- `offset: u32`
- `resv:   u32`
- `data:  *iovec`

#### `io_uring_rsrc_update2` (struct)

- `offset: u32`
- `resv:   u32`
- `data:  *iovec`
- `tags:  *u64`
- `nr:     u32`
- `resv2:  u32`

#### `io_uring_probe` (struct)

- `last_op:   IORING_OP`
- `ops_len:   u8`
- `resv:      u16`
- `resv2: [3] u32`
- `ops:   [0] io_uring_probe_op`

#### `io_uring_probe_op` (struct)

- `op:    IORING_OP`
- `resv:  u8`
- `flags: IO_URING_OP_Flags`
- `resv2: u32`

#### `io_uring_restriction` (struct)
```jai
io_uring_restriction :: struct {
    opcode: u16;
    union {
        register_op: u8;
        sqe_op:      u8;
        sqe_flags:   u8;
    }
    resv:      u8;
    resv2: [3] u32;
}
```

#### io_uring flag and opcode types

The enum bodies below are public. Implicit enum values begin at zero and increase by one in declaration order; explicit bit expressions retain their stated values.

```jai
using __kernel_rwf_t :: enum_flags s32 {
    RWF_HIPRI :: 0x00000001; RWF_DSYNC :: 0x00000002; RWF_SYNC :: 0x00000004;
    RWF_NOWAIT :: 0x00000008; RWF_APPEND :: 0x00000010;
    RWF_SUPPORTED :: RWF_HIPRI | RWF_DSYNC | RWF_SYNC | RWF_NOWAIT | RWF_APPEND;
}
using IORING_Setup_Flags :: enum_flags u32 {
    IORING_SETUP_IOPOLL :: 1 << 0; IORING_SETUP_SQPOLL :: 1 << 1; IORING_SETUP_SQ_AFF :: 1 << 2;
    IORING_SETUP_CQSIZE :: 1 << 3; IORING_SETUP_CLAMP :: 1 << 4; IORING_SETUP_ATTACH_WQ :: 1 << 5;
    IORING_SETUP_R_DISABLED :: 1 << 6; IORING_SETUP_SUBMIT_ALL :: 1 << 7;
    IORING_SETUP_COOP_TASKRUN :: 1 << 8; IORING_SETUP_TASKRUN_FLAG :: 1 << 9;
    IORING_SETUP_SQE128 :: 1 << 10; IORING_SETUP_CQE32 :: 1 << 11;
}
using IORING_FEAT_Flags :: enum_flags u32 {
    IORING_FEAT_SINGLE_MMAP :: 1 << 0; IORING_FEAT_NODROP :: 1 << 1; IORING_FEAT_SUBMIT_STABLE :: 1 << 2;
    IORING_FEAT_RW_CUR_POS :: 1 << 3; IORING_FEAT_CUR_PERSONALITY :: 1 << 4; IORING_FEAT_FAST_POLL :: 1 << 5;
    IORING_FEAT_POLL_32BITS :: 1 << 6; IORING_FEAT_SQPOLL_NONFIXED :: 1 << 7; IORING_FEAT_EXT_ARG :: 1 << 8;
    IORING_FEAT_NATIVE_WORKERS :: 1 << 9; IORING_FEAT_RSRC_TAGS :: 1 << 10; IORING_FEAT_CQE_SKIP :: 1 << 11;
    IORING_FEAT_LINKED_FILE :: 1 << 12;
}
using IORING_SQ_Flags :: enum_flags u32 {
    IORING_SQ_NEED_WAKEUP :: 1 << 0; IORING_SQ_CQ_OVERFLOW :: 1 << 1; IORING_SQ_TASKRUN :: 1 << 2;
}
using IORING_CQ_Flags :: enum_flags u32 { IORING_CQ_EVENTFD_DISABLED :: 1 << 0; }
using IORING_OP :: enum u8 {
    IORING_OP_NOP; IORING_OP_READV; IORING_OP_WRITEV; IORING_OP_FSYNC; IORING_OP_READ_FIXED; IORING_OP_WRITE_FIXED;
    IORING_OP_POLL_ADD; IORING_OP_POLL_REMOVE; IORING_OP_SYNC_FILE_RANGE; IORING_OP_SENDMSG; IORING_OP_RECVMSG;
    IORING_OP_TIMEOUT; IORING_OP_TIMEOUT_REMOVE; IORING_OP_ACCEPT; IORING_OP_ASYNC_CANCEL; IORING_OP_LINK_TIMEOUT;
    IORING_OP_CONNECT; IORING_OP_FALLOCATE; IORING_OP_OPENAT; IORING_OP_CLOSE; IORING_OP_FILES_UPDATE;
    IORING_OP_STATX; IORING_OP_READ; IORING_OP_WRITE; IORING_OP_FADVISE; IORING_OP_MADVISE; IORING_OP_SEND;
    IORING_OP_RECV; IORING_OP_OPENAT2; IORING_OP_EPOLL_CTL; IORING_OP_SPLICE; IORING_OP_PROVIDE_BUFFERS;
    IORING_OP_REMOVE_BUFFERS; IORING_OP_TEE; IORING_OP_SHUTDOWN; IORING_OP_RENAMEAT; IORING_OP_UNLINKAT;
    IORING_OP_MKDIRAT; IORING_OP_SYMLINKAT; IORING_OP_LINKAT; IORING_OP_MSG_RING; IORING_OP_FSETXATTR;
    IORING_OP_SETXATTR; IORING_OP_FGETXATTR; IORING_OP_GETXATTR; IORING_OP_SOCKET; IORING_OP_URING_CMD;
    IORING_OP_LAST;
}
using IOSQE_Flag_Bits :: enum {
    IOSQE_FIXED_FILE_BIT; IOSQE_IO_DRAIN_BIT; IOSQE_IO_LINK_BIT; IOSQE_IO_HARDLINK_BIT; IOSQE_ASYNC_BIT;
    IOSQE_BUFFER_SELECT_BIT; IOSQE_CQE_SKIP_SUCCESS_BIT;
}
using IOSQE_Flags :: enum_flags u8 {
    IOSQE_FIXED_FILE :: 1 << IOSQE_FIXED_FILE_BIT; IOSQE_IO_DRAIN :: 1 << IOSQE_IO_DRAIN_BIT;
    IOSQE_IO_LINK :: 1 << IOSQE_IO_LINK_BIT; IOSQE_IO_HARDLINK :: 1 << IOSQE_IO_HARDLINK_BIT;
    IOSQE_ASYNC :: 1 << IOSQE_ASYNC_BIT; IOSQE_BUFFER_SELECT :: 1 << IOSQE_BUFFER_SELECT_BIT;
    IOSQE_CQE_SKIP_SUCCESS :: 1 << IOSQE_CQE_SKIP_SUCCESS_BIT;
}
using IORING_ACCEPT_Flags :: enum_flags u16 { IORING_ACCEPT_MULTISHOT :: 1 << 0; }
using IORING_RECVSEND_Flags :: enum_flags u64 { IORING_RECVSEND_POLL_FIRST :: 1 << 0; }
using IORING_POLL_Flags :: enum_flags u32 {
    IORING_POLL_ADD_MULTI :: 1 << 0; IORING_POLL_UPDATE_EVENTS :: 1 << 1; IORING_POLL_UPDATE_USER_DATA :: 1 << 2;
}
using IORING_OP_Flags :: enum_flags u32 #specified {
    IORING_CLOSE_FD_AND_FILE_SLOT :: 1 << 0; IORING_FSYNC_DATASYNC :: 1 << 0;
    IORING_TIMEOUT_ABS :: 1 << 0; IORING_TIMEOUT_UPDATE :: 1 << 1; IORING_TIMEOUT_BOOTTIME :: 1 << 2;
    IORING_TIMEOUT_REALTIME :: 1 << 3; IORING_LINK_TIMEOUT_UPDATE :: 1 << 4; IORING_TIMEOUT_ETIME_SUCCESS :: 1 << 5;
    IORING_TIMEOUT_CLOCK_MASK :: IORING_TIMEOUT_BOOTTIME | IORING_TIMEOUT_REALTIME;
    IORING_TIMEOUT_UPDATE_MASK :: IORING_TIMEOUT_UPDATE | IORING_LINK_TIMEOUT_UPDATE;
    SPLICE_F_FD_IN_FIXED :: 1 << 31; IORING_ASYNC_CANCEL_ALL :: 1 << 0;
    IORING_ASYNC_CANCEL_FD :: 1 << 1; IORING_ASYNC_CANCEL_ANY :: 1 << 2;
}
using IOCQE_Flags :: enum_flags u32 {
    IORING_CQE_F_BUFFER :: 1 << 0; IORING_CQE_F_MORE :: 1 << 1; IORING_CQE_F_SOCK_NONEMPTY :: 1 << 2;
}
using IORING_ENTER :: enum_flags u32 {
    IORING_ENTER_GETEVENTS :: 1 << 0; IORING_ENTER_SQ_WAKEUP :: 1 << 1; IORING_ENTER_SQ_WAIT :: 1 << 2;
    IORING_ENTER_EXT_ARG :: 1 << 3; IORING_ENTER_REGISTERED_RING :: 1 << 4;
}
using IORING_REGISTER :: enum {
    IORING_REGISTER_BUFFERS :: 0; IORING_UNREGISTER_BUFFERS :: 1; IORING_REGISTER_FILES :: 2;
    IORING_UNREGISTER_FILES :: 3; IORING_REGISTER_EVENTFD :: 4; IORING_UNREGISTER_EVENTFD :: 5;
    IORING_REGISTER_FILES_UPDATE :: 6; IORING_REGISTER_EVENTFD_ASYNC :: 7; IORING_REGISTER_PROBE :: 8;
    IORING_REGISTER_PERSONALITY :: 9; IORING_UNREGISTER_PERSONALITY :: 10; IORING_REGISTER_RESTRICTIONS :: 11;
    IORING_REGISTER_ENABLE_RINGS :: 12; IORING_REGISTER_FILES2 :: 13; IORING_REGISTER_FILES_UPDATE2 :: 14;
    IORING_REGISTER_BUFFERS2 :: 15; IORING_REGISTER_BUFFERS_UPDATE :: 16; IORING_REGISTER_IOWQ_AFF :: 17;
    IORING_UNREGISTER_IOWQ_AFF :: 18; IORING_REGISTER_IOWQ_MAX_WORKERS :: 19; IORING_REGISTER_RING_FDS :: 20;
    IORING_UNREGISTER_RING_FDS :: 21; IORING_REGISTER_PBUF_RING :: 22; IORING_UNREGISTER_PBUF_RING :: 23;
    IORING_REGISTER_LAST;
}
using IO_WQ_Categories :: enum { IO_WQ_BOUND; IO_WQ_UNBOUND; }
using IORING_RSRC_Flags :: enum_flags u32 { IORING_RSRC_REGISTER_SPARSE :: 1 << 0; }
using IO_URING_OP_Flags :: enum_flags u16 { IO_URING_OP_SUPPORTED :: 1 << 0; }
using IO_URING_RESTRICTION_OP :: enum {
    IORING_RESTRICTION_REGISTER_OP :: 0; IORING_RESTRICTION_SQE_OP :: 1;
    IORING_RESTRICTION_SQE_FLAGS_ALLOWED :: 2; IORING_RESTRICTION_SQE_FLAGS_REQUIRED :: 3;
    IORING_RESTRICTION_LAST;
}
```

`IORING_OP` implicit values are their ordinal positions (NOP is 0; `IORING_OP_LAST` follows `IORING_OP_URING_CMD`). The records and operation set are described as new and untested by their public declaration; no precise kernel release/UAPI revision is specified. The `inotify_event.name` field is a zero-length flexible tail, so `size_of(inotify_event)` is the 16-byte fixed header; the event name occupies `len` bytes after it, so callers must use `len` and the containing read buffer.

#### `io_uring_buf` (struct)

- `addr: u64`
- `len:  u32`
- `bid:  u16`
- `resv: u16`

#### `io_uring_buf_ring` (struct)

```jai
io_uring_buf_ring :: struct {
    union {
        struct {
            resv1: u64;
            resv2: u32;
            resv3: u16;
            tail:  u16;
        };
        bufs: [0] io_uring_buf;
    };
}
```
#### `io_uring_buf_reg` (struct)

- `ring_addr:    u64`
- `ring_entries: u32`
- `bgid:         u16`
- `pad:          u16`
- `resv:     [3] u64`

#### `inotify_event` (struct)

- `wd: s32`
- `mask: u32`
- `cookie: u32`
- `len: u32`
- `name: [0] u8`

#### `sys_info` (struct)

- `uptime   : s64`
- `loads    : [3] u64`
- `totalram : u64`
- `freeram  : u64`
- `sharedram: u64`
- `bufferram: u64`
- `totalswap: u64`
- `freeswap : u64`
- `procs    : u16`
- `pad      : u16`
- `totalhigh: u64`
- `freehigh : u64`
- `mem_unit : u32`
- `_f       : [20-2*size_of(u64)-size_of(u32)] s8`

#### `input_absinfo` (struct)

- `value     : s32`
- `minimum   : s32`
- `maximum   : s32`
- `fuzz      : s32`
- `flat      : s32`
- `resolution: s32`

#### `input_event` (struct)

- `time: timeval`
- `type: u16`
- `code: u16`
- `value: s32`

## Behavioral contract and constraints

This module contains Linux-specific system bindings, including epoll and io_uring declarations. The io_uring portion is explicitly described as new and untested. These are low-level kernel interfaces: return values use Linux errno conventions, descriptors/rings are caller-managed, and kernel/header compatibility is required. It is not portable to non-Linux targets.

The public surface is a direct native binding: declared records, constants, and function signatures preserve the represented external ABI; this wrapper does not itself define a different ownership or error model. Native status/error values and handle lifetimes follow the external API. ABI compatibility across external-library versions is not specified unless a version note appears above.

The declarations use `OS == .LINUX` guards without CPU-specific variants. In particular, the visible syscall-number table and low-level record spellings do not establish an ARM64 ABI; callers must use only on a Linux target whose ABI matches those declarations. `epoll_event.data` is explicitly aligned to four bytes in the published Jai record; that is the declared representation, but no separate ARM64 layout is provided. The io_uring declarations are explicitly described as new and untested; their ABI revision is not stated, so compatibility with a particular kernel/UAPI revision is unknown. `inotify_event.name` is the conventional one-byte flexible-tail placeholder; actual names occupy `len` bytes after the fixed header, so callers must use `len` and the containing read buffer rather than treating that field as a full string.

External libraries/frameworks named in source: `libc`.
