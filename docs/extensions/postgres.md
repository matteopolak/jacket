# Postgres

A PostgreSQL client written from scratch over `Io` fibers: startup with TLS, every password method a server asks for (SCRAM-SHA-256 and its channel-bound `-PLUS` form, MD5, cleartext), the extended and simple query protocols, structured server errors, and text and binary codecs for the common types. It is layer 1 of the [Postgres design](../postgres/design.md); compile-time checked queries, migrations and the ORM build on it. Not on wasm.

```jai
Pg :: #import "Extensions/Postgres";

Io.io_run(() {
    config, error := Pg.parse_url("postgres://app@localhost/app_dev?sslmode=require");
    db:, error = Pg.connect(config);
    if error.kind != .NONE { print("%\n", Pg.error_text(error)); return; }
    defer Pg.close(db);

    result:, error = Pg.query_dynamic(db, "select id, name from users where id > $1", 10);
    defer Pg.free_result(*result);
    for row: 0..result.row_count - 1 {
        id := Pg.get(*result, row, 0, s64);
        name := Pg.get(*result, row, "name", Pg.Nullable(string));
    }
});
```

## How it works

- **Connecting.** `connect` runs on a fiber under one `Io.with_deadline(config.connect_timeout)`. A `host` starting with `/` is a Unix socket directory (no TLS). `connect` tries each host in `Config.hosts` until one logs in and passes `target_session_attrs`; pools use the same path, so they follow a failover.
- **TLS.** `SSLRequest` then handshake, or `ssl_direct` with ALPN `postgresql`. `VERIFY_FULL` checks the host name by libpq's rules (`tls_name.jai`). `ssl_mode = .ALLOW`/`.PREFER` retry the other way as libpq does.
- **Startup.** Asks for `max_protocol_version` (default 3.0; `.LATEST` is 3.2) and accepts a downgrade down to `min_protocol_version`. Sets `client_encoding=UTF8`, `DateStyle=ISO` and `IntervalStyle=postgres`, which the text decoders rely on.
- **Authentication.** SCRAM verifies the server's signature before accepting `AuthenticationOk`. `-PLUS` binds to the server certificate hash. Cleartext passwords go only over TLS or with `allow_cleartext_password`. The PBKDF2 step is cached per process so pool reconnects don't repeat it.
- **Queries.** `query_dynamic` sends Parse/Bind/Describe/Execute/Sync in one round trip with text results; `query_binary` asks for binary. `query_cached` and the typed queries use a named-statement cache. `execute_script` and `run_script` use the simple protocol; `run_script` reports each statement's result and handles COPY through `Script_Hooks`. `copy_in`/`copy_out` run COPY ([copy](../postgres/copy.md)). `describe_statement` returns parameter and column types without running anything.
- **Statement cache** (`statements.jai`). Up to `statement_cache_size` (256) prepared statements per connection, named from a hash of SQL and parameter types, evicted least-recently-used. A hit that fails with `0A000` or `26000` is dropped and resent once. `plan = .CUSTOM` or `Config.statement_cache = .OFF` use the unnamed statement (for PgBouncer before 1.21). `verify_statements` checks column counts and names against what a typed query was compiled against.
- **Parameters.** Integers, bools and strings go as text with OID 0 so the server infers the type; floats, bytea, uuid, dates, times and arrays go as binary with their OID. Arrays are one-dimensional. `Nullable(T)` with `null` sends NULL.
- **Results.** `Result` copies rows into one buffer; `get(*result, row, column, T)` decodes on demand and everything it returns stays valid until `free_result`. NULL into a non-`Nullable` type, or a value that doesn't fit, is a `.DECODE` error, never a silent zero.
- **JSON.** `json`/`jsonb` columns decode into `Json(T)` through the [Json](json.md) generated decoder, `Json_Raw`, or `string`. `result.json_options` (from `Config.json_options`) controls strictness. Decode errors name the column and jsonpath.
- **Enums.** A Jai enum maps to a Postgres enum by label. Labels that aren't plain names are spelled `__in_20progress` (`enum_member_name`, `enum_label` convert). [`generate`](../postgres/schema.md) writes these from the catalog.
- **Pipelining** (`pipeline.jai`). Semantics are libpq's pipeline mode: `pipeline_enter`, queue with `pipeline_query` and friends, `pipeline_sync`, then read with `pipeline_next_result` (`.RESULT`, `.ERROR`, `.ABORTED`, `.SYNC`). After an error the server skips to the next sync point. A pipeline writes before it reads, so read results every few hundred KB of queued statements or both sides can block. Synchronous calls fail with `USAGE` while one is active.
- **LISTEN/NOTIFY** (`notify.jai`, `listener.jai`). `wait_for_notification` takes the connection while it waits, so keep listening connections off the pool. `listen(pool, "a", "b")` takes a pool connection for good, queues notifications from a fiber, and re-LISTENs on a new connection after a failure (backing off 100 ms to 5 s). Notifications sent while disconnected are lost, as in Postgres; re-read your table when `listener.reconnects` changes.
- **Message trace** (`trace.jai`). `set_trace` emits one line per protocol message in libpq's `PQtrace` format; `regress` masks OIDs and pids.
- **Errors.** `Pg_Error.kind` says where it came from (`CONFIG`, `IO`, `TIMED_OUT`, `CANCELED`, `CLOSED`, `TLS`, `AUTH`, `PROTOCOL`, `SERVER`, `DECODE`, `USAGE`, `NOT_FOUND`). It's 40 bytes since hot calls return it by value; the other server fields come from `Pg.error_fields(error)`. `code` is a `Sqlstate` enum (`error.code == .UNIQUE_VIOLATION`). Free with `free_error`. A failure that leaves the stream out of step sets `connection.broken`.
- **Deadlines and cancel.** A call under `Io.with_deadline` or `Io.cancel` returns `TIMED_OUT`/`CANCELED`, and `recover_interrupted` keeps the connection usable: inside `Io.with_grace(config.cancel_timeout)` it opens a second connection, sends `CancelRequest`, and reads the main connection to `ReadyForQuery`. If that is too slow the connection is marked broken. A statement that finished before the cancel arrived has taken effect.
- **Pool** (`pool.jai`). One pool per scheduler (no locks; with `io_run_threads`, create one per thread). `acquire` hands out the most recently used idle connection, opens more up to `max_size`, otherwise waits up to `acquire_timeout`. `release` rolls back an open transaction and closes broken or over-age connections. A background fiber enforces `idle_timeout`, `max_lifetime` and `min_idle`.
- **Transactions.** `transaction(pool_or_connection, [data,] body, isolation, retries, read_only)` runs `body(tx, data) -> Pg_Error` between `BEGIN` and `COMMIT`. A body error, or a statement that failed while the body ignored it, rolls back. `40001`/`40P01` rerun the body up to `retries` times. It takes a procedure rather than `#code` so two transactions in a scope don't collide.

