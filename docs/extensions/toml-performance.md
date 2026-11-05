# TOML measurements

These measurements compare the general [TOML module](toml.md), its actual formatter consumer and pinned independent libraries. Allocation profiling and executable size are separate from timed parser/writer tasks.

## Storage and formatter artifacts

The actual formatter artifact comparison uses the same compiler binary and `-O2 -no_debug_info`, with the earlier formatter from repository base `d072effb268b18d2b16ebf27d69d36c14859909d` and this migration. The baseline stdlib is a physical owned source snapshot so internal module privacy rules remain active.

| Real formatter artifact | Earlier subset reader | General TOML consumer | Increase |
| --- | ---: | ---: | ---: |
| Native macOS arm64 | 565,920 B | 661,984 B | 96,064 B (17.0%) |
| WASI | 500,666 B | 586,015 B | 85,349 B (17.0%) |

These are whole formatter executable sizes, not isolated parser object sizes or browser compiler-bundle deltas. Executable size does not establish speed.

The storage decision compared three source-owned, stable-pointer prototypes before adding the final source-history and wide-table metadata. On the same 15,169-byte repository configuration (494 nodes), the compact union reduced payload substantially. Chunk allocation reduced allocation calls but retained more unused capacity, so the final design keeps individually allocated compact nodes. All prototypes parsed, traversed, looked up every root member and destroyed the document with the same checksum and zero retained allocations.

| Historical storage prototype | Node bytes | Allocations | Resizes | Retained payload | Peak payload |
| --- | ---: | ---: | ---: | ---: | ---: |
| Separate scalar fields, heap nodes | 256 | 1,706 | 33 | 185,710 B | 198,397 B |
| Compact union, heap nodes | 136 | 1,706 | 33 | 126,430 B | 139,117 B |
| Compact union, chunked nodes | 136 | 1,218 | 33 | 140,529 B | 153,216 B |

These are historical representation measurements, not final footprint or latency results. The chunked prototype retained 11.2% more payload than the compact heap on this configuration and about 22% more on the 4 KiB wide-table input. Stable node addresses alone do not require one allocation per node; retained capacity on actual configuration inputs determined this choice.

The current 144-byte node implementation was profiled separately on all nine valid benchmark inputs. Source history is disabled, as in the generic parser timings. Each row includes the owning source copy and requested capacity; every retained-byte ledger matched the counted allocator, and every destruction returned payload to zero. The repository input differs from the earlier prototype input because it includes this module's documentation registration.

| Current input | Input B | Nodes | Allocations / resizes | Retained payload B | Peak payload B |
| --- | ---: | ---: | ---: | ---: | ---: |
| config-1024 | 1,026 | 79 | 206 / 9 | 19,665 | 24,471 |
| config-8192 | 8,235 | 613 | 1,541 / 18 | 149,659 | 183,137 |
| config | 155 | 12 | 39 / 2 | 3,461 | 4,686 |
| records-102400 | 102,500 | 8,642 | 19,210 / 29 | 1,883,869 | 2,408,868 |
| repository-config | 15,195 | 495 | 1,297 / 36 | 128,894 | 146,118 |
| wide-102400 | 102,402 | 5,449 | 10,906 / 39 | 1,536,858 | 1,799,706 |
| wide-1048576 | 1,048,590 | 52,986 | 105,980 / 51 | 13,980,049 | 16,077,905 |
| wide-10485760 | 10,485,763 | 504,614 | 1,009,236 / 63 | 126,449,706 | 143,227,626 |
| wide-4096 | 4,102 | 235 | 478 / 19 | 58,996 | 67,892 |

Payload is not RSS and excludes allocation metadata and the instrumenting map. Competitor requested-payload accounting is not reported: their internal allocation and destruction contracts were not instrumented through a comparable public interface. Uninstrumented process RSS is collected separately for the cross-library resident-memory comparison.

Platform, memory and latency results must be reported with the command, input hash, binary/compiler identity and source snapshot. An unrun platform or unsupported writer is not a zero-time result. Requested allocation payload excludes allocator bookkeeping and differs from process RSS. Historical storage prototypes are not measurements of the final implementation.

## Timed results

