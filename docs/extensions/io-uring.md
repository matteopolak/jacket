# Io_Uring (`Extensions/Io_Uring`)

## What it is

The Linux io_uring kernel interface and nothing more: the kernel's constants, enums and structs, the three system calls, and a small `Ring` that maps the shared rings and keeps the head/tail protocol. A low-level binding like `POSIX`, not an IO API; the [Io engine](io-engine.md) is built on it. Linux only (x86-64, arm64): importing it elsewhere is a compile error.

```jai
Io_Uring :: #import "Extensions/Io_Uring";

ring: Io_Uring.Ring;
if Io_Uring.ring_init(*ring, 64) < 0 return;          // 0, or the kernel's -errno
defer Io_Uring.ring_deinit(*ring);

sqe := Io_Uring.get_sqe(*ring);                       // zeroed; null when the queue is full
Io_Uring.prep_rw(sqe, .READ, fd, buffer.data, cast(u32) buffer.count, 0);
sqe.user_data = 7;
Io_Uring.submit_and_wait(*ring, 1);                   // entries submitted, or -errno

cqe: *Io_Uring.io_uring_cqe;
Io_Uring.wait_cqe(*ring, *cqe);                       // cqe.res = bytes read or -errno
Io_Uring.cqe_seen(*ring, cqe);
```

## How it works

| File | Contents |
| --- | --- |
| `bindings.jai` | generated from `linux/io_uring.h`: every `#define` is a constant of the same name, named enums are Jai enums, structs keep the kernel's names and layout |
| `syscalls.jai` | `io_uring_setup`, `io_uring_enter`, `io_uring_register`; return what the kernel returns (0 or more, or negative errno) |
| `ring.jai` | `Ring`, `Probe`, `io_uring_sqe128`, `io_uring_cqe32` |

- Enums drop the prefix their members share (`IORING_OP_WRITE` is `io_uring_op.WRITE`); `io_uring_op` is `u8`, the others `u32`. The SQE still has `opcode: u8`: `sqe.opcode = cast(u8) io_uring_op.WRITE` or use `prep_rw`. Anonymous unions/structs stay anonymous, so `sqe.off`, `sqe.addr2` reach the right bytes. Flexible array members are `[0]` arrays at the struct end.
- With `IORING_SETUP_SQE128` cast what `get_sqe` returns to `io_uring_sqe128` (`sqe_command` gives the 80 command bytes); with `CQE32` cast the cqe to `io_uring_cqe32`.
- `#assert size_of(...)` / `offset_of(...)` at the end of `bindings.jai` pin every struct to C's layout, so a layout or generator bug breaks the module build instead of corrupting a ring. `tests/abi/manifest.txt` also compares older structs with the host's real header on Linux CI.
- `ring_init(ring, entries, flags)` / `ring_init_params`: setup, then `mmap` of the submission ring, completion ring (one mapping with `IORING_FEAT_SINGLE_MMAP`) and SQE array. `NO_MMAP`, `SQE_MIXED`, `CQE_MIXED`, `SQ_REWIND` return `-EINVAL` (use the raw syscalls). A failed init unmaps and closes everything.
- `get_sqe`, `submit`/`submit_and_wait(n)` (with `SQPOLL` they enter only to wake the poll thread), `publish_sqes` (no enter), `peek_cqe` (`-EAGAIN`), `wait_cqe`, `wait_cqe_timeout` (`-ETIME`, needs `FEAT_EXT_ARG`, 5.11), `cqe_seen`/`cq_advance(n)`. Rings that need an enter to see completions (`DEFER_TASKRUN`, `IOPOLL`, CQ overflow) enter for you. `probe_ops` + `supports(*probe, .WRITE)` ask which opcodes exist (5.6; `-EINVAL` on older).
- Memory ordering: kernel-advanced indices (SQ head, CQ tail) are read with `atomic_load`, program-advanced ones (SQ tail, CQ head) written with `atomic_store` (sequentially consistent in jaic, which is stronger than the acquire/release the protocol needs). One thread at a time per `Ring`; nothing locks. The mappings are owned by the OS (`mmap`/`munmap`), not the allocator.
- **Availability.** Needs Linux 5.1+. Container seccomp profiles (Docker default, gVisor), RHEL 9 defaults and `kernel.io_uring_disabled` make `ring_init` return `-ENOSYS`, `-EPERM` or `-EACCES`: treat that as "not available" and fall back or skip, not as a failure.

## How to change it

- **Regenerate bindings:** `python3 tools/gen_io_uring.py` (fetches the pinned header, verifies its sha256, writes `bindings.jai`); `--header FILE` uses a header on disk (a newer kernel's); `--check` exits 1 if `bindings.jai` is stale. No libclang: the parser handles only the C subset this header uses and stops with a message otherwise. To move the pin change `TAG` and `SHA256` at the top of the script. It checks struct sizes the ABI fixes (`KNOWN_SIZES`). For a new header you may touch `ENUM_TYPES`/`ENUM_PREFIXES` (narrow enums, odd prefixes), `MACRO_WIDTH`/`MACRO_SIGNED` (macros using `~` or negative), `EXTERNAL_STRUCTS`. The header is `(GPL-2.0 WITH Linux-syscall-note) OR MIT`; the script is original work.
- **New helper** goes in `ring.jai`; keep it thin. Policy (rings per thread, flags, fallbacks) belongs to the engine.
- **A setup flag that changes the mapping:** extend `ring_init_params`; flags it does not follow stay in the `-EINVAL` list.
- Gotchas: the SQE array is mapped separately at `IORING_OFF_SQES`; ring size arithmetic mirrors the kernel (`sq_off.array + entries * 4` unless `NO_SQARRAY`, `cq_off.cqes + entries * cqe_bytes`), so change both together. Index math relies on unsigned wrap-around.

## Configuration

None.

## Dependencies and tests

libc (`syscall`, `mmap`, `munmap`, `close`, `__errno_location`; declared inside the module so `POSIX` is not imported) and [`Atomics`](../specs/stdlib/modules/atomics.md). Tests: `tests/extensions/io-uring-raw.jai` (runs everywhere; on Linux with usable io_uring does NOP, full queue, read/write, probe, timed wait, 128-byte entries; elsewhere prints why it skipped and exits 0) and `tests/extensions/io-uring-protocol.jai` (the queue protocol over caller-owned memory plus a second thread, no kernel call; run with `python3 tests/extensions/run-uring-protocol.py`). `tests/stdlib-targets.txt` lists the module's intentional failure on other targets.
