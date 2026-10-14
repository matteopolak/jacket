# Io: readers, writers and deadlines (`Extensions/Io`)

Shared rules of Io, Tls, Http and Json (pointers or values, `close(x, now)`, errors, who frees, milliseconds): [conventions](extensions-conventions.md).

## What it is

One way to move bytes whatever they come from. `Io.Reader` and `Io.Writer` are small handles over a file (`Io_File`), a TCP connection (`Tcp_Stream`), an in-memory pipe (`*Pipe_Stream`) or a source of your own, and every source has the same five operations: `read`, `read_full`, `write`, `write_some` and `close`. A source that can say when a read would not wait (a TCP stream) also offers `wait_readable`. `with_deadline` is the one way to bound a wait, for all of them. `Io.read_json` decodes a document straight from a reader. Part of [`Extensions/Io`](io-fibers.md); the sockets, files and engine are in [the Io engine](io-engine.md). Everything that waits runs on a fiber inside `io_run`.

```jai
Io :: #import "Extensions/Io";

file, error := Io.open_file("log.txt", .READ);
defer Io.close(*file);

source := Io.reader_of(*file);

header: [16] u8;
count, error = Io.read_full(source, header);          // exactly 16 bytes, or .END_OF_FILE with what it got
rest, error = Io.read_all(source);                    // everything that is left, from the allocator

user, error2 := Io.read_json(Io.reader_of(conn), User);   // a Tcp_Stream, decoded as the bytes arrive
defer free_json(*user);

Io.with_deadline(2000, #code {
    count, error = Io.read(source, buffer);           // .TIMED_OUT after two seconds without data
});
```

## How it works

**The five operations** exist for each source directly and for `Reader`/`Writer`:

| Operation | Meaning |
| --- | --- |
| `read(src, buffer)` | waits for at least one byte and reads what is there; 0 is the end |
| `read_full(src, buffer)` | until the buffer is full; `.END_OF_FILE` with the bytes read if the source ended first |
| `write(dst, bytes)` / `write(dst, text)` | all of it, waiting as often as it takes; on an error, how many went out |
| `write_some(dst, bytes)` | at least one byte (unless empty); returns how many |
| `close(x, now := false)` | closes it; with `now`, without waiting |
| `wait_readable(src)` | returns when a `read` would not wait: bytes are there, the source ended or failed (the `read` says which). Takes no buffer and consumes nothing; `Tcp_Stream` and `Reader` only, `.UNSUPPORTED` when a reader has no `wait_proc` (`can_wait_readable(reader)` asks) |

`read_all(reader, size_hint := 0)` collects a reader to its end, and `read_file(path)` is that over a file (the size is the hint). `write_file(path, data)` creates or replaces a file.

**Representation: a vtable.** `Reader` is `{ data: *void, read_proc, close_proc, wait_proc }` and `Writer` is `{ data: *void, write_proc, close_proc }`, where the procedures take `data` first (`Read_Proc`, `Write_Proc`, `Close_Proc` in `stream.jai`). The built-in sources are made with `reader_of(x)` / `writer_of(x)`; a TCP connection carries its descriptor in `data` (it fits in a word), a file or pipe is pointed at. A source of your own fills the struct by hand:

```jai
source := Io.Reader.{ data = *mine, read_proc = my_read, close_proc = my_close };
```

and then works with `read_full`, `read_all`, `Io.read_json`, the HTTP connection loop and everything else written against a `Reader`. An enum union would have been smaller for the three built-in sources but closed to users, and every new kind (a TLS stream, a gzip reader) would mean editing Io. The cost of the vtable is one indirect call per read, which is nothing next to the system call behind it. `close_proc` may be null.