These complete timing tables identify the C5 snapshot below. A subsequent bounded insertion-cursor prototype is under validation; its refreshed scored comparison is pending.

Recorded on Apple M5 (10 logical CPUs), 16 GiB RAM, Darwin 27.0.0 arm64. All Jai clients and actual formatter consumers use the same source-built compiler with `-O2 -no_debug_info`; Rust uses the locked release example, and Python is 3.15.0. Compiler SHA-256: `3b4f6b601d51dbb68bd983e9e730ca007c87e0e1b3c3b50b2cc4b56c1f45b5b0`. Canonical source-map SHA-256: `fc2e851ed9ef6914231f2358fa130389945e5280380c5b5ddd693d93e52241a9`. Input sizes/hashes come from the frozen workload manifest; runtime binaries, full source maps and all raw samples remain in the result receipt.

The full comparison retains 3,015 warm samples (three unscored plus twelve measured rounds for each of 201 task/input combinations) and 2,412 cold samples. All 196 actual-executable semantic checks passed, with no unavailable task/input combinations. The earlier partial attempt stopped on a sub-resolution cold lookup and remains preserved; no sample from it enters these tables. Tables give **median [minimum–maximum] in microseconds**, unless another unit is named. Min/max include every measured sample; no outliers were removed.

In these tables, **previous** is the preserved original general implementation, **own** is the current optimized general implementation, and the formatter’s **earlier subset** is the older consumer without the general parser. Both general formatter consumers validate the whole TOML file; only the earlier subset has the narrower behavior.

### Materialization and destruction

| Input | previous | own | community | rust | python |
| --- | --- | --- | --- | --- | --- |
| config-1024 | 14.02 [13.89–14.15] | 9.66 [9.56–9.77] | 10.00 [9.94–10.19] | 7.83 [7.79–7.92] | 66.13 [65.74–66.92] |
| config-8192 | 108.05 [106.78–109.94] | 73.46 [72.42–74.83] | 75.43 [74.83–76.67] | 58.48 [57.83–66.70] | 513.55 [509.24–518.32] |
| config | 2.44 [2.43–2.47] | 1.69 [1.67–1.81] | 1.98 [1.96–1.99] | 1.40 [1.39–1.44] | 11.75 [11.60–11.94] |
| records-102400 | 1,518.04 [1,502.38–1,542.48] | 1,034.62 [1,019.35–1,043.85] | 1,510.95 [1,496.00–1,538.65] | 1,026.84 [999.30–1,056.01] | 8,904.99 [8,749.57–9,375.66] |
| repository-config | 150.99 [150.21–174.64] | 81.84 [80.93–82.88] | 105.63 [105.33–107.40] | 64.20 [63.61–65.45] | 637.04 [615.18–655.16] |
| wide-102400 | 2,316.49 [2,273.37–2,341.84] | 1,817.20 [1,789.08–1,989.97] | 1,561.55 [1,551.76–1,596.79] | 1,341.52 [1,316.53–1,366.09] | 7,352.03 [7,275.96–7,425.79] |
| wide-1048576 | 25,594.12 [25,294.00–26,060.25] | 20,488.62 [19,959.50–21,110.25] | 15,849.12 [15,705.00–16,781.25] | 14,609.51 [14,062.18–18,716.69] | 73,589.49 [71,761.78–76,081.18] |
| wide-10485760 | 279,514.00 [277,541.00–284,265.00] | 226,201.50 [224,604.00–261,370.00] | 169,251.50 [167,999.00–171,080.00] | 152,704.31 [149,493.25–157,663.38] | 743,304.75 [733,000.38–773,263.58] |
| wide-4096 | 82.27 [81.80–83.67] | 62.01 [61.29–62.46] | 67.31 [66.92–69.21] | 48.18 [47.52–58.14] | 309.65 [305.99–331.04] |

### Read, write and consume every output byte

