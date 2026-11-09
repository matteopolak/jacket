# Archive

`Extensions/Archive` reads and writes ZIP, TAR, and gzip-wrapped TAR. It indexes bounded metadata separately from payloads and provides extraction that rejects links and unsafe paths.

```jai
#import "Extensions/Archive";

items := Archive_Item.[.{ name = "hello.txt", data = cast([] u8) "hello" }];
bytes, error := encode_archive(items, .ZIP);
assert(error.kind == .NONE);
defer free(bytes.data);
archive, failure := read_archive(bytes, .ZIP);
assert(failure.kind == .NONE);
defer free_archive(*archive);
entry, entry_error := read_entry(*archive, 0);
assert(entry_error.kind == .NONE);
defer free_archive(*entry);
```

## How it works

`read_archive` accepts a byte view or `Archive_Source`, whose read-at callback supplies a known-size, seekable source. Short reads are supported. The source must stay unchanged and alive until the index and all borrowed entry views are released. An index owns its names and link targets; entries and internal fields are read-only to callers.

ZIP parsing follows the central directory, checks local names, sizes, CRC fields and data descriptors, and supports ZIP64 directory and entry fields. A central or local header with more than one ZIP64 extra record is `INVALID`. Stored and raw DEFLATE are supported. Names use UTF-8 when bit 11 is set and CP437 otherwise. Overlapping local records are rejected before any payload is expanded. Encryption, other compression methods, and multi-disk archives return `UNSUPPORTED`. ZIP timestamps are not decoded.

TAR supports ustar prefixes, pax `path`, `linkpath`, `size`, and fractional `mtime`, and GNU long names and links. Unknown pax keys are ignored except GNU sparse keys, which are rejected. An empty pax value deletes the keyword: in a global header it forgets the earlier global value, in a local header it also hides the global value for that entry, so the header block field is used. Header-only links, directories, and device entries do not consume a declared payload. Special files can be listed but cannot be extracted. Writers emit ustar and use pax for long paths and large sizes; long link targets and timestamps outside the ustar range are rejected. Global pax values apply to subsequent entries. Empty-value keyword deletion is not modeled; empty numeric overrides are rejected.

`read_entry` verifies ZIP CRC and exact decoded size. Stored byte-view entries borrow input without copying; seekable-source entries allocate only their payload. `copy_entry` sends stored data to an `Archive_Writer` in 64 KiB pieces. DEFLATE currently buffers a bounded compressed and decoded entry because Deflate has no incremental public codec. Gzip TAR buffers the bounded decoded container. This is not a complete incremental streaming codec.

`write_archive` writes to an `Archive_Writer` without seeking and uses ZIP data descriptors. Its inputs are byte views; it buffers one compressed entry and the central directory. `encode_archive` additionally buffers the output and returns caller-owned bytes. Release those bytes with `free(bytes.data)`. `free_archive` releases indexes, entry results, and error entry names. Error messages are static; error entry names are owned copies.

`Archive_Writer` is a pointer plus a `write_proc(data, bytes) -> bool`, so it needs no I/O layer. To write to an `Extensions/Io` file or socket, forward to `Io.write` from your own `write_proc` (example in the `Archive_Writer` docs).

## WebAssembly

Everything on byte buffers works on wasm: reading, `read_entry`, `copy_entry`, `encode_archive` and `write_archive` with your own `Archive_Writer`, for ZIP, TAR and gzip TAR. `extract` compiles but returns `UNSUPPORTED`, as on Windows, because it needs POSIX directory handles. Tests that touch files (`tests/extraction.jai`, `tests/conformance.jai`, `tests/io-writer.jai`) are skipped on wasm; everything else runs under `jacket test --interp -- -os=wasm`.

## Safe extraction

`extract(*archive, dest_dir, limits)` requires an existing destination directory. On macOS and Linux it pins directory descriptors and uses `openat` with `O_NOFOLLOW`; created files use `O_EXCL`. Existing files are never replaced. Existing symlinks cannot redirect traversal. Other operating systems, wasm included, return `UNSUPPORTED` before writing.

All archive symlinks, hardlinks, and special files are rejected. Preflight rejects absolute paths, backslashes and UNC paths, colons and drive letters, NULs, `..` components, duplicate normalized paths, and files used as parent directories. It checks the supplied extraction limits again before creating anything. Failures name the entry when one is known. Extraction may leave earlier files or a partial current file; use a new destination and discard it on failure. It does not recursively unpack nested archives.

## Configuration

Every reader and writer takes `Archive_Limits`. Defaults are 1 GiB of archive input/output, 16 MiB of metadata, 100,000 entries, 256 MiB per entry, 1 GiB total decoded payload, and a compression ratio of 1,000. Limits are nonnegative; the ratio must be positive. Increase limits deliberately for trusted large archives. Gzip decoding also bounds the decoded TAR container by the archive and total limits.

## How to change it

The public API and ownership rules are in `module.jai`; `zip.jai` validates ZIP records, `tar.jai` interprets TAR metadata, `write.jai` encodes both formats, and `extract.jai` handles filesystem containment. Keep arithmetic range checks before slices and allocations. Add malformed-input and counting-allocator tests before changing parser behavior. Incremental DEFLATE belongs in Deflate before Archive can expose it.

Fetch the Go fixtures (written to `target/archive-conformance/`; `tools/portable/build_portable.py data` does this too), then test:

```sh
export PYTHONPATH=tools
python3 tools/archive_conformance/fetch.py
jacket test --path stdlib/Extensions/Archive
python3 tools/archive_conformance/differential.py          # add --bench for the Python comparison
```

The pinned download contains [Go's ZIP/TAR fixture directories](https://github.com/golang/go/tree/go1.23.0/src/archive), covered by Go's BSD license; only fixture files are unpacked, into `target/`. `tests/conformance.jai` records expected errors, entry counts, and aggregate payload checksums per archive. `test-badbase.zip` deliberately expects rejection of its incorrect directory size, which Go tolerates. Sparse TAR and the bzip2-wrapped fixture are explicitly excluded; PNG/text payload resources are not archives. The Python differential test checks content and names in both directions, including forced ZIP64 streaming descriptors and fractional pax times. The benchmark compares in-memory indexing and filesystem extraction of identical archives, with build and input loading outside the timed region.

## Dependencies

Archive uses `Extensions/Deflate` for raw DEFLATE, gzip and CRC-32, `Extensions/Hash_Map` for extraction preflight, `Unicode` for strict UTF-8, and `Sort` for overlap validation. Safe extraction uses POSIX directory-relative filesystem operations. Python's standard library and `tools/toolkit/download.py` supply conformance fetching, differential validation, and benchmark comparison.

Run benchmarks with `jacket bench --path stdlib/Extensions/Archive`. Cases live in `benches/`; setup precedes `reset_timer`, and inputs/results use `black_box`.
Prepare the shared ZIP/TAR payloads with `PYTHONPATH=tools python3 tools/portable/build_portable.py data`. Indexing and extraction are separate cases; extraction includes removing its output after each sample.
