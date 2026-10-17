# Io_Uring

## What it is

`stdlib/Extensions/Io_Uring` is the Linux io_uring kernel interface and nothing more: the kernel's constants, enums and structs, the three system calls, and a small `Ring` type that maps the shared rings and keeps the head/tail protocol. It is a jaic extension (other Jai compilers have no such module) and a low-level binding, the way `POSIX` is. It is not an IO API: fibers over files and sockets are `Extensions/Io` ([fibers](io-fibers.md)), and the completion engine of `Io` is built on this module.

It is Linux only: importing it anywhere else is a compile error (`Extensions/Io_Uring is Linux only`).

## Canonical atomics and protocol verification

The ring uses the reviewed `atomic_load(pointer)` and
`atomic_store(pointer, value)` intrinsics, without extra ordering parameters.
The supplied compiler lowers both to sequentially consistent operations. This
preserves the required acquire observation of kernel-published SQ heads/CQ tails
and release publication of userspace SQ tails/CQ heads. The SQPOLL wakeup flag
read stays sequentially consistent; the former relaxed task/overflow flag read
is now stronger too. Index ownership is unchanged: userspace alone writes SQ
tail and CQ head, while the kernel alone writes SQ head and CQ tail. A Ring
still has one userspace owner at a time.

Sequential consistency can impose stronger barriers or locked instructions and
restrict reordering beyond the required acquire/release or relaxed operation.
The cost depends on target and workload; no Linux benchmark was run here. Kernel
records, generated ABI assertions, syscalls and mapping ownership are unchanged.
The OS owns mapped storage; real mmap/munmap remain the allocation/release path,
so no Basic allocator adapter or heap free is appropriate for these mappings.
The `cqe_seen` pointer parameter and its existing head-advance behavior are
retained, with the intentionally unused pointer explicitly discarded rather
than renaming a public parameter or suppressing a lint rule.

`tests/extensions/io-uring-protocol.jai` loads the production queue source
directly on the actual host. Caller-owned memory exercises SQ reservation/full
capacity, partial consumption, idempotent publication, u32 rollover, optional
index arrays, 64/128-byte SQE and 16/32-byte CQE strides, and completion consumption.
A second real host thread also exchanges 1000 submissions/completions through the
actual atomic publication paths. Interpreter/native O0/O2 pass on Darwin arm64.
These are userspace protocol tests; no kernel syscall is called or substituted.

`python3 tests/extensions/run-uring-protocol.py` repeats the host protocol,
Linux source checks on both CPUs, contextual strict lint and formatting checks,
writing actual command results to ignored
`artifacts/extensions-dependencies/uring-protocol/results.json`.

```sh
JAIC_STDLIB="$PWD/stdlib" target/debug/jaic -interp tests/extensions/io-uring-protocol.jai
JAIC_STDLIB="$PWD/stdlib" target/debug/jaic tests/extensions/io-uring-protocol.jai -O0 -exe artifacts/extensions-dependencies/uring-protocol-o0
artifacts/extensions-dependencies/uring-protocol-o0
JAIC_STDLIB="$PWD/stdlib" target/debug/jaic tests/extensions/io-uring-protocol.jai -O2 -exe artifacts/extensions-dependencies/uring-protocol-o2
artifacts/extensions-dependencies/uring-protocol-o2
```

The original dependency reduction and existing raw kernel test source-check on
Linux x64 and arm64. Strict lint passes on both Linux CPUs in the raw test's
actual context, and on the host protocol test. All 122 focused integration checks,
the original three fiber tests and two required-WASI formatter tests pass.
Actual Linux kernel execution is unavailable on this host. Mapping/fd teardown,
SQPOLL wakeup, timed waits and real kernel acceptance still need native Linux
evidence; source checks and caller-memory threads do not provide it. The raw
kernel suite was not run on macOS to turn its platform skip into a passing test.

## How it works

Three layers, loaded by `module.jai`:

| File | Contents |
|---|---|
| `bindings.jai` | Generated from `linux/io_uring.h`. Every `#define` is a constant of the same name; every named enum is a Jai enum; every struct keeps the kernel's name, field names and layout (`io_uring_sqe`, `io_uring_cqe`, `io_uring_params`, `io_sqring_offsets`, `io_cqring_offsets`, `io_uring_probe`, `io_uring_probe_op`, `io_uring_restriction`, `io_uring_buf_ring`, `io_uring_getevents_arg`, the register arguments, the zero-copy receive structs, ...). |
| `syscalls.jai` | `io_uring_setup`, `io_uring_enter`, `io_uring_register`. They return what the kernel returns: 0 or more, or a negative errno. |
| `ring.jai` | `Ring` and its helpers, the `Probe` type, `io_uring_sqe128` and `io_uring_cqe32`. |