| Input | previous | own | community | rust |
| --- | --- | --- | --- | --- |
| config-1024 | 21.64 [21.52–21.77] | 13.37 [13.23–13.56] | 12.86 [12.75–12.91] | 13.82 [13.63–14.17] |
| config-8192 | 162.57 [161.61–163.80] | 100.07 [99.25–101.92] | 95.82 [94.89–97.62] | 99.84 [97.51–106.44] |
| config | 4.19 [4.16–4.27] | 2.48 [2.45–2.68] | 2.60 [2.58–2.67] | 2.57 [2.55–2.59] |
| records-102400 | 3,764.62 [3,735.06–3,785.38] | 1,453.47 [1,440.97–1,471.44] | 1,931.72 [1,925.68–1,969.56] | 1,946.01 [1,928.17–2,163.65] |
| repository-config | 236.58 [234.97–239.77] | 113.04 [112.39–116.32] | 135.18 [134.17–138.70] | 111.24 [109.94–114.20] |
| wide-102400 | 4,520.35 [4,449.67–4,607.80] | 2,176.45 [2,134.77–2,261.30] | 1,796.80 [1,784.80–1,807.20] | 1,637.41 [1,611.00–1,672.22] |
| wide-1048576 | 49,815.67 [48,555.33–57,766.00] | 23,668.17 [23,262.33–25,826.33] | 18,310.83 [18,162.00–18,559.00] | 17,284.00 [16,809.43–22,042.44] |
| wide-10485760 | 532,602.50 [525,105.00–572,581.00] | 262,277.50 [259,618.00–268,360.00] | 193,361.50 [190,919.00–201,681.00] | 183,133.73 [178,067.08–190,894.21] |
| wide-4096 | 160.32 [159.09–162.69] | 76.49 [75.77–78.32] | 77.93 [76.96–80.54] | 61.61 [60.60–68.78] |

### Write prepared values and consume every output byte

| Input | previous | own | community | rust |
| --- | --- | --- | --- | --- |
| config-1024 | 7.36 [7.30–7.58] | 3.62 [3.57–4.30] | 2.71 [2.66–2.75] | 5.35 [5.28–5.49] |
| config-8192 | 53.98 [53.43–71.11] | 26.19 [25.84–26.39] | 19.82 [19.44–20.18] | 39.48 [38.70–47.31] |
| config | 1.72 [1.71–2.06] | 0.78 [0.77–0.82] | 0.59 [0.58–0.60] | 1.04 [1.01–1.05] |
| records-102400 | 2,252.05 [2,246.50–2,264.71] | 416.24 [412.40–420.69] | 426.15 [417.17–432.34] | 916.55 [908.35–942.87] |
| repository-config | 85.27 [83.89–87.22] | 31.22 [30.47–32.57] | 28.88 [28.50–29.38] | 44.61 [44.09–45.57] |
| wide-102400 | 2,196.86 [2,168.67–2,221.61] | 342.61 [340.47–354.62] | 236.33 [225.09–241.81] | 284.82 [277.61–340.35] |
| wide-1048576 | 24,240.05 [23,913.89–30,335.53] | 3,474.68 [3,447.47–3,704.26] | 2,357.47 [2,310.26–2,379.21] | 2,828.18 [2,715.97–2,917.00] |
| wide-10485760 | 254,162.00 [250,981.33–278,808.67] | 35,631.17 [35,042.00–36,338.00] | 23,044.50 [22,750.33–23,552.00] | 27,670.00 [27,026.93–28,588.47] |
| wide-4096 | 77.05 [76.18–82.40] | 14.57 [14.29–14.67] | 9.97 [9.81–10.44] | 12.65 [12.37–12.93] |

Serializer output sizes differ while independently reparsed values match:

| Input | Previous output B | Ours output B | Community output B | Rust output B |
| --- | --- | --- | --- | --- |
| config-1024 | 1,067 | 1,067 | 946 | 1,039 |
| config-8192 | 8,276 | 8,276 | 7,621 | 8,337 |
| config | 184 | 184 | 141 | 157 |
| records-102400 | 104,433 | 122,673 | 113,060 | 103,459 |
| repository-config | 11,486 | 11,486 | 11,022 | 11,486 |
| wide-102400 | 113,298 | 113,298 | 91,506 | 102,402 |
| wide-1048576 | 1,154,560 | 1,154,560 | 942,620 | 1,048,590 |
| wide-10485760 | 11,494,989 | 11,494,989 | 9,476,537 | 10,485,763 |
| wide-4096 | 4,570 | 4,570 | 3,634 | 4,102 |

