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
types, `Numeric`), `error.jai` (`Pg_Error`, `Sqlstate`), `pipeline.jai` (pipelining),
`notify.jai` (LISTEN/NOTIFY) and `trace.jai` (the message trace).

**Connecting.** `connect` must run on a fiber. The whole of it sits under one
`Io.with_deadline(config.connect_timeout)`.

1. A `host` starting with `/` is a socket directory: it connects to `<dir>/.s.PGSQL.<port>`
   with `Io.connect_unix`, and TLS is skipped. Otherwise it uses TCP.
2. TLS: `SSLRequest`, then the handshake if the server says `S`. With `ssl_direct` the
   handshake starts at once, with ALPN `postgresql` (PostgreSQL 17 and later). The chain is
   checked whenever roots are known (`VERIFY_CA` and `VERIFY_FULL` always; `REQUIRE` and
   `PREFER` when `ssl_root_cert_pem` is set), without the host name; `VERIFY_FULL` then
   checks the name by libpq's rules (`tls_name.jai`): a host name against the certificate's
   dNSName entries (`*.` wildcards for one label), an IP address against its iPAddress entries
   (any spelling of the same address) and, as text, its dNSName entries; the common name only
   when the certificate has no entry of the kind the host could match. The name is the
   `host`, never the `hostaddr` connected to. The host name is sent as SNI (not an address;
   `sslsni=0` turns it off). A client certificate is sent when `ssl_cert_pem` and `ssl_key` are
   set; a server alert after the handshake (TLS 1.3 shows a refused client certificate only on
   the next read) is reported as `SSL error: tlsv1 alert unknown ca`, as libpq words it.
3. The startup message asks for `max_protocol_version` (default 3.0, as libpq's; `.LATEST` is
   3.2) and accepts a `NegotiateProtocolVersion` downgrade unless it goes below
   `min_protocol_version`. It sets `client_encoding=UTF8`, `DateStyle=ISO` and `IntervalStyle=postgres`, which the text
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
- `query_cached` runs a statement as a named prepared statement kept in the connection's cache,
  and typed queries (`Pg.query`, `execute`, ...) always do (see Statement cache below).
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
as its text with the numeric OID (inside an array, in binary form). `Nullable(T)` with `null` set sends SQL NULL.
Arrays are one-dimensional: a multi-dimensional array is a `.DECODE` error, and a lower bound
other than 1 is dropped. `bool`, `bytea`, `"char"`, `name`, `oid`, the integer and float types,
`text`, `bpchar`, `varchar`, `uuid`, `date`, `time`, `timestamp`, `timestamptz`, `interval`,
`numeric`, `json` and `jsonb` have array forms. On servers from 17, an `interval` of `infinity`
or `-infinity` reads as every field at its limit. The
[binary round trip](../postgres/conformance.md) checks all of this against the server.

**Results.** `Result` copies each row's values into one buffer and keeps an offset and length per
value; `-1` marks NULL. `get(*result, row, column, T)` decodes on demand, in the column's
format. Strings and byte slices point into the result. Arrays, numerics from binary and
hex-decoded bytea are allocated by an allocator that records them in the result. Everything
`get` returns therefore stays valid until `free_result`. NULL into a non-`Nullable` type, or a
value that does not fit (`int2` 300 into `s8`), is a `.DECODE` error, never a silent zero.

