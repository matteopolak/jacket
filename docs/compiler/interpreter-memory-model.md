# Interpreter memory model

Compile-time code, `jaic -interp` and the playground see program memory as the target lays it out: the target's pointer width, byte order and record layouts, from its `DataLayout` ([targets](targets.md)). This page says which part of the interpreter owns each of those facts, so a new path that touches program memory knows which helper to use.

## What it is

Program memory is host memory that holds the target's bytes. Three facts can differ from the host:

| Fact | Where it is handled |
|---|---|
| Address width | the [address window](interpreter.md#data-layout) (`interp/window.rs`): a 2- or 4-byte program address is an offset into one host region |
| Byte order | `Interp::in_register_order` swaps every interpreted load and store; `DataLayout::read_*`/`write_*` swap the compiler's own reads and writes |
| Record layout | `DataLayout::pack` and the program's own types; never hard-coded offsets |

A program whose memory differs from the compiler's in any of them (`DataLayout::is_foreign_memory`) never calls native C: its foreign calls go to the sandbox host's emulated library, and plain `#run` stays on the target compiler instead of the [host twin](compile-time-host-execution.md).

## How it works

Every path that moves a value between the host and program memory goes through one of these, by who is moving it:

- **Interpreted code.** `Interp::load`/`store` and the frame accessors in `code/run.rs` take a program address, add the window base and swap. A frame slot of type `Ptr` is accessed at the target's pointer width. Atomics (`interp/atomics.rs`) swap too: exchange, compare-exchange and the bitwise operations work on swapped operands directly; add and subtract on swapped memory run as a compare-exchange loop.
- **The compiler.** `Interp::read_*`/`put_*`, `peek`/`poke` and the freezer read and write with the target `DataLayout`. The freezer asks `Interp::is_integer_address` whether a pointer can name compile-time memory at all: on a narrow target, values below the window's first offset or past its span are integers cast to pointers (`SIG_ERR`, registry roots).
- **The emulated C library.** Stand-ins in `interp/sandbox` reach memory only through `sandbox/guest.rs`. A call runs inside `guest::with_memory`, which sets the window base and the program's `DataLayout` for that call. `read_word`/`write_word` move pointers, `read_size`/`write_size` move `size_t`, and `read_uint`/`write_uint` move a field of any width in the program's byte order.
- **Records the library fills.** `struct stat`, `struct tm` and `struct timespec` are member lists placed with `DataLayout::pack` (`sandbox/records.rs`, `sandbox/time.rs`), so one list serves every CPU whose C library declares the record the same way; the alignment of a 64-bit field on i686 and armv7 comes from the layout. A C `long` is `guest::layout().c.long` bytes, except on wasm, whose private C ABI keeps `long` fields at 8 bytes (`SandboxHost::long_bytes`).
- **Addresses the interpreter hands out.** Anything the program gets a pointer to (a `setlocale` result, an allocation, a mapping) lives in program memory: `SandboxHost::alloc` and the window's page allocator. A host static's address is not a program address on a narrow target.
- **Mappings.** `mmap`, `mprotect` and `munmap` reserve and commit host pages through `interp/host_pages.rs` on every host, or take pages from the window. Unmapping part of a mapping decommits that part; the reservation is released with the last part.

## How to change it

- New libc stand-in: read and write through `guest`; use `write_word` for pointers and `write_size` for `size_t`, never `write_u64` for them.
- New filled record: add its member list to `sandbox/records.rs` and pick it by OS and CPU in the matching `*_members` function; check it against the stdlib's declaration for every CPU (the unit test there compares record sizes).
- New time64 entry point of a 32-bit C library (`__clock_gettime64` and friends): alias it in `sandbox/symbols.rs` to the call it stands for.
- New interpreted memory access (an intrinsic that writes memory): go through `Interp::store` or call `in_register_order` yourself.
- New target: if its record layouts follow an existing list, nothing changes here; otherwise add a list.

## Configuration

None. Everything follows `Options::layout`, chosen by `Target::data_layout`.

## Dependencies

`crate::layout::DataLayout`, `interp/window.rs`, `interp/host_pages.rs`, `interp/sandbox/guest.rs`, `interp/atomics.rs`. Tests: `crates/jaic-cli/tests/n_bit_targets.rs` runs programs for i686, armv7, wasm32, s390x and powerpc64 in the interpreter; `crates/jaic/src/interp/sandbox/tests.rs` checks the records byte by byte.
