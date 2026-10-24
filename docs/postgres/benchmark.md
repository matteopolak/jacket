# Postgres load test

## What it is

`stdlib/Extensions/Postgres/tests/bench/tpcb.jai` runs pgbench's built-in TPC-B transaction through the client: fibers sharing one `Pool`, each running transactions back to back. It is the benchmark the design names against libpq ([design](design.md#testing-against-real-behaviour)); pgbench itself links libpq, so it cannot drive this client.

## How it works

- **Load.** It creates the four `pgbench_*` tables with pgbench's row counts per scale step (1 branch, 10 tellers, 100,000 accounts) using `generate_series`, then runs `vacuum analyze` as its own statement (a multi-statement script runs in one transaction, which VACUUM refuses). `-no-load` reuses the tables.
- **Run.** `-clients N` fibers each pick random `aid`, `bid`, `tid` and `delta` and send `begin`, the three updates, the select, the insert and `end` as separate extended-protocol statements (`query_binary`), so the round trips are those of `pgbench -M extended`. With `-prepared` they go through the connection's statement cache (`query_cached`) instead: Bind and Execute only after the first run, as `pgbench -M prepared`. A failed statement rolls back.
- **Report.** Transactions, failures, tps and the average latency. Exit status 1 if any transaction failed.

```sh
jaic stdlib/Extensions/Postgres/tests/bench/tpcb.jai -exe tpcb -optimized
DATABASE_URL="postgres://postgres@localhost/bench?host=/tmp" ./tpcb -clients 8 -seconds 10 -scale 1
pgbench -h /tmp -n -c 8 -j 1 -T 10 -M extended bench    # the same load through libpq, on one thread
./tpcb -clients 8 -seconds 10 -scale 1 -no-load -prepared  # against pgbench -M prepared
```

Compare with `-j 1`: the client runs every fiber on one thread. At scale 1 every transaction updates the single branch row, so the run measures how quickly the lock holder's next statement goes out. A larger scale removes that contention.

## How to change it

- The transaction is the `statements` table in `run_transaction`. Keep it in step with pgbench's `tpcb-like` script so the numbers compare.
- To measure typed queries, replace `query_binary` with the typed calls; they use the statement cache like `-prepared`.

## Configuration

`DATABASE_URL`, plus `-clients`, `-seconds`, `-scale`, `-no-load` and `-prepared`. It drops and recreates `pgbench_*` tables, so point it at a scratch database.

## Dependencies

`Extensions/Postgres` (pool, `query_binary`, `execute_script`), `Extensions/Io` (fibers), a PostgreSQL server.
