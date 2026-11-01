# Friction log

## Fixed in the standard library

- `Io.open_file` uses positioned regular-file reads, so it cannot read a terminal or FIFO while other fibers handle network traffic. Fixed by "feat(io): open POSIX byte streams for fiber reads": `open_stream_file` opens a separate non-blocking descriptor with the engine's stream handle kind. The test is `stdlib/Extensions/Io/tests/stream-file.jai`: an empty FIFO read times out, another fiber writes, and a later line read succeeds. It also checks that `/dev/fd` duplicates are non-blocking and restore the original flags when closed. Windows explicitly returns `UNSUPPORTED`.

## Open

- `Shared_Memory_Channel` requires the reader to create its channel. `chat stats` creates a short-lived channel and the server polls for a writer connection, so stats start with up to one second of delay. The sandbox on this Mac returns `EPERM` from `shm_open`; smoke detects that exact OS restriction and skips cross-process stats verification.
- `Mail.mail_send` blocks its OS thread. Chat uses a bounded batch on one `Thread_Group` worker with a two-second timeout; failed sends are logged and dropped. Addresses and offline digests are not durable.
- Disk flush is synchronous. Small writes keep command ordering simple, but slow storage stalls the single network scheduler. The log is unbounded, recovery reads it into memory, and there is no process-level file lock or compaction.
- The client UI assumes a 24-row ANSI POSIX terminal and uses canonical stdin editing. Resize support, richer editing and an interactive Windows backend are left out. TLS clients currently verify only the hostname `localhost`.
- The shared build instruction names a Rust `jaifmt` package, but this checkout's formatter is Jai. Build `jaifmt/main.jai` with the newly built compiler instead.