### Lookup prepared tables

Each operation looks up every root member. Community and Python lookup are not scored: the community API has no compatible documented projection, and no Python lookup task was implemented. Those omissions are not zero-time results.

| Input | Ours µs | Rust µs | Paired ours / Rust |
| --- | --- | --- | --- |
| config-1024 | 0.03 [0.03–0.03] | 0.03 [0.03–0.03] | 0.84× [0.79–0.92] |
| config-8192 | 0.03 [0.03–0.03] | 0.03 [0.03–0.04] | 0.82× [0.75–0.88] |
| config | 0.02 [0.02–0.02] | 0.02 [0.02–0.02] | 0.86× [0.80–0.95] |
| records-102400 | 0.01 [0.01–0.01] | 0.01 [0.01–0.01] | 1.55× [1.38–1.85] |
| repository-config | 0.03 [0.02–0.03] | 0.03 [0.03–0.03] | 0.81× [0.71–0.88] |
| wide-102400 | 461.89 [441.66–498.22] | 296.92 [292.16–301.59] | 1.56× [1.48–1.71] |
| wide-1048576 | 5,434.50 [5,149.12–6,019.88] | 3,130.58 [3,020.00–3,192.21] | 1.73× [1.63–1.93] |
| wide-10485760 | 67,339.50 [65,069.50–76,643.00] | 33,161.54 [32,495.21–34,257.44] | 2.02× [1.92–2.36] |
| wide-4096 | 12.09 [11.33–12.48] | 7.68 [7.17–7.92] | 1.56× [1.48–1.71] |

### Reject malformed input

Early syntax errors, late errors and duplicate keys are separate tasks, not valid-input throughput. Every measured call must reject.

| Input | Previous µs | Ours µs | Community µs | Rust µs | Python µs |
| --- | --- | --- | --- | --- | --- |
| wide-102400-duplicate | 2,387.22 [2,351.14–2,453.91] | 1,868.86 [1,846.91–2,004.61] | 1,503.08 [1,494.41–1,529.30] | 810.57 [795.89–854.69] | 7,375.89 [7,295.61–7,999.06] |
| wide-102400-early | 33.98 [33.01–36.24] | 35.61 [33.25–39.75] | 41.43 [40.26–43.35] | 813.37 [786.49–829.05] | 71.48 [68.15–84.80] |
| wide-102400-late | 2,364.31 [2,336.18–2,420.39] | 1,875.47 [1,847.68–1,919.24] | 307.12 [304.31–317.98] | 785.86 [775.92–798.19] | 7,372.79 [7,265.57–7,420.29] |
| wide-1048576-duplicate | 26,464.33 [25,517.00–27,638.33] | 21,130.42 [20,708.33–24,832.83] | 15,262.08 [15,096.00–15,631.17] | 8,624.94 [8,347.61–9,570.61] | 72,954.67 [71,704.10–77,201.28] |
| wide-1048576-early | 348.96 [338.02–385.72] | 344.06 [337.08–385.18] | 354.12 [334.94–371.17] | 8,679.13 [8,577.36–8,996.33] | 840.29 [826.36–852.36] |
| wide-1048576-late | 26,208.38 [25,978.12–26,626.06] | 20,793.24 [20,497.00–21,148.94] | 2,986.00 [2,946.53–3,052.18] | 8,431.96 [8,172.57–9,167.42] | 73,225.31 [71,991.83–74,451.96] |
| wide-4096-duplicate | 84.89 [84.00–93.49] | 64.09 [63.44–65.61] | 72.67 [72.27–73.24] | 29.76 [29.53–30.28] | 313.41 [308.88–348.47] |
| wide-4096-early | 1.66 [1.62–1.86] | 1.70 [1.62–2.25] | 9.71 [9.61–10.62] | 29.88 [29.41–30.58] | 4.39 [4.20–4.99] |
| wide-4096-late | 85.09 [83.93–97.64] | 64.42 [63.70–66.32] | 20.93 [20.50–22.32] | 29.41 [28.94–30.84] | 312.12 [308.39–317.71] |

### Actual formatter configuration consumer

