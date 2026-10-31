# Postgres conformance

The Postgres client is checked against PostgreSQL's own test suites, not only tests written for it. The `pg-conformance` workflow fetches the suites at pinned versions (15 to 18, plus the next major in beta) and runs them against real servers. A `managed` job runs the [managed-server](managed.md) tests on Linux, macOS and Windows.

## How it works

PostgreSQL's suites drive the server through `psql` and, for URI parsing, `libpq_uri_regress`. Both are built here as Jai programs on our client (`tests/conformance/`), put first on `PATH`, and the suites run unchanged. The server, `initdb`, `pg_ctl` and `pg_regress` stay PostgreSQL's.

Runs are differential: each suite runs with the real `psql` and with the shim, and only tests that pass with the real one count. A difference is a client bug or a shim bug; both get fixed.

| Suite | Covers |
| --- | --- |
| Regression (`src/test/regress`) | text I/O of every built-in type, error fields and positions, notices, COPY, LISTEN/NOTIFY, transactions, big results |
| Binary round trip (`roundtrip.jai`, `edge_values.sql`) | every decoded column fetched as text and binary, compared, sent back as a parameter and compared on the server; json documents too |
| Authentication TAP | SCRAM, MD5, cleartext, `require_auth`, SASLprep, peer, login triggers |
| SSL TAP | `sslmode`, root certs, host names, client certs, CRLs, channel binding |
| libpq TAP | URI and key/value parsing, multiple hosts, `target_session_attrs` |
| psql TAP | options, `\watch`, notifications, query cancellation |
| Pipeline traces (`pipeline_traces.jai`) | the exact message sequence of nine pipeline scenarios |

Not applicable: pgbench and the isolation tester link libpq (the [load test](benchmark.md) stands in for pgbench). Skipped with reasons: GSS rows, SSPI (Windows), tab completion, libpq's C API tests, DNS load balancing, replication mode. The lists are `TAP_SKIPS`/`TAP_BASIC_SKIPS` in `run.py` and `tests/conformance/regress-skip.txt`.

### The psql shim

`tests/conformance/psql.jai` is psql's non-interactive half: `scan.jai` (psql's lexer), `print.jai` (every output format), `describe.jai` (the `\d` family with psql's catalog queries) and `psql.jai` (options, variables, `\if`, `\g*`, `\copy`, pipeline commands, error layout). It follows the newest psql, with `\d` and friends following the server's version. The line editor, `\e`, `\password` and `\prompt` are left out.

### The binary round trip

For each distinct value of each column the client decodes, it fetches text and binary and checks they decode to the same value (floats by bit pattern), then sends the value back and has the server check `$1::T is not distinct from $2::T` and the same text. Types it does not decode (ranges, `inet`, `bit`, multi-dimensional arrays) are counted as skipped. `edge_values.sql` adds awkward values: integer and float limits, `NaN`, huge numerics, BC dates, infinities, every `bytea` byte, arrays with nulls and lower bounds, odd json.

## How to change it

- **A new suite:** pin its source in `tools/pg_conformance/fetch.py`, add a step to `run.py` (TAP files go in `TAP_SUITES`, or `TAP_SKIPS` with a reason).
- **A failing regression file:** `PYTHONPATH=tools python3 tools/pg_conformance/run.py --only <name>`. `--port <n>` uses a running server, `--shim-only` reuses the last real-psql results. Outputs and diffs land in `target/pg-conformance/run/<major>/`. Fix the client or the shim, never `expected/`.
- **A failing round-trip value:** `tools/pg_conformance/roundtrip.py` prints table, column, original text and what came back. New awkward values are rows in `edge_values.sql`; a new decoded type is a `case` in `check_oid`.
- **Pipeline traces:** `run.py --pipeline-traces` prints the first differing line of each trace.
- **TAP only:** `run.py --tap`, `--only <path part>`, `--shim-only`.
- **SASLprep data:** `tools/pg_conformance/gen_saslprep.py` regenerates `saslprep_data.jai`.
- **A new PostgreSQL version:** add it to `VERSIONS` in `fetch.py` with the tarball SHA-256, and to the workflow matrix.

## Configuration

`VERSIONS` in `fetch.py`; the workflow matrix in `.github/workflows/pg-conformance.yml` (a beta major is reported but allowed to fail); `regress-skip.txt`.

## Dependencies

PostgreSQL source tarballs from `ftp.postgresql.org` (nothing vendored; `tests/conformance/COPYRIGHT.postgresql` carries the notice for the catalog queries the shim reuses), a server install with `pg_regress` and server headers, a C compiler for the helpers, a valid locale, and Perl with `IPC::Run` and `Test::More` for TAP.
