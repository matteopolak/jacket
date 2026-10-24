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
| Authentication TAP | `src/test/authentication/t/001_password.pl`, `002_saslprep.pl` | SCRAM, MD5, cleartext, `require_auth`, SASLprep of passwords | Applies |
| SSL TAP | `src/test/ssl/t/001_ssltests.pl`, `002_scram.pl` | `sslmode`, `sslrootcert`, host name checks, client certificates, CRLs, SCRAM channel binding | Applies. Client certificates, CRLs and `sslsni` are client features still to add. |
| libpq TAP | `src/interfaces/libpq/t/001_uri.pl` | connection URI and key/value parsing | Applies through the `libpq_uri_regress` shim |
| libpq TAP | `005_negotiate_encryption.pl` | the `sslmode` and `sslnegotiation` matrix | Applies (GSS rows do not) |
| libpq TAP | `003_load_balance_host_list.pl` | multiple hosts, `load_balance_hosts` | Applies once multi-host config exists |
| psql TAP | `src/bin/psql/t/020_cancel.pl` | query cancellation from a signal | Applies (the shim maps `SIGINT` to `Io.cancel`) |
| Pipeline traces | `src/test/modules/libpq_pipeline/traces/*.trace` | the exact message sequence of pipelines, errors inside them, `Sync` handling | Vectors: compared with our client's own trace |
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
  `tools/pg_conformance/run.py`, and a row to the table above.
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