All three executables call the real formatter configuration API. Full schema projections match the preserved Rust oracle before timing. The candidate intentionally validates the entire TOML file; the earlier reader handles only its subset.

| Input | Earlier subset µs | Previous general µs | Current general µs | Current / previous | Current / subset |
| --- | --- | --- | --- | --- | --- |
| config-1024 | 7.45 [7.42–7.59] | 17.14 [16.92–19.48] | 11.54 [11.51–11.71] | 0.68× [0.59–0.69] | 1.55× [1.53–1.57] |
| config-8192 | 56.92 [56.38–57.74] | 130.54 [129.03–143.05] | 87.43 [86.52–88.83] | 0.67× [0.62–0.68] | 1.54× [1.51–1.56] |
| config | 1.41 [1.39–1.43] | 3.07 [3.04–3.09] | 2.12 [2.10–2.14] | 0.69× [0.68–0.70] | 1.51× [1.49–1.53] |
| repository-config | 75.18 [74.34–77.04] | 167.66 [166.50–173.71] | 96.39 [95.56–112.81] | 0.57× [0.55–0.67] | 1.28× [1.25–1.51] |

### Matched optimization comparison

Previous and current use authenticated source archives, the same compiler and flags, identical workloads and rotated matched rounds. Previous is the original general implementation, not the older subset formatter reader.

| Input | Read current/previous | Read-write current/previous | Prepared write current/previous | Lookup current/previous |
| --- | --- | --- | --- | --- |
| config-1024 | 0.69× [0.68–0.70] | 0.62× [0.61–0.63] | 0.49× [0.48–0.59] | 1.02× [0.99–1.06] |
| config-8192 | 0.68× [0.67–0.69] | 0.62× [0.61–0.63] | 0.49× [0.37–0.49] | 1.01× [0.96–1.03] |
| config | 0.69× [0.68–0.75] | 0.59× [0.58–0.64] | 0.45× [0.38–0.48] | 1.00× [0.97–1.05] |
| records-102400 | 0.69× [0.67–0.69] | 0.39× [0.38–0.39] | 0.18× [0.18–0.19] | 0.93× [0.91–0.97] |
| repository-config | 0.54× [0.47–0.55] | 0.48× [0.47–0.49] | 0.37× [0.36–0.38] | 0.97× [0.95–1.00] |
| wide-102400 | 0.78× [0.77–0.86] | 0.48× [0.47–0.50] | 0.16× [0.15–0.16] | 0.99× [0.85–1.07] |
| wide-1048576 | 0.80× [0.78–0.83] | 0.48× [0.42–0.52] | 0.14× [0.11–0.15] | 0.97× [0.90–1.05] |
| wide-10485760 | 0.81× [0.80–0.93] | 0.49× [0.46–0.50] | 0.14× [0.13–0.14] | 0.96× [0.87–1.05] |
| wide-4096 | 0.75× [0.75–0.76] | 0.48× [0.47–0.48] | 0.19× [0.18–0.19] | 1.02× [0.88–1.09] |

### Paired task ratios

Ratios match the same input and measured round before taking their median/range. Values above 1 mean ours took longer; these are warm task ratios, not cold startup ratios.

