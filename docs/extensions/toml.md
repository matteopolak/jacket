# TOML

`Extensions/Toml` is an in-memory TOML 1.1 parser and semantic writer, plus a typed path (`from_toml`, `to_toml`) that maps structs without building a document. It has no null value. It keeps semantic values, not comments or layout, so it is not an in-place configuration editor. Typed decoding is on its own page: [toml-typed](toml-typed.md).

```jai
#import "Extensions/Toml";

doc, error := parse_toml("name = 'tool'\n");
assert(error.kind == .NONE);
assert(toml_get(doc.root, "name").text == "tool");
text, output_error := write_toml(doc.root);
assert(output_error.kind == .NONE);
free(text);
free_toml(*doc);
```

## How it works

- **Ownership.** `parse_toml` returns an owning `Toml_Document` and a structured error. The document keeps its own source copy, decoded strings and stable value nodes; a node pointer stays valid until `free_toml`. On failure all partial storage is released and the document is empty. `free_toml` uses the allocator captured at parse time and is safe to call twice. That allocator and its state must outlive the document.
- **Nodes.** Treat kind, container arrays and internal fields as read-only; change text, fractions and attachments through the builder procedures. Nodes belong to one document: builder insertion rejects cross-document nodes, shared children, roots and cycles, and nodes cannot be removed or reparented. Do not shallow-copy a document.
- **Lookup.** `toml_get(table, key)` treats dots literally and returns null for an absent key or a non-table. Wide tables use an index tree and small ones a linear scan; both keep insertion order.
- **Writer.** `write_toml` returns text allocated from the caller's current allocator; free it with that one. Keys of letters, digits, `_` and `-` are bare, others quoted. Plain values go first, then `[header]` tables and `[[header]]` arrays of tables (an empty table is a bare `[header]`); a table inside a mixed array stays inline. Floats use the shortest digits that read back as the same binary64, and always keep a fraction or exponent.
- **Dates and times.** Civil fields with exact fractional digits and fixed offsets; nothing reads a clock or timezone. `-00:00` stays distinct from `+00:00`. A `:60` second is kept for offset and local values without checking historical leap seconds; values above 60 are rejected. Fractions and offsets are never clamped.
- **Errors.** Static messages and numeric half-open byte spans, so they outlive the input and the document. Lines and Unicode-scalar columns count from one. A duplicate declaration carries the earlier span as `related`. Invalid UTF-8 is rejected.
- **Profile.** TOML 1.1 only; there is no 1.0 switch. The parser tracks how each table was introduced (implicit parent, header, dotted key, closed inline table, array of tables) to apply the duplicate and redefinition rules.
- **Statements.** `record_statements = true` keeps an ordered list of assignments and headers with value, byte span, line and header depth; `toml_statement_path` writes the decoded key path into caller storage. Events describe the original source, not later edits.

## Configuration

`Toml_Options` bounds input bytes, depth, nodes, each decoded key/string/fraction, numeric token size, retained allocation and output bytes. Exceeding one returns `.LIMIT`, separate from syntax and range errors. Retained limits count requested payload, not allocator metadata, and rejected operations spend no budget. `write_toml` applies depth, node, per-field and output limits.

| Option | Default |
| --- | ---: |
| Input | 16 MiB |
| Depth | 128 |
| Nodes | 1,048,576 |
| Each decoded key/string/fraction | 16 MiB |
| Numeric token | 4,096 B |
| Retained document allocation | 128 MiB |
| Written output | 32 MiB |
| Statement history | off |

## How to change it

- `module.jai` holds the public model; `reader.jai` the declaration state and grammar; `scalar.jai` UTF-8, strings, numbers and dates; `writer.jai` output; `index.jai` lookup and insertion.
- Keep declaration state separate from the semantic model. Add a regression before changing what is accepted, especially cross-notation duplicates, date precision, failure cleanup and spans.
- Typed decoding restates the declaration rules in `toml_enter` (`decode.jai`); change it together with `toml_parent`/`toml_header` in `reader.jai`.

Dependencies: `Basic` and `String`. The core does no file or network access; the test-only tagged-JSON adapter uses `Extensions/Json`.

