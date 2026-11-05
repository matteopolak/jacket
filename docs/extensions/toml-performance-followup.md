# TOML performance follow-up

The current source adds three measured changes: guarded byte comparison for wide tables, equality-only tiny duplicate checks plus ASCII bare keys, and plain keys that use the document-owned immutable source. The general parser, quotas, exact precision and raw-root writer checks remain. Rust also parses complete TOML; validation strictness does not explain the remaining gap.

These are targeted warm comparisons, not a replacement for the historical cold/consumer/conformance results. All nine original valid inputs and every round remain: three unscored warmups and twelve scored samples for each version/task/input. All 81 actual-executable checks passed in each comparison. The same compiler/flags, runtime harness and Rust general-value executable are authenticated; no source or binary is relabeled. Median parity is not every-round parity.

## Current warm comparison

C8 is the preceding writer candidate; C9 adds source-backed plain keys. Cells are median [minimum–maximum] microseconds. Ratios are median of paired samples, not ratios of the displayed medians; lower is faster. Every min/max includes all measured samples. Read materializes and destroys the complete tree; lookup queries every root key; prepared writing validates a prebuilt tree and consumes every output byte.

### read

| Input | C8 µs | C9 µs | Rust µs | C9/C8 | C9/Rust |
| --- | ---: | ---: | ---: | ---: | ---: |
| config-1024.toml | 9.851 [9.733–9.984] | 9.096 [9.027–10.832] | 7.788 [7.722–7.868] | 0.924 | 1.167 |
| config-8192.toml | 74.945 [74.192–106.636] | 68.998 [68.349–69.473] | 58.288 [57.645–60.061] | 0.918 | 1.186 |
| config.toml | 1.723 [1.708–1.735] | 1.581 [1.564–1.620] | 1.403 [1.389–1.413] | 0.916 | 1.130 |
| records-102400.toml | 1045.859 [1030.913–1065.739] | 981.728 [972.739–999.174] | 1020.214 [1006.736–1182.426] | 0.941 | 0.963 |
| repository-config.toml | 77.405 [76.766–78.581] | 73.723 [73.331–77.151] | 64.064 [63.533–66.850] | 0.957 | 1.152 |
| wide-102400.toml | 1386.829 [1380.237–1795.316] | 1259.382 [1242.868–1313.605] | 1345.108 [1320.154–1452.181] | 0.908 | 0.942 |
| wide-1048576.toml | 15168.500 [14871.000–15520.500] | 13750.750 [13587.500–13890.500] | 14364.260 [14088.896–15665.573] | 0.908 | 0.961 |
| wide-10485760.toml | 173837.500 [171538.000–177739.000] | 150063.000 [147652.000–221015.000] | 153953.834 [150024.417–160089.625] | 0.860 | 0.977 |
| wide-4096.toml | 50.782 [50.395–53.522] | 45.984 [45.340–46.963] | 48.725 [48.151–49.848] | 0.901 | 0.948 |

### lookup

| Input | C8 µs | C9 µs | Rust µs | C9/C8 | C9/Rust |
| --- | ---: | ---: | ---: | ---: | ---: |
| config-1024.toml | 0.028 [0.027–0.029] | 0.027 [0.026–0.041] | 0.033 [0.031–0.034] | 0.964 | 0.820 |
| config-8192.toml | 0.028 [0.027–0.029] | 0.027 [0.026–0.028] | 0.032 [0.030–0.033] | 0.963 | 0.834 |
| config.toml | 0.020 [0.019–0.027] | 0.019 [0.019–0.020] | 0.022 [0.020–0.023] | 0.978 | 0.875 |
| records-102400.toml | 0.011 [0.010–0.011] | 0.011 [0.010–0.011] | 0.007 [0.006–0.009] | 1.006 | 1.573 |
| repository-config.toml | 0.025 [0.024–0.026] | 0.025 [0.024–0.026] | 0.030 [0.029–0.033] | 0.998 | 0.831 |
| wide-102400.toml | 298.954 [291.420–301.316] | 285.652 [283.253–299.391] | 296.948 [293.766–318.294] | 0.958 | 0.963 |
| wide-1048576.toml | 3369.500 [3295.250–3505.125] | 3237.656 [3212.000–3975.375] | 3246.809 [3103.427–3531.581] | 0.967 | 1.004 |
| wide-10485760.toml | 48575.500 [47815.000–49838.000] | 32978.250 [32204.000–39267.500] | 33546.552 [32772.562–34749.188] | 0.682 | 0.983 |
| wide-4096.toml | 5.555 [5.406–5.815] | 5.281 [5.070–5.442] | 7.576 [7.277–7.831] | 0.956 | 0.703 |

