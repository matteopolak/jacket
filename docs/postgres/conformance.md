# Postgres conformance

## What it is

The Postgres client (`stdlib/Extensions/Postgres`) is checked against PostgreSQL's own test
suites, not only tests written for it. The suites are fetched at pinned versions and run against
real servers in the `pg-conformance` workflow. Every failure that applies is a client bug.

The same workflow runs the module's own server-backed suites (typed queries and the ORM, the `pg`
tool's schema commands, the editor's typed-query diagnostics) against each version's cluster, and a
`managed` job that runs the managed-server tests, including a typed-query build with no database
URL, on Linux, macOS and Windows ([managed.md](managed.md)).

## How it works

PostgreSQL's suites drive the server through two programs: `psql` and, for URI parsing,
`libpq_uri_regress`. We build both as Jai programs on our client (`tests/conformance/`), put
them first on `PATH` (or pass them with `pg_regress --bindir`), and run the suites unchanged. The
real server, `initdb`, `pg_ctl` and `pg_regress` stay the PostgreSQL ones.

The suites expect exact output: psql's aligned tables, its error layout (`LINE 1:` and the
caret), libpq's connection error texts. The shims print the same, so a difference is either a
client bug or a shim bug, and both get fixed.

Every run is differential. The same suite runs once with the real `psql` and once with the shim,
on the same server build. Only tests that pass with the real `psql` count, so locale, platform and
extension differences in the environment do not show up as client failures.

## The suites

