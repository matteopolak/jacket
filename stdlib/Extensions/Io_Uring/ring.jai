// A mapped ring and the userspace half of its head/tail protocol.
//
// The kernel and the program share two queues. The program fills submission entries (SQEs) and
// then publishes them by moving the submission tail; the kernel moves the submission head as it
// takes them. The kernel fills completion entries (CQEs) and publishes them by moving the
// completion tail; the program consumes them and moves the completion head. Whoever advances an
// index writes it with a release store after writing the entries, and whoever reads an index to
// learn what is ready reads it with an acquire load, so the entries are visible by then. An index
// the program owns (the submission tail, the completion head) is read plainly: nobody else writes
// it. One thread at a time may use a Ring; nothing here locks.

Ring :: struct {
    fd: s32 = -1;
    flags: u32; // IORING_SETUP_* the ring was created with
    features: u32; // IORING_FEAT_* the kernel reports

    sq_entries: u32;
    sq_mask: u32;
    sq_head: *u32; // advanced by the kernel
    sq_tail: *u32; // advanced by submit
    sq_flags: *u32; // IORING_SQ_*
    sq_dropped: *u32;
    sq_array: *u32; // the index array, null with IORING_SETUP_NO_SQARRAY
    sqes: *u8;
    sqe_bytes: u32; // 64, or 128 with IORING_SETUP_SQE128
    sqe_head: u32; // entries get_sqe handed out that submit has not published yet start here
    sqe_tail: u32; // and the next get_sqe hands out this one

    cq_entries: u32;
    cq_mask: u32;
    cq_head: *u32; // advanced by cqe_seen
    cq_tail: *u32; // advanced by the kernel
    cq_flags: *u32; // IORING_CQ_*
    cq_overflow: *u32;
    cqes: *u8;
    cqe_bytes: u32; // 16, or 32 with IORING_SETUP_CQE32

    sq_ring: *void;
    sq_ring_bytes: u64;
    cq_ring: *void; // the same mapping as sq_ring with IORING_FEAT_SINGLE_MMAP
    cq_ring_bytes: u64;
    sqes_bytes: u64;
}

// A 32-byte completion, as a ring created with IORING_SETUP_CQE32 delivers. Cast the *io_uring_cqe
// that peek_cqe and wait_cqe give you to this; `extra` is the 16 bytes the operation filled in.
io_uring_cqe32 :: struct {
    using cqe: io_uring_cqe;
    extra: [2] u64;
}

// A 128-byte submission entry, as a ring created with IORING_SETUP_SQE128 hands out. The first 64
// bytes are an ordinary io_uring_sqe; sqe_command gives the 80 bytes of command data from offset 48.
io_uring_sqe128 :: struct {
    using sqe: io_uring_sqe;
    extension: [64] u8;
}

#assert size_of(io_uring_cqe32) == 32;
#assert size_of(io_uring_sqe128) == 128;

// Creates a ring of at least `entries` submission entries (a power of two, the kernel rounds up)
// and maps it. `flags` are IORING_SETUP_* flags. Returns 0, or the negative errno of the failing
// step: -ENOSYS where the kernel has no io_uring, -EPERM where policy forbids it (a seccomp filter,
// kernel.io_uring_disabled), -ENOMEM when the locked-memory limit of an old kernel is too small.
// IORING_SETUP_NO_MMAP, SQE_MIXED, CQE_MIXED and SQ_REWIND change the memory layout in ways this
// type does not follow: they return -EINVAL; drive such rings with the system calls directly.
ring_init :: (ring: *Ring, entries: u32, flags: u32 = 0) -> s32 {
    params: io_uring_params;
    params.flags = flags;

    return ring_init_params(ring, entries, *params);
}

