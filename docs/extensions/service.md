# Compile-time services (`Extensions/Service`)

## What it is

A service is a long-lived Jai program that compile-time code asks questions. It keeps state that would be too slow to rebuild on every `#run`, such as a database connection used to learn a query's column types. The compiler starts the service the first time it is called and keeps it running between calls. In the editor, a slow answer never blocks a keystroke.

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

| `Status` | Meaning | What compile-time code usually does |
| --- | --- | --- |
| `.OK` | `reply` is the service's answer | use it |
| `.PENDING` | editor only: the answer is not in yet | `Service.unknown(...)` |
| `.UNAVAILABLE` | no service here: no `jaic` to run it, a sandbox, or the service said so | `Service.unknown(reply)` |
| `.FAILED` | the service crashed, timed out, ran out of memory or returned an error | report `reply` as an error |

`Service.unknown(message)` ends the compile-time call. In a build it is an error carrying `message`. In the editor it reports nothing and checking continues, the same way code an untrusted workspace may not run is handled.

## How it works

**Calls.** `Service.call` runs the `#compiler` hook `compile_time_service` (`Hook::Service`, `interp/calls.rs`). The hook passes the request to `Host::service` and copies the reply into memory that lives as long as the interpreter. Outside compile-time execution the hook is inactive, and the stub body answers `.UNAVAILABLE`.

- **Native builds.** `NativeHost` uses the process-wide `Services::global()` (`crates/jaic/src/service.rs`) and waits for the answer.
- **The sandbox host** (the language server) answers through an optional `ServiceProvider`.
  - If none is set, every call is `.UNAVAILABLE`. The playground and untrusted workspaces have none, and untrusted workspaces run no compile-time code at all.
  - Relative service paths start at the host's working directory. Pass an absolute path built from `#filepath`.

**Workers.** `Services` runs one manager thread per service file. On its first job, the manager starts `jaic <file> -interp` in the file's folder.

- **Wire format.** Frames are an 8-byte little-endian length followed by the bytes. An answer frame starts with a status byte. `Service.serve` reads frames from stdin and writes answers to stdout.
- **Restart on change.** A fingerprint covers the sizes and modification times of the `.jai` files up to two folders below the service's folder, plus `jaic` itself. The manager checks it at most once a second. When it changes, the worker is restarted and any backoff is forgotten.
- **Limits.** These are the `ServiceConfig` defaults:
  - A request has 30 s.
  - Each worker gets `JAIC_MEMORY_LIMIT=1G`, and exit code 120 is reported as "used more than 1024 MiB".
  - A worker idle for 10 minutes is stopped, and the next call starts it again.
- **After a crash or timeout.** The service is not restarted for 1 s. Each further failure doubles the wait, up to 60 s. A good answer resets it. The failure message includes the last 4 KiB of the worker's stderr, so a service that fails to compile shows its compile error.
- **Finding `jaic`.** It is the current process if that is `jaic`. Otherwise `find_jaic` looks for `jaic` next to the running executable (jailsp ships beside it), then on `PATH`.

**Editor.** `EditorServices` (`crates/jai-language-server/src/services.rs`) is the language server's provider. It is set on the sandbox host only for trusted compiles (`semantic/compile.rs`).

- **Memo.** Answers are remembered by `(service, request)`.
  - A new request waits 50 ms. If no answer arrives by then, the call answers `.PENDING` and the ticket is kept.
  - Failed and unavailable answers are asked again after 10 s.
  - A service's answers are dropped when its sources fingerprint changes.
- **Rechecks.** While any ticket is pending, the message loop in `main.rs` polls every 25 ms (`JsonSession::service_arrivals`). When an answer arrives, every cached compile is dropped and the open files are published again after the usual debounce.

**Unknown results.** `compile_time_unknown` traps with `TrapKind::Unknown`. `trap_diagnostic` turns that into an error of kind `DiagnosticKind::Unknown` (code `unknown-result`) at the `#run`, and `call_thunk` records the span in `IdeFacts::refused`. The editor treats it like `DiagnosticKind::Untrusted`:
- `StmtRecovery::Salvage` keeps checking the rest of the file.
- `known()` in `semantic/compile.rs` drops the error.

## How to change it

- **New status.** Add it to `ServiceStatus` (`service.rs`), `Status` (`module.jai`) and the table above. `read_answer` rejects statuses it does not know, and a service may never send `PENDING`.
- **Limits.** Change the `ServiceConfig` fields. Native builds use `ServiceConfig::default()`, and jailsp builds its own in `native_environment` (`main.rs`). Neither is configurable yet. Reading a `jaic` path and limits from `jacket.toml` is planned.
- **Editor memo or poll timings.** These are the constants at the top of `services.rs` and `SERVICE_POLL` in `main.rs`.
- **Rechecks.** `Cache::forget_compiles` drops every compile. Tracking which compiles saw a pending answer would let the others stay.
- **Gotchas.**
  - The manager serves one request at a time per service, so parallel compile-time calls to the same service queue.
  - The `SharedHost` borrow is released before the provider runs. Keep it that way, because the provider may wait.

## Configuration

| Setting | Where | Default |
| --- | --- | --- |
| `jaic` used for workers | `ServiceConfig::jaic`; else this process, a sibling `jaic`, or `PATH` | found |
| Request timeout | `ServiceConfig::timeout` | 30 s |
| Idle stop | `ServiceConfig::idle` | 10 min |
| Worker memory cap | `ServiceConfig::memory_bytes` (passed as `JAIC_MEMORY_LIMIT`) | 1 GiB |
| Crash backoff | `ServiceConfig::backoff_min` / `backoff_max` | 1 s to 60 s |
| Editor first wait, failure memo, poll | `FIRST_WAIT`, `FAILURE_KEPT` (`services.rs`), `SERVICE_POLL` (`main.rs`) | 50 ms, 10 s, 25 ms |

## Dependencies

- `jaic -interp` runs workers, so a service can use any module a program can, including `#foreign` libraries and sockets.
- `crate::memory_limit` provides the worker memory cap.
- Not available on wasm32: there are no processes there, so calls are `.UNAVAILABLE`.
- Tests:
  - Builds: `crates/jaic-cli/tests/compile_time_services.rs` and `stdlib/Extensions/Service/tests/call.jai`.
  - Editor: `crates/jai-language-server/tests/compile_time_services.rs`. It needs the workspace's `jaic` binary built and is skipped without it.
  - Toy service: `stdlib/Extensions/Service/tests/services/upper.jai`.
