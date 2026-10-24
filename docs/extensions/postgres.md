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
`query.jai` (`Result`, `query_dynamic`, `query_binary`, `execute_script`, `get`), `codecs.jai`
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
  Postgres runs the whole string as one implicit transaction unless it has its own `BEGIN`.
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

**Deadlines.** Any call can sit under `Io.with_deadline`. For now, a deadline or cancel that
fires while a query runs breaks the connection (`TIMED_OUT`/`CANCELED`). Sending `CancelRequest`
and draining, so the connection survives, comes with the pool.

**Notices** (`RAISE NOTICE`, warnings) go to `connection.on_notice`, by default `log`.
`connection.user_data` is there for the handler's state.

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
`channel_binding` (`DISABLE`, `PREFER`, `REQUIRE`), `connect_timeout` (ms, default 10 s),
`max_scram_iterations` (default 1,000,000), `allow_cleartext_password`.

`parse_url` reads `postgres://user:password@host:port/database?...`. Parameters: `host`, `port`,
`user`, `password`, `dbname`, `sslmode` (`allow` maps to `prefer`), `sslnegotiation=direct`,
`sslrootcert` (a PEM file), `channel_binding`, `application_name`, `options` and
`connect_timeout` (seconds). A config from `parse_url` owns its strings: call `free_config`.

Module parameter: `#import "Extensions/Postgres"(TLS = false)` leaves out Tls and mbedTLS for
programs that only use Unix sockets or cleartext TCP.

## Dependencies and tests

Depends on `Io` (fibers, TCP, Unix sockets, deadlines) and `Tls` (mbedTLS: handshake, peer
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
  - deadlines.

  It skips unless `JAI_PG_TEST_PORT` is set. To start a matching server:

  ```sh
  sh stdlib/Extensions/Postgres/tests/start_server.sh /tmp/pgtest 54329
  # prints: export JAI_PG_TEST_PORT=54329 JAI_PG_TEST_SOCKET_DIR=/tmp/pgtest/socket
  ```

  The script needs `initdb`, `pg_ctl` and `psql` (PostgreSQL 15 or later) on `PATH` or in
  `PG_BIN`. It creates the roles `jai_md5`, `jai_scram`, `jai_clear` and `jai_tls_scram`, with
  `pg_hba.conf` rules that force each method.