// ring_init with every parameter: `params` goes in (flags, cq_entries with IORING_SETUP_CQSIZE,
// the SQPOLL thread's CPU and idle time, wq_fd with ATTACH_WQ) and comes back as the kernel
// filled it in.
ring_init_params :: (ring: *Ring, entries: u32, params: *io_uring_params) -> s32 {
    unsupported :: IORING_SETUP_NO_MMAP | IORING_SETUP_SQE_MIXED | IORING_SETUP_CQE_MIXED | IORING_SETUP_SQ_REWIND;
    if params.flags & unsupported return -ERR_INVAL;

    fd := io_uring_setup(entries, params);
    if fd < 0 return fd;

    ring.* = .{};
    ring.fd = fd;
    ring.flags = params.flags;
    ring.features = params.features;
    ring.sqe_bytes = ifx params.flags & IORING_SETUP_SQE128 then 128 else 64;
    ring.cqe_bytes = ifx params.flags & IORING_SETUP_CQE32 then 32 else 16;

    has_array := (params.flags & IORING_SETUP_NO_SQARRAY) == 0;

    ring.sq_ring_bytes = cast(u64) params.sq_off.array;
    if has_array ring.sq_ring_bytes += cast(u64) params.sq_entries * size_of(u32);

    ring.cq_ring_bytes = cast(u64) params.cq_off.cqes + cast(u64) params.cq_entries * ring.cqe_bytes;
    single := (params.features & IORING_FEAT_SINGLE_MMAP) != 0;

    if single {
        // One mapping holds both rings, as large as the bigger of the two.
        if ring.cq_ring_bytes > ring.sq_ring_bytes ring.sq_ring_bytes = ring.cq_ring_bytes;
        ring.cq_ring_bytes = ring.sq_ring_bytes;
    }

    ring.sq_ring = mmap(null, ring.sq_ring_bytes, PROT_READ_WRITE, MAP_SHARED | MAP_POPULATE, fd, IORING_OFF_SQ_RING);
    if ring.sq_ring == MAP_FAILED {
        failure := -errno_value();
        ring.sq_ring = null;
        ring_deinit(ring);

        return failure;
    }

    if single {
        ring.cq_ring = ring.sq_ring;
    } else {
        ring.cq_ring = mmap(null, ring.cq_ring_bytes, PROT_READ_WRITE, MAP_SHARED | MAP_POPULATE, fd, IORING_OFF_CQ_RING);
        if ring.cq_ring == MAP_FAILED {
            failure := -errno_value();
            ring.cq_ring = null;
            ring_deinit(ring);

            return failure;
        }
    }

    ring.sqes_bytes = cast(u64) params.sq_entries * ring.sqe_bytes;
    sqes := mmap(null, ring.sqes_bytes, PROT_READ_WRITE, MAP_SHARED | MAP_POPULATE, fd, IORING_OFF_SQES);
    if sqes == MAP_FAILED {
        failure := -errno_value();
        ring.sqes_bytes = 0;
        ring_deinit(ring);

        return failure;
    }

    ring.sqes = cast(*u8) sqes;

    sq := *params.sq_off;

    ring.sq_head = cast(*u32) byte_offset(ring.sq_ring, sq.head);
    ring.sq_tail = cast(*u32) byte_offset(ring.sq_ring, sq.tail);
    ring.sq_mask = (cast(*u32) byte_offset(ring.sq_ring, sq.ring_mask)).*;
    ring.sq_entries = (cast(*u32) byte_offset(ring.sq_ring, sq.ring_entries)).*;
    ring.sq_flags = cast(*u32) byte_offset(ring.sq_ring, sq.flags);
    ring.sq_dropped = cast(*u32) byte_offset(ring.sq_ring, sq.dropped);

    cq := *params.cq_off;

    ring.cq_head = cast(*u32) byte_offset(ring.cq_ring, cq.head);
    ring.cq_tail = cast(*u32) byte_offset(ring.cq_ring, cq.tail);
    ring.cq_mask = (cast(*u32) byte_offset(ring.cq_ring, cq.ring_mask)).*;
    ring.cq_entries = (cast(*u32) byte_offset(ring.cq_ring, cq.ring_entries)).*;
    ring.cq_overflow = cast(*u32) byte_offset(ring.cq_ring, cq.overflow);
    ring.cqes = cast(*u8) byte_offset(ring.cq_ring, cq.cqes);
    ring.cq_flags = ifx cq.flags != 0 then cast(*u32) byte_offset(ring.cq_ring, cq.flags) else null;

    if has_array {
        // Entry i of the submission array names SQE i. Keeping it that way means a submission
        // only has to move the tail.
        ring.sq_array = cast(*u32) byte_offset(ring.sq_ring, sq.array);
        for 0..cast(s64) ring.sq_entries - 1 ring.sq_array[it] = cast(u32) it;
    }

    return 0;
}

// Unmaps the ring and closes its descriptor. Safe on a ring that was never initialized or already
// closed. Pending operations are dropped by the kernel.
ring_deinit :: (ring: *Ring) {
    if ring.sqes != null munmap(ring.sqes, ring.sqes_bytes);
    if ring.cq_ring != null && ring.cq_ring != ring.sq_ring munmap(ring.cq_ring, ring.cq_ring_bytes);
    if ring.sq_ring != null munmap(ring.sq_ring, ring.sq_ring_bytes);
    if ring.fd >= 0 close(ring.fd);

    ring.* = .{};
}

// The next free submission entry, zeroed, or null when the submission queue is full (submit, then
// ask again). Fill it in (prep_rw, prep_nop or by hand), set `user_data`, and call submit. With
// IORING_SETUP_SQE128 the entry is 128 bytes: cast it to io_uring_sqe128.
get_sqe :: (ring: *Ring) -> *io_uring_sqe {
    head := atomic_load(ring.sq_head, .ACQUIRE);
    if ring.sqe_tail - head >= ring.sq_entries return null;

    sqe := cast(*io_uring_sqe) byte_offset(ring.sqes, (ring.sqe_tail & ring.sq_mask) * ring.sqe_bytes);

    ring.sqe_tail += 1;
    memset(sqe, 0, ring.sqe_bytes);

    return sqe;
}

