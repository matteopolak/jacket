# Postgres client and compile-time ORM: design

Status: design only, nothing implemented. Branch `claude/pg-orm-design-e49892`.

## What it is

A plan for `Extensions/Postgres`, a jaic stdlib extension in four layers. Each layer works without the ones above it:

1. **Client.** A from-scratch implementation of the PostgreSQL frontend/backend protocol on [Io](../extensions/io-engine.md) fibers and [Tls](../extensions/tls.md). It covers authentication, the simple and extended query protocols, pipelining, COPY, LISTEN/NOTIFY, cancellation, binary codecs and a pool.
2. **Typed raw SQL.** `Pg.query(db, "... where id = %", id)` is checked at compile time. Postgres describes the statement and the macro emits the `$n` binding code and a row struct.
3. **Schema and migrations.** SQL migration files are the source of truth. Jai table structs are *generated* from the migrated schema and committed, so they can never disagree with what queries return.
4. **ORM.** A select builder whose filters are Jai expressions (`Code`) compiled to SQL. It fetches only the columns the loop body reads and uses joins instead of N+1 lookups. It lowers to layer 2, so it is typed and cached by the same machinery.

The recommendation is in [Recommendation and plan](#recommendation-and-plan). The questions for the maintainer are in [Open questions](#open-questions).

## How it works

### Constraints that shape the design

These facts about jaic decide most of the trade-offs below:

- **Compile-time code has full host access.** `#run` and macros can open sockets, run processes and call foreign code ([compile-time execution](../metaprogramming/compile-time-execution.md)). Io fibers and Tls already run under the interpreter, since their tests run with `-interp`. So the runtime client can also be the compile-time client, with no second implementation.
- **The language server can never reach a database.** Even trusted, `jailsp` runs compile-time code in `SandboxHost`, which has no network or process access and a block budget ([workspace trust](../tools/workspace-trust.md)). Editor support therefore needs everything on disk: a committed query cache and committed generated structs. A sqlx-style "connect at compile time" design alone would leave every query unknown in the editor.
- **No wasm engine exists.** `Extensions/Wasi_Runtime` is the guest side: the libc a `jaic -os wasm` program links. It cannot load or run someone else's `.wasm`. PGlite would need a wasm runtime inside the compiler first, through upstream PGlite or a third-party WASI build (see [How types are learned](#how-types-are-learned-at-compile-time)).
- **Available building blocks:**
  - SHA-1 and SHA-256 in `stdlib/internal/crypto/sha.jai`, and MD5 in `stdlib/md5`.
  - mbedTLS, which is vendored and has HMAC and PBKDF2, but is linked only by Tls and absent on wasm.
  - The Io resolver and TCP, deadlines, `cancel`, `Channel`, `Fiber_Mutex`, and a per-scheduler pool pattern in the HTTP client.

  Missing: HMAC-SHA-256, PBKDF2, an OS CSPRNG wrapper, and **Unix-domain sockets** in Io. Local Postgres listens on `/tmp/.s.PGSQL.5432` by default.
- **Metaprogramming that is available:**
  - Polymorphic structs whose bodies `#insert #run` generated members.
  - `#expand` macros that take `Code`, plus `for_expansion` (the loop body arrives as `Code`).
  - `#caller_code` and `compiler_get_nodes` with typed nodes; Check's `@PrintLike` already reads argument types this way.
  - `#modify`.

### Layer 1: the wire client

#### Connection startup

The client handles three request codes and one of two protocol versions:

- **TLS negotiation.** With `sslmode != disable`, the client sends `SSLRequest` (`80877103`) and reads one byte. `S` means it calls `Tls.tls_connect` over the same TCP stream; `N` means it continues in cleartext or fails, depending on `sslmode`. PostgreSQL 17 also accepts **direct TLS** (`sslnegotiation=direct`). Here the TLS handshake starts immediately with ALPN `postgresql`, which saves a round trip. Tls already does ALPN, so this is a config flag. It is allowed only with `sslmode=require` or stricter.
- **GSSENCRequest** (`80877104`) is out of scope.
- **Startup message.** `StartupMessage` carries protocol 3.0 (`196608`) and the parameters `user`, `database`, `application_name`, `client_encoding=UTF8` and `DateStyle=ISO`, plus any `options`.
- **Protocol 3.2.** PostgreSQL 18 adds protocol 3.2 (`196610`), whose `BackendKeyData`/`CancelRequest` key is variable length (256-bit in the release notes) instead of 4 bytes. Request 3.2 and accept `NegotiateProtocolVersion` downgrading to 3.0, so older servers keep working. Store the key as a byte slice, never a `u32`. Version 3.1 was never used.

#### Authentication

Authentication runs in a loop on `Authentication*` messages:

| Code | Method | Needs |
| --- | --- | --- |
| 0 | Ok | - |
| 3 | Cleartext | refuse unless TLS, or `allow_cleartext` |
| 5 | MD5 (deprecated in PostgreSQL 18, to be removed) | `"md5" ++ hex(md5(hex(md5(password ++ user)) ++ salt))`; stdlib `md5` |
| 10/11/12 | SASL: `SCRAM-SHA-256`, `SCRAM-SHA-256-PLUS` | SHA-256, HMAC-SHA-256, PBKDF2 (`Hi`), CSPRNG nonce, base64 |
| 7/8/9 | GSSAPI/SSPI | out of scope |
| 10 with `OAUTHBEARER` | PostgreSQL 18 OAuth | later |

SCRAM (RFC 5802/7677) is the default on every supported server. Its message flow:

- **Client-first message:** `n,,n=,r=<nonce>`. The user name is empty because Postgres uses the startup user.
- **Server-first message:** gives `r`, `s` and `i`.
- **Client-final message:** the client computes `SaltedPassword = PBKDF2(password, salt, i)`, then `ClientKey`, `StoredKey`, `ClientSignature` and `proof = ClientKey ^ ClientSignature`. It sends `c=biws,r=...,p=...`.
- **Server signature:** the client must check it before trusting `AuthenticationOk`. Skipping that check is a common bug.
- **Iteration cap:** a hostile server can send a huge `i` to burn CPU. Cap it, configurable; pg.zig does this.

Two details on SCRAM:

- **SASLprep.** Passwords that SASLprep cannot normalise (for example, not valid UTF-8) are used as raw bytes, which is what libpq does. ASCII passwords need no SASLprep. Implement it with the stdlib's Unicode tables (NFKC plus the prohibited-character checks). Test with a non-ASCII password such as U+2168, as node-postgres's CI does.
- **Channel binding.** `-PLUS` uses `tls-server-end-point`: a hash of the server's leaf certificate DER, using the certificate's signature hash with MD5/SHA-1 promoted to SHA-256. That needs the peer certificate from Tls. Check that `MBEDTLS_SSL_KEEP_PEER_CERTIFICATE` is on in `jai_mbedtls_config.h`, and expose `peer_certificate_der(stream)`.

**Write HMAC-SHA-256 and PBKDF2 in Jai, next to `sha.jai`**, rather than calling mbedTLS:

- they are about 60 lines;
- they keep the client working without linking mbedTLS (cleartext local connections, wasm later);
- they run identically at compile time.

Test them with RFC 4231 (HMAC) and RFC 7914 §11 (PBKDF2-HMAC-SHA-256) vectors and the RFC 7677 SCRAM example. The default of 4096 PBKDF2 iterations is about 8k SHA-256 compressions. Under the interpreter that is milliseconds, and caching `SaltedPassword` per `(password, salt, i)` makes pool reconnects free.

After authentication the client collects:

- `ParameterStatus`: `server_version`, `integer_datetimes`, `standard_conforming_strings`, `TimeZone`, `client_encoding`;
- `BackendKeyData`, for cancellation;
- the first `ReadyForQuery`, whose status is `I`, `T` or `E`.

#### Query protocols

- **Simple query** (`Q`) is for migrations and untyped scripts. It allows several statements, text results only, and no parameters.
- **Extended query** handles everything typed: `Parse` (statement name, SQL, parameter OIDs), `Bind` (portal, per-parameter format codes, values, per-column result format codes), `Describe`, `Execute` (row limit), `Close`, `Flush` and `Sync`. Typed queries know their parameter OIDs and column formats at compile time. So a cached statement needs `Bind`/`Execute`/`Sync` at runtime, an uncached one `Parse`/`Bind`/`Execute`/`Sync`, and neither needs `Describe`. That is one round trip per query.
- **Pipelining** sends many `Parse`/`Bind`/`Execute` groups before reading, with a `Sync` per batch. The rule that matters: after an `ErrorResponse`, the server discards everything up to the next `Sync`. The client must therefore track which queued requests were skipped and fail each one with "aborted by an earlier error in the pipeline". The libpq pipeline tests are the checklist for these cases. API: `Pg.pipeline(db) { a := Pg.query(...); b := Pg.query(...); }`. Each query returns a pending result, and leaving the block syncs. Without the block, queries are still sent back to back when several fibers share a connection (see [Pool](#pool)).
- **Errors.** `ErrorResponse`/`NoticeResponse` fields (`S V C M D H P q W s t c d n F L R`) become a `Pg_Error`, with SQLSTATE as a five-byte code plus a generated enum of the common classes (`UNIQUE_VIOLATION`, `SERIALIZATION_FAILURE`, ...) for `if err.code == .UNIQUE_VIOLATION`. Notices go to `context.logger` at INFO level by default.

#### COPY

- `copy_in(db, "copy t (a, b) from stdin (format binary)", rows)` takes a `[] Row` of a generated or checked struct. It writes the binary COPY header, and each tuple is a field count, then length-prefixed binary fields. This is the fast bulk insert, an order of magnitude faster than `INSERT ... VALUES` batches.
- `copy_out` streams `CopyData` into a reader or decodes it into rows.
- `CopyFail` on error or cancellation.

#### LISTEN/NOTIFY

`NotificationResponse` (`A`) can arrive at any time, including between other messages. The reader loop routes it to the connection's `Channel(Notification)`. `Pg.listen(pool, "channel")` takes a dedicated connection out of the pool, since a pooled connection would drop the subscription when recycled, and returns the channel. Reconnecting re-issues `LISTEN`; notifications sent while disconnected are lost, which is Postgres semantics, so this is documented rather than hidden.

#### Cancellation and deadlines

A query's fiber can be cancelled two ways: `Io.cancel(fiber)`, or an `Io.with_deadline` that expires while the fiber waits. Either way:

1. The client opens a **new** connection, through TLS if the main one used it, and sends `CancelRequest` (`80877102`, pid, key).
2. It keeps draining the original connection to `ReadyForQuery`, so the connection stays reusable.
3. If draining does not finish within a grace period (default 2 s), it closes the connection rather than returning it to the pool.

A cancel can arrive after the query already finished. That case is harmless, because the next `ReadyForQuery` resynchronises.

#### Formats per type

Typed queries know every column's OID at compile time. They therefore request **binary** for types whose binary form is simple and stable, and **text** for the rest. Runtime-only (untyped) queries use text.

| Type | Format | Jai type |
| --- | --- | --- |
| `bool`, `int2/4/8`, `float4/8`, `oid` | binary | `bool`, `s16/s32/s64`, `float32/64`, `u32` |
| `text`, `varchar`, `bpchar`, `name`, `citext` | binary (same bytes as text) | `string` |
| `bytea` | binary (text form is hex) | `[] u8` |
| `uuid` | binary (16 bytes) | `Pg.Uuid` (`[16] u8`) |
| `timestamptz`, `timestamp` | binary (int64 µs since 2000-01-01; integer datetimes are always on in supported servers) | `Pg.Timestamp`, `Pg.Local_Timestamp` |
| `date`, `time`, `interval` | binary | `Pg.Date` (days), `Pg.Time` (µs), `Pg.Interval` (months, days, µs) |
| `numeric` | binary (base-10000 digits, weight, sign, dscale) | `Pg.Numeric`: exact decimal, with conversions to string, `s64` and `float64` |
| `json` | text | `string`, or [Json](../extensions/json.md) value |
| `jsonb` | binary (version byte `1`, then text) | as `json` |
| arrays of the above | binary (ndim, has-null, element OID, dims, elements) | `[] T`, or `[] Nullable(T)` when elements may be null |
| enums | text | generated Jai `enum` from `pg_enum` |
| composites, ranges, `inet`, `money`, ... | text at first | `string`, plus later codecs |
| domains | as the base type | as the base type |

Codecs are table-driven and keyed by OID, in one `codecs.jai`. Each entry has a binary encoder/decoder, a text encoder/decoder and the Jai type. User types (enums, domains, composites) get OIDs at runtime. The compile-time description records *names*, so the runtime resolves `name → oid` once per connection, at the first use of that type.

#### Pool

Like the HTTP client's pool: scheduler-local, no locks, newest idle connection first. It has these settings:

- `max_size` and `min_idle`;
- an idle timeout;
- `max_lifetime`, with jitter so a fleet does not reconnect together;
- a health check (an empty `Sync`) after the connection has been idle more than N seconds.

A connection goes back to the pool only at `ReadyForQuery` status `I`. If the status is `T` or `E` (inside a transaction) when the borrower returns it, the pool rolls back or closes it.

**Transactions:**

```jai
Pg.transaction(pool) {
    `tx` is a connection pinned for the block
    rollback via `defer` unless the block finishes
}
```

`Pg.transaction(pool, .SERIALIZABLE, retries = 3)` retries on `40001`/`40P01`. Statement caches are per connection, so pinning matters (see [Statement and plan caching](#statement-and-plan-caching)).

**Multiplexing.** Several fibers may share one connection, with requests serialised in pipeline order. This gives higher throughput with fewer server connections. Later; v1 is one borrower per connection.

**Behind a pooler.** PgBouncer in transaction mode has tracked protocol-level named prepared statements since 1.21. If it is older, set `statement_cache = .OFF` (unnamed statements only).

### Layer 2: typed raw queries

#### What the user writes

```jai
Pg :: #import "Extensions/Postgres";

user_id: s64 = 42;
since := Pg.now() - Pg.days(7);

rows, err := Pg.query(db, #string SQL
    select u.id, u.email, u.display_name, count(p.id) as posts
    from users u left join posts p on p.author_id = u.id and p.created_at > %
    where u.id = %
    group by u.id
SQL, since, user_id);
if err { log_error("%", err); return; }

for rows print("% <%> wrote %\n", it.display_name, it.email, it.posts);
//  it: struct { id: s64; email: string; display_name: Nullable(string); posts: s64; }
```

Other forms:

- `Pg.query_one(db, sql, ..)` returns the row or `Pg_Error.NOT_FOUND`.
- `Pg.query_opt` returns `(row, found)`.
- `Pg.execute(db, sql, ..)` returns the affected row count.
- `Pg.query_scalar(db, "select count(*) from users")` returns the value.
- `Pg.query_as(User_Summary, db, sql, ..)` checks **your** struct against the described columns: names match fields, types are assignable, and nullable columns go into `Nullable` fields. This is how a hand-written type stays connected to SQL.

#### What happens at compile time

`query` is an `#expand` macro with `$sql: string`, `args: ..Any` and `call := #caller_code`.

1. **Rewrite placeholders.** `%` becomes `$1..$n`, and `%%` becomes a literal `%`, the same rule as `print`. `%` was chosen because Jai users already read it as a slot. The scanner skips SQL string literals, dollar-quoted strings, quoted identifiers and comments, so `'100%'` is not a parameter. Postgres' own `$n` is rejected in the source, to keep one style.
2. **Read argument types.** `compiler_get_nodes(call)` gives the typed argument nodes, the same path Check's `@PrintLike` uses.
3. **Describe.** Look up the cache key (next section). On a miss, `Parse` plus `Describe(statement)` against the compile-time database gives `ParameterDescription` (OIDs) and `RowDescription`, which has the following for each column:
   - name;
   - type OID and modifier;
   - **table OID and attribute number**, which are 0 for computed columns.
4. **Infer nullability** (below).
5. **Type-check arguments.** Each argument's Jai type must encode to its parameter OID. `s32` widens to `int8`. `string` goes to `text`/`varchar`/`citext`, and also to `json`. `Nullable(T)` and `null` (a typed `Pg.NULL`) are allowed for any parameter. A mismatch reports at the argument's location: `parameter 2 is int8 (users.id) but the argument is string`.
6. **Emit code.** The macro inserts:
   - binding code with a static format code per parameter;
   - the statement id, a 64-bit hash of the rewritten SQL;
   - the row decoder, with a static format code per column.

The row type is a polymorphic struct keyed by the statement id, so equal SQL in two places yields the same type:

```jai
Row :: struct ($id: u64) { #insert #run row_fields(id); }
```

`query` returns `Pg.Rows(Row(id))`, a `for_expansion`-able result.

#### Nullability

`RowDescription` says nothing about nulls. The approach, as in sqlx:

- A column with a table OID and attribute number is NOT NULL if `pg_attribute.attnotnull` is set, *and* the column does not come from the nullable side of an outer join. To find the join side, prepare the statement, run `EXPLAIN (VERBOSE, FORMAT JSON) EXECUTE stmt(null, ...)` with generic parameters, and walk the plan's join nodes. That gives a generic plan on PostgreSQL 12 and later. The plan can only make a column nullable, never non-null, which is how sqlx uses it.
- Computed columns are nullable unless they are:
  - `count(...)`;
  - a literal;
  - a `coalesce` whose last argument is not null;
  - an `exists(...)`;
  - a column of a `values` list with no nulls.

  Keep this list small and explicit.
- **Overrides are part of the column alias**, because they must survive caching and be visible in the SQL:
  - `as "posts!"` forces non-null;
  - `as "name?"` forces nullable;
  - `as "created_at: Pg.Local_Timestamp"` overrides the Jai type.

  The suffix is stripped from the field name. A forced non-null column that returns null at runtime is a decode error naming the column, not a silent zero.
- Parameters are non-null unless the argument's type is `Nullable(T)`.

Nullable values are `Nullable(T) :: struct { value: T; null: bool; }`, with `or_else(x, default)` and `if x.null`. A pointer (`*T`) was rejected because it allocates per field and is easy to dereference blindly. A sentinel was rejected because `0` and `""` are valid values. This is an [open question](#open-questions).

#### Rows and memory

- `for rows` streams. Each `it` holds strings and slices that point into the connection's receive buffer, valid for that iteration only. This is zero copy, and makes a 1M-row scan cheap.
- `Pg.fetch_all(rows)` copies into `context.allocator`, with one block per result so `free_rows` is one call.
- `Pg.fetch_all(rows, temp)` puts them in temporary storage.
- `Execute` with a row limit (`Pg.query(db, sql, ..; batch = 1000)`) uses a portal, so a huge result never sits in memory.

### How types are learned at compile time

The typing step needs a server that has the project's schema. Four ways to get one:

| | PGlite (WASI Postgres) in-process | Dev database (`DATABASE_URL`, like sqlx) | Managed ephemeral Postgres (downloaded binaries) | Committed offline cache |
| --- | --- | --- | --- | --- |
| Setup for the user | none | run Postgres yourself | none (first build downloads ~10-30 MB per platform) | none for readers; writers need one of the others |
| Fidelity | real Postgres, single user | exact (same server as prod, if you like) | exact, with a pinned major version | as of the last `prepare` |
| Extensions (`pgvector`, PostGIS...) | only those built into the bundle | whatever is installed | the bundle's own extensions only | as cached |
| Speed | interpreted wasm: initdb plus migrations would take many seconds at least | ~1 ms per describe | server start ~0.5-1 s, then ~1 ms per describe | instant |
| Works in the editor (`jailsp`) | no (sandbox, budget) | no (no network) | no (no processes) | **yes** |
| Works in CI and fresh clones | yes | needs a service | yes | **yes** |
| Cost for us | **a wasm runtime in jaic**, plus a third-party WASI build; see below | none beyond the client | a downloader plus process management | a file format and a check command |

On PGlite:

- **Upstream** PGlite is an Emscripten build whose host is its JavaScript glue. It is single-user mode, with one connection.
- **Third-party WASI builds exist** and run outside JS:
  - `pglite-oxide`, a Rust crate that runs PostgreSQL 17.5 in Wasmtime, ships a prebuilt data-directory template and exposes a local TCP URL;
  - `wasipg` runs in Go with wazero;
  - `pglite-rust-bindings` is an early draft.

  They are young. wasipg's README says some WASI builds abort on a failing query, and that wasipg recovers only by lowering setjmp/longjmp to wasm exceptions. A typing server that dies on the first bad query is a problem, since users write bad queries constantly.
- **PGlite does not answer the nullability question either.** Its `describeQuery` returns parameter and result type OIDs only, so the `pg_attribute` and `EXPLAIN` steps below are still needed.
- **Embedding means a wasm runtime in jaic:**
  - vendoring Wasmtime, as pglite-oxide does, which is feasible since jaic is Rust, but adds a large dependency and binary size against the dependency policy; or
  - writing an interpreter, which is a large project and too slow for initdb plus migrations.

  The compile-time interpreter runs *Jai*, not wasm.

So PGlite is possible, but the cost and maturity are not worth it for typing alone. The managed-binaries row gets the same zero-setup result with a real server. Revisit if the WASI builds mature, or if jaic gains a wasm host for other reasons. The design keeps it pluggable: the compile-time path needs only "a server URL that has the schema", and pglite-oxide already provides one.

**Chosen pipeline:**

1. **Offline cache first.** At compile time, if a cache entry matches the key, use it, with no server needed.
2. **On a miss,** connect to the server named by `JACKET_DATABASE_URL`, falling back to `DATABASE_URL`, then write the entry.
   - With no URL and **managed mode on** (later phase), start the managed server instead (next paragraph).
   - With neither, the miss is a compile error that names the query and says how to fix it: `set DATABASE_URL or run jacket pg prepare`.
3. **`JACKET_PG_OFFLINE=1`** (CI, editor) turns misses into errors and never connects.

**Migrated-schema cache.** Neither a dev database nor the managed server is trusted to have the right schema. The compile-time path:

1. **Clone a template database.**
   1. Hash the migration files: sorted names plus contents, SHA-256. This is the **schema hash**.
   2. Look for a template database `jacket_<hash12>` on the compile-time server.
   3. If it is missing, create it from `template0`, apply all migrations and mark it a template.
2. **Describe queries in a clone of it.** A clone is cheap, because `CREATE DATABASE ... TEMPLATE` is a file copy; one clone per build is enough, since describes don't write. The user's dev database is never touched.

Old `jacket_*` templates beyond the newest N are dropped. This is the "cache the migrated schema by a hash of the migration files" idea from the TODO, and it works the same against any server.

#### Cache keys and invalidation

```
.jacket/queries/<statement-id>.json        (committed)
{
  "sql": "select ... where u.id = $2",          // after % rewriting, exactly what is sent
  "schema_hash": "9f2c…",                         // migrations that produced the description
  "deps": { "public.users": "a41b…", "public.posts": "77e0…" },   // per relation fingerprint
  "params":  [ { "type": "timestamptz" }, { "type": "int8" } ],
  "columns": [ { "name": "id", "type": "int8", "nullable": false, "origin": "public.users.id" }, … ]
}
```

- **Statement id** = SHA-256 of the rewritten SQL plus the override aliases, truncated to 64 bits for the file name; the full hash is checked inside. sqlx uses the full SHA-256 of the query text and writes `.sqlx/query-<hash>.json`.
- **Coarse validity:** `schema_hash` equals the current one. This is always correct, but every migration invalidates every query.
- **Fine validity:** when `schema_hash` differs, the entry is still valid if every relation and type it depends on has the same fingerprint. A fingerprint is SHA-256 over that relation's catalog rows: columns, types, `attnotnull`, enum labels, domain bases. After a migration, the compile-time path computes all fingerprints in one catalog query and re-describes only the queries whose dependencies changed. So adding a column to `posts` leaves queries that don't touch `posts` alone. Column-level granularity is not worth it: `select *` and views make "which columns" fragile, and a re-describe costs a millisecond.
- **Finding dependencies:**
  - Table OIDs from `RowDescription` cover the selected columns but not tables used only in `where`, functions or types.
  - The plan's relation list from `EXPLAIN (VERBOSE)` covers all scanned relations, for `SELECT` and DML alike.
  - Types come from parameter and column OIDs, followed through domains, arrays and composites.

  An option to test in a spike: for `SELECT`s, `CREATE TEMP VIEW ... AS <sql>` and read `pg_depend`. That is the server's own dependency list, functions included.
- **`search_path`** and the server major version are part of the key. A query described on 17 may describe differently on 18.

#### Staleness check

`jacket pg check` (CI) runs in five steps:

1. Migrate a fresh server.
2. Re-describe every cache entry.
3. Fail if any entry differs.
4. Fail if an entry is unused. That needs the list of statement ids actually compiled, which the build writes as `.jacket/queries/used.txt` in check mode.
5. Fail if a compiled query has no entry.

`jacket pg prepare` rewrites the entries. This is `cargo sqlx prepare --check` with dependency-aware invalidation.

### Layer 3: schema, migrations and table structs

#### The three options

**A. SQL migration files are the truth; Jai types are derived from the migrated schema.**

- Good:
  - one source of truth;
  - all of Postgres is available: partial and expression indexes, `check`, generated columns, triggers, RLS, extensions, `create index concurrently`;
  - migrations are reviewable SQL that DBAs understand;
  - query results and table structs come from the same catalog through the same OID-to-Jai mapping, so they **cannot** disagree;
  - no diff engine to build or trust;
  - the same model as sqlx, sqlc, Diesel (`print-schema`) and jOOQ.
- Bad:
  - you write DDL by hand;
  - table structs exist only if generated, and generated code must be visible to the editor;
  - renaming a column means a migration plus fixing compile errors. That is the right workflow, but it's two steps.

**B. Jai structs plus notes define tables; migrations are generated by diffing.**

```jai
User :: struct @pg_table("users") {
    id: s64;              @pg_primary_key @pg_identity
    email: string;        @pg_unique
    display_name: Nullable(string);
}
```

- Good: one language, types visible where you write them, and refactors feel local. This is the Prisma, Drizzle, Django and EF experience.
- Bad:
  - **A diff engine.** It needs schema introspection, a struct-to-schema model, a DDL generator, rename detection (ambiguous without a `@pg_renamed_from` note), and data migrations still written in SQL. That is the largest single piece of work in this whole design, with subtle failure modes.
  - **The notes become a second DDL language** that will never cover Postgres. Every missing feature needs an escape hatch, and once escape hatches exist, the struct no longer describes the table.
  - **It does not remove the "disconnect".** Raw queries are still typed by *describing them against the migrated database*, because only Postgres knows the type of `count(*)`, `coalesce(a, b)` or a view column. So the structs are the truth for DDL, the database is the truth for queries, and the two agree only if the generated migrations were applied unedited. A hand-edited migration, a `psql` hotfix or a migration from another branch makes them drift silently, unless you also check the structs against the catalog. Once you do that, the database is the truth anyway.

**C. Hybrid: SQL is the truth, and generated structs are committed.**

- `jacket pg generate` writes `db/schema.jai`, a committed and formatted file. It is generated from the migrated template database, like `diesel print-schema` or `sqlc generate`, and contains:
  - a struct per table and view;
  - an enum per Postgres enum;
  - the notes the ORM needs (`@pg_table`, `@pg_primary_key`, foreign keys).
- A compile-time check (`#run Pg.check_schema(#file)`) compares the file's hash with the schema hash recorded in it. A stale file is a compile error saying "run `jacket pg generate`". Like the query cache, this check needs no server.
- Optionally, **struct-first authoring as scaffolding only.** `jacket pg diff` compares hand-edited structs in `db/schema.jai` with the template database and *writes a proposed migration file* for you to review and edit. It never applies anything. This gives B's authoring feel for the easy 80% (add a table or a column) without making the diff engine a source of truth. It is a later phase and can stay simple, because it is only a starting point.

#### The "disconnected" concern

Raw query results and table structs stay connected under C in three ways:

1. **Same origin.** Both come from `RowDescription`/`pg_attribute` after the same migrations, mapped by the same codec table. `users.email` is `string` everywhere or nowhere.
2. **Row type identity.** When a query's columns are exactly one table's columns in order, by table OID and attribute number, the macro returns the **table struct** (`User`) instead of a new anonymous struct:
   - `select * from users`
   - `select u.* from users u join ...`
   - `returning *`

   Results then flow into code that takes `User`.
3. **Checked user structs.** For a subset, `Pg.query_as(User_Card, ...)` checks your struct. `Pg.Partial(User, "id, email")` is a struct generated from `User` holding only those fields, and it converts to `User` with defaults where needed.

Under B, by contrast, connecting them means adding a check of the structs against the catalog. That makes the database the real source of truth while you maintain a second one. This is the core reason to choose C.

#### The migration runner

- **Files.** `db/migrations/NNNN_name.sql`, forward only. Down migrations are rarely run and rarely correct. A file whose first line is `-- jacket: no-transaction` runs outside a transaction, as needed by `create index concurrently`.
- **Embedding.** `Pg.MIGRATIONS :: #run Pg.load_migrations("db/migrations")` embeds the names, contents and SHA-256 sums in the binary, so production needs no files on disk.
- **Runtime.** `Pg.migrate(db, Pg.MIGRATIONS)` works like this:
  1. Take an advisory lock (`pg_advisory_lock(hash('jacket_migrations'))`), so two app instances never race.
  2. Create the table `jacket_migrations(version, name, sha256, applied_at, duration_ms)` if it is missing.
  3. Apply each pending migration in its own transaction, using the simple protocol so multi-statement files work.
  4. **Refuse** if an applied migration's checksum changed or a migration is missing from the embedded list. Override with `allow_modified`.
- **Commands.** `jacket pg migrate`, `new <name>` and `status` wrap the same code.

### Layer 4: the ORM

The ORM is a compile-time **SQL generator** that hands its output to layer 2. It has no runtime query objects, no reflection, and no SQL built at runtime.

```jai
#import "db/schema.jai";     // User, Post, Comment (generated)

min_posts := 3;
for Pg.select(db, User,
              where    = #code it.active && it.post_count >= min_posts,
              order_by = #code it.created_at, desc = true,
              limit    = 20)
{
    print("% %\n", it.email, it.team.name);   // only email + team.name are read
}
```

How it compiles:

- **Filters as Code.** `select` is a macro with `where: Code` and `order_by: Code`. `compiler_get_nodes` walks the expression:
  - field accesses on `it` become columns;
  - literals stay literals;
  - any other identifier or expression from the caller (`min_posts`) becomes a `%` parameter, carried in through a backtick and typed by the compiler;
  - `&&`, `||`, `!`, comparisons, `== null` and arithmetic map to SQL.

  The ORM also provides `Pg.like`, `Pg.in(it.id, ids)` (`= any($n)`) and `Pg.lower`. A construct it cannot translate is a compile error pointing at that node. It never silently falls back to runtime filtering. This is Ecto's `^pinned` model with Jai's own syntax.
- **Column pruning from the loop body.** `Pg.select` returns a value whose `for_expansion` receives the loop body as `Code`. Before the SQL is built, the expansion walks the body for `it.<field>` and `it.<relation>.<field>` reads, and selects exactly those columns plus the primary key.
  - If `it` escapes (passed whole to a procedure, stored, returned, or `*it` taken), the body could read anything, so all columns are selected. That is still correct.
  - `Pg.fetch_all(Pg.select(...))` with no loop selects all columns. `Pg.select(..., fields = .[.id, .email])` lets you choose. In both cases the row type is then `Partial(User, ...)`.
- **Relations without N+1.** Foreign keys in `db/schema.jai` become relation notes (`team: Team; @pg_belongs_to("team_id")`, `posts: [] Post; @pg_has_many("author_id")`).
  - **To-one** (`it.team.name`) becomes a `join`: `inner` when the foreign-key column is NOT NULL, otherwise `left`, with the relation then `Nullable`.
  - **To-many** (`for p: it.posts`) becomes a second query after the first, `where author_id = any($1)` with the parent keys, stitched in memory. This is Ecto's preload. It avoids the row explosion of joining two one-to-many relations, and costs 2 queries rather than N+1.

    A `strategy = .JSON_AGG` alternative builds one query with a `lateral (select json_agg(...))` subquery, as Drizzle's relational queries do.

    Both strategies come from the loop body. The user never writes `include`.
- **Mixing in raw SQL:**
  - `where = #code it.active && Pg.sql("lower(%) = lower(%)", it.email, email)` mixes a raw fragment with ORM conditions, with the same `%` rule.
  - `Pg.select(db, User, from_sql = "select * from users_visible_to(%)", viewer)` takes a raw `from` source.
  - Anything bigger is a raw `Pg.query`, which still returns `User` when the columns match (row type identity).
- **Writes:**
  - `Pg.insert(db, user)` and `Pg.insert_many(db, users)` use `insert ... returning *`, or COPY above a threshold.
  - `Pg.update(db, User, set = #code it.display_name = name, where = #code it.id == id)` and `Pg.delete(...)` are compiled the same way as `select`.
  - No dirty tracking or unit of work, and no lazy loading.
- **Dynamic filters.** Everything above is static SQL, which is what makes it typed and cacheable. Optional filters use `Pg.if_some(x, #code it.team_id == x)`, which compiles to `($n::int8 is null or team_id = $n)`. That is one statement, but it may get a generic plan. Large dynamic search screens should use several static queries or a raw query. A runtime builder is deliberately left out of v1 (see [Open questions](#open-questions)).

#### Statement and plan caching

- **Every typed statement has a compile-time id.** Each connection keeps an LRU (default 256) from id to server statement name (`j<hex id>`). The first use on a connection pipelines `Parse(name, sql, oids)` with the `Bind`/`Execute`, so it is still one round trip. Later uses send only `Bind`/`Execute`/`Sync`. Eviction sends `Close`.
- **Drift check.** On the first `Parse` per connection, `verify_statements` (on in debug builds, off in release) adds a `Describe` and compares the result with the compiled description. If someone migrated the database under a running binary, this fails loudly with "statement X was compiled against a different schema", instead of misdecoding bytes.
- **Plan caching** is the server's job. Named statements switch to a generic plan after five executions when it is not worse (`plan_cache_mode`). Parameter-sensitive queries can opt out per statement (`Pg.query(..., plan = .CUSTOM)`), which uses the unnamed statement each time.
- **Errors.** The cache entry is dropped on `0A000 cached plan must not change result type` and on `26000`, and the query is retried once.

## How to change it

The intended layout, so this section also guides the first implementation:

- `stdlib/Extensions/Postgres/`:
  - `protocol.jai` (message encode/decode, no I/O);
  - `connection.jai` (startup, auth, read loop);
  - `auth.jai` (MD5/SCRAM);
  - `codecs.jai` (OID table);
  - `pool.jai`;
  - `typed.jai` (the `query` macro and describe cache);
  - `schema.jai` (generation and checks);
  - `migrate.jai`;
  - `orm.jai`.
- `stdlib/internal/crypto/`: HMAC-SHA-256, PBKDF2, OS random bytes.
- `stdlib/Extensions/Io`: Unix-domain sockets.
- `jacket pg ...` subcommands, which are thin: each one compiles and runs a Jai tool program from `Extensions/Postgres/tools/`, so all the logic lives in Jai and runs the same in `#run`.
- Adding a type: one row in `codecs.jai`, with the binary and text codecs and the Jai type, plus round-trip cases in the conformance suite. If it has no stable binary form, register it text-only.
- Adding an ORM operator: one case in the `Code` to SQL walker in `orm.jai`, plus a test that compiles a query with it and compares the generated SQL text.
- Gotchas:
  - Changing the `%` rewrite, the override syntax or the description format changes every statement id and invalidates every committed cache. Bump a format version in the key.
  - Everything the compile-time path does must also work under `SandboxHost` when it reads from the cache. That means no network or process calls on the cache-hit path.

## Testing against real behaviour

What "up to spec" means here: the [protocol chapter](https://www.postgresql.org/docs/current/protocol.html) as implemented by real servers. The options:

| Option | What it checks | Verdict |
| --- | --- | --- |
| **Real servers in CI**: a matrix of supported majors × auth (trust, md5, scram, scram+TLS, scram-PLUS, client cert) × `sslnegotiation` | everything, against the implementation that defines the protocol | **primary** |
| **Differential against libpq**: run the same statements through `psql`/`PQexecParams` (binary results) and through the client, and compare decoded values | that our codecs match the reference client for every type and edge value | **primary, for codecs** |
| **Recorded traces**: capture real client↔server byte streams once, commit them, and replay them against a scripted fake server. go-pgvcr does TCP-level record and replay; pgproto3 has a trace test. libpq_pipeline's `traces/` files are reference message sequences for pipelines. | the protocol state machine, deterministically, on every OS without Postgres, including fault injection (truncation, errors mid-pipeline, unsolicited notices and notifications, a cancel racing completion) | **secondary**: fast and runs on Windows/macOS on every push |
| **Fuzzing** the server-to-client decoder with `Extensions/Fuzz` (seed corpus: pgproto3's `testdata/fuzz`) | that hostile or garbled input never crashes or over-reads | **secondary**, in the nightly fuzz job |
| pgproto3 (Go) / rust-postgres / node-postgres test suites | written for their own APIs; not runnable against ours | mine their value lists (edge values per type) and scenario lists as checklists, nothing more |
| `pg_regress` | server behaviour (SQL), not a client | not useful |
| `pgbench` | drives libpq itself; cannot drive our client | not useful for conformance; a Jai port of its TPC-B script is the **benchmark** against libpq/pgx |
| `src/test/modules/libpq_pipeline` | libpq's own pipeline cases | port its scenarios (errors mid-pipeline, sync handling) as test cases |
| RFC 4231 / 7914 / 7677 vectors | HMAC, PBKDF2, SCRAM | unit tests |

No reusable cross-language protocol conformance suite exists. Drivers test against real servers: pgx runs PostgreSQL 14-18 with md5, SCRAM, Unix-socket and client-certificate users (and OAuth on 18); node-postgres runs 13-18.

**Pick:** real servers plus libpq differential as the conformance suite, recorded traces and fuzzing as the fast and robustness layers. A new `pg_conformance.py` harness, next to `http_conformance.py` in tools/stdlib, runs them, a path-filtered workflow `pg-conformance.yml` on the same model as `http-conformance.yml`.

- **Linux (every relevant push):** GitHub `services:` containers, one job per major (`postgres:14` … `postgres:18`, plus the newest beta as allowed-to-fail). The job writes `pg_hba.conf` and `postgresql.conf` and restarts the server, to enable md5 and scram users, TLS with the Tls test CA from `stdlib/Extensions/Tls/tests/data`, client-certificate auth and `sslnegotiation=direct`. `psql` from the same image is the libpq oracle.
- **macOS and Windows (manual dispatch and nightly):** no service containers. `ikalnytskyi/action-setup-postgres` (Linux, macOS, Windows; 15-18) installs a server, as pgx's Windows job does. Once managed mode exists, also run it from the downloaded binaries (`theseus-rs/postgresql-binaries`: x64 and arm64 for Linux, macOS and Windows, with Windows arm64 from PostgreSQL 16), so that path is tested too. Unix sockets are covered on macOS/Linux; Windows uses TCP.
- **Every push, all OSes:** trace replay, codec unit tests and SCRAM vectors, inside the normal stdlib runtime tests (natively and under `-interp`, since the client must also work at compile time).

## Configuration

| Setting | Where | Meaning |
| --- | --- | --- |
| `JACKET_DATABASE_URL`, then `DATABASE_URL` | environment | compile-time server for describing queries (`postgres://user:pass@host:port/db?sslmode=…`); also the default for `jacket pg` |
| `JACKET_PG_OFFLINE=1` | environment | never connect at compile time; cache misses are errors (CI, editor) |
| `[postgres]` in `jacket.toml` | `migrations = "db/migrations"`, `schema = "db/schema.jai"`, `cache = ".jacket/queries"`, `server_version = 17`, `managed = false` | paths and modes for `jacket pg` and the compile-time path |
| connection options | `Pg.Config` / URL | `sslmode` (`disable`, `prefer`, `require`, `verify-ca`, `verify-full`), `sslnegotiation`, `channel_binding`, `application_name`, `connect_timeout`, `statement_timeout`, `statement_cache` (size or `.OFF`), `verify_statements` |
| pool options | `Pg.Pool_Config` | `max_size`, `min_idle`, `idle_timeout`, `max_lifetime`, `health_check_after` |

The `jacket.toml` section needs a schema entry in `crates/jacket-config`. Compile-time code also reads it (the `[project]` import paths already set a precedent).

## Dependencies

- Internal:
  - Io (TCP, the resolver, fibers, deadlines, and Unix sockets once added);
  - Tls (with peer certificate DER for channel binding);
  - `internal/crypto` (SHA-256, plus the new HMAC/PBKDF2/random);
  - `md5`, Base64, [Json](../extensions/json.md) (for `json`/`jsonb` values and cache files);
  - the `Compiler` module (`compiler_get_nodes`, `#caller_code`, typed nodes).
- External at runtime: none (no libpq).
- External at compile time: a Postgres server only on cache misses.
- CI: Postgres server images and binaries, and `psql`.

## Prior art

| Library | Copy | Avoid |
| --- | --- | --- |
| **sqlx** (Rust) | Raw SQL typed by asking the server (`Describe`). `query_as!` checking user structs. Column-alias overrides (`"col!"`, `"col?"`, `"col: T"`). Committed offline cache and `prepare --check`. Nullability from `attnotnull` plus `EXPLAIN` join analysis. Migrations as SQL files with checksums. | A live database on every fresh build unless offline mode is set up. Coarse invalidation, where any change means re-prepare everything. Nullability inference that is sometimes wrong for outer joins and expressions, which overrides exist to patch. No answer for dynamic queries. A separate CLI needed to keep the cache fresh. |
| **Diesel** (Rust) | `diesel print-schema`: Rust schema generated from the migrated database and committed, which is option C. SQL-file migrations. Compile-time-checked query DSL with joins. | Trait-heavy types whose errors are famously unreadable. The DSL rewrites SQL, so anything unusual becomes `sql_query` with no typing. Long compile times. |
| **SeaORM** (Rust) | Entities generated from a live schema (`sea-orm-cli generate entity`). Relation definitions and a loader that batches `IN (...)` to avoid N+1. | Queries typed only at runtime (string columns, runtime errors). ActiveModel boilerplate. A migration DSL that duplicates SQL. |
| **jOOQ** (Java) | Code generation from the schema *after running migrations in a throwaway database* (Flyway plus Testcontainers), which is exactly the template-database step. A DSL that is SQL, not an object model. Every Postgres feature reachable. Typed records per table. | Codegen as a separate build step that is easy to forget. Verbose DSL. Dynamic SQL everywhere by default, so nothing is checked until runtime unless you use the generated types. |
| **Prisma** (TS) | `migrate dev`: edit a schema, get a *proposed* SQL migration you can edit, which is the scaffolding in option C. TypedSQL (6.12+): `.sql` files in `prisma/sql` typed by asking a live database at `generate --sql`, with `-- @param` comments for parameter names and nullability. Relation loading without N+1 in its client. | Its own schema language (PSL), a second DDL language that lags Postgres. The struct/schema-to-query disconnect B suffers from (raw SQL results were untyped until TypedSQL). Historically, a separate query-engine binary. |
| **sqlc** (Go and others) | SQL-first: annotated `.sql` queries become generated, committed typed code. Nullable parameters marked explicitly (`sqlc.narg`). Generated code you can read and grep. | Typing by its own analysis over a schema parsed with libpg_query (the real Postgres parser, but not its type checker), which misses or mistypes some SQL. It later added a database-backed analyzer (`analyzer.database`, `database.managed`) to fix this. Asking the server avoids that class of bug. |
| **Ecto** (Elixir) | Queries are macros compiled at compile time, with `^pinned` caller values as parameters; this is the model for `where = #code ...`. `preload` via join or a separate query. Changesets for validation. `Repo.transaction` with rollback. | Schemas declared in code *and* migrations in a DSL, maintained separately. They can disagree, which is the exact "disconnected" problem (Ecto relies on discipline). |
| **Drizzle** (TS) | A schema in code whose types equal the SQL types. A builder that reads like SQL. A `sql` tagged template for raw fragments, typed by hand (`sql<T>`) and never checked. Relational queries compiled to one statement with `json_agg`/lateral. `drizzle-kit` proposing migrations from schema diffs. | Heavy type-level programming. `push` (applying diffs directly), which is fine for prototypes and dangerous for production. Raw `sql` fragments are untyped. |

Common lessons:

- Ask the server for types rather than reimplementing Postgres' type system (sqlx, TypedSQL and sqlc's analyzer all converge on this).
- Generate and commit the schema code (Diesel, sqlc, jOOQ).
- Keep raw SQL a first-class, typed citizen.
- Make migrations SQL, with tooling that proposes them.

## Recommendation and plan

**Recommendation:**

- **Client:** the from-scratch wire client.
- **Typing:** sqlx-style typed raw SQL with a committed, dependency-aware offline cache.
- **Schema:** SQL migrations as the single source of truth, with Jai table structs generated from the migrated schema and committed (option C).
- **ORM:** `Code`-based selects that prune columns from the loop body and join instead of N+1, compiled down to the same typed raw-query path.
- **Compile-time server:** a dev database at first, then managed downloaded binaries for zero setup. Not PGlite, unless jaic gains a wasm host anyway.

Phases (each lands on this branch, then goes to `main` when it is useful on its own; none blocks a release):

1. **Spikes**, small throwaway programs, to settle the risky parts first:
   1. A `Row($id)` struct with `#insert #run` fields plus `#caller_code` argument types read in a macro, in jaic today.
   2. `for_expansion` body `Code` walking that finds `it.<field>`.
   3. A Parse/Describe round trip from `#run` under the interpreter against a local server.
   4. The temp-view and `pg_depend` dependency trick.
2. **Client:**
   1. Unix sockets in Io; HMAC, PBKDF2 and random in `internal/crypto`.
   2. Startup with TLS, MD5 and SCRAM (including `-PLUS`), simple and extended protocols, errors, core binary codecs.
   3. Pool, transactions, cancellation.
   4. The `pg-conformance` workflow.

   Then pipelining, COPY, LISTEN/NOTIFY and protocol 3.2.
3. **Typed raw queries:** `query`/`query_one`/`execute`/`query_as`, describe from `DATABASE_URL`, nullability, overrides, the offline cache, `jacket pg prepare`/`check`.
4. **Migrations and schema:** the runner, embedded migrations, the template-database cache, `jacket pg generate`, the `db/schema.jai` staleness check, row type identity.
5. **ORM:** `select` with `Code` filters, column pruning, to-one joins, to-many batched loading, insert/update/delete.
6. **Zero setup:** managed per-platform Postgres for compile-time typing; `jacket pg diff` migration proposals.

## Open questions

1. **Source of truth.** SQL migrations, with Jai structs generated and committed (recommended)? Or do you want structs-first authoring badly enough to build the diff engine (option B)?
2. **Compile-time server.** Start with `DATABASE_URL` plus the offline cache and add managed binaries later? Or is zero setup from day one a requirement? And is setting PGlite aside acceptable, given that it needs Wasmtime (or our own wasm runtime) in jaic and its WASI builds are young?
3. **Parameter syntax.** Positional `%` with trailing arguments (like `print`, recommended)? Or inline `%name`/`%(expr)` that captures caller variables, which reads better in long queries but hides arguments inside the string?
4. **Null representation.** `Nullable(T)` struct (recommended), `*T`, or something else?
5. **Row memory.** Zero-copy rows valid per iteration, with `fetch_all` copying (recommended)? Or always copy, which is simpler and slower?
6. **Dynamic queries.** Is "static SQL plus `if_some` optional filters, else raw SQL" enough for v1, or is a runtime builder (typed columns, runtime-assembled SQL, no cache) needed early?
7. **Oldest server.** 14 (the oldest pgx tests, and in support until November 2026) or 15+? A newer minimum simplifies nothing in the protocol, but it shrinks the CI matrix.
8. **Tooling home.** `jacket pg ...` subcommands that run Jai tool programs (recommended), or a standalone Jai program the user builds?
9. **Placement.** `Extensions/Postgres` in this repo, or a separate repo like the other "later" extensions?