| Suite | Where in the source | What it covers | Status |
| --- | --- | --- | --- |
| Regression | `src/test/regress` (`parallel_schedule`, `sql/`, `expected/`) | text I/O of every built-in type (including `json`, `jsonb`, `jsonpath`), errors with every field and position, notices, COPY in and out, LISTEN/NOTIFY, transactions, multiple statements, big results | Applies. The shim covers the meta-commands in the table below. |
| Binary round trip | the tables the regression run leaves behind, plus `edge_values.sql` | binary codecs: every value is fetched in text and in binary, decoded, compared, then sent back as a parameter and compared on the server | Ours: `tools/pg_conformance/roundtrip.py`, in the workflow after the regression step |
| Authentication TAP | `src/test/authentication/t/001_password.pl`, `002_saslprep.pl`, `003_peer.pl`, `004_file_inclusion.pl`, `006_login_trigger.pl` | SCRAM, MD5, cleartext, `require_auth`, password files, SASLprep of passwords, peer authentication, login event triggers | Passes: 178, 20, 53, 2 and 32 tests. `005_sspi.pl` is Windows only. |
| SSL TAP | `src/test/ssl/t/001_ssltests.pl`, `002_scram.pl` | `sslmode`, `sslrootcert`, host name checks, client certificates, CRLs, SCRAM channel binding | Applies. `001_ssltests.pl`: 201 of 205 tests (the real `psql` passes 200; the four others are the suite's own TODO skips); `002_scram.pl`: 21 of 24, the three left being the RSASSA-PSS certificate test, which mbedTLS cannot do (`TAP_SSL_SKIPS` in `run.py`). |
| libpq TAP | `src/interfaces/libpq/t/001_uri.pl` | connection URI and key/value parsing | Passes (60 tests) through the `libpq_uri_regress` shim |
| libpq TAP | `003_load_balance_host_list.pl` | several hosts, the next one after a failure, `load_balance_hosts=random`, `target_session_attrs` | Applies: `connect` tries the hosts in order or shuffled (not run locally; needs a PostgreSQL source tree) |
| libpq TAP | `004_load_balance_dns.pl`, `005_negotiate_encryption.pl` | one name with several addresses; the `sslmode` and `sslnegotiation` matrix | Skipped: the client tries the first address of a name only; the matrix has GSS encryption rows |
| psql TAP | `src/bin/psql/t/020_cancel.pl` | query cancellation from a signal | Passes (2 tests): the shim turns `SIGINT` into a cancel request (`cancel_query`) |
| psql TAP | `src/bin/psql/t/001_basic.pl` | options, `\timing`, `\watch`, notifications, `\errverbose`, `--single-transaction`, `\g` into a program, `\restrict` | 105 of 112 tests; the rest are listed below |
| Pipeline traces | `src/test/modules/libpq_pipeline/traces/*.trace` and `libpq_pipeline.c` | the exact message sequence of pipelines, errors inside them, `Sync` handling, prepared statements, single-row and chunked results | Applies. `tests/conformance/pipeline_traces.jai` replays all nine scenarios and matches every trace line for line on 17.11 (`run.py --pipeline-traces`; PostgreSQL 17 and later). |
| pgbench | `src/bin/pgbench` | load | Does not apply: pgbench links libpq. Its built-in TPC-B script runs on our pool as a load test. |
| Isolation | `src/test/isolation` | concurrency | Does not apply: the tester links libpq. |

### The psql shim

`tests/conformance/psql.jai` is psql's non-interactive half on our client:

- `scan.jai`: psql's lexer (statement splitting, variables, backslash arguments);
- `print.jai`: every output format (aligned, wrapped, unaligned, CSV, HTML, AsciiDoc, LaTeX,
  troff-ms), expanded or not, and tables printed in pieces for `FETCH_COUNT`;
- `describe.jai`: the `\d` family, `\l`, `\z`, `\lo_list`, `\sf` and `\sv`, with psql's
  catalog queries for servers 15 and later;
- `psql.jai`: options, variables and their hooks, `\if`, `\g` and its options, `\gset`,
  `\gexec`, `\gdesc`, `\bind`, `\crosstabview`, `\copy`, the large-object commands, `\o`,
  `\i`, `\!`, error and notice layout, `ON_ERROR_ROLLBACK` and `AUTOCOMMIT`.

Interactive features (the line editor, `\e`, `\password`, `\prompt`, `\watch`) are left out:
the suites never use them. `FETCH_COUNT` prints in pieces only in the aligned layouts; the others
print the whole result at the end.

The shim follows the newest psql it is run against (18): `\bind_named`, `\parse`,
`\close_prepared` and pipeline mode (`\startpipeline`, `\endpipeline`, `\syncpipeline`,
`\sendpipeline`, `\flush`, `\flushrequest`, `\getresults` and the `PIPELINE_*` variables) run on
the client's pipeline API; `\pset`, `\d` and the other listings follow the server's major
version, so 15, 16, 17 and 18 all print what their own psql prints.

Every regression file that passes with the real psql passes with the shim on 15, 16, 17 and 18.
Files that ever need what the shim does not do are listed in `tests/conformance/regress-skip.txt`, each with
its reason.

### The TAP suites

`python3 tools/pg_conformance/run.py --tap` runs them. For each file it puts a bin directory first
on `PATH` (the shim `psql` and `libpq_uri_regress`, then the server's programs), sets `PG_REGRESS`
and runs `prove` with the fetched `src/test/perl` and IPC::Run on `PERL5LIB`. IPC::Run is taken
from perl when it has it, else downloaded from CPAN (pinned by digest) into `target/pg-conformance`.
The run is differential as for the regression suite: each file runs with the real programs and with
the shims, and a test that passes with the real ones must pass with the shims. `libpq_uri_regress`
is not installed with PostgreSQL, so `001_uri.pl` runs only with the shim and its own expectations
are the oracle. `--only <part of a path>` picks files; `--shim-only` skips the real run. Logs are
under `target/pg-conformance/run/<major>/tap/<shim|real>/<file>/`.

Not run (`TAP_SKIPS` in `run.py`): `005_sspi.pl` (Windows), `010_tab_completion.pl` (a terminal and
a line editor), `002_api.pl` (libpq's C API), `004_load_balance_dns.pl` (the client tries the first address of a
name only), `005_negotiate_encryption.pl` (GSS rows).

Tests of `001_basic.pl` that do not apply (`TAP_BASIC_SKIPS`): `\help` (the shim has no SQL
command help text) and the replication-mode response (`unexpected PQresultStatus: 8`: the client does
not speak CopyBoth). The tab completion file and
`interactive_psql` features beyond prompts and `\password` are interactive and left out; typed
passwords are not hidden.

The shim reads its input as it arrives, shows `PROMPT1` and `PROMPT2` on a terminal, and takes a
connection string as the database name (`-d`, `\c`), as psql does.

### Driver suites

Mature drivers' suites (pgx and its `pgproto3`, pgjdbc, psycopg, rust-postgres, npgsql) test the
driver they ship with: none can be pointed at another client. What we can reuse from them is
narrower:

- pgx's `pgtype` tables of values and their binary encodings, as codec vectors;
- psycopg's and npgsql's lists of awkward values (infinities, out-of-range dates, `NaN` numerics,
  empty arrays and arrays with lower bounds).

These go into the binary round-trip test as extra SQL, `tests/conformance/edge_values.sql`: every
value there is written for this suite, from those lists' ideas (its header says so). It covers
integer limits, `-0.0` and the float limits, `NaN` and infinities of floats and numerics, numerics
with a huge weight, a huge scale and a negative scale, BC dates and the year 294276 and 5874897
limits, the `infinity` values of dates, timestamps and (from 17) intervals, intervals of both signs,
every byte of a `bytea`, Unicode text, empty arrays, arrays with NULLs and with lower bounds, and
json documents with duplicate keys, deep nesting and escapes.

### The binary round trip

`tests/conformance/roundtrip.jai` takes every column of every type the client decodes, in the
tables of the schemas it is given, and for each distinct value:

1. fetches the column in text (`query_dynamic`) and in binary (`query_binary`) and decodes both
   into the Jai type (`bool`, `s16`... `Pg.Numeric`, `Pg.Interval`, `[] Pg.Nullable(T)` for
   arrays...); the two must be the same value (floats by their bits, so `-0.0` differs from `0.0`);
2. sends the decoded value back as a parameter, with the original text as a second one, and has
   the server check `$1::T is not distinct from $2::T` and that both print the same text (so a
   numeric's scale and `-0` count). Arrays are compared through `to_jsonb`, and their text only when
   the original has no lower bounds, which a Jai slice cannot hold;
3. for `json` and `jsonb`: decodes the document into a `Json_Value` from both formats, writes it
   back with `Pg.json`, and has the server compare it with the original (`jsonb` equality and
   text; for `json` the jsonb forms, when the text has one).

A column of another type (ranges, `inet`, `bit`, enums, arrays of domains...) is listed as not
decoded. Multi-dimensional arrays are counted as skipped: the client only reads one dimension.

`tools/pg_conformance/roundtrip.py` runs it. A temporary cluster runs the regression schedule with
the real `psql` (its tables stay in the `regression` database), then `roundtrip.jai` checks every
non-system schema plus `edge_values.sql`, and every `json`/`jsonb` literal harvested from the
`json`, `jsonb`, `jsonpath`, `jsonb_jsonpath`, `json_encoding`, `jsonpath_encoding` and `sqljson*`
regression files (string literals cast to `json`/`jsonb`, or that look like a document; the
server's refusals, the invalid ones in those files, are counted and skipped). The session time zone
is `Europe/Amsterdam`, so `timestamptz` text carries the odd offsets of old local mean time.

On 17.11 it checks about 157,000 distinct values (1.3 million fetched) and 390 JSON documents with
no failure. Two known limits are counted, not failed: a `json` text with a lone surrogate escape
(`"\ud83d"`) cannot be a `Json_Value` string and is read as `Json_Raw`, and `json` text with a
`\u0000` escape has no `jsonb` form to compare with.

### Results by version

Run locally against the managed binaries (`stdlib/Extensions/Postgres/managed.jai` pins 15.19.1,
16.15.1, 17.11.1 and 18.6.1; `pg_regress` and its `regress` library come from the archive's
`lib/pgxs` and `include/server`, and `run.py` builds the library as `regress.so` up to 15 and
`regress.dylib` from 16 on macOS):

| Suite | 15.19 | 16.15 | 17.11 | 18.6 |
| --- | --- | --- | --- | --- |
| Regression (shim / real psql) | 217/217 | 220/220 | 225/225 | 231/231 |
| Pipeline traces | n/a | n/a | all ok | all ok |
| Binary round trip | ok | ok | ok | ok |
| `tests/server.jai` | ok | ok | ok | ok (incl. protocol 3.2) |
| TAP: `001_uri` | 57/57 | 60/60 | 60/60 | 63/63 |
| TAP: authentication files | all pass | all pass | all pass | all pass |
| TAP: `001_basic` (rest are listed skips) | 60/68 | 79/87 | 105/112 | 124/131 |
| TAP: `020_cancel` | 2/2 | 2/2 | 2/2 | 2/2 |

Protocol 3.2 (PostgreSQL 18 only): `test_protocol_versions` connects with `max_protocol_version =
.V3_2`, expects protocol minor 2 and a 32-byte cancel key, and cancels through it. Before 18 the
same connection settles on 3.0 with a 4-byte key.

Version-dependent texts: libpq 18 words `require_auth=!scram-sha-256` failures as "SCRAM-SHA-256
authentication" where earlier ones say "SASL"; the client follows 18 and `TAP_PASSWORD_SKIPS`
excludes the four older expectations. psql before 17 declares a cursor for `FETCH_COUNT`, which
shifts the error's `LINE`; the shim uses chunked mode, so that `\errverbose` test is skipped
before 17. TAP files a version does not ship (`003_peer` before 16, `006_login_trigger` before 17)
print "not in PostgreSQL <version>".

## How to change it

- **A new suite:** add its pinned source to `tools/pg_conformance/fetch.py`, a runner step to
  `tools/pg_conformance/run.py` (TAP files go in `TAP_SUITES`, or `TAP_SKIPS` with a reason), and a
  row to the table above.
- **SASLprep data:** `python3 tools/pg_conformance/gen_saslprep.py` regenerates
  `stdlib/Extensions/Postgres/saslprep_data.jai`.
- **A pipeline trace that differs:** `python3 tools/pg_conformance/run.py --pipeline-traces
  [--port <n>]` builds `pipeline_traces.jai`, starts a temporary cluster (or uses the running
  one) and prints the first differing line of each trace, libpq's and ours. The scenarios follow
  `libpq_pipeline.c` (fetched with the traces) step by step; the traces only hold messages, so
  the test also checks each step's result. Our trace is the connection's `set_trace` output in
  regress mode (`docs/extensions/postgres.md`).
- **A failing regression file:** run `python3 tools/pg_conformance/run.py --only <name>`
  (add `--port <n>` to use a running server instead of temporary clusters, and `--shim-only` to
  reuse the last real-psql results). Both outputs and the diffs stay under
  `target/pg-conformance/run/<major>/`; each run's `summary.txt` holds pg_regress's report,
  since it deletes `regression.out` when everything passes. Fix the client or the shim. Never
  edit `expected/`.
- **A failing round-trip value:** `python3 tools/pg_conformance/roundtrip.py` prints the
  table, column, original text and what was sent back. It builds `roundtrip.jai` and keeps the
  JSON literals under `target/pg-conformance/roundtrip/<major>/`. `--skip-regress` checks the
  edge values and literals only; `--only int4 numeric` shortens the regression run. To run it by
  hand against a server: `./roundtrip --database <db> --schema '*' --script edge_values.sql`, with
  `PGHOST`, `PGPORT` and `PGUSER` set. A new awkward value is a row in `edge_values.sql`; a new
  decoded type is a `case` in `check_oid`.
- **The C helpers:** `run.py` compiles `regress.c` and contrib/spi's `autoinc.c` and `refint.c`
  (with `REFINT_VERBOSE`, as its Makefile does) against the server's headers into the run's
  `lib/`, which pg_regress gets as `--dlpath`.
- **A new PostgreSQL version:** add it to `VERSIONS` in `fetch.py` with the tarball's SHA-256,
  and to the workflow matrix.

## Configuration

- `VERSIONS` in `tools/pg_conformance/fetch.py`: PostgreSQL versions and source tarball digests.
- The workflow matrix in `.github/workflows/pg-conformance.yml`: the server versions tested
  (15 to 18), plus the next major while it is in beta (19, from its release candidate), which is
  reported but allowed to fail (`beta: true`). When that major is released, pin its release in
  `VERSIONS`, move it into the main list and drop `beta`.
- `tests/conformance/regress-skip.txt`: regression files that do not apply yet, with reasons.

## Dependencies

- PostgreSQL source tarballs from `ftp.postgresql.org`. The PostgreSQL License lets us download
  and run them; no test file is vendored into this repository. The shim follows psql's sources
  and reuses its catalog queries, so `tests/conformance/COPYRIGHT.postgresql` carries that
  license's notice.
- A PostgreSQL server install with `pg_regress` (it ships in PGXS) and its server headers
  (`postgresql-server-dev-<major>` on Debian), plus a C compiler for the helpers.
- A valid locale: macOS postmasters refuse to start without one, so `run.py` sets
  `en_US.UTF-8` when `LC_ALL` is empty.
- Perl with `IPC::Run` and `Test::More`, for the TAP suites.