// Entries handed out by get_sqe and not yet submitted.
sq_pending :: (ring: *Ring) -> u32 {
    return ring.sqe_tail - ring.sqe_head;
}

// Publishes the entries handed out so far, so the kernel can see them, and returns how many it
// has not yet taken. Does not enter the kernel; submit does.
publish_sqes :: (ring: *Ring) -> u32 {
    tail := ring.sq_tail.*;

    count := ring.sqe_tail - ring.sqe_head;

    if count != 0 {
        if ring.sq_array != null {
            for 1..count {
                ring.sq_array[tail & ring.sq_mask] = ring.sqe_head & ring.sq_mask;
                tail += 1;
                ring.sqe_head += 1;
            }
        } else {
            tail += count;
            ring.sqe_head += count;
        }

        atomic_store(ring.sq_tail, tail, .RELEASE);
    }

    return tail - atomic_load(ring.sq_head, .ACQUIRE);
}

// Publishes the pending entries and tells the kernel about them. Returns how many it took, or a
// negative errno (-EBUSY: completions are backed up, reap some first; -EAGAIN: out of resources).
submit :: (ring: *Ring) -> s32 {
    return submit_and_wait(ring, 0);
}

// submit, then sleeps until at least `wait_for` completions are ready.
submit_and_wait :: (ring: *Ring, wait_for: u32) -> s32 {
    unconsumed := publish_sqes(ring);

    flags: u32;

    if wait_for > 0 || needs_enter(ring) flags |= IORING_ENTER_GETEVENTS;

    if ring.flags & IORING_SETUP_SQPOLL {
        // A polling thread takes entries on its own; only wake it when it went to sleep.
        if atomic_load(ring.sq_flags, .SEQ_CST) & IORING_SQ_NEED_WAKEUP {
            flags |= IORING_ENTER_SQ_WAKEUP;
        } else if flags == 0 {
            return cast(s32) unconsumed;
        }
    } else if unconsumed == 0 && flags == 0 {
        return 0;
    }

    return io_uring_enter(ring.fd, unconsumed, wait_for, flags, null, 0);
}

// Completions ready to be consumed.
cq_ready :: (ring: *Ring) -> u32 {
    return atomic_load(ring.cq_tail, .ACQUIRE) - ring.cq_head.*;
}

// The oldest unconsumed completion in `cqe`, without waiting: returns 0, or -EAGAIN when there is
// none. The pointer stays valid until cqe_seen. A ring that only produces completions when it
// enters the kernel (DEFER_TASKRUN, IOPOLL, or one with overflowed completions) is entered once
// to collect them.
peek_cqe :: (ring: *Ring, cqe: **io_uring_cqe) -> s32 {
    collected := false;

    while true {
        head := ring.cq_head.*;
        tail := atomic_load(ring.cq_tail, .ACQUIRE);

        if head != tail {
            cqe.* = cast(*io_uring_cqe) byte_offset(ring.cqes, (head & ring.cq_mask) * ring.cqe_bytes);

            return 0;
        }

        if collected || !needs_enter(ring) return -ERR_AGAIN;

        result := io_uring_enter(ring.fd, 0, 0, IORING_ENTER_GETEVENTS, null, 0);
        if result < 0 && result != -ERR_INTR return result;

        collected = true;
    }

    return -ERR_AGAIN;
}

// Waits until a completion is ready and returns it in `cqe` (see peek_cqe). Submitted entries must
// have been passed to submit first. Returns 0 or a negative errno.
wait_cqe :: (ring: *Ring, cqe: **io_uring_cqe) -> s32 {
    while true {
        result := peek_cqe(ring, cqe);
        if result != -ERR_AGAIN return result;

        result = io_uring_enter(ring.fd, 0, 1, IORING_ENTER_GETEVENTS, null, 0);
        if result < 0 && result != -ERR_INTR return result;
    }

    return -ERR_AGAIN;
}

// wait_cqe with a limit: returns -ETIME when no completion arrived within `nanoseconds`.
// Needs IORING_FEAT_EXT_ARG (Linux 5.11); returns -EOPNOTSUPP without it.
wait_cqe_timeout :: (ring: *Ring, cqe: **io_uring_cqe, nanoseconds: s64) -> s32 {
    result := peek_cqe(ring, cqe);
    if result != -ERR_AGAIN return result;
    if !(ring.features & IORING_FEAT_EXT_ARG) return -ERR_OPNOTSUPP;

    timeout: __kernel_timespec;
    timeout.tv_sec = nanoseconds / 1000000000;
    timeout.tv_nsec = nanoseconds % 1000000000;

    argument: io_uring_getevents_arg;
    argument.ts = cast(u64) cast(*void) *timeout;

    while true {
        result = io_uring_enter(ring.fd, 0, 1, IORING_ENTER_GETEVENTS | IORING_ENTER_EXT_ARG, *argument, size_of(io_uring_getevents_arg));
        if result < 0 && result != -ERR_INTR return result;

        result = peek_cqe(ring, cqe);
        if result != -ERR_AGAIN return result;
    }

    return -ERR_AGAIN;
}