### write

| Input | C8 µs | C9 µs | Rust µs | C9/C8 | C9/Rust |
| --- | ---: | ---: | ---: | ---: | ---: |
| config-1024.toml | 3.319 [3.283–3.345] | 3.264 [3.230–3.290] | 5.350 [5.246–5.460] | 0.987 | 0.611 |
| config-8192.toml | 23.992 [23.904–24.390] | 24.084 [23.932–24.283] | 39.021 [37.916–44.140] | 1.004 | 0.619 |
| config.toml | 0.718 [0.708–0.806] | 0.697 [0.690–0.725] | 1.030 [1.020–1.049] | 0.970 | 0.677 |
| records-102400.toml | 389.707 [385.883–392.477] | 388.257 [383.486–392.081] | 931.387 [893.295–964.258] | 0.998 | 0.417 |
| repository-config.toml | 29.676 [29.030–32.256] | 29.400 [28.919–30.255] | 44.566 [44.047–46.607] | 0.990 | 0.659 |
| wide-102400.toml | 250.324 [248.507–263.356] | 244.324 [242.639–249.122] | 285.877 [278.257–321.537] | 0.978 | 0.860 |
| wide-1048576.toml | 2535.810 [2496.143–2580.524] | 2509.190 [2473.571–2708.333] | 2821.852 [2744.540–3162.476] | 0.985 | 0.892 |
| wide-10485760.toml | 26151.000 [25844.667–27532.667] | 24275.333 [24134.333–25171.000] | 27522.708 [26990.639–28825.194] | 0.928 | 0.882 |
| wide-4096.toml | 10.244 [10.070–10.351] | 10.059 [9.929–10.388] | 12.724 [12.373–13.536] | 0.985 | 0.791 |

The 10 MiB read paired C9/Rust range is 0.936–1.435 and lookup 0.931–1.198; the medians do not establish an every-round advantage. Small 1 KiB/8 KiB/repository reads retain median gaps of 16.7%/18.6%/15.2% versus Rust. The 8 KiB prepared writer is 1.0037× C8 (eight of twelve slower pairs, range 0.995–1.013); this result is retained. Profiles show ordinary grammar/allocation work and remaining comparison/tree traversal, not another identified unused staging path. Separate adaptive node storage or direct typed parsing would need new measured ownership/memory tradeoffs.

## Changes and retained regressions

Earlier paired ratios are retained below rather than selecting only the best wide cases. C7 uses the guarded byte comparator against the current-stack control. C8 adds equality-only tiny duplicate checks and safe bare-key output. Ratios show candidate/control median [min–max].

### C7 paired candidate/control

| Input | Read | Lookup | Prepared write |
| --- | ---: | ---: | ---: |
| config-1024.toml | 1.009 [0.992–1.026] | 1.018 [0.954–1.033] | 1.016 [1.004–1.029] |
| config-8192.toml | 1.014 [1.004–1.030] | 1.015 [0.959–1.072] | 1.013 [1.000–1.028] |
| config.toml | 1.007 [0.982–1.113] | 0.984 [0.949–1.010] | 1.015 [0.654–1.046] |
| records-102400.toml | 1.008 [0.988–1.025] | 0.996 [0.973–1.043] | 1.021 [0.995–1.034] |
| repository-config.toml | 0.988 [0.962–1.034] | 1.004 [0.969–1.047] | 1.001 [0.955–1.019] |
| wide-102400.toml | 0.889 [0.856–0.907] | 0.638 [0.524–0.671] | 0.974 [0.958–0.988] |
| wide-1048576.toml | 0.885 [0.846–1.087] | 0.613 [0.553–0.634] | 0.972 [0.931–1.013] |
| wide-10485760.toml | 0.881 [0.846–0.896] | 0.711 [0.677–0.759] | 0.963 [0.945–1.064] |
| wide-4096.toml | 0.920 [0.909–0.932] | 0.449 [0.415–0.500] | 0.973 [0.952–1.029] |

### C8 paired candidate/control

