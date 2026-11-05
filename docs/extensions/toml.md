# TOML values and configuration

`Extensions/Toml` is an in-memory TOML 1.1 parser and semantic writer. It represents strings, signed 64-bit integers, binary64 floats, booleans, arrays, tables and the four TOML date/time categories. It has no null value. Typed reflection mapping and comment-preserving editing are outside this module.

## Ownership and flow

`parse_toml(text)` returns an owning `Toml_Document` and a structured error. On success the document keeps its own source copy, decoded keys/strings and stable value nodes. On failure it releases all partial storage and returns an empty document. `free_toml(*doc)` uses the allocator captured during parsing and clears the document, so a second call does nothing.

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

`toml_get(table, key)` treats dots literally and returns null for an absent key or non-table. A node pointer remains valid until document destruction.

For document-owned nodes, treat kind, container arrays and internal fields as read-only; change text/fractions and attachments through the builder procedures. Scalar fields may be assigned only for their matching fixed kind. Array/member storage can move on growth. Nodes belong to one document. Builder insertion rejects cross-document nodes, shared children, roots and cycles. Nodes cannot be removed or reparented. `write_toml` returns a string allocated with the active caller allocator, which must also be used to free that output; the document allocator is independent. The captured allocator and its state must outlive the document; changing the active context allocator before destruction is supported. Wide tables use a balanced member-index tree; small tables use linear lookup, and both preserve insertion order. Parsed key allocations move into their semantic members after capacity admission. Temporary key vectors are reused by active parsing scope, with their cached capacity charged until parser cleanup. Recursive scopes keep separate vectors, so nested inline values cannot overwrite an outer assignment path.

Do not shallow-copy document ownership or change its source/bookkeeping. Value spans and statement events describe the original immutable source, including after scalar edits.

Errors use static messages and numeric source spans, so they survive input and document release. Spans are half-open byte ranges. Display lines and Unicode scalar columns count from one; tabs count as one column. Duplicate declarations carry an earlier related span. Invalid UTF-8 is rejected without normalization.

Private lookup/insertion helpers live in `index.jai`. An absent-key cursor keeps only bounded node indices and directions across detached value parsing; snapshot/path checks protect reuse, and stale cursors re-probe before mutation. Capacity relocation preserves indices. The private helper regression loads that same source against concrete module field types, without exporting test hooks.

## Profiles and limits

The parser targets TOML 1.1; there is no strict 1.0 switch. Table provenance distinguishes implicit parents, explicit declarations, dotted-key definitions, closed inline tables and arrays of tables. The writer emits quoted keys and inline nested tables in insertion order. Finite floats use scientific notation with 17 significant digits, independent of display style; this preserves binary64 round trips and both zero signs. The writer reuses float conversion scratch and emits validated temporal prefixes/suffixes directly, keeping exact fractional digits. It preserves semantic values rather than comments or the original layout; do not use it as an in-place configuration editor.

`Toml_Options` bounds input bytes, depth, nodes, each decoded key/string/fraction, numeric token bytes, retained allocation capacity and output bytes. Limits report `.LIMIT` separately from syntax/range errors. Retained limits count requested allocation payload, including capacity and parser provenance, rather than allocator metadata. Rejected operations do not consume budget; replacement accounts for the retained size difference. Copy-before-free replacement can temporarily hold both old and new strings, so this is a retained-storage limit, not a peak-allocation guarantee. `write_toml` applies depth, node, per-field string/fraction and output-byte limits. Input and numeric-token limits apply only to parsing; retained-byte limits apply to parser/builder document storage, not writer scratch allocation. Allocation failure follows the owning Basic allocator contract; the module does not promise recoverable OOM when that allocator aborts.

Default options are deliberately finite:

| Option | Default |
| --- | ---: |
| Input | 16 MiB |
| Depth | 128 |
| Nodes | 1,048,576 |
| Each decoded key/string/fraction | 16 MiB |
| Numeric token | 4,096 B |
| Retained document allocation payload | 128 MiB |
| Written output | 32 MiB |
| Source statement history | Disabled |