| Input | Read ours/community | Read ours/Rust | Read ours/Python | Write ours/community | Write ours/Rust |
| --- | --- | --- | --- | --- | --- |
| config-1024 | 0.97× [0.95–0.98] | 1.23× [1.22–1.25] | 0.15× [0.14–0.15] | 1.33× [1.30–1.60] | 0.68× [0.65–0.80] |
| config-8192 | 0.97× [0.94–0.99] | 1.26× [1.10–1.28] | 0.14× [0.14–0.15] | 1.32× [1.29–1.34] | 0.66× [0.55–0.68] |
| config | 0.85× [0.84–0.92] | 1.20× [1.18–1.26] | 0.14× [0.14–0.15] | 1.33× [1.29–1.38] | 0.75× [0.74–0.81] |
| records-102400 | 0.69× [0.67–0.70] | 1.00× [0.99–1.03] | 0.12× [0.11–0.12] | 0.98× [0.95–1.00] | 0.45× [0.44–0.46] |
| repository-config | 0.77× [0.76–0.78] | 1.27× [1.25–1.29] | 0.13× [0.12–0.13] | 1.08× [1.05–1.14] | 0.70× [0.69–0.72] |
| wide-102400 | 1.16× [1.14–1.27] | 1.36× [1.32–1.48] | 0.25× [0.24–0.27] | 1.46× [1.41–1.52] | 1.21× [1.00–1.24] |
| wide-1048576 | 1.29× [1.23–1.34] | 1.41× [1.08–1.48] | 0.28× [0.27–0.29] | 1.48× [1.46–1.57] | 1.23× [1.18–1.36] |
| wide-10485760 | 1.34× [1.32–1.54] | 1.49× [1.43–1.67] | 0.31× [0.29–0.35] | 1.55× [1.52–1.59] | 1.29× [1.23–1.31] |
| wide-4096 | 0.92× [0.90–0.93] | 1.28× [1.07–1.31] | 0.20× [0.19–0.20] | 1.45× [1.40–1.48] | 1.15× [1.11–1.17] |

### Fresh-process reads and resident memory

The following representative inputs show the first read operation separately from whole-process CPU, peak RSS and wall time. All nine valid inputs were collected; the full receipt retains the others and all writer/lookup/rejection cold tasks. Whole-process metrics include startup, input loading and preparation. Wall time has approximately 5 ms polling resolution, so small startup differences are not precise. Process RSS is uninstrumented and includes runtime/input/allocator state; it is not document-retained payload.

| Input | Library | Operation µs | Process CPU ms | Peak RSS MiB | Process wall ms |
| --- | --- | --- | --- | --- | --- |
| config | previous | 19.00 [17.00–21.00] | 5.67 [5.46–5.90] | 1.98 [1.98–2.00] | 8.56 [7.37–8.74] |
| config | own | 17.00 [14.00–19.00] | 5.63 [5.55–8.15] | 1.98 [1.97–1.98] | 8.55 [6.99–14.27] |
| config | community | 22.50 [20.00–28.00] | 5.71 [5.58–7.98] | 2.11 [2.09–2.11] | 8.62 [8.37–14.08] |
| config | rust | 20.75 [17.79–22.25] | 5.87 [5.65–7.89] | 2.08 [2.06–2.08] | 8.60 [7.63–9.32] |
| config | python | 464.96 [455.46–510.54] | 20.94 [20.62–21.82] | 15.58 [15.52–15.66] | 26.27 [23.48–31.14] |
| config-8192 | previous | 150.00 [145.00–154.00] | 5.78 [5.66–6.17] | 2.27 [2.27–2.28] | 8.60 [7.56–8.73] |
| config-8192 | own | 108.50 [104.00–218.00] | 5.79 [5.60–6.69] | 2.20 [2.19–2.20] | 8.60 [8.01–14.07] |
| config-8192 | community | 112.00 [108.00–213.00] | 5.77 [5.66–6.81] | 2.39 [2.39–2.39] | 8.61 [7.15–13.59] |
| config-8192 | rust | 112.63 [109.25–121.96] | 5.90 [5.75–6.33] | 2.59 [2.58–2.61] | 8.66 [7.76–14.16] |
| config-8192 | python | 997.69 [979.71–1,042.92] | 21.96 [21.27–22.80] | 15.60 [15.58–15.67] | 30.80 [23.63–31.51] |
| repository-config | previous | 200.50 [194.00–211.00] | 5.83 [5.69–7.06] | 2.20 [2.19–2.20] | 8.56 [8.39–13.74] |
| repository-config | own | 127.50 [122.00–132.00] | 5.78 [5.71–6.75] | 2.17 [2.17–2.17] | 8.58 [7.77–8.73] |
| repository-config | community | 154.00 [149.00–163.00] | 5.76 [5.64–6.03] | 2.42 [2.42–2.42] | 8.56 [8.38–13.86] |
| repository-config | rust | 148.10 [142.21–155.92] | 5.94 [5.78–6.31] | 2.55 [2.55–2.55] | 8.59 [8.42–13.72] |
| repository-config | python | 1,140.35 [1,122.38–1,153.04] | 21.67 [21.45–22.42] | 15.66 [15.64–15.75] | 28.64 [23.36–31.31] |
| wide-1048576 | previous | 28,103.00 [27,581.00–28,519.00] | 34.63 [34.02–35.17] | 31.22 [31.22–31.23] | 38.74 [36.18–43.97] |
| wide-1048576 | own | 22,064.50 [21,530.00–22,734.00] | 28.28 [27.72–28.98] | 23.92 [23.92–23.92] | 31.25 [30.57–37.25] |
| wide-1048576 | community | 18,096.00 [17,891.00–19,836.00] | 24.03 [23.73–26.68] | 39.45 [39.45–39.45] | 30.85 [28.92–31.50] |
| wide-1048576 | rust | 16,900.35 [16,326.21–18,016.12] | 22.91 [22.34–23.98] | 35.69 [35.69–35.69] | 30.78 [29.02–31.33] |
| wide-1048576 | python | 73,043.17 [72,503.21–74,353.42] | 94.06 [93.24–95.63] | 25.78 [25.77–25.88] | 98.78 [96.25–104.98] |
| wide-10485760 | previous | 302,440.50 [299,287.00–321,014.00] | 317.47 [314.06–335.70] | 270.44 [270.44–270.45] | 323.34 [319.57–344.37] |
| wide-10485760 | own | 243,001.00 [238,590.00–257,317.00] | 254.55 [250.28–266.23] | 201.12 [201.12–201.12] | 259.31 [255.12–276.80] |
| wide-10485760 | community | 188,953.50 [185,867.00–207,143.00] | 196.81 [194.94–212.55] | 335.81 [335.81–335.86] | 206.10 [198.04–218.24] |
| wide-10485760 | rust | 175,222.60 [171,943.88–194,712.75] | 182.04 [178.57–200.68] | 269.71 [269.70–269.73] | 187.57 [183.11–207.14] |
| wide-10485760 | python | 741,626.31 [734,824.29–753,783.58] | 763.84 [756.55–777.15] | 111.06 [111.05–111.14] | 772.89 [764.22–789.43] |