| Input | Read | Lookup | Prepared write |
| --- | ---: | ---: | ---: |
| config-1024.toml | 0.972 [0.962–1.377] | 1.004 [0.983–1.070] | 0.835 [0.776–0.847] |
| config-8192.toml | 0.959 [0.948–0.979] | 1.024 [0.991–1.043] | 0.834 [0.799–0.849] |
| config.toml | 0.974 [0.951–0.985] | 1.044 [0.996–1.062] | 0.834 [0.822–0.854] |
| records-102400.toml | 0.975 [0.950–0.987] | 1.000 [0.970–1.032] | 0.868 [0.859–1.020] |
| repository-config.toml | 0.985 [0.961–1.002] | 0.979 [0.920–1.016] | 0.940 [0.795–0.996] |
| wide-102400.toml | 1.036 [0.979–1.053] | 1.013 [0.999–1.060] | 0.740 [0.664–0.760] |
| wide-1048576.toml | 1.038 [1.016–1.076] | 1.024 [0.995–1.085] | 0.752 [0.711–0.812] |
| wide-10485760.toml | 1.041 [1.022–1.052] | 1.012 [0.996–1.050] | 0.783 [0.753–0.825] |
| wide-4096.toml | 1.023 [1.006–1.041] | 1.076 [1.027–1.137] | 0.724 [0.700–0.746] |

C7 improves wide comparison paths but regresses several small reads/writes. C8 improves every writer median while wide reads/lookups regress against C7. Those interventions do not establish why unchanged reader code timing moved. C9 read medians improve on C8 for all nine inputs; all output hashes remain unchanged from C8.

## Output and allocation

C8 deliberately emits nonempty ASCII `[A-Za-z0-9_-]+` keys without quotes. Other keys—including empty names, dots, spaces, controls and Unicode—keep the checked quoted path; string values remain quoted. Writer output size changes are part of the work, not hidden normalization. All four wide outputs match Rust byte-for-byte by SHA-256.

| Input | C7 bytes | C8/C9 bytes | Rust bytes |
| --- | ---: | ---: | ---: |
| config-1024.toml | 1,067 | 981 | 1,039 |
| config-8192.toml | 8,276 | 7,656 | 8,337 |
| config.toml | 184 | 166 | 157 |
| records-102400.toml | 122,673 | 114,991 | 103,459 |
| repository-config.toml | 11,486 | 11,240 | 11,486 |
| wide-102400.toml | 113,298 | 102,402 | 102,402 |
| wide-1048576.toml | 1,154,560 | 1,048,590 | 1,048,590 |
| wide-10485760.toml | 11,494,989 | 10,485,763 | 10,485,763 |
| wide-4096.toml | 4,570 | 4,102 | 4,102 |

Requested payload is measured separately with matched DEBUG allocation-ledger clients; it is not RSS or latency. Source history is disabled. Every ledger equals live requested payload, every document is looked up/traversed and destroyed, and live payload/map entries return to zero. Nodes remain 144 bytes and retain stable addresses. Timed Jai batches already use Flat_Pool: the request count is not a claim of that many system mallocs.

| Input | C8 requests | C9 requests | C8 retained B | C9 retained B | C9 peak B |
| --- | ---: | ---: | ---: | ---: | ---: |
| config-1024.toml | 206 | 153 | 19,665 | 19,438 | 24,244 |
| config-8192.toml | 1,541 | 1,132 | 149,659 | 148,186 | 181,664 |
| config.toml | 39 | 30 | 3,461 | 3,394 | 4,619 |
| records-102400.toml | 19,210 | 14,410 | 1,883,869 | 1,869,462 | 2,394,461 |
| repository-config.toml | 1,297 | 1,048 | 128,894 | 125,383 | 142,607 |
| wide-102400.toml | 10,906 | 5,458 | 1,536,858 | 1,476,930 | 1,739,778 |
| wide-1048576.toml | 105,980 | 52,995 | 13,980,049 | 13,397,214 | 15,495,070 |
| wide-10485760.toml | 1,009,236 | 504,623 | 126,449,706 | 120,898,963 | 137,676,883 |
| wide-4096.toml | 478 | 244 | 58,996 | 56,422 | 65,318 |

On 10 MiB, 504,613 removed requests and 5,550,743 fewer retained bytes equal exactly one 11-byte payload per plain key. Caller input is still copied; source views belong to that retained immutable copy. Escaped/empty keys and builder insertions keep owned payloads. The older chunk/heap comparison measured memory, not latency superiority.

## Actual consumer, artifacts and reproduction

