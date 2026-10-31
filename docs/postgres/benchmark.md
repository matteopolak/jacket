# Postgres load test

`stdlib/Extensions/Postgres/tests/bench/tpcb.jai` runs pgbench's TPC-B transaction through the client: fibers sharing one `Pool`, each running transactions back to back. pgbench links libpq, so it cannot drive this client; this is the comparable load.

```sh
jaic stdlib/Extensions/Postgres/tests/bench/tpcb.jai -exe tpcb -optimized
DATABASE_URL="postgres://postgres@localhost/bench?host=/tmp" ./tpcb -clients 8 -seconds 10 -scale 1
pgbench -h /tmp -n -c 8 -j 1 -T 10 -M extended bench      # the same load through libpq
./tpcb -clients 8 -seconds 10 -scale 1 -no-load -prepared # against pgbench -M prepared
```

## How it works

- It creates the four `pgbench_*` tables with pgbench's row counts, then `vacuum analyze`. `-no-load` reuses them.
- Each client picks random `aid`, `bid`, `tid`, `delta` and sends `begin`, three updates, a select, an insert and `end` as separate extended-protocol statements, like `pgbench -M extended`. `-prepared` uses the statement cache (`query_cached`), like `-M prepared`.
- It reports transactions, failures, tps and average latency, and exits 1 on any failure.

Compare with `-j 1`, since the client runs all fibers on one thread. At scale 1 every transaction updates the single branch row, so the run measures lock hand-off; a larger scale removes that.

## How to change it

The transaction is the `statements` table in `run_transaction`; keep it in step with pgbench's `tpcb-like` script. To measure typed queries, swap `query_binary` for the typed calls.

## Configuration

`DATABASE_URL`, `-clients`, `-seconds`, `-scale`, `-no-load`, `-prepared`. It drops and recreates `pgbench_*` tables: use a scratch database.