## Conformance

Decoder and encoder run the whole applicable `toml-lang/toml-test` inventory with the explicit 1.1 profile. The version, commit and hashes are pinned in `tools/stdlib/toml/pin.json`; updating the pin means reviewing hashes, licensing and case counts. The runner compares every result id against the profile manifest and rejects missing, duplicate or skipped cases. Decoder errors must exit exactly 1, so a crash is not a valid rejection. The suite checks only millisecond date precision and treats signed float zeros as equal, so local tests cover those. Rust `toml` and Python `tomllib` serve as independent oracles; the community Jai parser is compared only inside the `tools/bench` sandbox.

```sh
export PYTHONPATH="$PWD/tools"
python3 tools/stdlib/toml/run.py --jaic /absolute/path/to/jaic --cache /tmp/toml-suite --report /tmp/toml-result.json
python3 tools/stdlib/toml/run.py --jaic /absolute/path/to/jaic --cache /tmp/toml-suite --report /tmp/toml-statements.json --record-statements
python3 tools/stdlib/toml/run_wasi.py --jaic /absolute/path/to/jaic --cache /tmp/toml-wasi --report /tmp/toml-wasi.json
python3 tools/stdlib/toml/compare_conformance.py --cache /tmp/toml-suite --python --rust /absolute/path/to/toml-oracle
python3 tools/stdlib/toml/settings.py --own /absolute/path/to/settings-adapter --rust /absolute/path/to/toml-oracle --report /tmp/toml-settings.json
python3 tools/stdlib/toml/differential.py --own /tmp/toml-suite/decoder --rust /absolute/path/to/toml-oracle --report /tmp/toml-differential.json
```

The Python reader needs TOML 1.1 support; its version is `python_toml` in `.github/pins.json`, separate from the ordinary CI Python (`tools/ci/pins.py sync` updates the job-local setting). CI runs both profiles (default and `--record-statements`) once per native platform and CPU, and under WASI with Node on Linux x64. The pinned runner has no Windows ARM64 controller, so ARM64 jobs pass `--runner-asset windows-amd64.exe` and `--expected-windows-cpu arm64` to check that the parser itself is native.

## Performance

The comparison measures read (parse and free the tree), lookup (every root key), write (serialize a prebuilt tree) and the `jaifmt` configuration load, against the Rust `toml-oracle` example, the pinned community Jai parser and Python.

- Same compiler flags for every Jai client: `-O2 -no_debug_info`.
- Inputs come from `tools/stdlib/toml/workloads.py` (valid files from 155 B to 10 MiB, including the repository `jacket.toml`). Every result is first checked for the same semantic output.
- Three unscored warm-up rounds, then twelve scored rounds per task and input. Report medians of paired samples, not ratios of medians.
- Do not time anything while a build runs. On a shared machine, alternate old and new runs and report medians with the load average.
- Writing a table built by hand (no lookup index) checks duplicate keys with a per-write hash set; tables built with `toml_insert` or parsed carry an index and skip it.

```sh
python3 tools/stdlib/toml/prepare_community.py --archive /tmp/toml-jai.tar.gz --output /tmp/toml-imports
python3 tools/stdlib/toml/build_oracle.py --target-dir /tmp/toml-target --receipt /tmp/toml-oracle.json
python3 tools/stdlib/toml/workloads.py --output /tmp/toml-inputs
python3 benchmarks/compare/toml/measure.py --scratch /tmp/toml-bench --jaic /path/to/jaic \
  --community-dir /tmp/toml-imports --community-archive /tmp/toml-jai.tar.gz --build
python3 benchmarks/compare/toml/measure.py --scratch /tmp/toml-bench --rust /tmp/toml-target/release/examples/toml-oracle \
  --rust-receipt /tmp/toml-oracle.json --workloads /tmp/toml-inputs --invalid-workloads /tmp/toml-inputs/invalid --verify-only
```

Drop `--verify-only` for `--pilot`, then `--freeze-plan` and `--batch-plan <plan> --run` for a full comparison. `benchmarks/compare/toml/formatter_batch.jai <file> <iterations>` times the `jaifmt` config load.
