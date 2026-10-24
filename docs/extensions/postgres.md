# Postgres (`Extensions/Postgres`)

## What it is

A PostgreSQL client written from scratch over `Io` fibers: connection startup with TLS, every
password method a server asks for (SCRAM-SHA-256 and its channel-bound `-PLUS` form, MD5,
cleartext), the extended and simple query protocols, structured server errors, and text and binary
codecs for the common types.

This is layer 1 of the [Postgres design](../postgres/design.md). The compile-time checked queries,
migrations and ORM of the later phases are built on it. Not on wasm.

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

**Files.** `config.jai` (`Config`, `parse_url`), `connection.jai` (`connect`, `close`, the
`Connection` struct), `startup.jai` (transport, TLS negotiation, the startup message, the
authentication loop), `scram.jai`, `wire.jai` (message framing and the `Cursor` reader),
`query.jai` (`Result`, `query_dynamic`, `query_binary`, `execute_script`, `run_script`, `get`), `codecs.jai`
(`Oid`, decoding and parameter encoding), `types.jai` (`Nullable`, `Uuid`, the date and time
types, `Numeric`), `error.jai` (`Pg_Error`, `Sqlstate`).

**Connecting.** `connect` must run on a fiber. The whole of it sits under one
`Io.with_deadline(config.connect_timeout)`.

1. A `host` starting with `/` is a socket directory: it connects to `<dir>/.s.PGSQL.<port>`
   with `Io.connect_unix`, and TLS is skipped. Otherwise it uses TCP.
2. TLS: `SSLRequest`, then the handshake if the server says `S`. With `ssl_direct` the
   handshake starts at once, with ALPN `postgresql` (PostgreSQL 17 and later).
3. The startup message asks for protocol 3.2 and accepts the server's downgrade to 3.0. It
   sets `client_encoding=UTF8`, `DateStyle=ISO` and `IntervalStyle=postgres`, which the text
   decoders rely on.
4. Authentication. SCRAM verifies the server's signature before it accepts `AuthenticationOk`,
   so a server that does not know the password cannot fake a login. `-PLUS` hashes the server
   certificate with its signature hash (`Tls.peer_certificate_signature_hash`), with MD5 and
   SHA-1 raised to SHA-256 as RFC 5929 says. A cleartext password is sent only over TLS, or
   with `allow_cleartext_password`.

**Queries.**

- `query_dynamic` sends Parse, Bind, Describe, Execute and Sync together: one round trip,
  unnamed statement and portal, text results.
- `query_binary` is the same with binary results.
- `execute_script` uses the simple protocol: several statements, no parameters, rows dropped.
- `run_script(db, sql, *hooks)` is the same protocol with every statement's outcome reported to
  `Script_Hooks`: `on_result` gets each statement's `Result` (rows, `command` tag, `returns_rows`)
  in order, `copy_in` feeds `COPY ... FROM STDIN` (or fails it with a message), and `copy_out`
  receives `COPY ... TO STDOUT` data. `on_copy` runs when a COPY starts, before any data moves
  (`Copy_Info.incoming` tells the two directions apart). The returned error is the one that
  stopped the script, after the results before it were reported. With `chunk_rows` set,
  `on_result` gets rows in pieces of that many as they arrive (`result.partial`), then the rest
  with the command tag, so a large result never sits in memory whole. Embed the hooks with `#as using` to carry your own state.
  Postgres runs the whole string as one implicit transaction unless it has its own `BEGIN`.
- `describe_statement(db, sql)` prepares the unnamed statement and returns its parameter types
  and result columns (`Statement_Info`, freed with `free_statement_info`) without running it.
  A statement with no result has no columns.
- After an `ErrorResponse` the reader drains to `ReadyForQuery`, so the connection stays usable
  and `connection.status` is current (`IDLE`, `IN_TRANSACTION`, `FAILED`).

**Parameters.** Integers, bools and strings go as text with OID 0, so the server infers their type
from the query (`$1 = 'abc'`, `$2 + 5`). Floats, `[] u8` (bytea), `Uuid`, the date and time
types and arrays go as binary with their OID, so nothing is lost to formatting. `Numeric` goes
as its text with the numeric OID. `Nullable(T)` with `null` set sends SQL NULL.

**Results.** `Result` copies each row's values into one buffer and keeps an offset and length per
value; `-1` marks NULL. `get(*result, row, column, T)` decodes on demand, in the column's
format. Strings and byte slices point into the result. Arrays, numerics from binary and
hex-decoded bytea are allocated by an allocator that records them in the result. Everything
`get` returns therefore stays valid until `free_result`. NULL into a non-`Nullable` type, or a
value that does not fit (`int2` 300 into `s8`), is a `.DECODE` error, never a silent zero.

