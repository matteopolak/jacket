# Compile-time services

A service is a long-lived Jai program that compile-time code asks questions. It keeps state that would be too slow to rebuild on every `#run`, such as a database connection used to learn a query's column types. The compiler starts it on the first call and keeps it running. In the editor, a slow answer never blocks a keystroke.

```jai
// service.jai: the service side
Service :: #import "Extensions/Service";
main :: () {
    Service.serve((request: string) -> (Service.Status, string) {
        return .OK, to_upper_copy(request);
    });
}
```

```jai
// any file: the compile-time side
Service :: #import "Extensions/Service";
SERVICE :: #run tprint("%service.jai", #filepath);

ANSWER :: #run -> string {
    status, reply := Service.call(SERVICE, "hello");
    if status != .OK  Service.unknown(reply);   // pending, unavailable or failed
    return reply;
};
```

| `Status` | Meaning |
| --- | --- |
| `.OK` | `reply` is the answer: use it |
| `.PENDING` | editor only, answer not in yet: `Service.unknown(...)` |
| `.UNAVAILABLE` | no service here (no `jaic`, a sandbox, or the service said so): `Service.unknown(reply)` |
| `.FAILED` | crashed, timed out, out of memory or errored: report `reply` as an error |

`Service.unknown(message)` ends the compile-time call. In a build it is an error carrying `message`; in the editor it reports nothing and checking continues.

## How it works

- `Service.call` runs the `#compiler` hook `compile_time_service` (`interp/calls.rs`), which passes the request to `Host::service` and copies the reply into interpreter memory. Outside compile-time execution the stub answers `.UNAVAILABLE`.
- Native builds use the process-wide `Services::global()` (`crates/jaic/src/service.rs`). The language server's sandbox host answers through an optional `ServiceProvider`; without one (the playground, untrusted workspaces) every call is `.UNAVAILABLE`. Pass an absolute service path built from `#filepath`.
- `Services` runs one manager thread per service file; on its first job it starts `jaic <file> -interp` in the file's folder.
- Wire format: frames of an 8-byte little-endian length plus bytes; an answer starts with a status byte. `Service.serve` moves answers to a copy of stdout and points stdout at stderr, so handler output can't corrupt a frame and lands in the stderr tail kept for crash messages.
- A fingerprint of the `.jai` files up to two folders below the service (plus `jaic`) is checked at most once a second; a change restarts the worker.
- Defaults: 30 s per request, `JAIC_MEMORY_LIMIT=1G` per worker (exit 120 reports as "used more than 1024 MiB"), idle workers stop after 10 minutes. After a crash or timeout the service isn't restarted for 1 s, doubling to 60 s; a good answer resets it. Failure messages include the last 4 KiB of stderr.
- `jaic` is the current process if it is `jaic`, else one next to the executable, else on `PATH`.
- **Editor.** `EditorServices` (`crates/jai-language-server/src/services.rs`) answers only for trusted compiles. Answers are memoised by `(service, request)`; a new request waits 50 ms and otherwise answers `.PENDING`. Failed and unavailable answers are asked again after 10 s. While any ticket is pending the message loop polls every 25 ms; when an answer arrives all cached compiles are dropped and open files are re-published.
- `unknown` traps with `TrapKind::Unknown`, which becomes the `unknown-result` diagnostic at the `#run`. The editor treats it like `Untrusted`: the rest of the file is still checked and `known()` drops the error.

## How to change it

- New status: add it to `ServiceStatus` (`service.rs`), `Status` (`module.jai`) and the table above. `read_answer` rejects unknown statuses and a service may never send `PENDING`.
- Limits live in `ServiceConfig`; native builds use the default and jailsp builds its own in `native_environment`. Neither reads `jacket.toml` yet.
- Editor timings: constants at the top of `services.rs` and `SERVICE_POLL` in `main.rs`.
- Gotchas: the manager serves one request at a time per service, so parallel calls queue. The `SharedHost` borrow is released before the provider runs; keep it that way, since the provider may wait. `Cache::forget_compiles` drops every compile, which is coarse.
- Not available on wasm32 (no processes).
- Tests: `crates/jaic-cli/tests/compile_time_services.rs`, `stdlib/Extensions/Service/tests/call.jai`, and `crates/jai-language-server/tests/compile_time_services.rs` (skipped without a built `jaic`).