Across all cold tasks, 65 samples in 10 task/input combinations were below their timer resolution. They remain in the raw report with their zero reading and a censoring flag; they are not interpreted as zero execution cost. No cold ratios are claimed.


## Reproducing the comparison

The native benchmark uses the existing `tools/bench` sandbox, process measurements and statistics. The measured community writer preset is scientific with 16 fractional digits (17 significant digits) and no zero removal; it passes the complete valid/encoder inventory and avoids the padding generated by zero removal in fixed-width display formatting. Output byte counts remain part of every writer result. The community comparison harness currently supports macOS/Linux, where its compiler/runtime subprocesses have a supported sandbox launcher. Windows core correctness remains required; community comparison is not run there because these subprocess wrappers do not use the Windows AppContainer launcher.

Prepare the pinned community source opaquely, without opening its implementation. Build the locked Rust example with a source/binary receipt. Build timing executables separately from execution:

```sh
python3 tools/stdlib/toml/prepare_community.py --archive /tmp/toml-jai.tar.gz --output /tmp/toml-imports
python3 tools/stdlib/toml/build_oracle.py --target-dir /tmp/toml-target --receipt /tmp/toml-oracle.json
python3 tools/stdlib/toml/workloads.py --output /tmp/toml-inputs
python3 benchmarks/compare/toml/measure.py --scratch /tmp/toml-bench --jaic /absolute/path/to/jaic --community-dir /tmp/toml-imports --community-archive /tmp/toml-jai.tar.gz --baseline-stdlib /absolute/path/to/baseline/stdlib --build
python3 benchmarks/compare/toml/measure.py --scratch /tmp/toml-bench --rust /tmp/toml-target/release/examples/toml-oracle --rust-receipt /tmp/toml-oracle.json --workloads /tmp/toml-inputs --invalid-workloads /tmp/toml-inputs/invalid --verify-only
```

Use the coordinated heavy-job gate for builds and an exclusive quiet-machine lease for the next pilot/run commands. Do not hold a quiet lease indefinitely while core correctness builds are waiting. The baseline is a physical owned stdlib checkout of the recorded repository commit; symlinking its internal modules to another root violates module privacy checks.