```jai
error := Pg.transaction(pool, user_id, (tx: *Pg.Connection, id: s64) -> Pg.Pg_Error {
    _, error := Pg.query_dynamic(tx, "update users set seen = now() where id = $1", id);
    return error;
}, .SERIALIZABLE, retries = 3);
```

## How to change it

- New type: add its OID to `Oid` and a branch to `decode` in `codecs.jai` for both formats; if it can be a parameter, a branch in `encode_binary`; for an array, the pair in `ARRAY_TYPES`; and a row in `test_values` in `tests/server.jai`.
- New pipeline step: add a `Pipeline_Step`, a `queue_*` function that writes its messages and calls `add_step`, and a case in `consume` saying which message ends it.
- Traced message: add it to `trace_frontend` or `trace_backend` and compare against a `.trace` file.
- New auth method: a case in `answer_authentication` (`startup.jai`). Keep the rule that a method weaker than `channel_binding = .REQUIRE` allows fails before sending anything.
- JSON decoding is the `is_json(T)` and `Json_Raw` branches of `decode`; what a document may contain is Json's business.
- The typed queries use the names `query`, `query_one`, `execute`, so runtime calls are `query_dynamic`, `query_binary` and `execute_script`.
- Gotchas: `cast(u32) at[0] << 24` casts the whole shift; write `(cast(u32) at[0]) << 24`. A procedure that forwards `(string, ..Any)` to one call is treated as a formatter and warns about `$1`; `query_dynamic` copies `args` to a local first.