`wait_proc` (`Wait_Proc`, `(data) -> Io_Error`) may stay null; it is how a reader says it can wait for readability. `reader_of(Tcp_Stream)` sets it, so a program that holds many connections can wait for each one without a buffer and take one only when something has arrived (the Http server does, see [its memory notes](http-server.md#memory)). It is public because the call is clean on its own: a deadline and `cancel` end it like any wait, it consumes nothing, and a reader that cannot offer it says so instead of pretending. Pipes and files do not offer it (a file is always ready, and a pipe's waiters are not an engine operation).

A reader and a writer over one TCP stream or pipe end share the connection: `close` on either closes it, so close only one of them. A file's reader and writer share its position.

**The receiver rule.** A handle goes by value; anything with a cursor or other state goes by pointer. `Tcp_Stream`, `Tcp_Listener`, `Reader`, `Writer` and `Fiber_Handle` are handles. `Io_File` has a `position`, so `read`, `read_full`, `read_at`, `write`, `write_some`, `write_at`, `file_size`, `close` and `reader_of`/`writer_of` all take `*Io_File`, like `*Pipe_Stream`, `*Channel`, `*Wait_Group` and `*Fiber_Mutex`.

**Deadlines.** `with_deadline(ms, #code { ... })` is the only deadline primitive. It limits every IO call and `sleep` in the block to `ms` from now; a call still waiting then returns `.TIMED_OUT`, and so does a call made after the limit, whatever the source (a pipe too, even with bytes waiting). Deadlines nest and the earlier one wins; the previous deadline is back when the block ends. `0` is a limit that has already passed. A negative value, `NO_DEADLINE`, adds no limit, so a timeout setting that can be off (the Http config's `0` means off, mapped by `within(timeout)` in `Http/link.jai`) needs no second code path. Channel and lock operations are not IO and ignore it. jaic does not accept a trailing block after a call, so the body is a `#code` argument; assign results to variables declared before it, and do not `return` from inside it (that leaves the macro).

**`close(x, now = true)`** closes without going through the engine, for code that cannot block (a `Local_Drop`, or a fiber that must not give up its turn). Fibers still waiting on the stream are not woken, so use the plain call unless the scheduler is ending. It is an option of the one `close`, for `Tcp_Stream`, `Tcp_Listener`, `*Io_File` and `Reader`/`Writer`; a pipe ignores it.

**`connect` and `tcp_listen`.** `connect(string)` takes `"host:port"`, an address literal or `"[::1]:80"` and looks a name up; `connect(Address)` takes a resolved address. The `host, port` form is gone (`resolve(host, port)` then `connect(address)` is the long way). `tcp_listen` has the same two forms.

**JSON from a reader.** `Io.read_json(reader, T, options)` reads to the end and decodes with `from_json`, so the result and the errors are those of the same bytes as text. `Extensions/Json` does not import Io (it builds for WebAssembly, where Io does not), so Json takes a `Json_Source` (`data` and a `fill` callback returning bytes and a failure text) and `json.jai` here makes one from a `Reader`. A reader that fails gives a `Json_Error` of kind `.READ` with the `Io_Error` named; the whole document is buffered once before it is decoded.

## How to change it

- A new kind of source: write its read/write/close procedures with the `Read_Proc`/`Write_Proc`/`Close_Proc` signatures and add `reader_of`/`writer_of` overloads in `stream.jai`. Give the source direct `read`, `read_full`, `write`, `write_some` and `close` as well, so every source stays uniform (see `pipe.jai`: `read_full` and `write` are one line over the generic loop).
- The generic loops (`read_full`, `read_all`, `write`) are in `stream.jai`; sources implement only `read` and `write_some`.
- A pipe checks the deadline before each call (`deadline_passed` in `pipe.jai`) because it has no engine to do it; a new in-memory source should too.
- `with_deadline` and its two helpers (`push_deadline`, `pop_deadline`, module-private) are in `runtime.jai`; the deadline itself is `fiber.deadline`, which the engine, the pipe and the resolver read.

## Configuration

None of its own. Timeouts are arguments (`with_deadline`) or settings of the code using them (`Config.idle_timeout`, `Client.timeout`, ...). Io's module parameters and `JAI_IO_*` variables are in [fibers](io-fibers.md) and [the engine](io-engine.md).

## Dependencies

[`Extensions/Json`](json.md) for `read_json` (only `json.jai`), the Io engine for files and sockets. Tests: `stdlib/Extensions/Io/tests/streams.jai` (each operation over a file, TCP and a pipe, directly and through a reader; `read_all`; a custom source; deadlines on readers; `read_json` from a pipe written in 1-, 3- and 7-byte pieces, a file and a socket, with errors, a failing reader and the size limit) and `timeouts.jai` (deadlines and cancellation over TCP).
