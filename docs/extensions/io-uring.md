# Io_Uring

The Linux io_uring kernel interface and nothing more: constants, enums and structs, the three system calls, and a small `Ring` that maps the shared rings and keeps the head/tail protocol. A low-level binding like `POSIX`, not an IO API; the [Io engine](io-engine.md) is built on it. Linux only (x86-64, arm64).

```jai
Io_Uring :: #import "Extensions/Io_Uring";

ring: Io_Uring.Ring;
if Io_Uring.ring_init(*ring, 64) < 0 return;          // 0, or the kernel's -errno
defer Io_Uring.ring_deinit(*ring);

sqe := Io_Uring.get_sqe(*ring);                       // zeroed; null when full
Io_Uring.prep_rw(sqe, .READ, fd, buffer.data, cast(u32) buffer.count, 0);
sqe.user_data = 7;
Io_Uring.submit_and_wait(*ring, 1);

cqe: *Io_Uring.io_uring_cqe;
Io_Uring.wait_cqe(*ring, *cqe);                       // cqe.res = bytes read or -errno
Io_Uring.cqe_seen(*ring, cqe);
```

## How it works

- `bindings.jai` is generated from `linux/io_uring.h`: macros become constants, enums drop their shared prefix (`io_uring_op.WRITE`), structs keep the kernel layout, and `#assert`s pin sizes and offsets. `syscalls.jai` has the raw calls (negative errno on failure). `ring.jai` has `Ring`.
- With `IORING_SETUP_SQE128` cast `get_sqe`'s result to `io_uring_sqe128`; with `CQE32` cast the cqe to `io_uring_cqe32`.
- `ring_init` maps the submission ring, completion ring and SQE array. `NO_MMAP`, `SQE_MIXED`, `CQE_MIXED` and `SQ_REWIND` return `-EINVAL`; use the raw syscalls.
- Kernel-advanced indices are read with `atomic_load`, program-advanced ones written with `atomic_store`. One thread at a time per `Ring`; nothing locks.
- Needs Linux 5.1+. Container seccomp profiles and `kernel.io_uring_disabled` make `ring_init` return `-ENOSYS`, `-EPERM` or `-EACCES`: treat that as "not available", not failure.

## How to change it

- Regenerate bindings: `PYTHONPATH=tools python3 tools/generators/gen_io_uring.py` (`--header FILE` for a newer header, `--check` for staleness). Move the pin by changing `TAG` and `SHA256` in the script. It has no libclang, so it stops with a message on C it can't parse; `ENUM_TYPES`, `ENUM_PREFIXES`, `MACRO_WIDTH`, `MACRO_SIGNED` and `EXTERNAL_STRUCTS` tune it.
- New helpers go in `ring.jai` and stay thin; policy belongs to the engine.
- Gotcha: the SQE array is mapped separately at `IORING_OFF_SQES`, and the ring size arithmetic mirrors the kernel. Change both sides together.
- Tests: `tests/raw.jai` (skips with a message where io_uring is unusable) and `tests/protocol.jai` (queue protocol over caller memory with a second thread).