**JSON.** A `json` or `jsonb` column decodes into `Json(T)` with the [Json](json.md) module's
generated decoder for `T`, or into `Json_Raw` (the document's text, undecoded) or a plain
`string`. Binary `jsonb` must carry version byte 1. Arrays of either (`json[]`, `jsonb[]`) decode
into `[] Json(T)`.

```jai
settings := Pg.get(*result, row, "settings", Pg.Json(Settings));    // settings.value.theme
maybe := Pg.get(*result, row, "extra", Pg.Nullable(Pg.Json(Extra))); // SQL NULL
```

- `result.json_options` (a `Json_Options`, copied from `Config.json_options`) controls unknown
  fields, missing fields, duplicate keys and limits. Change it before `get` for one query.
- Strings in the document that need no unescaping point into the result, like `string` columns.
- SQL NULL is the column's `Nullable`. JSON `null` and a missing key are told apart by the Json
  module's `Json_Maybe(T)` (`.NULL`, `.MISSING`, `.SET`).
- A decode error names the column and the place in the document in jsonpath form:
  `column "settings" (jsonb) of row 3: at $.tags[2]: ...`.
- As a parameter, `Pg.json(value)` (a `Json(T)`) and `Json_Raw` go as text with OID 0, so the
  server reads them as `json` or `jsonb` as the statement needs. In arrays they go as binary
  `jsonb[]`. `Json(T)` carries its encoder in its `write` field, because parameters arrive as
  `Any`; a zeroed `Json(T)` has none and is refused.

**Enums.** A Jai `enum` maps to a Postgres enum by label: as a parameter it goes as its label in text
(OID 0, so the server reads the enum the statement wants), an array of them as an array literal, and
a column decodes by matching the label to a member name, in either format. A label that is not a
plain name (`in progress`, `if`) is spelled `__in_20progress` and `__if`: `enum_member_name` and
`enum_label` convert. An integer column is not read into an enum, and an enum is never sent
as its number. [`generate`](../postgres/schema.md) writes these enums from the catalog.
`Json(Json_Raw)` is a json column kept as text (the document is `.value.text`), for columns
without a mapped type.

**Protocol versions.** `Config.max_protocol_version` and `min_protocol_version` (`.V3_0`,
`.V3_2`, `.LATEST`) pick the range. PostgreSQL 17 and older answer a 3.2 request with
`NegotiateProtocolVersion` naming 3.0; the connection then speaks 3.0, or fails with a `PROTOCOL`
error if `min_protocol_version` is higher. `connection.protocol_minor` says what is in use. The
difference that matters is the cancel key: four bytes in 3.0, 4 to 256 in 3.2 (PostgreSQL 18
sends 32). The key is kept as bytes (`connection.cancel_key`) and sent whole in `CancelRequest`.
Connection strings set them with `min_protocol_version` and `max_protocol_version` (`3.0`, `3.2`
or `latest`), parsed in `config.jai`.

**Pipelining** (`pipeline.jai`). A pipeline queues several statements and reads their results
later, in order. The semantics are libpq's pipeline mode.

```jai
p, error := Pg.pipeline_enter(db);
Pg.pipeline_query(p, "insert into log(line) values ($1)", "one");
Pg.pipeline_query(p, "select count(*) from log");
Pg.pipeline_sync(p);                        // queue a sync point and send everything
while true {
    item, error := Pg.pipeline_next_result(p);
    defer Pg.free_pipeline_result(*item);
    if item.kind == .SYNC break;            // also .NONE, if nothing is queued
    // .RESULT: item.result; .ERROR: item.error; .ABORTED: skipped
}
Pg.pipeline_exit(p);
```

- `pipeline_enter(db)` returns the connection's `*Pipeline` (entering twice returns the same one).
  While it is active, `query_dynamic`, `execute_script` and the other one-request calls fail with
  a `USAGE` error ("synchronous command execution functions are not allowed in pipeline mode").
- Queueing: `pipeline_query(p, sql, args..)` and `pipeline_query_binary` (typed arguments, as
  `query_dynamic`), `pipeline_query_text(p, sql, types, values, format)` (arguments already text,
  with the parameter OIDs to declare), `pipeline_prepare`, `pipeline_execute_prepared` and
  `pipeline_execute_prepared_text`, `pipeline_describe_statement` and `pipeline_describe_portal`,
  `pipeline_close_statement` and `pipeline_close_portal`. A query is Parse, Bind, Describe portal
  and Execute; a prepared one leaves out Parse. Queueing only writes into the output buffer.
- Sending: `pipeline_sync` queues a sync point and sends. `pipeline_queue_sync` queues without
  sending. `pipeline_flush_request` queues a `Flush` message (the server then sends its answers
  without ending the batch) and sends. `pipeline_flush` only sends. Reading a result sends
  whatever is still unsent first.
- Results: `pipeline_next_result(p)` gives the oldest statement's outcome: `.RESULT` (rows and
  command tag in `item.result`; for describes the columns, and `parameter_types` for a statement),
  `.ERROR` (`item.error`, a `SERVER` error), `.ABORTED` or `.SYNC`. The `Pg_Error` it returns is
  for the connection. After an error the server skips the rest of the batch up to the next sync
  point; those statements come back as `.ABORTED` without a round trip, and the `.SYNC` result
  ends the abort (`p.aborted`). Without a sync point an error is not recovered from: send one.
  `connection.status` follows the server's `ReadyForQuery` at each `.SYNC`.
- `pipeline_chunked_rows(p, n)` (and `pipeline_single_row(p)`, n = 1) makes the statement being
  read arrive in `.ROWS` results of `n` rows (`result.partial`), then a `.RESULT` with the
  command tag and no rows. It must be called before the statement's first result.