## Configuration

- `Config` holds `host`/`hosts`, `port`, `user` (default `$USER`), `password`, `database`, `application_name`, `options`, `ssl_*` (mode, roots, client cert and key, CRL, SNI, TLS version range), `channel_binding`, protocol versions, `connect_timeout` (10 s), `cancel_timeout` (2 s), `statement_cache`, `statement_timeout`, `target_session_attrs`, `load_balance_hosts`, `require_auth`, `passfile`, `parameters`, `on_notice`, `json_options`. The source of truth is `config.jai`.
- `parse_url` and `parse_conninfo(text, use_environment := false)` read libpq URIs and `host=db port=5432` strings with libpq's keywords and error texts, including the `PG*` environment variables and `~/.postgresql` files. Free with `free_config`.
- `Pool_Config` (milliseconds; negative turns a limit off): `max_size` 10, `min_idle` 0, `idle_timeout` 10 min, `max_lifetime` 30 min, `health_check_after` 30 s, `acquire_timeout` 30 s.
- Notices (`RAISE NOTICE`) go to `connection.on_notice`, by default `log`. Set `Config.on_notice` to catch notices during login.
- `#import "Extensions/Postgres"(TLS = false)` leaves out Tls and mbedTLS for Unix-socket or cleartext programs.

## Tests

`stdlib/Extensions/Postgres/tests`:

- `offline.jai` needs no server (URLs, SQLSTATEs, UUIDs, numerics).
- `fake_server.jai` uses a scripted server and mutates its replies (bit flips, cuts, duplicated runs); a broken reply must give an error, never a crash or hang.
- `conformance/pipeline_traces.jai` replays libpq's pipeline scenarios and compares traces ([conformance](../postgres/conformance.md)).
- `server.jai` covers auth methods, TLS, Unix sockets, every type in both formats, errors, scripts, cancel, the pool, transactions, the statement cache, pipelines, notifications and protocol versions. It skips unless `JAI_PG_TEST_PORT` is set:

```sh
sh stdlib/Extensions/Postgres/tests/start_server.sh /tmp/pgtest 54329
# prints: export JAI_PG_TEST_PORT=54329 JAI_PG_TEST_SOCKET_DIR=/tmp/pgtest/socket
```

The script needs `initdb`, `pg_ctl` and `psql` (PostgreSQL 15+) on `PATH` or in `PG_BIN`.

## Compared with the community libraries

`benchmarks/compare/postgres/benches/postgres.jai` (`tools/bench/library_compare.py postgres`, see [library comparisons](../tools/library-comparisons.md)) runs against a local PostgreSQL server over loopback, so it measures the client and protocol, not the network. Jacket versus the `jai-postgres` libpq wrapper, time per iteration:

| Workload | Jacket | libpq wrapper |
| --- | --- | --- |
| point select | 33.8 us | 32.9 us |
| point select over TLS | 38.4 us | 39.1 us |
| point select, prepared | 24 us | n/a |
| 1,000 inserts in a transaction | 27.3 ms | 28.2 ms |
| 1,000 inserts, pipelined | 6.19 ms | n/a |
| 1,000 rows by binary COPY | 3.9 ms | n/a |
| fetch 1,000 rows x 24 columns | 1.02 ms | 1.28 ms |

Single statements are level because the time is the server round trip. Jacket wins where the protocol allows it: pipelining is 4x quicker than a loop, COPY 7x, and wide rows decode 1.25x faster. The wrapper has no pool, pipelining, COPY, LISTEN/NOTIFY, cancel or fiber-friendly waiting, and has partial type coverage (no arrays, json, intervals).