Dates and times use format-owned civil fields with exact fractional digits and fixed offsets. Parsing never reads a clock or timezone and does not import tzdb. Negative-zero offsets remain distinct. A `:60` spelling is retained for offset and local values without validating historical insertion events, including structurally valid non-event dates. Values above 60 are rejected. This explicit implementation policy follows the [upstream leap-second guidance](https://github.com/toml-lang/toml-test/issues/160#issuecomment-2187827569); it does not provide RFC 3339 event validation or instant conversion. Fractions and offsets are never clamped or normalized.

`record_statements = true` retains an ordered source projection for consumers needing declaration history. Each event identifies its value, byte span, source line and enclosing header depth. `toml_statement_path` writes borrowed decoded key segments into caller storage, omitting array element indices. Events describe the original parsed source and do not track subsequent edits. Jai_Format uses this projection to preserve unknown-setting names, header history and raw documentation overrides without a second parser.

## Extending and verifying

The public model and lifetime boundary are in `module.jai`; `reader.jai` owns declaration state and document grammar; `scalar.jai` owns UTF-8, strings, numbers and temporal syntax; `writer.jai` owns output policy. Keep declaration state separate from the semantic model. Add a regression before changing acceptance, especially cross-notation duplicate rules, temporal precision, failure cleanup and source spans.

Runtime dependencies are the owned `Basic` and `String` modules. The test-only tagged-JSON adapter uses `Extensions/Json` and explicit stdin/stdout file operations. The core performs no file/network access. Independent conformance is pinned to `toml-lang/toml-test` v2.2.0 with the explicit 1.1 profile; decoder and encoder must both pass the complete applicable inventory before release. Pin updates require reviewed hashes, licensing and case-count changes, rather than silently following upstream main.

## Coverage and independent comparisons

The independent inventory is pinned in `tools/stdlib/toml/pin.json`: toml-test v2.2.0, commit `ce08da1ddb075d1c7596d663c7fcba9a2ae02c5c`, explicit `-toml=1.1`. Its 895 outcomes comprise 214 valid decoder cases, 214 semantic encoder cases and 467 invalid decoder cases. The runner compares every result ID against the profile manifest, rejects missing/duplicate/skipped cases and retains verbose JSON. Decoder errors must exit exactly 1; crashes do not count as valid rejection. The suite checks only millisecond temporal precision and considers signed floating zero equivalent, so its result does not establish those stronger contracts.

| Contract | Local regression or evidence | Scope |
| --- | --- | --- |
| Full 1.1 grammar, read and semantic write | Pinned decoder and encoder inventory | Complete applicable inventory; no selected exclusions |
| Header, dotted, implicit and inline declaration conflicts | `tests/declarations.jai`, `tests/api.jai` | Includes per-element array-of-table state and earlier declaration spans |
| Stable lookup pointers and wide tables | `tests/index.jai`, `tests/insert-hint.jai`, `tests/builder-state.jai` | Rotations, sorted/reverse/zigzag insertion, detached nested values, stale snapshots, path-cap fallback, writer prefix indices and lookup across growth |
| Ownership and failure cleanup | `tests/ownership.jai`, `tests/builder-state.jai` | Source release, changed allocator, truncations, cross-document/shared/root/cyclic attachment rejection |
| Limits and diagnostics | `tests/errors.jai`, `tests/builder-state.jai` | UTF-8 byte location, Unicode columns, CRLF, depth, per-field size and transactional retained capacity |
| Temporal precision and leap policy | `tests/temporal.jai`, `tests/api.jai` | Exact >9-digit write/reparse, signed-zero offset, real/shifted/local/structural `:60`, rejection above 60 and fractions without seconds |
| Parser allocation and bounded string scanning | `tests/key-storage.jai`, `tests/string-spans.jai`, `tests/number-staging.jai` | Single owned key payload, reused scope vectors, nested/truncated/quota cleanup, decoded UTF-8 widths, plain-span limits and no unused integer normalization |
| Constructed-value validation | `tests/writer.jai`, `tests/writer-index.jai`, `tests/writer-integers.jai`, `tests/writer-floats.jai`, `tests/utf8-values.jai` | Canonical signed64 decimal, 527 finite binary64 bit-pattern round trips including signed zero/subnormals, and no added per-value result-string allocation beyond the float conversion primitive, UTF-8 diagnostics, checked cached-index witness, corrupt-cache fallback, output spans, invalid date, duplicate key, negative floating zero and display-style independence |
| Source projection | `tests/statements.jai` | Assignment/header order, nested inline values, array-of-table identity and caller path buffer |
| Actual formatter consumer | `Jai_Format/tests/toml-config.jai`, `toml-warning-history.jai`, existing golden cases | Strict whole-file migration, valid settings, raw overrides, unknown-name/header history and command-line precedence |

Competitor execution uses identical pinned fixture IDs where a compatible adapter exists. The community Jai library is pinned separately in `tools/stdlib/toml/community-pin.json`; its implementation is opaque to this project. Compilation and execution occur only through `tools/bench` Sandbox. Its documented public read/write API does not expose a documented tagged-value projection, so this comparison is a **read/write interoperability pipeline**: community parse/write followed by Python tagged conversion for decoding, and independently constructed TOML followed by community parse/write for encoding. A valid-case failure therefore does not identify parsing versus writing as the cause. Accepted invalid syntax is observable at the read API. Default and explicitly configured float formatting are separate runs.

| Library/profile | Read | Write | Precision and behavior boundary | Evidence source |
| --- | --- | --- | --- | --- |
| `Extensions/Toml`, 1.1 | General semantic tree | Semantic tree | s64/binary64, exact fraction digits, signed floating/offset zero | Local regressions and pinned independent inventory |
| `sjorsdonkers/toml-jai`, pinned revision | Public general-value API | Public general-value API | README claims broad 1.0/1.1 support; measured interoperability is narrower than direct tagged conformance | Pinned README/public API and black-box pipeline; no implementation inspection |
| Rust `toml` 1.1.2 | General value tree | Semantic value tree | Independent formatter-settings oracle; observed fractions retain 9 digits | Existing pinned Cargo dependency and `toml-oracle` example |
| Python `tomllib` 3.15 | General value tree, 1.1 | No writer | Observed fractions retain 6 digits; no writer | Installed runtime and official Python documentation; decoder-only suite |

| Practical coverage | Ours | Existing Rust settings/parser | Python reader | Community public API |
| --- | --- | --- | --- | --- |
| Formatter consumer | Real `Jai_Format` file/schema, `--set`, raw override and warning-history regressions | Existing `jacket-config` schema remains independent; shared settings checked | Generic reader comparison only | Generic read/write pipeline only |
| Diagnostics and limits | Error kinds/path/primary/related spans; configured input/depth/nodes/string/token/retained/output boundaries tested | Oracle captures message/optional span; default depth probe checked; other quotas not assessed | Expected errors classified; depth probe checked; other quotas not assessed | Adapter observes public success flags; structured errors and configurable quotas not assessed |
| Executed platforms in this comparison | macOS native/interpreter, full WASI suite and browser-engine consumer; wider native CI pending | macOS release oracle | macOS Python 3.15 | macOS sandboxed black box; other targets not run |

The separate authored feature probes retain original inputs and raw outputs, so datetime adapters cannot hide precision loss. These are observations of the pinned configuration, not inferred upper bounds or unsupported-platform claims.

| Identical probe | Ours | Rust `toml` 1.1.2 projection | Python 3.15.0 projection | Community scientific read/write (17 fractional digits) |
| --- | --- | --- | --- | --- |
| 12 fractional digits, offset datetime and local time | All 12 retained | First 9 retained | First 6 retained | Output retains first 3 |
| Floating `-0.0` | Sign retained | Sign retained | Sign retained | Sign retained |
| Offset `-00:00` | Distinct spelling retained | Becomes `+00:00` | Becomes `+00:00` | Negative-zero spelling retained |
| Historical `2016-12-31T23:59:60Z` | Retained | Retained | Rejected | Retained |
| Structural non-event `2024-02-29T12:00:60Z` | Retained by documented policy | Retained | Rejected | Retained |
| Integer `9223372036854775808` | Range rejection | Range rejection | Integer retained | Accepted; output changes to floating syntax |
| 129 nested arrays, default options | Limit rejection | Rejected | Accepted in this probe | Accepted in this probe |

Community roundtrip output does not establish its internal read representation. The fraction row therefore attributes the observed loss to the read/write path. Leap spelling acceptance does not imply historical event validation for any implementation. The depth probe does not establish that accepting libraries have no upper limit.

The measured native conformance snapshot on macOS arm64 gives the following counts. A pipeline encoder row is not a direct community tagged-value encoder claim.

| Implementation / configuration | Valid decoder (214) | Encoder (214) | Invalid rejected (467) | Skips |
| --- | ---: | ---: | ---: | ---: |
| Ours, default | 214 | 214 | 467 | 0 |
| Ours, source statements enabled | 214 | 214 | 467 | 0 |
| Ours, WASI default | 214 | 214 | 467 | 0 |
| Ours, WASI source statements enabled | 214 | 214 | 467 | 0 |
| Rust `toml` 1.1.2 | 214 | 214 | 467 | 0 |
| Python `tomllib` 3.15.0 | 214 | Not supplied | 467 | 0 |
| Community pipeline, default float formatting | 205 | 205 | 449 | 0 |
| Community pipeline, shortest float formatting | 209 | 209 | 449 | 0 |
| Community pipeline, scientific, 16 fractional digits, no zero removal | 214 | 214 | 449 | 0 |

The community negative failures comprise missing array separators, nonstandard infinity/NaN spellings, missing key newline, string concatenation/termination/Unicode escape cases and text following strings. Configure float formatting before using its valid-case roundtrip counts; default precision loss is not evidence that its parser read the original number incorrectly. Earlier Rust oracle failures for omitted seconds were adapter normalization errors, fixed before the table above.

Local contract validation also passed 17 interpreted core tests, 18 native tests and 18 WASI tests (the extra case exercises the real formatter), all 30 existing formatter golden cases, 50 shared settings-schema cases against Rust, 324 authored-value/mutation differential cases against Rust/Python, and 3,000 lifecycle-aware mutation/construction fuzz calls with zero skips/findings. These counts describe distinct checks; they are not multiplied into the independent suite inventory.

The browser compiler engine passed the signed-zero, minimum-subnormal and formatter consumer on the C5 source bundle under Node 24.12.0. The current insertion-cursor source is refreshed by the browser CI gate before publication. These checks exercise the browser engine, not Chrome UI behavior. Linux/Windows execution and required final-head CI remain separate acceptance gates.

[TOML measurements](toml-performance.md) records the storage decision, final allocation payload, formatter artifact cost and equivalent-task benchmark procedure.

Reproduce the native suite through the shared local heavy-job gate:

```sh
python3 tools/stdlib/toml/run.py --jaic /absolute/path/to/jaic --cache /tmp/toml-suite --report /tmp/toml-result.json
python3 tools/stdlib/toml/run.py --jaic /absolute/path/to/jaic --cache /tmp/toml-suite --report /tmp/toml-statements.json --record-statements
python3 tools/stdlib/toml/run_wasi.py --jaic /absolute/path/to/jaic --cache /tmp/toml-wasi --report /tmp/toml-wasi.json
python3 tools/stdlib/toml/compare_conformance.py --cache /tmp/toml-suite --python --rust /absolute/path/to/toml-oracle
python3 tools/stdlib/toml/settings.py --own /absolute/path/to/settings-adapter --rust /absolute/path/to/toml-oracle --report /tmp/toml-settings.json
python3 tools/stdlib/toml/features.py --own /tmp/toml-suite/decoder --rust /absolute/path/to/toml-oracle --community /absolute/path/to/community --scratch /tmp/toml-features --report /tmp/toml-features.json
python3 tools/stdlib/toml/differential.py --own /tmp/toml-suite/decoder --rust /absolute/path/to/toml-oracle --report /tmp/toml-differential.json
python3 tools/stdlib/toml/workloads.py --output /tmp/toml-workloads
```

The independent Python reader needs TOML 1.1 support. Its exact `python_toml` version is managed in `.github/pins.json` separately from the ordinary CI-tools Python version; `tools/ci/pins.py sync` updates the job-local setting. The first Linux tooling job uses that interpreter for all subsequent Python steps, including its later helper/producer checks; other legs retain the ordinary tools pin.

CI runs both complete modes once per represented native platform/CPU using the existing production compiler builds: the first Linux tooling part, each macOS suite leg and the first Windows shard. Linux x64 also runs both complete profiles under actual WASI with Node. Both profiles run even if one fails; either failure fails the job. Complete JSON outcomes and the expected inventory are retained, and missing profile reports fail the job.

The [pinned runner release](https://github.com/toml-lang/toml-test/releases/expanded_assets/v2.2.0) publishes a Windows x64 controller but no Windows ARM64 controller. ARM64 jobs explicitly select `--runner-asset windows-amd64.exe`; [Windows 11 supports x64 application emulation](https://learn.microsoft.com/en-us/windows/arm/apps-on-arm-x86-emulation). The parser adapter remains native ARM64: `--expected-windows-cpu arm64` verifies its PE machine separately. Reports record controller asset and parser CPU; there is no silent architecture fallback or parser emulation claim. Actual Windows acceptance remains pending its required CI execution.
