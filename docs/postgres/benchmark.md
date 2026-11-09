# Postgres load test

`stdlib/Extensions/Postgres/benches/tpcb.jai` runs pgbench's TPC-B transaction through eight fibers sharing one `Pool`. Each benchmark sample starts a managed scratch server, loads scale-1 tables and stops it afterwards.

```sh
jacket bench --path stdlib/Extensions/Postgres
jacket bench --path stdlib/Extensions/Postgres tpcb_prepared
```

## How it works

Setup creates the four `pgbench_*` tables with pgbench's row counts, then runs `vacuum analyze`. Each iteration runs one transaction on each of eight clients. A transaction sends `begin`, three updates, a select, an insert and `end` as separate extended-protocol statements. `tpcb_prepared` uses the statement cache; `tpcb_simple` uses unnamed statements. Errors fail the case. The runner reports time and transactions per second.

At scale 1 every transaction updates the single branch row, so the run measures lock hand-off. The scheduler runs on one thread.

## How to change it

Keep the `statements` table in `run_transaction` in step with pgbench's `tpcb-like` script. The fixture constants control client count and scale. Keep server/table initialization before `reset_timer`.

## Configuration

Jacket benchmark flags control sampling. `JACKET_OUTPUT_DIR` relocates the scratch cluster from `target/tpcb-bench`. No external database or `DATABASE_URL` is used.

## Dependencies

`Extensions/Postgres` managed-server support downloads its pinned PostgreSQL binaries when absent. `Extensions/Io` supplies fibers and pooling.