**Errors.** `Pg_Error.kind` says where the failure came from: `CONFIG`, `IO`, `TIMED_OUT`,
`CANCELED`, `CLOSED`, `TLS`, `AUTH`, `PROTOCOL`, `SERVER`, `DECODE` or `USAGE`. Server errors carry
every `ErrorResponse` field (`code`, `detail`, `hint`, `position`, `table`, `column`,
`constraint`, ...). `code` is a `Sqlstate`: an enum over the five characters, so
`error.code == .UNIQUE_VIOLATION` works, and `sqlstate_class` keeps only the class. An error owns
its text: call `free_error` when done. A failure that leaves the stream out of step (I/O, a
protocol error, a FATAL) sets `connection.broken`, and later calls fail with `CLOSED`.

**Deadlines and cancellation.** Any call can sit under `Io.with_deadline`, and `Io.cancel` of the
fiber works the same way. When either ends the wait for an answer, the call returns `TIMED_OUT` or
`CANCELED`, and `recover_interrupted` (in `query.jai`) keeps the connection usable:

1. Inside `Io.with_grace(config.cancel_timeout)`, so it runs past the expired deadline or cancel,
   it opens a second connection to the same server and sends `CancelRequest` (pid and key from
   `BackendKeyData`), over TLS if the main connection used it, reusing its TLS settings.
2. It reads the main connection up to `ReadyForQuery`, answering a pending `COPY FROM STDIN`
   with `CopyFail`.
3. If that does not finish in time, the connection is marked broken.

A deadline loses no bytes (Io and Tls leave the stream in step), so `fill` does not break the
connection for these two errors. A failed *write* still does: part of a message may have gone.
A statement that finished before the cancel arrived has taken effect. Inside a transaction, the
canceled statement leaves it `FAILED`.

**Pool** (`pool.jai`). `create_pool(config, Pool_Config)` makes a pool for the fibers of one
scheduler: no locks, so with `io_run_threads` create one per thread (it asserts this).

- `acquire` hands out the most recently used idle connection, so the rest go stale and are closed
  when the load drops. It opens a new one while fewer than `max_size` exist, and otherwise waits on
  an `Io.Condition` for up to `acquire_timeout`.
- A connection idle longer than `health_check_after` gets `ping` (an empty `Sync`) first; a dead
  one is dropped, and the next is tried.
- `release` rolls back a connection left in a transaction (under `with_grace`, so a canceled
  borrower still cleans up). It closes one that is broken or past `max_lifetime`.
- A background fiber closes idle connections past `idle_timeout` or `max_lifetime`, and opens
  connections up to `min_idle`.
- `close_pool` closes the idle connections. The pool is freed when the last borrowed one comes
  back and the background fiber has stopped (`Pool.maintaining`).

**Transactions.** `transaction(pool_or_connection, [data,] body, isolation, retries, read_only)`
runs `body(tx, data) -> Pg_Error` between `BEGIN` and `COMMIT`:

- an error from `body` rolls back and is returned;
- so does a statement that failed while `body` ignored it (`25P02`);
- a `40001` or `40P01` from `body` or from `COMMIT` reruns `body`, up to `retries` times.

It takes a procedure, not a `#code` block: a macro would have to declare `tx` in the caller's scope,
and two transactions in one scope would then collide. A procedure can also `return` its error.

```jai
error := Pg.transaction(pool, user_id, (tx: *Pg.Connection, id: s64) -> Pg.Pg_Error {
    _, error := Pg.query_dynamic(tx, "update users set seen = now() where id = $1", id);
    return error;
}, .SERIALIZABLE, retries = 3);
```

**Notices** (`RAISE NOTICE`, warnings) go to `connection.on_notice`, by default `log`.
`connection.user_data` is there for the handler's state. Set `Config.on_notice` and
`Config.user_data` instead to also catch notices sent while logging in.

## How to change it

- **A new type.** Add its OID to `Oid` and a branch to `decode` in `codecs.jai`, for both
  formats. If it can be a parameter, add a branch to `encode_binary`. For an array, add the
  pair to `ARRAY_TYPES`. Add a row to `tests/server.jai` in `test_values`, which runs in both
  formats.
- **A new auth method.** Add a case to `answer_authentication` in `startup.jai`. Keep the rule
  that a method weaker than what `channel_binding = .REQUIRE` allows fails before it sends
  anything.