### The bindings

- Enums drop the prefix their members share: `IORING_OP_WRITE` is `io_uring_op.WRITE`, `IORING_REGISTER_PROBE` is `io_uring_register_op.REGISTER_PROBE`. `io_uring_op` is a `u8` enum (its field in the SQE is a byte); the others are `u32`. The sqe still has `opcode: u8`, so write `sqe.opcode = cast(u8) io_uring_op.WRITE` (or use `prep_rw`).
- Unnamed enums and all `#define`s are plain constants with the C name (`IORING_SETUP_SQPOLL`, `IORING_ENTER_GETEVENTS`, `IORING_FEAT_SINGLE_MMAP`, `IORING_OFF_SQES`, `IORING_CQE_F_MORE`, `IOSQE_IO_LINK`, ...).
- Anonymous unions and structs of C are anonymous `union { }` and `struct { }` members, so `sqe.off`, `sqe.addr2` and `sqe.cmd_op` all reach the right bytes. A C flexible array member (`ops[]`, `big_cqe[]`, `bufs[]`) is a `[0]` array at the end of its struct; the entries start right after the struct.
- The 128-byte SQE and 32-byte CQE are selected by setup flags, as in the kernel. With `IORING_SETUP_SQE128` entries are 128 bytes: cast what `get_sqe` returns to `io_uring_sqe128` (the 64-byte `io_uring_sqe` followed by 64 bytes; `sqe_command` gives the 80 bytes of command data from offset 48). With `IORING_SETUP_CQE32` completions are 32 bytes: cast the `*io_uring_cqe` to `io_uring_cqe32` for the extra 16 bytes.
- At the end of `bindings.jai`, `#assert size_of(...)` and `#assert offset_of(...)` pin the size of every struct and the offset of every member to the values C's layout rules give them. A jaic layout bug, or a generator bug, breaks the build of the module instead of corrupting a ring. `tests/abi/manifest.txt` also compares the older structs and constants with the host's real `<linux/io_uring.h>` on Linux CI ([native bindings](../specs/stdlib/README.md)).

### Regenerating the bindings

`tools/gen_io_uring.py` writes `bindings.jai`. It has no dependencies beyond Python 3, and no libclang: the parser handles the C subset this one header uses (integer `#define`s, enums, nested anonymous structs and unions, `__DECLARE_FLEX_ARRAY`, fixed-width kernel integers) and stops with a message on anything else.

```sh
python3 tools/gen_io_uring.py                      # fetch the pinned header (Linux v7.0), verify its sha256, write bindings.jai
python3 tools/gen_io_uring.py --header io_uring.h  # use a header on disk, e.g. a newer kernel's
python3 tools/gen_io_uring.py --check              # exit 1 if bindings.jai is not what the pinned header generates
```

When a kernel adds something: take its `include/uapi/linux/io_uring.h`, run with `--header`, review the diff (the header only ever grows), and commit it. To move the pin, change `TAG` and `SHA256` at the top of the script (`sha256sum` of the file at that tag) and run it without arguments. The script computes each struct's size and member offsets itself and compares the sizes the kernel ABI fixes (`KNOWN_SIZES`: SQE 64, CQE 16, params 120, ...), so a parser mistake is caught before the output is written. Things you may have to touch for a new header:

- `ENUM_TYPES`, `ENUM_PREFIXES`: an enum that must be narrower than `u32`, or whose member prefix is not the longest common one (a single-member enum).
- `MACRO_WIDTH`, `MACRO_SIGNED`: a macro that uses `~` or is negative needs its C width stated; the script refuses to guess.
- `EXTERNAL_STRUCTS`: structs from other kernel headers that the file uses (`__kernel_timespec`).

The header is `(GPL-2.0 WITH Linux-syscall-note) OR MIT` licensed; the generated file says where it came from. The script is original work and is not derived from liburing or any other library.

### The ring

```jai
Io_Uring :: #import "Extensions/Io_Uring";

ring: Io_Uring.Ring;
made := Io_Uring.ring_init(*ring, 64);            // 0, or the kernel's -errno
if made < 0 return;
defer Io_Uring.ring_deinit(*ring);

sqe := Io_Uring.get_sqe(*ring);                   // zeroed; null when the queue is full
Io_Uring.prep_rw(sqe, .READ, fd, buffer.data, cast(u32) buffer.count, 0);
sqe.user_data = 7;

Io_Uring.submit_and_wait(*ring, 1);               // entries submitted, or -errno

cqe: *Io_Uring.io_uring_cqe;
Io_Uring.wait_cqe(*ring, *cqe);                   // cqe.user_data == 7, cqe.res = bytes read or -errno
Io_Uring.cqe_seen(*ring, cqe);
```

