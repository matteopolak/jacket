# Library comparison benchmarks

## What it is

Benchmarks that run a Jacket library next to the community libraries it replaces, on the same inputs, with [`jacket bench`](jacket-bench.md). The programs are in `benchmarks/compare/<name>/` and `tools/bench/library_compare.py` runs them. The results, the feature matrices and the method of each comparison are in the module's own page: [Json](../extensions/json.md#compared-with-the-community-libraries), [Regex](../extensions/regex.md#compared-with-the-community-libraries), [Postgres](../extensions/postgres.md#compared-with-the-community-libraries), [Hash_Map](../extensions/hash-map.md#measured-against-hash_table-and-the-community-map). The HTTP server has no comparison yet.

```
PYTHONPATH=tools python3 tools/bench/library_compare.py json --libraries jacket,jaison,contiguous --out target/compare/json.md
PYTHONPATH=tools python3 tools/bench/library_compare.py regex --libraries jacket,uniform,rematch --bench-arguments "-E 'test(log_)'"
```

## How it works

**Sandbox.** The community sources are third-party and unreviewed, so the build and the run of every comparison go inside `bench.sandbox.Sandbox` ([the bench sandbox](benchmarks.md)): no network, file reads limited to the system, the jacket install, the stdlib and `corpus/upstream`, writes limited to a scratch folder. The Postgres comparison uses the sandbox's `loopback` setting, which allows the loopback interface and nothing else; the other comparisons have no network at all. Nothing from `corpus/upstream` is copied: a benchmark imports a library by its folder name through `-import_dir` (`corpus/upstream`).

**Naming.** A benchmark is named `WORKLOAD__LIBRARY` (`decode_api__jaison`). `library_compare.py` groups the results of one workload into a row, one column per library, with the ratio to the first library named in `--libraries`. `--per-item` divides each time by the items one iteration handles (`set_items`).

**The loop.** Every comparison uses one shape: input prepared before the timer starts, `reset_timer(b)`, then a loop of `b.iterations` calls with the input and the result passed through `black_box`, so the harness adds no per-iteration cost of its own. Allocation is the same for every library of a comparison (a bump pool emptied after each call in the Json comparison; the default allocator elsewhere) and a benchmark exits when a library rejects the input or finds a different count, so a failure path is never timed.

**Database.** The Postgres comparison needs a server on `127.0.0.1:54329` (`PG_BENCH_URL` changes it) with the tables of `benchmarks/compare/postgres/setup.sql` in a database named `bench`. Start one bound to loopback, for example with Homebrew's `postgresql`:

```
initdb -D "$DATA" -U postgres && pg_ctl -D "$DATA" -o "-h 127.0.0.1 -p 54329" start
createdb -h 127.0.0.1 -p 54329 -U postgres bench
psql -h 127.0.0.1 -p 54329 -U postgres bench -f benchmarks/compare/postgres/setup.sql
```

Stop it when done (`pg_ctl -D "$DATA" stop`). The libpq side needs `libpq` (`LIBPQ_DIR` names its folder; Homebrew's is found).

## How to change it

- **A new comparison:** a folder `benchmarks/compare/<name>/benches/<name>.jai` with `WORKLOAD__LIBRARY` benchmarks, run by name through `library_compare.py`. Put a library in `LOOPBACK` (`library_compare.py`) only when it talks to a server on this machine. A shim module that only the scratch copy needs goes through `write_shims` (the Regex comparison uses one to relax a capacity assert in `Unicode` that Rematch trips; the file says which lines).
- **Check every library gives the same answer** before trusting a number: the Regex benchmarks compare their match counts with Jacket's, and the Json and Postgres benchmarks stop when a library rejects the document or a row.
- Gotchas: never run a comparison while a build or another benchmark runs (the machine has 16 GiB); the drivers print `load_average` in the report header and the numbers in the docs say what the load was. The files must pass `jacket fmt --check` and the lints (`magic_number` is allowed for `benchmarks/compare/**` in `jacket.toml`: sizes and seeds are the content).

## Configuration

| Setting | Where | Effect |
| --- | --- | --- |
| `--jacket`, `--upstream` | both drivers | the `jacket` binary (default `target/release/jacket`) and the community sources (default `corpus/upstream`) |
| `--libraries` | `library_compare.py` | columns, first one is the baseline of the ratios |
| `--bench-arguments`, `--compiler-flags` | `library_compare.py` | extra arguments for `jacket bench` (a filter, `--samples`) and for jaic |
| `PG_BENCH_URL`, `LIBPQ_DIR` | Postgres comparison | server address (default `postgres://postgres@127.0.0.1:54329/bench`) and libpq folder |

## Other overlapping libraries in the corpus

A sweep of `corpus/upstream` (183 projects) and `corpus/upstream/_modules` (links to some of them under the names programs import) for libraries that overlap with a Jacket module. The Json, Regex, Postgres and Hash_Map rows are compared; the rest are not, because they do not do the same task or are not alternatives of the module.

| Jacket module | Compared | Not compared, and why |
| --- | --- | --- |
| Json | `rluba--jaison`, `kujukuju--ContiguousJsonJai` | `kujukuju--KodaSerializer` and `n00bmind--reflector` (binary serialisation, no JSON), `segcore--jai-protobuf` (protobuf) |
| Regex | `rluba--uniform`, `Lunatoid--Rematch` | `isomorphic-tech--fuzzy-match-jai` and `sjorsdonkers--match-jai` (fuzzy matching and sum-type matching, not regular expressions) |
| Postgres | `rluba--jai-postgres` | `rluba--jai-redis` (Redis, no Jacket module), `rezich--SQLite` and `isomorphic-tech--jai-sqlite` (SQLite bindings, an embedded database) |
| Http server | none yet (`rluba--hyperserve`, `smari--jai-simplehttp` and `farzher--Jai-HTTP-Server`, which has no macOS backend, are the candidates) | `smari--Cavy` (a toy daemon its author advises against using), `rluba--cluster` (starts and restarts copies of a process, shares a listening socket; not a server) |
| Hash_Map | `666rayen999--hash_map` (does not give correct answers, see the page) | |
| Testing | none yet | `rluba--stubborn` (matchers), `n00bmind--unotest`, `dbechrd--jai-assertive` (assertions) |
| Io fibers | none | `sjorsdonkers--yield-jai` (generators), `kujukuju--JaiParallel` (threads) |
| Jai_Format | none | `OrangeLightning219--jai-format` (a formatter with its own config) |
| Deflate | none | `kujukuju--lz4_static` (LZ4, a different codec) |
| SDL3, WebGPU | none | `isomorphic-tech--jai-sdl3`, `isaplane--sdl3-jai`, `SogoCZE--jai_wgpu_native` (bindings of the same C libraries) |
| no Jacket module | | `sjorsdonkers--toml-jai`, `smari--jai-xml`, `rluba--jai-csv`, `rluba--jai-date`, `smari--jai-crypto` |

## Dependencies

`bench.sandbox` (`tools/bench/sandbox.py`), `jacket bench` and `Extensions/Testing` (`Bench`, `black_box`), the native libraries of `JAIC_NATIVE_LIBS` for the programs that use TLS, `libpq` and a local PostgreSQL for the database comparison, and `corpus/upstream` ([upstream corpus](upstream-corpus.md)).
