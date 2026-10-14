# Conventions of Io, Tls, Http and Json (`Extensions/`)

## What it is

The four network-facing extensions ([Io](io-engine.md), [Tls](tls.md), [Http](http-client.md) and [Json](json.md)) follow the same few rules, so what you learn from one holds for the others. A new procedure in any of them should follow them too.

## The rules

**Handles by value, stateful objects by pointer.** A handle that only names something the engine owns is passed by value: `Tcp_Stream`, `Tcp_Listener`, `Fiber_Handle`, `Io.Reader` and `Io.Writer`. A value that carries state the call changes is passed by pointer: `*Io_File` (it has a position), `*Tls_Stream`, `*Pipe_Stream`, `*Channel(T)`, `*Wait_Group`, `*Fiber_Mutex`, `*Parser`, `*Client` and `*Client_Response` (its body is used up as it is read). A config you fill in and hand over (`Tls_Config`, `Server_Config`, `Client`) is a plain struct with defaults for its fields.

**Type first for free decoders, receiver first for methods.** A procedure that builds a `T` from text takes the type as its first argument: `from_json(T, text)`, `from_query(T, pairs)`, `from_param(T, text)`. A procedure that acts on an object takes the object first and the type after it: `Io.read_json(reader, T)`, `read_json(*response, T)`. The same split holds for the other readers (`read_text(*response)`, `Io.read_all(reader)`).

**`close(x, now := false)`.** Every closable thing has a `close` that waits for what it must (a draining body, a pending write) and takes `now = true` to close at once without waiting, for code that cannot block: `Io.close`, `Tls.close`, `Http.close(*response, now)`, and `close` on a `Reader`, `Writer` and `Link`. `defer close(x)` is always right. (`close(*Channel(T))` never waits, so it has no `now`.)

**A value plus an error.** A call that can fail returns its result first and an error last: `value, error := from_json(T, text)`, `count, error := Io.read(...)`, `response, error := get(*client, url)`. The error is a small value with a `kind` whose `.NONE` means success (`Io_Error`, `Json_Error_Kind`, `Tls_Error_Kind`, `Client_Error_Kind`); on failure the value is the zero value. `error_text(error)` makes one line for logs. The kinds that name the same event share a name: `TIMED_OUT` (`Io_Error`, `Client_Error_Kind`), `CANCELED`, `CLOSED`.

**Who frees what.** Results the module allocates come from `context.allocator` and are yours: `read_text` and `read_bytes` (`free` the data), `Io.read_all`, `to_json` (`free` the string), and anything `from_json` decodes, released by the one `free_json(*value)` overload set, which also takes a `Json_Value` and a `Json_Error`. A `Client_Error` that owns strings (`owned`) is released by `free_error`. A `Tls_Error` holds its text inline and needs no freeing. Text made for a message is in temporary storage and is valid until it is reset: `Tls.error_text`, `Tls.detail`, `Json`'s `error_text` and `Http_Error`'s. (`error_text(Client_Error)` can return the error's own `detail`, so use it before `free_error`.)

**Milliseconds, and `NO_DEADLINE`.** Every duration is an integer number of milliseconds: `Io.sleep`, `Io.with_deadline`, `Client.timeout`, `client_settings.idle_timeout`, every `Server_Config` timeout. `Io.with_deadline(ms, #code { ... })` is the one deadline primitive; a negative value (`Io.NO_DEADLINE`) adds no limit, so a setting that may be off needs no second code path. `Server_Config` timeouts treat 0 or less as no limit, except `shutdown_timeout` (see the [server configuration](http-server.md#configuration)).

**Diagnostics.** A module exposes at most one `<module>_statistics()` returning a `<Module>_Statistics` struct, for tests and curiosity only: `Io.io_statistics()` and `Http.http_statistics()`. `Json.json_index_stats` is the one older global of this kind and is documented configuration.

## How to change it

Check a new procedure against each rule above, and keep helpers that only the module needs below a `#scope_module` line so they stay out of the public surface. Renames of public names go in the `Breaking changes` table of `CHANGELOG.md`.