- `ring_init(ring, entries, flags)` / `ring_init_params(ring, entries, params)`: `io_uring_setup`, then `mmap` of the submission ring, the completion ring (one mapping for both when the kernel reports `IORING_FEAT_SINGLE_MMAP`) and the SQE array. It fills in the pointers from the offsets the kernel returns. With the submission index array (not `IORING_SETUP_NO_SQARRAY`) entry `i` is set to name SQE `i` once, so submitting only moves the tail. `NO_MMAP`, `SQE_MIXED`, `CQE_MIXED` and `SQ_REWIND` change the layout in ways `Ring` does not follow and return `-EINVAL`: drive those with the system calls.
- `get_sqe` hands out the next entry, zeroed, or null when `sq_entries` are in flight. `submit` / `submit_and_wait(n)` publish the pending entries and enter the kernel. With `IORING_SETUP_SQPOLL` they enter only to wake a sleeping poll thread. `publish_sqes` publishes without entering, for programs that call `io_uring_enter` themselves.
- `peek_cqe` returns the oldest completion or `-EAGAIN`; `wait_cqe` sleeps until there is one; `wait_cqe_timeout` returns `-ETIME` after a limit (needs `IORING_FEAT_EXT_ARG`, Linux 5.11). A ring whose completions only appear when the process enters the kernel (`DEFER_TASKRUN`, `IOPOLL`, or after a CQ overflow) is entered for you. `cqe_seen` / `cq_advance(n)` hand completion slots back.
- `probe_ops(ring, *probe)` fills a `Probe` (`IORING_REGISTER_PROBE`, Linux 5.6): `supports(*probe, .WRITE)` says whether this kernel knows an opcode, and `probe.last_op` how many it has. It returns `-EINVAL` on an older kernel.
- Memory ordering follows the kernel's contract: indices the kernel advances (SQ head, CQ tail) are read with acquire loads; indices the program advances (SQ tail after the entries are written, CQ head after the entry was read) are written with release stores ([Atomics](../specs/stdlib/modules/atomics.md)). The indices the program owns are read plainly. One thread at a time may use a `Ring`; nothing locks.
- Ring construction and teardown can also fail partway: every step that fails unmaps what was mapped and closes the descriptor before returning.

### Availability

io_uring needs Linux 5.1 or newer (`ring_init` itself; probing and the `READ`/`WRITE` opcodes need 5.6). Whether it works also depends on policy: container runtimes (Docker's default seccomp profile, gVisor) and RHEL 9 style defaults refuse or disable it, and the `kernel.io_uring_disabled` sysctl turns it off. `ring_init` then returns `-ENOSYS`, `-EPERM` or `-EACCES`. Code and tests should treat those as "not available" and fall back or skip, never as failures. `tests/stdlib/io-uring-raw.jai` does that.

## How to change it

- New kernel feature: regenerate (above). If it needs behaviour in `Ring` (a new setup flag that changes the mapping), extend `ring_init_params` and document it here; flags it does not follow should stay in the `-EINVAL` list.
- New helper: add it to `ring.jai`. Keep it thin: this module is the OS binding, and the policy (which ring per thread, which flags, fallbacks) belongs to the engine that will sit on top of it.
- Gotchas: the SQE array is mapped at `IORING_OFF_SQES` separately from the rings; the ring size arithmetic mirrors what the kernel allocates (`sq_off.array + entries * 4` unless `NO_SQARRAY`, `cq_off.cqes + entries * cqe_bytes`), so change both together. `byte_offset` and the `u32` index arithmetic rely on unsigned wrap-around (`tail - head`).

## Configuration

None at build time. The test writes its scratch file under `tests/stdlib/.scratch/io-uring-raw/`.

## Dependencies

libc (`syscall`, `mmap`, `munmap`, `close`, `__errno_location`, declared inside the module so it does not import `POSIX`), [`Atomics`](../specs/stdlib/modules/atomics.md) for the head/tail protocol. Linux x86-64 and arm64.

## Tests

`tests/stdlib/io-uring-raw.jai` runs on every platform and mode of the [stdlib runtime tests](../tools/stdlib-runtime-tests.md). On Linux with a usable io_uring it does a NOP, a batch that fills the submission queue, a write and a read of a file (`IORING_OP_WRITE`, `IORING_OP_READ`, an offset read, a read at the end of the file, a bad descriptor), the probe, a timed wait, and a ring with 128-byte entries and 32-byte completions. Everywhere else, and on a Linux that refuses io_uring, it prints why it skipped and exits 0. `tests/stdlib-targets.txt` lists the module's intentional failure on the other targets ([stdlib target check](../tools/stdlib-target-check.md)).