- **Gotcha: casts bind loosely.** `cast(u32) at[0] << 24` casts the whole shift. Write
  `(cast(u32) at[0]) << 24`, as `wire.jai` does.
- **Gotcha: `(string, ..Any)` procedures.** The compiler's print-format check treats a
  procedure that forwards both to one call as a formatter, and warns about `$1` arguments.
  `query_dynamic` copies `args` to a local first to avoid it.
- The typed queries of phase 3 will take the names `query`, `query_one`, `execute` and so on.
  The runtime calls are therefore named `query_dynamic`, `query_binary` and `execute_script`.

## Configuration

`Config` fields: `host`, `port`, `user` (default `$USER`), `password`, `database` (default the user),
`application_name`, `options`, `ssl_mode` (`DISABLE`, `PREFER` (default), `REQUIRE`,
`VERIFY_CA`, `VERIFY_FULL`), `ssl_direct`, `ssl_root_cert_pem` (default: the system roots),
`channel_binding` (`DISABLE`, `PREFER`, `REQUIRE`), `connect_timeout` (ms, default 10 s), `cancel_timeout` (ms, default 2 s: the time allowed for
cancel and drain after a deadline or cancel),
`max_scram_iterations` (default 1,000,000), `allow_cleartext_password`, `parameters` (extra
startup parameters such as `TimeZone`; a name the client sends itself, `client_encoding`,
`DateStyle` or `IntervalStyle`, replaces its value. Text-format dates and intervals only decode
under `DateStyle=ISO` and `IntervalStyle=postgres`), `send_styles` (default true; false sends
neither style, leaving the server's or role's settings in place, for programs that only show text
values), `on_notice` and `user_data` (see Notices above).

`parse_url` reads `postgres://user:password@host:port/database?...`. Parameters: `host`, `port`,
`user`, `password`, `dbname`, `sslmode` (`allow` maps to `prefer`), `sslnegotiation=direct`,
`sslrootcert` (a PEM file), `channel_binding`, `application_name`, `options` and
`connect_timeout` (seconds). A config from `parse_url` or `copy_config` owns its strings: call `free_config`.

`Pool_Config` fields (milliseconds; a negative value turns the limit off): `max_size` (10),
`min_idle` (0), `idle_timeout` (10 min), `max_lifetime` (30 min, less a jitter of up to a tenth),
`health_check_after` (30 s), `acquire_timeout` (30 s).

Module parameter: `#import "Extensions/Postgres"(TLS = false)` leaves out Tls and mbedTLS for
programs that only use Unix sockets or cleartext TCP.

## Dependencies and tests

Depends on `Io` (fibers, TCP, Unix sockets, deadlines, `with_grace`, `Condition`) and `Tls` (mbedTLS: handshake, peer
certificate). It also uses `internal/crypto` for SHA-256/384/512, HMAC, PBKDF2 and random bytes
([crypto](../stdlib/crypto.md)), plus `md5` and `Base64`.

Tests (`stdlib/Extensions/Postgres/tests`):

- `offline.jai` needs no server: URLs, SQLSTATEs, UUIDs, calendar arithmetic, numerics.
- `server.jai` covers:
  - each auth method and its failures;
  - TLS with SCRAM-PLUS, verify-full against the Tls fixtures' CA, and direct TLS;
  - Unix sockets;
  - every type in both formats, arrays and parameters;
  - server errors and recovery, scripts and transaction status;
  - `run_script` hooks including COPY, startup parameters and `send_styles`, login-time notice
    handlers, and `describe_statement`;
  - deadlines and `Io.cancel`, which leave the connection usable (plain, TLS, direct TLS, in a
    transaction);
  - the pool: limits and waiting, newest first, rollback on release, health checks, lifetime,
    `min_idle`, closing with borrowed connections;
  - transactions: commit, rollback, ignored failures, retries.

  It skips unless `JAI_PG_TEST_PORT` is set. To start a matching server:

  ```sh
  sh stdlib/Extensions/Postgres/tests/start_server.sh /tmp/pgtest 54329
  # prints: export JAI_PG_TEST_PORT=54329 JAI_PG_TEST_SOCKET_DIR=/tmp/pgtest/socket
  ```

  The script needs `initdb`, `pg_ctl` and `psql` (PostgreSQL 15 or later) on `PATH` or in
  `PG_BIN`. It creates the roles `jai_md5`, `jai_scram`, `jai_clear` and `jai_tls_scram`, with
  `pg_hba.conf` rules that force each method.
