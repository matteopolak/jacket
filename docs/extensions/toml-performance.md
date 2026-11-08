# TOML performance

How the [TOML module](toml.md) is measured, what it costs today, and how to rerun it.

## Method

- Same compiler and flags for every Jai client: `-O2 -no_debug_info`.
- Inputs come from `tools/stdlib/toml/workloads.py` (nine valid files from 155 B to 10 MiB, including the repository `jacket.toml`). Every result is checked for the same semantic output first.
- Comparators: the Rust `toml-oracle` example, the pinned community Jai parser and Python.
- Warm runs: three unscored warm-up rounds, then twelve scored rounds per task and input. Report medians of paired samples, not ratios of medians.
- Tasks: read (materialise and free the tree), lookup (query every root key), write (serialise a prebuilt tree), plus whole-formatter config load.
- Do not time anything while another build runs. When the machine is shared, alternate old and new runs and report medians with the load average.

## Current results

Apple M5, 16 GiB, warm, median microseconds, read task (parse plus free):

| Input | Jai | Rust |
| --- | ---: | ---: |
| config-1024 (1 KiB) | 9.1 | 7.8 |
| repository-config (15 KB) | 73.7 | 64.1 |
| records-102400 | 982 | 1020 |
| wide-1048576 | 13,750 | 14,360 |
| wide-10485760 | 150,060 | 153,950 |

Lookups of every root key are faster than Rust on all inputs (about 0.03 µs). Typed decoding ([toml-typed.md](toml-typed.md)) skips the tree and is what `jaifmt` uses to read its configuration; a 15 KB config loads about 20% faster than through the tree.

Writing a table built by hand (no lookup index) checks duplicate keys with a per-write hash set; 150k keys take about 5 ms instead of 27 ms. Tables built with `toml_insert` or parsed carry an index and skip the check.

## Rerun

```sh
python3 tools/stdlib/toml/prepare_community.py --archive /tmp/toml-jai.tar.gz --output /tmp/toml-imports
python3 tools/stdlib/toml/build_oracle.py --target-dir /tmp/toml-target --receipt /tmp/toml-oracle.json
python3 tools/stdlib/toml/workloads.py --output /tmp/toml-inputs
python3 benchmarks/compare/toml/measure.py --scratch /tmp/toml-bench --jaic /path/to/jaic \
  --community-dir /tmp/toml-imports --community-archive /tmp/toml-jai.tar.gz --build
python3 benchmarks/compare/toml/measure.py --scratch /tmp/toml-bench --rust /tmp/toml-target/release/examples/toml-oracle \
  --rust-receipt /tmp/toml-oracle.json --workloads /tmp/toml-inputs --invalid-workloads /tmp/toml-inputs/invalid --verify-only
```

Drop `--verify-only` for `--pilot`, then `--freeze-plan` and `--batch-plan <plan> --run` for a full comparison. For two builds of this module, use `benchmarks/compare/toml/targeted-compare.py` with `--control` and `--candidate` build receipts. `benchmarks/compare/toml/formatter_batch.jai <file> <iterations>` times the `jaifmt` config load.
