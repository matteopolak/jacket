# Postgres conformance

## What it is

The Postgres client (`stdlib/Extensions/Postgres`) is checked against PostgreSQL's own test
suites, not only tests written for it. The suites are fetched at pinned versions and run against
real servers in the `pg-conformance` workflow. Every failure that applies is a client bug.

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
| Binary round trip | the same SQL | binary codecs: every result is fetched again in binary and compared with the text decode | Ours, driven by the regression SQL |
| Authentication TAP | `src/test/authentication/t/001_password.pl`, `002_saslprep.pl`, `003_peer.pl`, `004_file_inclusion.pl`, `006_login_trigger.pl` | SCRAM, MD5, cleartext, `require_auth`, password files, SASLprep of passwords, peer authentication, login event triggers | Passes: 178, 20, 53, 2 and 32 tests. `005_sspi.pl` is Windows only. |
| SSL TAP | `src/test/ssl/t/001_ssltests.pl`, `002_scram.pl` | `sslmode`, `sslrootcert`, host name checks, client certificates, CRLs, SCRAM channel binding | Applies. Client certificates, CRLs and `sslsni` are client features still to add. |
| libpq TAP | `src/interfaces/libpq/t/001_uri.pl` | connection URI and key/value parsing | Passes (60 tests) through the `libpq_uri_regress` shim |
| libpq TAP | `005_negotiate_encryption.pl` | the `sslmode` and `sslnegotiation` matrix | Applies (GSS rows do not); with the SSL suites |
| libpq TAP | `003_load_balance_host_list.pl`, `004_load_balance_dns.pl` | multiple hosts, `load_balance_hosts` | Skipped: the config holds every host but connecting uses the first |
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

On 17.11 every regression file that passes with the real psql passes with the shim. Files that
ever need what the shim does not do are listed in `tests/conformance/regress-skip.txt`, each with
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
a line editor), `002_api.pl` (libpq's C API), `003`/`004_load_balance_*.pl` (the client connects to
the first host only), `005_negotiate_encryption.pl` (with the SSL suites).

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

These go into the binary round-trip test as extra SQL, with a note of where each came from.

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
- **The C helpers:** `run.py` compiles `regress.c` and contrib/spi's `autoinc.c` and `refint.c`
  (with `REFINT_VERBOSE`, as its Makefile does) against the server's headers into the run's
  `lib/`, which pg_regress gets as `--dlpath`.
- **A new PostgreSQL version:** add it to `VERSIONS` in `fetch.py` with the tarball's SHA-256,
  and to the workflow matrix.

## Configuration

- `VERSIONS` in `tools/pg_conformance/fetch.py`: PostgreSQL versions and source tarball digests.
- The workflow matrix in `.github/workflows/pg-conformance.yml`: the server versions tested
  (15 to 18).
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
