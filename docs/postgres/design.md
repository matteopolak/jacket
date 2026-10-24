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
- **The language server cannot reach a database today.** Even trusted, `jailsp` runs compile-time code in `SandboxHost`, which has no network or process access and a block budget ([workspace trust](../tools/workspace-trust.md)). Instant editor feedback on raw queries is a goal, so phase 0 adds **compile-time services**: a narrow call that a trusted workspace serves from a long-lived `jaic -interp` worker (see [Editor feedback](#editor-feedback-phase-0)). The committed cache remains the offline and CI fallback.
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
| `json` | text | `Pg.Json(T)`, or `Pg.Json_Raw` when no `T` is mapping ([JSON and jsonb](#json-and-jsonb)) |
| `jsonb` | binary (version byte `1`, then text) | as `json` |
| `json[]`, `jsonb[]` | binary array of the above | `[] Pg.Json(T)` |
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
error := Pg.transaction(pool, data, (tx: *Pg.Connection, data: T) -> Pg.Pg_Error {
    // `tx` is a connection pinned for the call; an error return rolls back
}, .SERIALIZABLE, retries = 3);
```

It takes a procedure rather than a `#code` block. A macro would declare `tx` in the caller's scope, so two transactions in one scope would collide, and a `#code` body cannot `return` its error. `retries` reruns the body on `40001`/`40P01`. Statement caches are per connection, so pinning matters (see [Statement and plan caching](#statement-and-plan-caching)).

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
    from users u left join posts p on p.author_id = u.id and p.created_at > %since
    where u.id = %user_id and u.email like '%@example.com'
    group by u.id
SQL);
if err { log_error("%", err); return; }

for rows print("% <%> wrote %\n", it.display_name, it.email, it.posts);
//  it: struct { id: s64; email: string; display_name: Nullable(string); posts: s64; }
```

Other forms:

Parameters are written inline: `%name`, `%name.field`, `%array[i]`, or `%(any expression)`. `%%` is a literal `%`. Inside SQL string literals, quoted identifiers and comments, `%` is plain text, so `like '%@example.com'` needs no escaping.

- `Pg.query_one(db, sql)` returns the row or `Pg_Error.NOT_FOUND`.
- `Pg.query_opt` returns `(row, found)`.
- `Pg.execute(db, sql)` returns the affected row count.
- `Pg.query_scalar(db, "select count(*) from users")` returns the value.
- `Pg.query_as(User_Summary, db, sql)` checks **your** struct against the described columns: names match fields, types are assignable, and nullable columns go into `Nullable` fields. This is how a hand-written type stays connected to SQL.

#### What happens at compile time

`query` is an `#expand` macro with `$sql: string`, `args: ..Any` and `call := #caller_code`, marked with an `@Interpolates("sql")` note ([Inline parameters](#inline-parameters)).

1. **Rewrite placeholders.** The compiler has already turned each `%name` hole into a positional `%` plus a trailing argument. The macro rewrites `%` to `$1..$n`, using the same SQL-aware scanner. Postgres' own `$n` is rejected in the source, to keep one style. The statement id hashes the *rewritten* SQL, so renaming a variable doesn't invalidate the cache.
2. **Read argument types.** `compiler_get_nodes(call)` gives the typed argument nodes, the same path Check's `@PrintLike` uses.
3. **Describe.** Look up the cache key (next section). On a miss, `Parse` plus `Describe(statement)` against the compile-time database gives `ParameterDescription` (OIDs) and `RowDescription`, which has the following for each column:
   - name;
   - type OID and modifier;
   - **table OID and attribute number**, which are 0 for computed columns.
4. **Infer nullability** (below).
5. **Type-check arguments.** Each argument's Jai type must encode to its parameter OID. `s32` widens to `int8`. `string` goes to `text`/`varchar`/`citext`, and also to `json`. `Nullable(T)` and `null` (a typed `Pg.NULL`) are allowed for any parameter. A mismatch reports at the hole inside the string: `%user_id is string, but parameter 2 is int8 (users.id)`.
6. **Emit code.** The macro inserts:
   - binding code with a static format code per parameter;
   - the statement id, a 64-bit hash of the rewritten SQL;
   - the row decoder, with a static format code per column.

The row type is a polymorphic struct keyed by the statement id, so equal SQL in two places yields the same type:

```jai
Row :: struct ($id: u64) { #insert #run row_fields(id); }
```

`query` returns `Pg.Rows(Row(id))`, a `for_expansion`-able result.

#### Inline parameters

Inline holes read better in long queries, but they hide expressions inside a string literal. To keep the editor fully aware of them, the compiler does the work once, rather than every macro scanning strings itself:

- **A note on the procedure.** `@Interpolates("sql")` marks the procedure's `$string` parameter as holding holes. The argument must be a literal or a `#string` block.
- **Desugaring at the call site.** Before type checking, the compiler parses each hole as an expression in the caller's scope, with source spans pointing *into the string*. It rewrites the call to positional form:

  ```jai
  Pg.query(db, "... %user_id ...")   // becomes
  Pg.query(db, "... % ...", user_id)
  ```

  The arguments are real expressions with real locations. So type errors, go to definition, find references, rename and hover on `user_id` inside the string all work through the normal paths. Rename edits the string.
- **Scanner modes.** The note's argument picks the skip grammar, so `%` inside quoted text stays literal:
  - `"plain"`: skip nothing;
  - `"sql"`: skip `'...'` (with `''`), `E'...'`, `$tag$...$tag$`, `"..."`, `--` and nested `/* */`.

  The grammar is fixed and small, inside the compiler. A module cannot plug in its own. A `%` that is followed by a space, or that is not followed by an identifier or `(`, stays a plain `%`, so SQL's modulo `a % b` works. `a %b` would be read as a hole and must be written `a %% b`, or with the space.
- **Language server support beyond `@PrintLike`.** `@PrintLike` only checks argument counts and types against a format string. Inline holes also need:
  - completion after `%` inside a marked string (locals and fields in the caller's scope);
  - semantic highlighting of holes as code;
  - diagnostics at the hole's span.

  The desugaring gives the last two most of the way. Completion is new `jailsp` work: it must know, from the note alone, that the cursor is in a hole. This works without running any compile-time code, so it works in untrusted workspaces too.
- **Outside Postgres.** Any procedure can use the note, for example a `print` variant with `%name` holes. That is a jaic extension; official Jai has no such note.

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

#### JSON and jsonb

A JSON column is typed by the Jai type its documents hold, and decoded with the [Json](../extensions/json.md) module's generated decoders.

```jai
Settings :: struct @json(rename_all=camelCase) {
    theme: string;
    font_size := 14;            @json(default)
    beta: Json_Maybe(bool);     // missing, null, or a value
}

row := Pg.query_one(db, "select id, settings as \"settings: Json(Settings)\" from users where id = %id");
print("%\n", row.settings.value.theme);
```

- **`Pg.Json(T) :: struct { value: T; }`.** It marks a value as a JSON document. As a parameter, it encodes with `to_json`: `json` gets text, and `jsonb` gets binary (version byte `1`, then the same text). As a column, it decodes with `from_json(T)`. A `jsonb` value whose version byte is not `1` is a decode error naming the version.
- **`Pg.Json_Raw :: struct { text: string; }`.** This is the untyped fallback, and the type of any JSON column without a mapping. It is zero copy: a slice of the receive buffer, valid for one iteration like any other string. `Pg.decode_json(T, raw)` decodes it later, and `Pg.json_value(raw)` gives a `Json_Value` tree. A plain `string` argument still binds to `json`/`jsonb` as raw text.
- **Which `T` a column holds:**
  - **Per query:** the column alias override, `as "settings: Json(Settings)"`, the same mechanism as the other [overrides](#nullability). It applies to `json[]` too: `as "events: [] Json(Event)"`. `Pg.query_as(Row, ...)` reads it from the struct field instead: a `Json(T)` field means that column holds `T`.
  - **In the ORM:** `db/schema.jai` is generated, so the mapping lives in a hand-written `db/json_columns.jai` that it imports:

    ```jai
    Json_Columns :: Pg.Json_Column.[
        .{ "users.settings", Settings },
        .{ "events.payload", Event_Payload },
    ];
    ```

    The generator emits `settings: Pg.Json(Pg.json_column_type("users.settings"));`. That is a compile-time lookup in `Json_Columns`, falling back to `Json_Raw` (the field is then `Json(Json_Raw)`, whose `.value.text` is the document), so changing a mapping does not regenerate the schema. A mapping naming a column that is not JSON is a compile error.
- **Strictness.** `Pg.Json_Options` is a `Json_Options` with Postgres defaults, set per query (`json = .{ unknown_fields = .DENY }`), per `Json_Columns` entry, or per connection in `Config`:
  - `unknown_fields`: `.IGNORE` (the default) or `.DENY`;
  - `missing_fields`: `.ERROR` (the default) or `.DEFAULT`, which keeps each field's initializer;
  - `duplicate_keys`: `.LAST` (the default; what `jsonb` keeps, and what `json` text may contain) or `.ERROR`.

  A struct's own `@json(deny_unknown_fields)` and `@json(default)` notes still win.
- **Errors name the column and the path.** A failed decode is a `Pg_Error` of kind `.DECODE`, whose message reads `column "settings" (users.settings), row 3, at $.theme.colors[2]: expected string, found number`. The path uses jsonpath syntax, so it can be pasted into `jsonb_path_query`. The Json error's offset and line are kept as fields.
- **Null, missing and SQL NULL are three different things:**
  - SQL NULL is the column's nullability: `Nullable(Json(T))`, like any other column.
  - A JSON `null` document, or a field holding `null`, decodes into `Json_Maybe(T)` as `.NULL`.
  - A missing key decodes into `Json_Maybe(T)` as `.MISSING`.

  `Json_Maybe(T) :: struct { state: enum u8 { MISSING; NULL; SET; }; value: T; }` belongs to the Json module, not Postgres, since HTTP bodies need the same thing. Encoding omits a `.MISSING` field and writes `null` for `.NULL`, so a decoded document re-encodes to the same keys. A plain `T` field that receives `null` is an error unless it is a pointer or a `Json_Value`.
- **Zero copy.** In a `for rows` loop, decoded strings without escapes point into the receive buffer, like the row's other strings. Strings with escapes are unescaped into `temp`. This uses a borrowing mode, `Json_Options.borrow`, which the Json module gains for this purpose. `Pg.fetch_all` deep-copies through `free_json`'s generated walker, which gains a matching `clone_json`.
- **Numbers.** `jsonb` numbers are `numeric`: arbitrary precision. A `Pg.Numeric` field decodes from the number's exact text, and `s64`/`float64` fields use the Json module's range checks. A `Json_Value` keeps the original number text next to its `float64`, so the round trip is exact.
- **Typed operators** in the ORM: see [Layer 4](#layer-4-the-orm).

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
   - With neither, the miss is a compile error that names the query and says how to fix it: `set DATABASE_URL, or run the pg tool's prepare`.
3. **`JACKET_PG_OFFLINE=1`** (CI) turns misses into errors and never connects.

Steps 1 and 2 run inside the Postgres describe service ([Editor feedback](#editor-feedback-phase-0)), in the build and in the editor alike. The macro only builds a request and reads a response.

**Migrated-schema cache.** Neither a dev database nor the managed server is trusted to have the right schema. The compile-time path:

1. **Find a schema database.**
   1. Hash the migration files in order: each one's name and contents, chained with SHA-256. The hash after the last file is the **schema hash**; the hash after each file names that prefix.
   2. Look for a database `jacket_<hash12>` for the longest prefix that has one.
   3. If none matches the whole set, the missing migrations are applied on top of the prefix database **inside a transaction that is rolled back** (see below). A new database for the whole set is built in the background, from `template0`, and used once it is ready.
2. **Describe queries in that transaction.** Describes don't write, so no clone is needed: the worker connects to the schema database, `BEGIN`s, applies any missing migrations, describes, and `ROLLBACK`s. The user's dev database is never touched.

Old `jacket_*` databases beyond the newest N are dropped. The **benchmark** behind this (Postgres 17, local, `fsync=off`, generated migrations with tables, foreign keys, indexes, checks, enums, views and triggers; medians, in ms, each including a ~12 ms client round trip):

| Migrations (lines) | Build from `template0` | Clone (`TEMPLATE`) | Clone (`FILE_COPY`) | Drop | Last migration in a rolled-back transaction | All migrations in a rolled-back transaction |
| --- | --- | --- | --- | --- | --- | --- |
| 30 (273) | 170-240 | 120-160 | 200 | 310-520 | 13 | 41 |
| 100 (910) | 290-340 | 180-520 | 230 | 400-690 | 12 | 179 |
| 300 (2730) | 730-1420 | 690-860 | 550 | 980-1070 | 13 | 338 |

Clones and drops are not cheap: since Postgres 15, `CREATE DATABASE` copies through the WAL, and drops checkpoint. Applying the edited migration in a transaction costs about a millisecond. DDL is transactional in Postgres, so a rollback leaves nothing behind. This is the "cache the migrated schema by a hash of the migration files" idea from the TODO, and it works the same against any server.

### Editor feedback (phase 0)

Goal: in a trusted workspace, editing a raw query gives feedback on the next analysis, with no build and no command:

- Postgres errors underline the exact span in the SQL string;
- `it.` completes the columns;
- hover shows each column's type and origin.

Untrusted workspaces keep today's behaviour: compile-time code is refused and its results are unknown.

#### Why today's sandbox can't do it

`jailsp` runs compile-time code in `SandboxHost` even when trusted ([workspace trust](../tools/workspace-trust.md)). That host has:

- an overlay file system (unsaved buffers, writes kept in memory);
- no processes and no network;
- a virtual clock, and cooperative "inline" threads that suspend by unwinding the interpreter;
- a `BLOCK_BUDGET` per analysis.

Fibers are refused there by one host check (`native_linking()` in `interp/fibers.rs`).

The sandbox is not only a security boundary. **The language server re-runs every `#run` on every analysis**, many times a minute while typing. A metaprogram that writes a file, starts `cmake` or POSTs to a server must not do that on each keystroke, even when trusted. Any new capability has to be safe to repeat.

#### Options

**A. Grant capabilities inside the sandbox, and run the Jai client in the language server's interpreter.** Ordinary `#run` code in a trusted workspace would get:

- network to localhost and the configured database host;
- real environment variables;
- fibers;
- native foreign calls (for mbedTLS);
- a real clock while waiting on the network;
- a larger budget;
- connections kept across analyses.

This is more than "allow sockets":

- **Io's OS surface.** Io talks to kqueue, epoll/io_uring or IOCP and runs resolver and file-pool threads. The sandbox would need to virtualise those per OS, mapping its descriptor numbers onto real sockets inside `kevent`/`epoll_event` structs, or Io would need a host-backed engine for compile time.
- **Fibers and inline threads.** Lifting the fiber check is one line, but the sandbox's inline threads suspend only in the outermost interpreter call. A thread that blocks inside a fiber, which runs on its own native stack, cannot suspend. Io's resolver and file-pool threads would hit that. Fixing it means suspending through fiber stacks, which is real interpreter work. A spike that lifts the check needs explicit approval, since it changes the interpreter's sandbox; it has not been run.
- **Clock.** The virtual clock jumps forward when nothing can run, so every network deadline would fire at once. Waits must use real time.
- **TLS** needs native foreign calls into mbedTLS, which is arbitrary native code inside the language server process.
- **Keeping connections across analyses** needs host-owned handles that outlive each fresh interpreter, plus reaping when an analysis is abandoned mid-query (budget, newer edit).
- **SCRAM** costs about 8k interpreted SHA-256 compressions per connect, unless the connection or the salted password survives across analyses.
- **Repetition.** If the policy is general, every trusted metaprogram can reach the network on every keystroke. If it is scoped, the sandbox needs a way to tell "the Postgres module's describe" from any other code, which per-symbol policies cannot do.

**B. A host-side describe service written in Rust inside `jailsp`.** It is fast, with persistent connections, and has a narrow API (`describe(url, sql) -> columns`). But:

- it is a second implementation of the protocol, auth and TLS, in Rust, next to the Jai client;
- it puts Postgres-specific code (and probably a crate) in the language server;
- the build (`jaic` CLI) would either duplicate it or diverge from the editor.

**C. Trusted compile-time services: one narrow, general call, served by a long-lived `jaic -interp` worker.** Recommended.

- **The call.** The Compiler module gains `compile_time_service(service_file: string, request: string) -> (response: string, status: Service_Status)`. A service is an ordinary Jai program with a handler:

  ```jai
  // Extensions/Postgres/describe_service.jai
  #import "Extensions/Postgres";
  handle :: (request: string) -> string { /* connect (kept open), describe, return the JSON cache entry */ }
  #run Compiler.serve(handle);      // the worker's loop: framed requests on stdin, responses on stdout
  ```

- **The worker.** The host runs the service as a child process, `jaic -interp <service_file>`, using the normal native interpreter host. So the service gets real fibers, sockets, TLS, environment variables and a real clock, which is everything the build already has, and the Jai client runs unchanged. The worker stays alive and keeps its connections and schema databases warm. Its lifecycle:

  - **One worker per service file.** It is restarted when that file or anything it loads changes. It is killed when the workspace closes or after 10 minutes idle.
  - **Each request has a timeout:** 30 s in builds, with the editor's shorter wait described next.
  - **A memory cap.** An `RLIMIT_AS`/job-object limit, default 1 GiB, kills a runaway service.
  - **Crash backoff.** A crashed worker restarts after 1 s, doubling to 60 s. While it is down, calls return `.UNAVAILABLE`.

  It is a separate process, not a thread in `jailsp` with the native host, so a crash or hang in a service can never take the language server down.
- **Generic.** `jaic` and `jailsp` know nothing about Postgres. They move opaque request and response strings for whatever service file a call names. GraphQL or OpenAPI schema typing, or a protobuf registry, can use the same mechanism later.
- **Finding `jaic`.** `jailsp` looks next to its own executable first (release archives ship both), then on `PATH`. A path set in `jacket.toml` is planned. If none is found, calls return `.UNAVAILABLE`, and the caller decides whether that is worth a note.
- **Pending results in the editor.** `jailsp` memoises responses by `(service, request bytes)`. On a miss it sends the request and waits a short time (default 50 ms); a warm describe of a local database is about 2-5 ms, so most answers arrive in that window. If the reply is late, the call returns `.PENDING`. The macro then reports the result as unknown, which suppresses follow-on errors the same way a refused `#run` does today ([workspace trust](../tools/workspace-trust.md): `semantic::known`). When the response lands, `jailsp` invalidates the compiles that asked and republishes, so typing never blocks on the database.
- **In the build.** `jaic` uses the same worker for the length of a build, so a build keeps one connection, not one per query. Builds always run services, in any workspace, as they run `#run` today. Running the handler in-process is a later optimisation.
- **Same answers, different persistence.** The build and the editor send identical requests to the identical service, so they get the same answers. Only the build writes results to disk, as the committed offline cache (`db/queries/`). The editor keeps answers in memory, keyed by request bytes, and never writes the cache.
- **Scoped policy.** Ordinary `#run` code keeps the sandbox in every workspace. Only code that goes through a declared service file gets host access, in a separate process, and the contract that a request may be memoised makes repetition safe. Untrusted workspaces run no compile-time code at all, so no service is ever asked there; query results are unknown, as for any refused `#run`.
- **One general thing in the sandbox:** in trusted workspaces, `getenv` reads the real environment. It is read-only and idempotent, and it lets the macro hash `DATABASE_URL` into its request so a changed URL re-describes. `.env` files need no compiler support: the Postgres module reads them through the overlay file system like any other file.

| | A. capabilities in the sandbox | B. Rust service in `jailsp` | **C. compile-time service worker** |
| --- | --- | --- | --- |
| Warm latency per changed query | a few ms, plus interpreted auth unless handles persist | ~1 ms | ~2-5 ms (pipe plus describe) |
| Cold start | connect and auth on each miss | connect once | worker start ~0.5 s, then connect once |
| Protocol implementations | one (Jai) | **two** (Rust and Jai) | one (Jai) |
| Compiler and language-server changes | large: socket and readiness emulation per OS, fibers through inline threads, real clock, native calls, persistent handles | Postgres code and a dependency in `jailsp` | one Compiler call, a worker manager, pending results |
| TLS to a remote dev database | native mbedTLS inside the language server | needs a Rust TLS dependency | works as in the build |
| Side effects on every keystroke | any trusted metaprogram, if the policy is general | none | only declared, memoised services |
| A crash in the database path | inside the language server | inside the language server | the worker dies; `jailsp` restarts it |
| Useful beyond Postgres | yes | no | yes (GraphQL or OpenAPI schemas, protobuf registries, anything a macro wants typed from a live system) |

C keeps the Jai client as the one source of truth, gives the editor exactly what the build sees, and adds one general mechanism instead of a growing list of sandbox exceptions. B is ruled out by the duplicate protocol code. A is the right long-term direction only if the playground or ordinary metaprograms ever need networking, and it can be added later without changing the Postgres module's code, which only calls `compile_time_service`.

#### Schema changes while editing

- The macro reads the migration files through the overlay file system. They therefore land in the compile's `Reads`, and editing one, even unsaved, re-runs the queries that depend on it. The schema hash is part of every describe request.
- **Unsaved edits count.** The migrations are read through the overlay, so an edited migration takes effect on the next analysis. Editing the newest migration (the usual case) costs about 1 ms per describe batch: it is applied in the rolled-back transaction on top of the database for the migrations before it. Editing an older one applies every later migration in that transaction too, which is 40-340 ms for 30-300 migrations, still within the pending-answer path.
- The worker builds a schema database for a new prefix in the background, at most one at a time, debounced 2 s after the last migration change, so a migration being typed never builds one. The newest 3 are kept.
- **Statements that cannot run in a transaction** (`CREATE INDEX CONCURRENTLY`, `ALTER TYPE ... ADD VALUE` followed by its use, `VACUUM`) cannot be overlaid. A migration that has them is applied only by a background build, and until that is ready, describes answer from the previous schema, marked `stale_schema`, and the editor shows a hint: "checked against the schema before migration 0007; updating".
- **A migration that fails to apply** is reported on the `load_migrations` call and on the queries that need that schema: "migration 0007_add_teams.sql failed at line 3: relation "team" does not exist". `jailsp` can also publish that diagnostic on the `.sql` file's own URI, so it shows in the migration file when it is open.

#### What the feedback looks like

- **Errors with exact spans.** `ErrorResponse` carries `P`, a 1-based *character* position in the SQL as sent, and `p`/`q` for errors inside internal queries. The macro maps it back in four steps:
  1. character index to byte offset in the rewritten SQL;
  2. through the `$n`-to-hole rewrite map to an offset in the source string;
  3. through the literal's escapes, which are trivial for `#string` blocks;
  4. to a file, line and column, reported with `compiler_report`.

  The message includes the SQLSTATE, `M`, `D` and `H`. Undefined columns (`42703`) and relations (`42P01`) also suggest the nearest name from the catalog. Step 3 is userspace: `Extensions/String_Literal` re-reads the literal through the compiler's file system and maps byte `n` of its value to a line and column ([String_Literal](../extensions/string-literal.md)).
- **Argument errors** (`parameter 2 is int8 (users.id) but the argument is string`) are reported on the argument, from the typed caller nodes.
- **Completions** on `it.` come from the generated row struct. `jailsp` completes and hovers fields of a polymorphic struct whose members come from `#insert #run` (`crates/jai-language-server/tests/generated_fields.rs`).
- **Hover.** Generated fields carry doc comments (`/// users.email: text, not null`), so hover on `it.email` shows the column, its Postgres type and its nullability. Hover on the query call shows the parameter list.

#### Offline and degraded behaviour

| Situation | Editor | Build |
| --- | --- | --- |
| Database reachable | full checking | full checking; writes cache entries |
| Unreachable, committed entry matches the schema hash | types from the cache; an information-level note on the query: "not re-checked: no database at localhost:5432 (connection refused)" | uses the cache, silently |
| Unreachable, no entry or a stale one | row is unknown (no cascading errors); a warning on the query naming the reason and the fix | error, unless `JACKET_PG_OFFLINE` is unset and a database becomes reachable |
| Response still pending | row is unknown, without a warning until 2 s pass; republished on arrival | waits (per-request timeout 30 s) |
| Untrusted workspace | today's behaviour: `#run` refused, results unknown | n/a |

The worker backs off on reconnects (1 s doubling to 30 s), so a stopped database does not slow every analysis. A connect error is memoised for the backoff period.

#### Cache keys and invalidation

```
db/queries/<statement-id>.json        (committed)
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
- **Finding dependencies:** the server's own dependency list, read inside the describe transaction and rolled back with it.
  - A `SELECT` becomes `CREATE TEMP VIEW pg_temp.q AS <sql>`. DML (and anything with parameters) becomes `CREATE FUNCTION pg_temp.q(<param types>) RETURNS TABLE (<columns>) LANGUAGE sql BEGIN ATOMIC <sql>; END`, with the types from the describe that came first.
  - `pg_depend` rows of the view's rewrite rule, or of the function, list every table and column used anywhere in the statement (`where` clauses included), every function and every type.
  - Views in that list are followed through their own rewrite rules, so a query on a view depends on the view's tables.
  - The spike measured 0.3 ms for the view and 0.5 ms for the function on a local server. Table OIDs from `RowDescription` alone miss tables used only in `where`, functions and types, so they are not used.
- **`search_path`** and the server major version are part of the key. A query described on 17 may describe differently on 18.

#### Staleness check

`check` (CI) runs in five steps:

1. Migrate a fresh server.
2. Re-describe every cache entry.
3. Fail if any entry differs.
4. Fail if an entry is unused. That needs the list of statement ids actually compiled, which the build writes as `db/queries/used.txt` in check mode.
5. Fail if a compiled query has no entry.

`prepare` rewrites the entries. This is `cargo sqlx prepare --check` with dependency-aware invalidation.

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

- The `generate` command writes `db/schema.jai`, a committed and formatted file. It is generated from the migrated schema database, like `diesel print-schema` or `sqlc generate`, and contains:
  - a struct per table and view;
  - an enum per Postgres enum;
  - the notes the ORM needs (`@pg_table`, `@pg_primary_key`, foreign keys).
- A compile-time check (`#run Pg.check_schema(#file)`) compares the file's hash with the schema hash recorded in it. A stale file is a compile error saying "run the pg tool's `generate`". Like the query cache, this check needs no server.
- Optionally, **struct-first authoring as scaffolding only.** A `diff` command compares hand-edited structs in `db/schema.jai` with the schema database and *writes a proposed migration file* for you to review and edit. It never applies anything. This gives B's authoring feel for the easy 80% (add a table or a column) without making the diff engine a source of truth. It is a later phase and can stay simple, because it is only a starting point.

#### The "disconnected" concern

Raw query results and table structs stay connected under C in three ways:

1. **Same origin.** Both come from `RowDescription`/`pg_attribute` after the same migrations, mapped by the same codec table. `users.email` is `string` everywhere or nowhere.
2. **Row type identity.** When a query's columns are exactly one table's columns in order (by origin and name) and the caller can see that table's struct, the macro returns the **table struct** (`User`) instead of a new anonymous struct:
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
- **Commands.** `migrate`, `new <name>` and `status` in the pg tool wrap the same code.

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
  - any other identifier or expression from the caller (`min_posts`) becomes a parameter, carried in through a backtick and typed by the compiler;
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
  - `where = #code it.active && Pg.sql("lower(%(it.email)) = lower(%email)")` mixes a raw fragment with ORM conditions, with the same inline holes.
  - `Pg.select(db, User, from_sql = "select * from users_visible_to(%viewer)")` takes a raw `from` source.
  - Anything bigger is a raw `Pg.query`, which still returns `User` when the columns match (row type identity).
- **Writes:**
  - `Pg.insert(db, user)` and `Pg.insert_many(db, users)` use `insert ... returning *`; `Pg.copy_many(db, users)` is the COPY path, which returns no rows.
  - `Pg.update(db, User, set = #code it.display_name = name, where = #code it.id == id)` and `Pg.delete(...)` are compiled the same way as `select`.
  - No dirty tracking or unit of work, and no lazy loading.
- **JSON fields.** A `Pg.Json(T)` column can be navigated inside filters and loop bodies, with keys taken from `T`'s `@json` names and renames:
  - `it.settings.theme == "dark"` becomes `settings->>'theme' = $1`. A non-string leaf is cast from its text: `(settings->>'font_size')::int8`. Comparisons are typed against the field: `it.settings.font_size > "x"` is a compile error.
  - Nested objects and arrays chain `->`: `it.settings.layout.columns[0]` becomes `settings->'layout'->'columns'->>0`.
  - `Pg.contains(it.settings, value)` becomes `@> $1::jsonb`, `Pg.has_key(it.settings, "beta")` becomes `?`, and `Pg.json_path(it.settings, "$.tags[*] ? (@ == \"x\")")` becomes `@?`. They are `jsonb`-only, and a compile error on `json`.
  - `Json_Maybe` fields compare with `.MISSING` and `.NULL`: `it.settings.beta.state == .MISSING` becomes `not (settings ? 'beta')`, and `== .NULL` becomes `settings->'beta' = 'null'::jsonb`.
  - Column pruning extends into documents: a loop that reads only `it.settings.theme` selects `settings->'theme'`, not the whole document.
  - `Json_Raw` columns have no typed navigation; use `Pg.sql` fragments for them.
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
- `Extensions/Postgres/describe_service.jai`: the compile-time service; the only code that talks to the compile-time server.
- Compiler and language server (phase 0):
  - `call`, `unknown` and `serve` in `Extensions/Service` ([compile-time services](../extensions/service.md));
  - the worker manager (`crates/jaic/src/service.rs`) and the editor's memo (`crates/jai-language-server/src/services.rs`);
  - pending results treated as unknown (`DiagnosticKind::Unknown`);
  - string-literal sub-span locations in `Extensions/String_Literal`;
  - real `getenv` in trusted sandboxes.
- **The pg tool is userspace.** `Extensions/Postgres/Tool` provides `run_tool(args)`, with `prepare`, `check`, `generate`, `diff`, `migrate`, `new` and `status`. A project adds a three-line `pg_tool.jai` that loads its `db/postgres.jai` (below) and calls it, and runs it with `jaic -interp pg_tool.jai -- prepare`. Nothing in the jacket CLI knows about Postgres. A general `[commands]` table in `jacket.toml` (`pg = "jaic -interp pg_tool.jai --"`, giving `jacket pg prepare`) would be a separate, Postgres-agnostic jacket feature.
- Adding a type: one row in `codecs.jai`, with the binary and text codecs and the Jai type, plus round-trip cases in the conformance suite. If it has no stable binary form, register it text-only.
- Adding an ORM operator: one case in the `Code` to SQL walker in `orm.jai`, plus a test that compiles a query with it and compares the generated SQL text.
- Gotchas:
  - Changing the hole syntax, the `%`-to-`$n` rewrite, the override syntax or the description format changes every statement id and invalidates every committed cache. Bump a format version in the key.
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
| PostgreSQL's own suites: `pg_regress` through a Jai psql, the auth/SSL/libpq/psql TAP tests, `libpq_pipeline` traces | every built-in type's text I/O, error and notice layout, COPY, auth, TLS, URI parsing, pipeline message sequences | **primary**: fetched at pinned versions, run unchanged against the shims ([Conformance](conformance.md)) |
| `pgbench` | drives libpq itself; cannot drive our client | not useful for conformance; a Jai port of its TPC-B script is the **benchmark** against libpq/pgx |
| RFC 4231 / 7914 / 7677 vectors | HMAC, PBKDF2, SCRAM | unit tests |

No reusable cross-language protocol conformance suite exists. Drivers test against real servers: pgx runs PostgreSQL 14-18 with md5, SCRAM, Unix-socket and client-certificate users (and OAuth on 18); node-postgres runs 13-18.

**Pick:** PostgreSQL's own suites on real servers plus libpq differential as the conformance suite, recorded traces and fuzzing as the fast and robustness layers.

**JSON** is checked from the `json`, `jsonb`, `jsonpath`, `jsonb_jsonpath`, `json_encoding` and `sqljson*` regression files. Their text output already runs through the shim. On top of that, every `json`/`jsonb` literal they use is decoded into `Json_Value`, re-encoded as a parameter (text for `json`, binary for `jsonb`), and compared with the original on the server (`=` for `jsonb`, `::text` for `json`). Typed decoding is covered by fixtures that hold their expected values and error paths. A new `pg_conformance.py` harness, next to `http_conformance.py` in tools/stdlib, runs them from a path-filtered workflow, `pg-conformance.yml` on the same model as `http-conformance.yml`.

- **Linux (every relevant push):** GitHub `services:` containers, one job per major (`postgres:15` … `postgres:18`, plus the newest beta as allowed-to-fail). The job writes `pg_hba.conf` and `postgresql.conf` and restarts the server, to enable md5 and scram users, TLS with the Tls test CA from `stdlib/Extensions/Tls/tests/data`, client-certificate auth and `sslnegotiation=direct`. `psql` from the same image is the libpq oracle.
- **macOS and Windows (manual dispatch and nightly):** no service containers. `ikalnytskyi/action-setup-postgres` (Linux, macOS, Windows; 15-18) installs a server, as pgx's Windows job does. Once managed mode exists, also run it from the downloaded binaries (`theseus-rs/postgresql-binaries`: x64 and arm64 for Linux, macOS and Windows, with Windows arm64 from PostgreSQL 16), so that path is tested too. Unix sockets are covered on macOS/Linux; Windows uses TCP.
- **Every push, all OSes:** trace replay, codec unit tests and SCRAM vectors, inside the normal stdlib runtime tests (natively and under `-interp`, since the client must also work at compile time).

## Configuration

| Setting | Where | Meaning |
| --- | --- | --- |
| `JACKET_DATABASE_URL`, then `DATABASE_URL` (also read from `.env`) | environment | compile-time server for describing queries (`postgres://user:pass@host:port/db?sslmode=…`); also the pg tool's default. Needs `CREATEDB` for schema databases |
| `JACKET_PG_OFFLINE=1` | environment | never connect at compile time; cache misses are errors (CI) |
| service limits | Compiler module constants | editor wait before `.PENDING` (50 ms), build request timeout (30 s), worker idle exit (10 min), memory cap (1 GiB), crash backoff (1-60 s), database reconnect backoff (1-30 s) |
| `jaic` location for `jailsp` | `jacket.toml` | used only when `jaic` is neither next to `jailsp` nor on `PATH` |
| module parameters | `Pg :: #import "Extensions/Postgres"(MIGRATIONS = "db/migrations", SCHEMA = "db/schema.jai", QUERY_CACHE = "db/queries", MIN_SERVER_VERSION = 15);` | paths and modes for the compile-time path and the pg tool |
| connection options | `Pg.Config` / URL | `sslmode` (`disable`, `prefer`, `require`, `verify-ca`, `verify-full`), `sslnegotiation`, `channel_binding`, `application_name`, `connect_timeout`, `statement_timeout`, `statement_cache` (size or `.OFF`), `verify_statements` |
| pool options | `Pg.Pool_Config` | `max_size`, `min_idle`, `idle_timeout`, `max_lifetime`, `health_check_after` |

Put that import in one file (`db/postgres.jai`) that both the program and `pg_tool.jai` load, so the two always agree. There is no `jacket.toml` section.

## Dependencies

- Internal:
  - Io (TCP, the resolver, fibers, deadlines, and Unix sockets once added);
  - Tls (with peer certificate DER for channel binding);
  - `internal/crypto` (SHA-256, plus the new HMAC/PBKDF2/random);
  - `md5`, Base64, [Json](../extensions/json.md) (for `json`/`jsonb` values and cache files);
  - the `Compiler` module (`compiler_get_nodes`, `#caller_code`, typed nodes, ), plus `Extensions/Service` and `Extensions/String_Literal`.
- External at runtime: none (no libpq).
- External at compile time: a Postgres server only on cache misses.
- CI: Postgres server images and binaries, and `psql`.

## Prior art

| Library | Copy | Avoid |
| --- | --- | --- |
| **sqlx** (Rust) | Raw SQL typed by asking the server (`Describe`). `query_as!` checking user structs. Column-alias overrides (`"col!"`, `"col?"`, `"col: T"`). Committed offline cache and `prepare --check`. Nullability from `attnotnull` plus `EXPLAIN` join analysis. Migrations as SQL files with checksums. | A live database on every fresh build unless offline mode is set up. Coarse invalidation, where any change means re-prepare everything. Nullability inference that is sometimes wrong for outer joins and expressions, which overrides exist to patch. No answer for dynamic queries. A separate CLI needed to keep the cache fresh. |
| **Diesel** (Rust) | `diesel print-schema`: Rust schema generated from the migrated database and committed, which is option C. SQL-file migrations. Compile-time-checked query DSL with joins. | Trait-heavy types whose errors are famously unreadable. The DSL rewrites SQL, so anything unusual becomes `sql_query` with no typing. Long compile times. |
| **SeaORM** (Rust) | Entities generated from a live schema (`sea-orm-cli generate entity`). Relation definitions and a loader that batches `IN (...)` to avoid N+1. | Queries typed only at runtime (string columns, runtime errors). ActiveModel boilerplate. A migration DSL that duplicates SQL. |
| **jOOQ** (Java) | Code generation from the schema *after running migrations in a throwaway database* (Flyway plus Testcontainers), which is exactly the schema-database step. A DSL that is SQL, not an object model. Every Postgres feature reachable. Typed records per table. | Codegen as a separate build step that is easy to forget. Verbose DSL. Dynamic SQL everywhere by default, so nothing is checked until runtime unless you use the generated types. |
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
- **Editor:** compile-time services (option C in [Editor feedback](#editor-feedback-phase-0)). A trusted workspace runs the Postgres describe service in a long-lived `jaic -interp` worker, so raw queries are checked on the next analysis with no build. Ordinary `#run` code keeps the sandbox.

Phases (each lands on this branch, then goes to `main` when it is useful on its own; none blocks a release):

0. **Editor feedback groundwork** (compiler and language server), tested with a toy service that upper-cases its request or sleeps. Done; the spikes all worked:
   1. `compile_time_service`/`serve`, with the worker in `jaic` builds and in `jailsp` (trusted only): framed pipe protocol, restart on source change, idle exit, memoised responses.
   2. Pending results: the call returns `.PENDING` after the editor's wait, the result counts as unknown (no cascading errors), and the arrival invalidates and republishes.
   3. `.UNAVAILABLE` (no `jaic`, a sandbox, or the service says so) handled the same way; worker memory cap, request timeout and crash backoff.
   4. The source location of byte `n` of a string literal, for exact error spans (userspace, `Extensions/String_Literal`).
   5. Real `getenv` in trusted sandboxes.
   6. Spikes:
      1. a `Row($id)` struct with `#insert #run` fields, plus `#caller_code` argument types read in a macro;
      2. `jailsp` completion and hover on those generated fields;
      3. `for_expansion` body `Code` walking that finds `it.<field>`.
1. **Spike on the client side:** the temp-view and `pg_depend` dependency trick. Done; it works, and DML works through `BEGIN ATOMIC` functions ([Cache keys](#cache-keys-and-invalidation)).
2. **Client:**
   1. Unix sockets in Io; HMAC, PBKDF2 and random in `internal/crypto`. Done.
   2. Startup with TLS, MD5 and SCRAM (including `-PLUS`), simple and extended protocols, errors, core binary codecs. Done: [Postgres](../extensions/postgres.md). The runtime calls are `query_dynamic`, `query_binary` and `execute_script`, which leaves `query`/`execute` for the typed macros.
   3. Pool, transactions, cancellation. Done. Io gained `with_grace` (cleanup past a deadline or cancel) and `Condition` (a wait that honours deadlines) for it.
   4. The `pg-conformance` workflow, with PostgreSQL's regression suite through a Jai psql. Done; the TAP, URI, trace and driver-vector suites follow ([Conformance](conformance.md)).
   5. JSON: `Json(T)`, `Json_Raw`, `Json_Maybe`, binary `jsonb`, `json[]`, strictness options, error paths, borrowing decode.

   6. Pipelining (`Pipeline`, libpq's semantics, checked against `libpq_pipeline`'s traces), LISTEN/NOTIFY (queue, callback, `wait_for_notification`), the message trace and protocol 3.2 (`min`/`max_protocol_version`, negotiation, long cancel keys). Done; 3.2 itself runs only against PostgreSQL 18 in CI.

   The typed query layer (phase 3) will pipeline Parse with Bind and Execute through the same queue.
3. **Typed raw queries:** `query`/`query_one`/`execute`/`query_as`, JSON column overrides, the describe service (schema databases, nullability, overrides), the offline cache, the pg tool's `prepare`/`check`. Editor feedback lands here: error spans, completions, hover, offline notes. *Landed* ([typed-queries.md](typed-queries.md)): everything above except pipelined execution of typed queries, portal streaming (`Rows` buffers the whole result), and user-defined types in `query` alias overrides (use `query_as`). The editor test is written but unrun.
4. **Migrations and schema:** the runner, embedded migrations, the schema-database cache, `generate` (with `db/json_columns.jai`), the `db/schema.jai` staleness check, row type identity. *Landed* ([migrations.md](migrations.md), [schema.md](schema.md)): the runner, embedded migrations, the pg tool's `migrate`, `status`, `new` and `generate` (from the migrations' schema database, `db/json_columns.jai` through `Pg.json_column_type`), the `db/schema.jai` staleness check, Jai enums for Postgres enums, and the relation notes the ORM reads. The schema-database cache and row type identity landed with typed queries ([typed-queries.md](typed-queries.md#schema-databases), [row type identity](typed-queries.md#row-type-identity)).
5. **ORM:** `select` with `Code` filters, typed JSON navigation, column pruning, to-one joins, to-many batched loading, insert/update/delete. *Landed* ([orm.md](orm.md)): `select` with `Code` filters (comparisons, `like`/`ilike`/`in`/`lower`/`upper`, `if_some`, `== null`, `contains`/`has_key`/`json_path`, `Json_Maybe` states, `Pg.sql` fragments), `from_sql`, filters through to-one relations (subqueries), batched loading of both relation kinds (`load_many`, `load_one`), typed JSON navigation in filters, `order_by`/`desc`/`limit`/`offset`, `insert`/`insert_many`/`update`/`delete`, and `copy_many` over binary `copy_in` ([copy.md](copy.md)).
6. **Zero setup:** managed per-platform Postgres for compile-time typing; `diff` migration proposals. *Managed binaries landed* (`stdlib/Extensions/Postgres/managed.jai`, see [managed.md](managed.md)): download, verify, cache, `initdb` and start. The describe service uses them when `JACKET_PG_MANAGED=1` and the project has migrations (see [typed-queries.md](typed-queries.md)). *`diff` landed* (`stdlib/Extensions/Postgres/diff.jai`, see [diff.md](diff.md)): it writes a proposed migration for hand-edited structs and applies nothing.

## Decisions

1. **Source of truth:** SQL migrations, with Jai structs generated from the migrated schema and committed.
2. **Compile-time server:** `DATABASE_URL` plus the offline cache first; managed binaries later; PGlite set aside.
3. **Parameters:** inline `%name`/`%(expr)` holes, through an `@Interpolates("sql")` note that the compiler desugars and the language server understands ([Inline parameters](#inline-parameters)).
4. **Nulls:** `Nullable(T)`.
5. **Rows:** zero copy, valid for one iteration; `fetch_all` copies.
6. **Dynamic queries:** static SQL plus `if_some` optional filters, otherwise raw SQL; no runtime builder in v1.
7. **Oldest server:** PostgreSQL 15.
8. **Tooling:** all userspace (`Extensions/Postgres/Tool`, run with `jaic -interp`); nothing Postgres-specific in jacket. Generic `[commands]` in `jacket.toml` is a possible separate jacket feature.
9. **Placement:** `stdlib/Extensions/Postgres`.
10. **Schema databases** on the `DATABASE_URL` server; requiring `CREATEDB` is fine.
11. **Editor feedback through compile-time services** (option C in [Editor feedback](#editor-feedback-phase-0)), not networking inside the sandbox. C gives instant feedback in trusted workspaces without letting every trusted metaprogram reach the network on every keystroke. It keeps one protocol implementation, and isolates crashes in a separate process. The mechanism is generic, with no Postgres knowledge in `jaic` or `jailsp`.

12. **Live migration edits:** unsaved migration buffers are used. The changed suffix is applied in a rolled-back transaction on the database for the unchanged prefix, about 1 ms for the newest migration ([benchmark](#how-types-are-learned-at-compile-time)). No clones.

13. **JSON:** columns are typed by the document's Jai type (`Json(T)`), mapped per query by alias override and per column in `db/json_columns.jai`, with `Json_Raw` as the untyped fallback. JSON null, a missing key and SQL NULL stay distinct (`Json_Maybe`, `Nullable`).

## Open questions

None open.
