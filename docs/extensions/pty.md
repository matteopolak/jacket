# Pty

Nonblocking pseudoterminal sessions on macOS and Linux: start a shell on a PTY, read its output, send input, resize it and reap it, all without blocking a UI frame. `@experimental`. The [terminal example](../tools/dogfood.md) is built on it. On Windows `open_session` returns false (no ConPTY yet).

```jai
Pty :: #import "Extensions/Pty";

session, ok := Pty.open_session("/bin/sh", "printf hello", cols = 80, rows = 24);
if ok {
    defer Pty.close_session(*session);
    buffer: [4096] u8;
    while !Pty.poll_exit(*session) {
        count := Pty.read_session(*session, buffer);   // 0: nothing yet, -1: closed
        if count > 0 write_string(string.{count, buffer.data});
    }
}
```

## How it works

- `open_session(shell, command, cols, rows)` opens a master/slave pair, forks, makes the child a session leader with the slave as its controlling terminal, and runs `shell` (interactive, or `-c command`). The child gets `TERM=xterm-256color`. Call it before starting other threads, since it forks.
- The master descriptor is nonblocking. `read_session` returns bytes read, `0` when nothing is waiting and `-1` at EOF or error. `write_session` writes what fits and returns the count; keep the unwritten rest and retry next frame.
- `resize_session` sets the window size, and the kernel signals the foreground group with `SIGWINCH`.
- `poll_exit` reaps without blocking and stores the native wait status in `status` (not an exit code). `close_session` closes the master, ends a child that's still running and reaps it; it's safe on an empty session.

## How to change it

- Platform calls are behind `#scope_file` at the bottom of `module.jai`. Windows support would be a ConPTY branch with the same five procedures.
- Don't copy a live `Session`; pass a pointer.

Tests: `jacket test --path stdlib/Extensions/Pty` (spawns `/bin/sh`). Depends on `Basic` and the POSIX libc.