- `pipeline_exit(p)` fails while results are unread or statements unsent. A pipeline never has
  to end with a sync point (libpq's `pipeline_idle` case: flush request, read, exit).
- Deadlines: `pipeline_next_result` under `Io.with_deadline` returns `TIMED_OUT` and keeps what
  it read, so the call can be repeated. `pipeline_abandon(p)` gives up: it cancels the running
  statement, drains the answers to the last sync point and leaves pipeline mode (the connection
  breaks if that takes longer than `cancel_timeout`).
- Outside a pipeline: `describe_prepared`, `describe_portal`, `close_prepared` and
  `close_portal` run one Describe or Close and a Sync.
- A pipeline writes before it reads. If so many statements queue that the server's answers fill
  the socket buffers before any is read, both sides block: read results every few hundred
  kilobytes of queued statements. A pool drops a connection released with unread results.

**LISTEN/NOTIFY** (`notify.jai`). The server sends `NotificationResponse` whenever the session is
outside a transaction, and the client reads it wherever it reads the connection: in a query, a
script, a pipeline or a wait. With `connection.on_notification` set, each notification goes to
that callback (freed after the call). Otherwise they queue on the connection.

```jai
Pg.listen(db, "jobs");                          // channel names are quoted
n, error := Pg.wait_for_notification(db);       // blocks this fiber; deadline and cancel work
defer Pg.free_notification(*n);                 // n.channel, n.payload, n.backend_pid
n, found := Pg.next_notification(db);           // the queue, without waiting
Pg.notify(other, "jobs", "payload");            // pg_notify($1, $2)
```

`wait_for_notification` takes the connection (`busy`) while it waits, so nothing else can use it:
keep a listening connection off the pool. After `TIMED_OUT` or `CANCELED` the connection is as it
was. A FATAL from the server while waiting (shutdown, idle session timeout) comes back as the
error and breaks the connection. `unlisten`, `unlisten_all` and `drop_notifications` complete
the set. Reconnecting does not re-issue `LISTEN`, and notifications sent while disconnected are
lost, as in Postgres.

**Message trace** (`trace.jai`). `set_trace(db, proc, data, regress = true)` calls `proc` with one
line per protocol message sent (`F`) or received (`B`) from then on, in the text format of libpq's
`PQtrace` (tab-separated: direction, length, message name, fields). With `regress` the output is
that of `PQTRACE_SUPPRESS_TIMESTAMPS | PQTRACE_REGRESS_MODE`: OIDs and process ids print as
`NNNN`, an error's length as `NN` and its file, line and routine as `SSSS`. Pass a null `proc`
to stop. Tracing costs nothing when off (one pointer test per message).

**Errors.** `Pg_Error.kind` says where the failure came from: `CONFIG`, `IO`, `TIMED_OUT`,
`CANCELED`, `CLOSED`, `TLS`, `AUTH`, `PROTOCOL`, `SERVER`, `DECODE`, `USAGE` or `NOT_FOUND` (a typed `query_one` with no row). Server errors carry
every `ErrorResponse` field (`code`, `detail`, `hint`, `position`, `table`, `column`,
`constraint`, ...). `code` is a `Sqlstate`: an enum over the five characters, so
`error.code == .UNIQUE_VIOLATION` works, and `sqlstate_class` keeps only the class. An error owns
its text: call `free_error` when done. A failure that leaves the stream out of step (I/O, a
protocol error, a FATAL) sets `connection.broken`, and later calls fail with `CLOSED`. When the
server sends a FATAL error and then closes (a crash, `pg_terminate_backend`), the call returns that
error rather than the close.

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

- **JSON.** Decoding is the `is_json(T)` and `Json_Raw` branches of `decode`, with
  `json_document` stripping the jsonb version byte. Parameters go through
  `json_parameter_text`. What a document may contain is the Json module's business: change
  `Json_Options` there.
- **A new type.** Add its OID to `Oid` and a branch to `decode` in `codecs.jai`, for both
  formats. If it can be a parameter, add a branch to `encode_binary`. For an array, add the
  pair to `ARRAY_TYPES`. Add a row to `tests/server.jai` in `test_values`, which runs in both
  formats.
- **A pipeline step.** `Pipeline_Step` lists what a queued request expects back. A new request
  adds a step, a `queue_*` function that writes its messages and calls `add_step`, and a case in
  `consume` (`pipeline.jai`) that says which message ends it. The typed queries of a later phase
  queue Parse together with Bind and Execute through the same `queue_query`.
- **A traced message.** Add its name and fields to `trace_frontend` or `trace_backend`
  (`trace.jai`); the field helpers print as libpq does. Check against a `.trace` file.
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

**Statement cache** (`statements.jai`). Each connection keeps up to `statement_cache_size`
(default 256) prepared statements, named `j` and 16 hex digits (a hash of the SQL and the
parameter types). The first run of a query sends Parse, Describe, Bind, Execute and Sync in one
round trip, and remembers the result columns; later runs send Bind, Execute and Sync only. When
the cache is full the least recently used statement is closed in the same batch as the next
Parse.

- A hit that fails with `0A000` ("cached plan must not change result type", a table changed
  under `select *`) or `26000` (the statement is gone: `DEALLOCATE ALL`, a pooler's reset) is
  dropped and sent again once. Inside a transaction the error aborts it, so it is returned as
  it is, and the statement is prepared afresh on its next run.
- `plan = .CUSTOM` on a typed query or `query_cached` uses the unnamed statement instead, planned
  with the actual values each time. `Config.statement_cache = .OFF` does that for every query,
  for poolers that do not track named statements (PgBouncer before 1.21).
- `Config.verify_statements` makes the first Parse per connection a round trip of its own, and
  compares the statement's columns with the ones the typed query was compiled against (count and
  names). A mismatch is a `.USAGE` error "statement X was compiled against a different schema",
  before anything runs. Off by default.
- `query_dynamic`, `query_binary`, scripts and pipelines do not use the cache. A statement
  interrupted by a deadline stays in the cache marked as possibly present, and the next run closes
  it before preparing it again.

`Config.statement_timeout` (ms) is sent as the `statement_timeout` startup parameter. A connection
string can set it through `options=-c statement_timeout=5000` as well.

## Configuration

`Config` fields: `host`, `port`, `user` (default `$USER`), `password`, `database` (default the user),
`application_name`, `options`, `ssl_mode` (`DISABLE`, `PREFER` (default), `REQUIRE`,
`VERIFY_CA`, `VERIFY_FULL`), `ssl_direct`, `ssl_root_cert_pem` (PEM; empty means the system
roots for `VERIFY_*` and no check otherwise; the chain must end in a self-signed root from it),
`ssl_crl_pem` (revocation lists: the server's certificate needs one from its issuer, and must not
be on it), `ssl_cert_pem` and `ssl_key` (the client certificate chain and its key, PEM or DER)
with `ssl_key_password`, `ssl_cert_mode` (`DISABLE`, `ALLOW` (default), `REQUIRE`), `ssl_sni`
(default true), `ssl_min_protocol_version` and `ssl_max_protocol_version` (`TLS_1_0` to `TLS_1_3`;
TLS 1.2 and 1.3 are what is spoken, a minimum below 1.2 is raised to it),
`channel_binding` (`DISABLE`, `PREFER`, `REQUIRE`), `min_protocol_version` and
`max_protocol_version` (`.V3_0` (default), `.V3_2`, `.LATEST`), `connect_timeout` (ms, default 10 s), `cancel_timeout` (ms, default 2 s: the time allowed for
cancel and drain after a deadline or cancel),
`statement_cache` (`ON`, `OFF`), `statement_cache_size`, `verify_statements` and
`statement_timeout` (see Statement cache above),
`max_scram_iterations` (default 1,000,000), `allow_cleartext_password`, `parameters` (extra
startup parameters such as `TimeZone`; a name the client sends itself, `client_encoding`,
`DateStyle` or `IntervalStyle`, replaces its value. Text-format dates and intervals only decode
under `DateStyle=ISO` and `IntervalStyle=postgres`), `send_styles` (default true; false sends
neither style, leaving the server's or role's settings in place, for programs that only show text
values), `on_notice` and `user_data` (see Notices above), `json_options` (see JSON above).

`parse_conninfo(text, use_environment := false)` reads libpq's connection strings: a URI as below,
or `host=db port=5432 user='a b'` (single quotes, `\` escapes). Errors have libpq's texts. Several
hosts (`host1:5432,host2:5433`) fill `Config.hosts`; `connect` uses `host` and `port`, the first,
and does not try the others. Further keywords: `hostaddr` (connected to in place of the host, which
TLS still checks), `passfile`, `require_auth`, `client_encoding`, `replication`,
`min_protocol_version` and `max_protocol_version` (`3.0`, `3.2`, `latest`), `sslrootcert=system`
(system roots, and `sslmode` defaults to `verify-full`; `$SSL_CERT_FILE` and `$SSL_CERT_DIR` replace
the system's, as for OpenSSL; any other `sslmode` than `verify-full` is an error). The files are read
when the string is parsed, as libpq does at connect time: `sslrootcert` (a missing file is fine
unless `sslmode` is `verify-ca` or `verify-full`: `root certificate file "x" does not exist`;
default `~/.postgresql/root.crt`, used when it exists), `sslcert` and `sslkey` (defaults
`~/.postgresql/postgresql.crt` and `.key`; a missing certificate means none is sent, a certificate
without a key is `certificate present, but not private key file "x"`, and a key that group or others
can access is refused as libpq does), `sslpassword`, `sslcertmode` (`disable`, `allow`, `require`;
`require` fails after login unless a certificate was configured to be sent; unlike libpq it does not ask whether the server requested one), `sslcrl` (PEM file; a missing one is
ignored; default `~/.postgresql/root.crl`) and `sslcrldir` (every PEM file in the directory),
`sslsni`, `ssl_min_protocol_version` and `ssl_max_protocol_version` (`TLSv1` to `TLSv1.3`).
With `use_environment` the keywords not
given come from `PGHOST`, `PGPORT`, `PGUSER`, `PGPASSWORD`, `PGDATABASE`, `PGPASSFILE`,
`PGREQUIREAUTH`, `PGSSLMODE` and the other variables libpq reads. Lower level:
`parse_conninfo_options` (the options as set, with `conninfo_get`/`conninfo_set`, `is_conninfo`)
and `config_from_conninfo`. Free with `free_config` / `free_conninfo`.

`Config.require_auth` is libpq's: `password`, `md5`, `gss`, `sspi`, `scram-sha-256`, `none`, or the
same with `!` to forbid; a login by another method fails with `authentication method requirement
"..." failed: ...`. `Config.passfile` (default `$PGPASSFILE`, then `~/.pgpass`) supplies a password
when `Config.password` is empty; a file readable by others is ignored. Passwords are prepared with
SASLprep before SCRAM (`saslprep(text)`; normalization works per code point, so decomposed accents
are not recomposed, and unassigned code points are not rejected). `scram_secret(password,
iterations)` and `md5_secret(password, user)` build the secret for `ALTER ROLE ... PASSWORD`.
`cancel_query(connection)` sends a cancel request for the statement another fiber is waiting on.

`parse_url` is `parse_conninfo` for URIs: `postgres://user:password@host:port/database?...`. Parameters: `host`, `port`,
`user`, `password`, `dbname`, `sslmode` (`allow` maps to `prefer`), `sslnegotiation=direct`,
`sslrootcert` (a PEM file, or `system`) and the other `ssl*` keywords above, `channel_binding`, `application_name`, `options` and
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

- `conformance/pipeline_traces.jai` replays the scenarios of PostgreSQL's `libpq_pipeline` test and
  compares the client's message trace with libpq's `.trace` files, line for line
  ([Conformance](../postgres/conformance.md)).
- `offline.jai` needs no server: URLs, SQLSTATEs, UUIDs, calendar arithmetic, numerics.
- `fake_server.jai` needs no server either: a scripted server on a local socket answers one query per connection with a valid reply (every column type decoded and checked), the same with an unsolicited notice, notification and parameter change, the reply cut at every byte, and 400 seeded mutations (flipped bits, edge lengths, cuts, duplicated runs). A broken reply must give an error, never a crash or a hang. Raise `MUTATIONS` and change the seed to search harder.
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
  - transactions: commit, rollback, ignored failures, retries;
  - the statement cache (one Parse for repeated runs, eviction, off, `.CUSTOM`, a deallocated or
    reshaped statement) and `statement_timeout`;
  - pipelines (typed statements, errors and aborts, prepared statements, a deadline in the
    middle, `pipeline_abandon`), notifications (waiting, callback, queue, a deadline, `Io.cancel`,
    another fiber sending) and protocol versions (negotiating down to 3.0, `min_protocol_version`
    failing, and 3.2 with its longer cancel key when the server is PostgreSQL 18).

  It skips unless `JAI_PG_TEST_PORT` is set. To start a matching server:

  ```sh
  sh stdlib/Extensions/Postgres/tests/start_server.sh /tmp/pgtest 54329
  # prints: export JAI_PG_TEST_PORT=54329 JAI_PG_TEST_SOCKET_DIR=/tmp/pgtest/socket
  ```

  The script needs `initdb`, `pg_ctl` and `psql` (PostgreSQL 15 or later) on `PATH` or in
  `PG_BIN`. It creates the roles `jai_md5`, `jai_scram`, `jai_clear` and `jai_tls_scram`, with
  `pg_hba.conf` rules that force each method.
