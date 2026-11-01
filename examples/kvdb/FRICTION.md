# Friction log

Rough edges hit while writing kvdb. Fixed ones name the commit subject; open ones say so.

## Fixed in the tools

- After a compile error in the tests, `jacket test` printed the diagnostics followed by pages of the Discovery plugin's records (the generated test table and the manifest). Fixed by "fix(jacket): leave the plugin's records out of a failed test build"; the unit test is `a_failed_compile_shows_the_diagnostics_without_the_plugins_records`.
- Two unit-test files that `#load` the same helper file are a `module-already-loaded` error, because all test files are one program. The helpers are a local module (`tests/support`, imported with `#import,dir "support"`) instead. Fixed by "docs(jacket): say that test files share one program": `docs/tools/jacket-test.md` now says so.
- `jacket test` in `examples/kvdb` found nothing, because `examples/jacket.toml` makes `examples/` the project. Fixed by "feat(jacket): test the enclosing folder that has its own tests": a folder with its own `tests/` below the project is the root. `[test] paths = ["kvdb"]` stays so `jacket test` in `examples/` still runs kvdb.
- `expect_equal` took no message, and passing one gave a confusing parameter-type error. Fixed by "feat(testing): let expect_equal take a message"; the cut-point tests use it.
- `Extensions/Io` had no line reader, so `protocol/lines.jai` held a 60-line buffered `read_line`. Fixed by "feat(io): add a buffered line reader" (`Io.make_line_reader`, `Io.read_line`); kvdb sets its line limit and uses it.

## Open

- `File` cannot shorten a file (no truncate), so recovery writes the valid prefix beside the log and renames it over. Safe, but it copies the log. A `file_truncate` needs the interpreter's sandbox (`crates/jaic/src/interp/sandbox`) and `Wasi_Runtime` to learn `ftruncate` as well as the two native back ends; left for its own change.
- `File` cannot force data to the device: `file_flush` hands bytes to the operating system only (its doc says so). kvdb survives a killed process, not a power cut. A `file_sync` is the missing half.
- `File.file_move` replaces the target on Unix and fails on Windows; the portable atomic replace is `File_Utilities.atomic_file_replace_or_just_rename_if_destination_file_does_not_exist`. It works, but the name is a sentence, so the store aliases it (`replace_file`).
- `md5` returns only the 32-character hex string, allocated for each call. The record checksum parses the first eight digits back into a `u32`. A raw 16-byte digest, or a streaming interface, would avoid the round trip. `meow_hash` is x86-64 with AES-NI only, so it could not be the checksum on an Apple Silicon Mac. `md5` is a reference-compatible module, so its exports are not changed without checking the reference.
- `Table.table_remove` returns the value but not the stored key, so an index whose keys are allocated strings keeps the key inside the value (`Item`) to be able to free it. `Hash_Table` is reference-compatible too.

## Notes

- The server uses fibers (`Extensions/Io`) rather than a writer thread and reader threads. One scheduler thread runs one request at a time, so there is nothing to lock and a test can run server and clients in one process. The cost is that a blocking disk write (and `COMPACT`) holds up every client. `Thread` and `Mutex` were not needed.
- A fiber's temporary storage is small (`TEMPORARY_BYTES`, 2 KB). Scans of a thousand rows and a 600 kB value still work through `temp` (it grows), but `write_line` and the reply path use the heap on purpose.
- The first version of the line reader moved its pending bytes to the front on every read, which made a 600 kB request 10 times slower under `--interp`. Found by the large-request test; `Io.read_line` compacts once per read.

## Coverage

| Area | Used | Not used |
|---|---|---|
| Storage | File, File_Utilities, Hash_Table, Sort | |
| Hashing and encoding | md5, Base64 | meow_hash (needs AES-NI), xxHash (game2d uses it) |
| Network | Extensions/Io (fibers, TCP, line reader) | Socket, Thread |
| Tools | Extensions/Args (subcommands, `@global`, `@env`), Extensions/Testing, `jacket test` | |