Current formatter correctness was refreshed with twelve full settings projections against Rust, thirty golden cases through the rebuilt production browser engine, and thirty through actual WASI/native formatter executables. A new consumer timing attempt did not admit within the sixty-second quiet deadline; no child or sample started. The earlier broad consumer timings remain explicitly historical. This snapshot records local acceptance; hosted CI results are separate.

Matched shipping artifact builds use the same pinned compiler and `-O2 -no_debug_info`. These are whole formatter executables, not isolated parser objects. C8 uses its authenticated physical stdlib snapshot; source paths are retained in receipts.

| Formatter artifact | C8 bytes | C9 bytes |
| --- | ---: | ---: |
| Native macOS arm64 | 696,192 | 663,184 |
| WASI | 612,816 | 597,423 |

The fresh production browser compiler bundle is 13,642,741 bytes and embeds the current stdlib; its Node engine contract check passed. This is not a Chrome UI claim or an isolated TOML size delta. All 122 focused DEBUG commands, 3,580 full native/WASI conformance outcomes, 50 shared settings cases, 324 differential cases and 3,000 lifecycle fuzz calls passed. The new benchmark recording/archive/input checks pass separately.

## Reproduce or update

Use Python 3.15 and the locked Rust oracle described in [the broad comparison procedure](toml-performance.md). Keep one compiler binary and `-O2 -no_debug_info` for both Jai clients. Build two revisions in the same clean isolated checkout path, saving each build in a different output directory; archive each before switching the checkout. Keep these controller scripts in a separate controller checkout so they remain available while switching source revisions. Community compilation stays opaque and inside the existing bench sandbox.

```sh
TOML_SOURCE_CHECKOUT=/absolute/source/checkout
TOML_CONTROLLERS=/absolute/controller/checkout/benchmarks/compare/toml
PYTHONPATH="$TOML_SOURCE_CHECKOUT/tools" python3 "$TOML_SOURCE_CHECKOUT/tools/stdlib/toml/workloads.py" --output /tmp/toml-inputs --repository-config "$TOML_CONTROLLERS/fixtures/repository-config.toml"
PYTHONPATH="$TOML_SOURCE_CHECKOUT/tools" python3 "$TOML_SOURCE_CHECKOUT/benchmarks/compare/toml/measure.py" --scratch /tmp/toml-control --jaic /absolute/path/to/jaic --community-dir /tmp/toml-imports --community-archive /tmp/toml-jai.tar.gz --build
python3 "$TOML_CONTROLLERS/archive_sources.py" --root "$TOML_SOURCE_CHECKOUT" --build /tmp/toml-control/build.json
# Switch the source checkout; repeat --build and archive_sources with /tmp/toml-candidate.
```

`frozen-followup-plan.json` preserves every input hash and used family count from the reported plan. The repository fixture regenerates all eighteen valid/invalid inputs exactly; the targeted run requires all nine valid inputs. For a new oracle build, copy this plan and bind its `rust_binary_sha256` to the new oracle build receipt. Label that as a new experiment; do not change historical receipts or mix samples. The controller records the complete new compiler/source/oracle/machine identities.

```sh
python3 "$TOML_CONTROLLERS/targeted-compare.py" --prepare --root "$TOML_SOURCE_CHECKOUT" --control /tmp/toml-control/build.json --candidate /tmp/toml-candidate/build.json --rust /absolute/path/to/toml-oracle --rust-receipt /tmp/oracle-build.json --iterations /tmp/frozen-plan.json --workloads /tmp/toml-inputs --output /tmp/toml-targeted
# Run the same command with --run only inside the shared exclusive quiet lease.
```

Preparation verifies actual read/write output, exact numeric/civil values, lookup consumption and writer hashes. An intentional writer spelling change requires `--spelling-change`; otherwise output SHA-256 must match. Keep the plan, validation, append-only samples, result, binaries, build receipts and source archives together. Never remove a slow input or sample. Refresh every table from the new complete report, retaining regressions and ranges. These targeted warm results do not replace the historical cold, community, Python or actual-consumer tasks.

Result receipts:

- C7: 1,215 rows/81 groups; SHA-256 `ce7a322442466c2c1ef0aebb85ef74ccbed265b97a22745eb2d249c545251602`.
- C8: 1,215 rows/81 groups; SHA-256 `d81748659b8a89ed0801ead4fe082086473e0636f06a7032f6ae686abbe9d1b8`.
- C9: 1,215 rows/81 groups; SHA-256 `8a22d9a4b4abc288fca014484ea6fe5d584eabca0c44c4f27f9d4d2fe1582af0`.