// Marks `cqe`, the one peek_cqe or wait_cqe returned, as consumed, handing its slot back to the
// kernel. The pointer must not be used afterwards.
cqe_seen :: (ring: *Ring, cqe: *io_uring_cqe) {
    cq_advance(ring, 1);
}

// Consumes `count` completions at once (each CQE32 counts as one).
cq_advance :: (ring: *Ring, count: u32) {
    atomic_store(ring.cq_head, ring.cq_head.* + count, .RELEASE);
}

// The buffer id of a completion that used a provided buffer (IORING_CQE_F_BUFFER).
cqe_buffer_id :: (cqe: *io_uring_cqe) -> u16 {
    return cast(u16) (cqe.flags >> IORING_CQE_BUFFER_SHIFT);
}

// Sets the fields most operations share: read, write, accept, send, recv... `address` and `length`
// describe the buffer, `offset` the file position (or the operation's second argument).
prep_rw :: (sqe: *io_uring_sqe, opcode: io_uring_op, fd: s32, address: *void, length: u32, offset: u64) {
    sqe.opcode = cast(u8) opcode;
    sqe.fd = fd;
    sqe.addr = cast(u64) address;
    sqe.len = length;
    sqe.off = offset;
}

prep_nop :: (sqe: *io_uring_sqe) {
    sqe.opcode = cast(u8) io_uring_op.NOP;
}

// The 80 bytes of command data of a 128-byte entry (IORING_SETUP_SQE128), at offset 48.
sqe_command :: (sqe: *io_uring_sqe) -> *u8 {
    return cast(*u8) byte_offset(sqe, 48);
}

// Which opcodes the running kernel supports.
Probe :: struct {
    last_op: u8; // the highest opcode the kernel knows
    supported: [256] bool; // indexed by opcode
}

// Asks the kernel which operations it supports (IORING_REGISTER_PROBE, Linux 5.6). Returns 0, or
// -EINVAL on an older kernel, where only the original operations exist.
probe_ops :: (ring: *Ring, probe: *Probe) -> s32 {
    // A struct io_uring_probe followed by 256 io_uring_probe_op, as 4-byte words.
    storage: [(size_of(io_uring_probe) + 256 * size_of(io_uring_probe_op)) / 4] u32;
    raw := cast(*io_uring_probe) storage.data;

    result := io_uring_register(ring.fd, cast(u32) io_uring_register_op.REGISTER_PROBE, raw, 256);

    if result < 0 return result;

    probe.* = .{};
    probe.last_op = raw.last_op;

    ops := cast(*io_uring_probe_op) byte_offset(raw, size_of(io_uring_probe));

    for 0..cast(s64) raw.ops_len - 1 {
        if ops[it].flags & IO_URING_OP_SUPPORTED probe.supported[ops[it].op] = true;
    }

    return 0;
}

supports :: (probe: *Probe, opcode: io_uring_op) -> bool {
    return probe.supported[cast(u8) opcode];
}

#scope_file
#import "Atomics";

libc :: #system_library "libc";

mmap :: (address: *void, length: u64, protection: s32, flags: s32, descriptor: s32, offset: s64) -> *void #foreign libc;
munmap :: (address: *void, length: u64) -> s32 #foreign libc;
close :: (descriptor: s32) -> s32 #foreign libc;
errno_location :: () -> *s32 #foreign libc "__errno_location";

PROT_READ_WRITE :: 3;
MAP_SHARED :: 0x01;
MAP_POPULATE :: 0x8000;
MAP_FAILED :: cast(*void) -1;

errno_value :: () -> s32 {
    return errno_location().*;
}

byte_offset :: (base: *void, bytes: $T) -> *void {
    return cast(*void) (cast(u64) base + cast(u64) bytes);
}

// Does the ring need a trip into the kernel to see completions that are already due: overflowed
// ones, task work waiting to run, or a mode that never posts them on its own.
needs_enter :: (ring: *Ring) -> bool {
    if ring.flags & (IORING_SETUP_DEFER_TASKRUN | IORING_SETUP_IOPOLL) return true;

    return (atomic_load(ring.sq_flags, .RELAXED) & (IORING_SQ_CQ_OVERFLOW | IORING_SQ_TASKRUN)) != 0;
}