```sh
python3 benchmarks/compare/toml/measure.py --scratch /tmp/toml-bench --rust /tmp/toml-target/release/examples/toml-oracle --rust-receipt /tmp/toml-oracle.json --workloads /tmp/toml-inputs --invalid-workloads /tmp/toml-inputs/invalid --pilot
python3 benchmarks/compare/toml/measure.py --scratch /tmp/toml-bench --freeze-plan
python3 benchmarks/compare/toml/measure.py --scratch /tmp/toml-bench --rust /tmp/toml-target/release/examples/toml-oracle --rust-receipt /tmp/toml-oracle.json --workloads /tmp/toml-inputs --invalid-workloads /tmp/toml-inputs/invalid --batch-plan /tmp/toml-bench/batch-plan.json --run
```

On this Mac, `clock_getres(CLOCK_MONOTONIC)` reports a 1,000 ns resolution for the Basic clock used by both Jai clients. Python's reported performance-counter resolution is 42 ns; Rust's `Instant` resolution is not queried. A single cold lookup can therefore report zero elapsed nanoseconds. Such samples are retained and marked **below timer resolution**, not reported as zero cost; warm batches must still report positive elapsed time. Cold summaries retain the count of censored samples. Do not compute cold point-latency ratios when either side contains censored samples. Whole-process measurements remain separate.

The pilot fixes batch counts per input/task family before scoring. Warm batches use three internal warmup operations; separate fresh processes perform exactly one operation without internal warmup, with the OS filesystem cache already warm. Cold process wall/CPU/RSS include input loading and preparation, while the first-operation clock excludes setup. Whole-process wall time uses the existing 5 ms polling interval and is reported with that resolution limitation. The warm run retains three unscored warmup batches and twelve rotated measured rounds; the cold process run retains twelve rounds. Order reverses on alternate rounds to balance paired consumers. All samples survive in an append-only journal and the final report; failed or slow samples are not removed. Reports name the CPU/OS/memory, compiler and executable hashes, full owned stdlib and Python-tool snapshots, authenticated community archive/tree, Rust build receipt and input hashes. A changed source, executable, runtime or input invalidates the frozen plan.

Tasks separate materialization/destruction, read plus write, writing prebuilt values, lookup and invalid-input rejection. Benchmark parsing raises the retained-payload limit to 1 GiB for the 10 MiB wide document; other production defaults remain active. Python supplies no writer. Community lookup is not scored because the documented general-value API lacks a compatible projection; absence is not zero time. Both Jai libraries use the same resettable pool, but ours also executes `free_toml` before resetting, while the public community API uses reset alone. Two lazily committed 1 GiB address-space reservations do not imply 2 GiB of resident memory.

Parser tasks include UTF-8 validation/decoding of preloaded bytes. Writer tasks consume every output byte. Untimed checks compare exact numeric types, signed floating zero, civil fields and offsets on the shared millisecond profile; they verify the actual Rust binary and both actual formatter benchmark executables. Full formatter projections are checked against the preserved Rust schema, and measured lookup/serialization/consumer checksums must match those verified values. Raw precision/leap probes remain separate so Python conversion cannot conceal stronger-contract differences.

The separate allocation profiler parses, traverses all values, looks up every root key and destroys the document. It asserts that the retained-byte ledger matches the counted allocator and that destruction leaves zero payload. Compile separately, then execute under the same exclusive profiling lease:

```sh
jaic tools/stdlib/toml/memory.jai -O2 -no_debug_info -exe /tmp/toml-memory -output_path /tmp/toml-memory-build
/tmp/toml-memory /tmp/toml-inputs/repository-config.toml
```

It reports requested allocation calls, resize calls, retained and peak payload; tracker storage is excluded. Instrumented process RSS includes tracker overhead and must not be presented as production parser RSS.

The scripts verify release/archive checksums before use. Review a pin update as a fixture change: verify the resolved revision, release hashes and license, inspect the profile inventory/count difference, rerun both modes and all comparators, then update this matrix and measured tables together. Do not silently substitute a new installed Python, Rust dependency or community revision in an existing result.
