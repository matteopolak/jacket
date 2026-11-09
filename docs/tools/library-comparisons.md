# Library comparison benchmarks

Benchmarks that run a Jacket library next to the community libraries it replaces, on the same inputs, with [`jacket bench`](jacket-bench.md). Programs live in `benchmarks/compare/<name>/`; `tools/bench/library_compare.py` runs them. Results and feature matrices are on each module's page: [Json](../extensions/json.md#compared-with-the-community-libraries), [Regex](../extensions/regex.md#compared-with-the-community-libraries), [Postgres](../extensions/postgres.md#compared-with-the-community-libraries), [Hash_Map](../extensions/hash-map.md#measured-against-hash_table-and-the-community-map), [Protobuf](../extensions/protobuf.md#compared-with-jai-protobuf-and-c), [Csv](../extensions/csv.md#compared-with-the-community-libraries). The [Http server](../extensions/http-server.md#compared-with-the-community-servers) comparison uses `tools/bench/http_compare.py`.

```sh
PYTHONPATH=tools python3 tools/bench/library_compare.py json --libraries jacket,borrow,jaison,contiguous --out target/compare/json.md
PYTHONPATH=tools python3 tools/bench/library_compare.py regex --libraries jacket,uniform,rematch --bench-arguments "-E 'test(log_)'"
```

## How it works

- **Sandbox.** The community sources are third-party and unreviewed, so every build and run goes inside `bench.sandbox.Sandbox`: no network, limited reads, writes to a scratch folder. Postgres uses the `loopback` setting. Libraries are imported from `corpus/upstream` through `-import_dir`, never copied.
- **Naming.** Benchmarks are called `WORKLOAD__LIBRARY` (`decode_api__jaison`). The driver makes one row per workload, one column per library, with ratios to the first `--libraries` entry. `--per-item` divides by `set_items`.
- **The loop.** Input is prepared before the timer, then `reset_timer(b)`, then a loop with input and result through `black_box`. Allocation is the same for every library, and a benchmark exits when a library rejects the input or returns a different count, so a failure path is never timed.
- **Database.** Postgres needs a server on `127.0.0.1:54329` (`PG_BENCH_URL` changes it) with a `bench` database loaded from `benchmarks/compare/postgres/setup.sql`, and `libpq` (`LIBPQ_DIR`).

```sh
initdb -D "$DATA" -U postgres && pg_ctl -D "$DATA" -o "-h 127.0.0.1 -p 54329" start
createdb -h 127.0.0.1 -p 54329 -U postgres bench
psql -h 127.0.0.1 -p 54329 -U postgres bench -f benchmarks/compare/postgres/setup.sql
```

## How to change it

- **New comparison:** add `benchmarks/compare/<name>/benches/<name>.jai` with `WORKLOAD__LIBRARY` benchmarks. List a library in `LOOPBACK` only when it talks to a local server. A shim only the scratch copy needs goes through `write_shims`.
- Check that every library gives the same answer before trusting a number.
- Don't run a comparison while a build or another benchmark runs. The report header prints the load average.

## Configuration

- `--jacket` (default `target/release/jacket`), `--upstream` (default `corpus/upstream`), `--libraries`, `--bench-arguments`, `--compiler-flags`.
- `PG_BENCH_URL`, `LIBPQ_DIR` for the Postgres comparison.

## Dependencies

`tools/bench/sandbox.py`, `jacket bench`, `Extensions/Testing`, the [upstream corpus](upstream-corpus.md), and for Postgres a local server and `libpq`.
