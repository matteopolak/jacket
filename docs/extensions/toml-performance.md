# TOML measurements

The [current follow-up measurements](toml-performance-followup.md) include the key-comparison, writer and source-key changes. The broad C6 tables below remain historical, including cold/consumer/community/Python results.

These measurements compare the general [TOML module](toml.md), its actual formatter consumer and pinned independent libraries. Allocation profiling and executable size are separate from timed parser/writer tasks.

## Historical C6 storage and formatter artifacts

The actual formatter artifact comparison uses the same compiler binary and `-O2 -no_debug_info`, with the earlier formatter from repository base `d072effb268b18d2b16ebf27d69d36c14859909d` and this migration. The baseline stdlib is a physical owned source snapshot so internal module privacy rules remain active.

| Real formatter artifact | Earlier subset reader | General TOML consumer | Increase |
| --- | ---: | ---: | ---: |
| Native macOS arm64 | 565,920 B | 663,240 B | 97,320 B (17.2%) |
| WASI | 500,666 B | 595,327 B | 94,661 B (18.9%) |

These are whole formatter executable sizes, not isolated parser object sizes or browser compiler-bundle deltas. Executable size does not establish speed.

The storage decision compared three source-owned, stable-pointer prototypes before adding the final source-history and wide-table metadata. On the same 15,169-byte repository configuration (494 nodes), the compact union reduced payload substantially. Chunk allocation reduced allocation calls but retained more unused capacity, so the final design keeps individually allocated compact nodes. All prototypes parsed, traversed, looked up every root member and destroyed the document with the same checksum and zero retained allocations.

| Historical storage prototype | Node bytes | Allocations | Resizes | Retained payload | Peak payload |
| --- | ---: | ---: | ---: | ---: | ---: |
| Separate scalar fields, heap nodes | 256 | 1,706 | 33 | 185,710 B | 198,397 B |
| Compact union, heap nodes | 136 | 1,706 | 33 | 126,430 B | 139,117 B |
| Compact union, chunked nodes | 136 | 1,218 | 33 | 140,529 B | 153,216 B |

These are historical representation measurements, not final footprint or latency results. The chunked prototype retained 11.2% more payload than the compact heap on this configuration and about 22% more on the 4 KiB wide-table input. Stable node addresses alone do not require one allocation per node; retained capacity on actual configuration inputs determined this choice.

Before the source-key changes, the 144-byte node implementation was profiled separately on all nine valid benchmark inputs. Source history is disabled, as in the generic parser timings. Each row includes the owning source copy and requested capacity; every retained-byte ledger matched the counted allocator, and every destruction returned payload to zero. The repository input differs from the earlier prototype input because it includes this module's documentation registration.

| Historical C6 input | Input B | Nodes | Allocations / resizes | Retained payload B | Peak payload B |
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

## Historical C6 timed results

Recorded on Apple M5 (10 logical CPUs), 16 GiB RAM, Darwin 27.0.0 arm64. All Jai clients and actual formatter consumers use the same source-built compiler with `-O2 -no_debug_info`; Rust uses the locked release example, and Python is 3.15.0. Compiler SHA-256: `3b4f6b601d51dbb68bd983e9e730ca007c87e0e1b3c3b50b2cc4b56c1f45b5b0`. Canonical source-map SHA-256: `05f346438ae1ef9f608f23f9d7ca26da785db37ed0e5216364de21da4443340c`. Input sizes/hashes come from the frozen workload manifest; runtime binaries, full source maps and all raw samples remain in the result receipt.

The full comparison retains 3,015 warm samples (three unscored plus twelve measured rounds for each of 201 task/input combinations) and 2,412 cold samples. All 196 actual-executable semantic checks passed, with no unavailable task/input combinations. The earlier partial attempt stopped on a sub-resolution cold lookup and remains preserved; no sample from it enters these tables. Tables give **median [minimum–maximum] in microseconds**, unless another unit is named. Min/max include every measured sample; no outliers were removed.

In these tables, **previous** is the preserved original general implementation, **own** is the C6 optimized general implementation, and the formatter’s **earlier subset** is the older consumer without the general parser. Both general formatter consumers validate the whole TOML file; only the earlier subset has the narrower behavior.

### Materialization and destruction

| Input | previous | own | community | rust | python |
| --- | --- | --- | --- | --- | --- |
| config-1024 | 14.35 [14.17–14.88] | 10.13 [10.05–10.43] | 9.88 [9.81–10.12] | 7.86 [7.79–8.15] | 66.51 [65.64–67.40] |
| config-8192 | 110.14 [109.53–111.21] | 77.00 [76.24–126.38] | 74.53 [74.19–75.68] | 58.80 [58.11–59.60] | 518.20 [511.85–522.37] |
| config | 2.49 [2.46–2.73] | 1.77 [1.75–1.81] | 1.94 [1.93–1.97] | 1.41 [1.40–1.41] | 11.86 [11.80–11.96] |
| records-102400 | 1,533.90 [1,522.57–1,550.13] | 1,068.86 [1,053.04–1,094.11] | 1,477.15 [1,469.98–1,501.74] | 1,021.21 [1,013.73–1,034.10] | 8,961.85 [8,831.18–9,191.80] |
| repository-config | 154.60 [153.56–158.35] | 80.96 [79.69–83.54] | 104.19 [103.48–105.21] | 64.72 [63.40–66.72] | 636.57 [617.05–704.19] |
| wide-102400 | 2,340.26 [2,291.29–2,415.42] | 1,515.66 [1,486.08–1,536.21] | 1,520.99 [1,509.08–1,530.39] | 1,342.92 [1,303.41–1,371.91] | 7,359.20 [7,265.82–7,663.04] |
| wide-1048576 | 25,819.38 [25,433.00–26,735.75] | 16,939.50 [16,559.25–17,261.25] | 15,578.75 [15,457.00–15,775.50] | 14,430.82 [14,176.79–14,910.12] | 73,392.12 [72,055.44–75,959.97] |
| wide-10485760 | 281,011.50 [278,397.00–287,176.00] | 192,504.00 [189,509.00–195,459.00] | 167,650.00 [164,538.00–176,406.00] | 153,225.06 [150,164.21–160,332.50] | 750,707.62 [743,808.96–781,881.88] |
| wide-4096 | 83.42 [82.45–110.97] | 54.22 [53.59–56.50] | 65.50 [65.06–67.21] | 48.44 [47.84–49.07] | 311.64 [305.19–318.93] |

### Read, write and consume every output byte

| Input | previous | own | community | rust |
| --- | --- | --- | --- | --- |
| config-1024 | 21.93 [21.80–34.95] | 14.20 [14.10–15.03] | 12.92 [12.84–14.20] | 13.81 [13.73–14.02] |
| config-8192 | 165.10 [163.89–177.58] | 107.02 [105.70–110.99] | 96.20 [95.76–97.84] | 99.32 [98.43–104.34] |
| config | 4.25 [4.23–4.45] | 2.64 [2.61–2.68] | 2.60 [2.58–2.65] | 2.57 [2.54–2.62] |
| records-102400 | 3,864.59 [3,850.44–4,067.69] | 1,508.05 [1,488.59–1,555.41] | 1,929.75 [1,915.47–1,973.84] | 1,948.32 [1,931.85–1,980.73] |
| repository-config | 240.85 [238.23–245.60] | 113.39 [112.21–119.93] | 135.79 [135.02–140.42] | 111.21 [109.45–114.03] |
| wide-102400 | 4,556.55 [4,494.71–4,681.29] | 1,866.19 [1,838.68–1,882.94] | 1,787.26 [1,770.52–1,800.74] | 1,630.23 [1,613.35–1,661.77] |
| wide-1048576 | 50,005.83 [49,218.00–51,173.33] | 20,484.83 [20,047.00–20,740.67] | 18,200.83 [17,977.67–18,916.33] | 17,305.76 [17,075.49–18,170.00] |
| wide-10485760 | 540,510.00 [534,738.00–547,970.00] | 226,993.50 [222,404.00–262,879.00] | 192,771.50 [189,786.00–194,830.00] | 183,523.73 [179,280.71–187,510.71] |
| wide-4096 | 162.06 [160.83–163.72] | 68.91 [68.30–70.90] | 76.98 [76.65–78.15] | 61.72 [60.85–63.60] |

### Write prepared values and consume every output byte

| Input | previous | own | community | rust |
| --- | --- | --- | --- | --- |
| config-1024 | 7.43 [7.40–7.74] | 3.94 [3.91–4.01] | 2.72 [2.66–2.77] | 5.36 [5.29–5.49] |
| config-8192 | 54.05 [53.82–59.42] | 28.73 [28.18–29.38] | 19.26 [18.95–19.57] | 38.95 [38.58–40.12] |
| config | 1.75 [1.73–1.77] | 0.84 [0.83–0.87] | 0.59 [0.59–0.60] | 1.03 [1.01–1.07] |
| records-102400 | 2,343.20 [2,329.15–2,355.11] | 442.57 [438.17–453.41] | 427.98 [424.77–437.75] | 923.24 [913.35–1,362.83] |
| repository-config | 85.93 [84.95–87.05] | 31.74 [31.28–32.51] | 28.74 [28.42–29.20] | 45.19 [44.46–47.75] |
| wide-102400 | 2,245.54 [2,213.60–2,263.80] | 348.23 [341.38–349.63] | 238.79 [236.20–240.42] | 284.86 [280.84–292.22] |
| wide-1048576 | 24,541.10 [24,046.90–24,740.00] | 3,517.33 [3,463.33–4,441.76] | 2,363.12 [2,331.05–2,418.43] | 2,808.41 [2,743.64–2,891.86] |
| wide-10485760 | 261,540.17 [256,388.67–284,067.00] | 35,659.67 [35,303.00–36,089.67] | 23,281.67 [22,636.00–24,727.33] | 27,622.68 [26,801.33–28,818.31] |
| wide-4096 | 79.03 [78.14–79.82] | 14.57 [14.27–14.87] | 9.94 [9.88–10.23] | 12.65 [12.49–13.06] |

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
| config-1024 | 0.03 [0.03–0.03] | 0.03 [0.03–0.03] | 0.84× [0.79–0.89] |
| config-8192 | 0.03 [0.03–0.03] | 0.03 [0.03–0.03] | 0.86× [0.81–0.88] |
| config | 0.02 [0.02–0.02] | 0.02 [0.02–0.02] | 0.88× [0.84–1.01] |
| records-102400 | 0.01 [0.01–0.01] | 0.01 [0.01–0.01] | 1.59× [1.41–1.62] |
| repository-config | 0.03 [0.02–0.03] | 0.03 [0.03–0.03] | 0.84× [0.79–0.91] |
| wide-102400 | 467.41 [444.72–495.25] | 298.88 [288.15–304.47] | 1.57× [1.46–1.67] |
| wide-1048576 | 5,421.66 [5,239.50–5,691.12] | 3,150.87 [3,001.42–3,237.33] | 1.73× [1.68–1.83] |
| wide-10485760 | 68,093.25 [65,866.00–69,096.00] | 33,370.02 [32,461.92–33,798.42] | 2.03× [1.96–2.12] |
| wide-4096 | 11.48 [10.52–13.69] | 7.58 [7.45–7.80] | 1.49× [1.39–1.81] |

### Reject malformed input

Early syntax errors, late errors and duplicate keys are separate tasks, not valid-input throughput. Every measured call must reject.

| Input | Previous µs | Ours µs | Community µs | Rust µs | Python µs |
| --- | --- | --- | --- | --- | --- |
| wide-102400-duplicate | 2,382.97 [2,342.61–2,534.20] | 1,575.62 [1,555.81–1,632.46] | 1,464.19 [1,456.68–1,527.53] | 813.90 [782.28–998.58] | 7,432.59 [7,311.48–8,292.06] |
| wide-102400-early | 34.78 [33.44–36.61] | 33.93 [33.28–48.89] | 42.07 [40.56–44.25] | 815.08 [786.33–862.38] | 71.22 [69.86–76.19] |
| wide-102400-late | 2,383.96 [2,350.24–2,481.72] | 1,566.62 [1,535.27–1,674.67] | 297.19 [294.29–309.13] | 796.80 [783.39–847.72] | 7,454.28 [7,369.16–7,682.99] |
| wide-1048576-duplicate | 26,379.00 [25,951.83–27,174.17] | 17,662.33 [17,122.67–18,268.67] | 14,763.17 [14,706.00–14,964.50] | 8,825.63 [8,526.93–9,078.15] | 73,713.98 [72,457.74–76,745.18] |
| wide-1048576-early | 368.48 [342.42–427.82] | 346.12 [338.58–405.52] | 351.86 [336.97–426.28] | 8,683.11 [8,367.83–9,344.71] | 845.43 [834.11–850.39] |
| wide-1048576-late | 26,323.00 [25,720.11–27,540.28] | 17,366.00 [16,880.28–17,820.17] | 2,887.89 [2,820.50–3,096.22] | 8,461.71 [8,300.97–10,018.40] | 73,817.01 [72,320.81–76,821.27] |
| wide-4096-duplicate | 85.52 [84.33–85.86] | 56.40 [55.81–56.88] | 70.47 [70.04–71.03] | 30.01 [29.26–30.62] | 313.28 [307.47–321.77] |
| wide-4096-early | 1.70 [1.62–2.10] | 1.67 [1.63–1.80] | 9.70 [9.58–9.92] | 29.85 [29.31–30.18] | 4.40 [4.29–4.48] |
| wide-4096-late | 84.54 [84.20–85.53] | 56.21 [55.53–57.61] | 20.29 [20.02–23.04] | 29.47 [29.02–29.77] | 312.23 [308.39–317.47] |

### Actual formatter configuration consumer

All three executables call the real formatter configuration API. Full schema projections match the preserved Rust oracle before timing. The candidate intentionally validates the entire TOML file; the earlier reader handles only its subset.

| Input | Earlier subset µs | Previous general µs | Current general µs | Current / previous | Current / subset |
| --- | --- | --- | --- | --- | --- |
| config-1024 | 8.18 [8.06–8.28] | 17.21 [17.12–17.53] | 12.39 [12.27–12.47] | 0.72× [0.70–0.73] | 1.52× [1.49–1.54] |
| config-8192 | 62.03 [61.04–63.61] | 131.83 [131.35–149.87] | 93.50 [92.68–94.48] | 0.71× [0.63–0.72] | 1.51× [1.48–1.53] |
| config | 1.52 [1.50–1.57] | 3.09 [3.07–3.14] | 2.26 [2.22–2.32] | 0.73× [0.71–0.75] | 1.49× [1.43–1.52] |
| repository-config | 82.90 [81.52–85.25] | 171.59 [169.09–176.30] | 96.91 [94.58–98.31] | 0.56× [0.55–0.58] | 1.17× [1.14–1.20] |

### Matched optimization comparison

Previous and current use authenticated source archives, the same compiler and flags, identical runtime harness bytes, identical workloads and rotated matched rounds. The previous control rebuilds the original complete general stdlib with the current harness; its historical measurements remain separately preserved. Previous is the original general implementation, not the older subset formatter reader.

| Input | Read current/previous | Read-write current/previous | Prepared write current/previous | Lookup current/previous |
| --- | --- | --- | --- | --- |
| config-1024 | 0.70× [0.68–0.73] | 0.65× [0.43–0.66] | 0.53× [0.51–0.54] | 1.01× [0.98–1.05] |
| config-8192 | 0.70× [0.69–1.14] | 0.65× [0.61–0.66] | 0.53× [0.49–0.54] | 1.02× [0.98–1.05] |
| config | 0.71× [0.66–0.72] | 0.62× [0.59–0.63] | 0.48× [0.48–0.49] | 0.98× [0.93–1.05] |
| records-102400 | 0.70× [0.68–0.71] | 0.39× [0.37–0.40] | 0.19× [0.19–0.19] | 0.96× [0.92–0.99] |
| repository-config | 0.52× [0.52–0.53] | 0.47× [0.46–0.49] | 0.37× [0.36–0.38] | 0.99× [0.98–1.03] |
| wide-102400 | 0.65× [0.63–0.67] | 0.41× [0.40–0.41] | 0.16× [0.15–0.16] | 0.96× [0.73–1.07] |
| wide-1048576 | 0.65× [0.63–0.67] | 0.41× [0.39–0.42] | 0.14× [0.14–0.18] | 0.93× [0.87–1.01] |
| wide-10485760 | 0.68× [0.67–0.70] | 0.42× [0.41–0.48] | 0.14× [0.13–0.14] | 0.92× [0.68–0.99] |
| wide-4096 | 0.65× [0.51–0.66] | 0.43× [0.42–0.44] | 0.18× [0.18–0.19] | 0.96× [0.89–1.14] |

### Paired task ratios

Ratios match the same input and measured round before taking their median/range. Values above 1 mean ours took longer; these are warm task ratios, not cold startup ratios.

| Input | Read ours/community | Read ours/Rust | Read ours/Python | Write ours/community | Write ours/Rust |
| --- | --- | --- | --- | --- | --- |
| config-1024 | 1.02× [1.00–1.05] | 1.28× [1.25–1.33] | 0.15× [0.15–0.16] | 1.46× [1.42–1.50] | 0.74× [0.72–0.75] |
| config-8192 | 1.03× [1.02–1.67] | 1.31× [1.29–2.12] | 0.15× [0.15–0.24] | 1.49× [1.47–1.53] | 0.74× [0.72–0.75] |
| config | 0.91× [0.90–0.94] | 1.26× [1.24–1.29] | 0.15× [0.15–0.15] | 1.42× [1.40–1.47] | 0.82× [0.79–0.86] |
| records-102400 | 0.72× [0.70–0.74] | 1.05× [1.03–1.07] | 0.12× [0.12–0.12] | 1.03× [1.02–1.06] | 0.48× [0.33–0.49] |
| repository-config | 0.78× [0.76–0.79] | 1.25× [1.21–1.28] | 0.13× [0.12–0.13] | 1.11× [1.08–1.13] | 0.70× [0.67–0.73] |
| wide-102400 | 1.00× [0.97–1.01] | 1.12× [1.10–1.16] | 0.21× [0.19–0.21] | 1.46× [1.43–1.48] | 1.22× [1.18–1.24] |
| wide-1048576 | 1.08× [1.06–1.11] | 1.17× [1.13–1.21] | 0.23× [0.22–0.24] | 1.49× [1.45–1.88] | 1.25× [1.21–1.55] |
| wide-10485760 | 1.15× [1.09–1.18] | 1.25× [1.21–1.29] | 0.26× [0.25–0.26] | 1.53× [1.44–1.59] | 1.29× [1.25–1.33] |
| wide-4096 | 0.83× [0.80–0.86] | 1.12× [1.10–1.15] | 0.17× [0.17–0.18] | 1.46× [1.42–1.50] | 1.16× [1.11–1.18] |

### Fresh-process reads and resident memory

The following representative inputs show the first read operation separately from whole-process CPU, peak RSS and wall time. All nine valid inputs were collected; the full receipt retains the others and all writer/lookup/rejection cold tasks. Whole-process metrics include startup, input loading and preparation. Wall time has approximately 5 ms polling resolution, so small startup differences are not precise. Process RSS is uninstrumented and includes runtime/input/allocator state; it is not document-retained payload.

| Input | Library | Operation µs | Process CPU ms | Peak RSS MiB | Process wall ms |
| --- | --- | --- | --- | --- | --- |
| config | previous | 20.00 [17.00–33.00] | 5.82 [5.71–6.42] | 2.00 [2.00–2.00] | 8.67 [7.77–8.89] |
| config | own | 20.50 [18.00–32.00] | 5.83 [5.71–11.61] | 2.02 [2.02–2.02] | 8.74 [7.85–15.85] |
| config | community | 23.00 [22.00–27.00] | 5.76 [5.65–6.20] | 2.11 [2.11–2.11] | 8.63 [7.75–14.18] |
| config | rust | 19.83 [18.00–21.71] | 6.00 [5.78–6.19] | 2.08 [2.08–2.08] | 8.64 [6.96–13.82] |
| config | python | 469.62 [459.71–497.29] | 21.56 [20.96–22.85] | 15.59 [15.52–15.62] | 30.30 [23.20–31.29] |
| config-8192 | previous | 152.50 [149.00–164.00] | 5.89 [5.77–6.14] | 2.28 [2.28–2.28] | 8.69 [8.54–8.79] |
| config-8192 | own | 115.00 [112.00–119.00] | 5.88 [5.71–6.33] | 2.23 [2.23–2.23] | 8.66 [7.40–14.89] |
| config-8192 | community | 114.00 [110.00–131.00] | 5.91 [5.72–6.09] | 2.39 [2.38–2.39] | 8.70 [8.14–8.78] |
| config-8192 | rust | 119.08 [114.33–139.92] | 6.09 [5.96–6.35] | 2.60 [2.58–2.61] | 8.76 [7.71–15.35] |
| config-8192 | python | 1,011.06 [982.38–1,061.08] | 22.07 [21.74–23.87] | 15.61 [15.55–15.66] | 30.55 [27.14–31.38] |
| repository-config | previous | 206.00 [201.00–223.00] | 6.06 [5.82–6.76] | 2.20 [2.20–2.20] | 8.61 [7.26–14.44] |
| repository-config | own | 132.50 [123.00–137.00] | 5.95 [5.81–6.43] | 2.19 [2.19–2.19] | 8.63 [7.40–8.83] |
| repository-config | community | 157.50 [152.00–170.00] | 6.07 [5.81–6.34] | 2.42 [2.42–2.42] | 8.66 [8.50–13.96] |
| repository-config | rust | 154.19 [146.08–163.96] | 6.12 [5.91–6.60] | 2.55 [2.55–2.55] | 8.60 [7.76–15.01] |
| repository-config | python | 1,148.33 [1,132.67–1,193.04] | 22.00 [21.88–22.82] | 15.66 [15.62–15.73] | 30.47 [27.79–31.31] |
| wide-1048576 | previous | 28,231.50 [27,729.00–28,612.00] | 35.09 [34.68–35.55] | 31.23 [31.23–31.23] | 37.69 [36.58–42.86] |
| wide-1048576 | own | 18,576.50 [18,085.00–20,882.00] | 25.04 [24.38–32.42] | 23.94 [23.94–23.94] | 31.03 [26.95–37.09] |
| wide-1048576 | community | 17,609.00 [17,258.00–18,620.00] | 23.69 [23.30–24.97] | 39.45 [39.45–39.45] | 30.92 [28.21–31.41] |
| wide-1048576 | rust | 16,748.50 [16,545.42–17,624.42] | 22.84 [22.66–23.58] | 35.69 [35.67–35.70] | 30.02 [28.65–31.35] |
| wide-1048576 | python | 74,139.75 [72,585.21–76,837.79] | 95.26 [93.68–97.92] | 25.79 [25.73–25.88] | 102.46 [100.13–104.61] |
| wide-10485760 | previous | 303,539.00 [299,187.00–313,432.00] | 319.81 [314.42–329.28] | 270.45 [270.45–270.45] | 325.72 [319.18–336.74] |
| wide-10485760 | own | 207,007.00 [204,538.00–211,995.00] | 219.25 [216.38–224.39] | 201.14 [201.14–201.14] | 222.82 [218.98–237.56] |
| wide-10485760 | community | 186,161.00 [182,421.00–189,944.00] | 195.27 [190.44–198.27] | 335.81 [335.81–335.83] | 203.05 [197.00–205.99] |
| wide-10485760 | rust | 178,367.44 [173,254.71–181,244.75] | 185.67 [180.16–188.66] | 269.70 [269.70–269.73] | 189.85 [184.92–198.82] |
| wide-10485760 | python | 757,537.23 [744,488.17–767,012.25] | 780.07 [767.54–789.50] | 111.06 [110.98–111.14] | 790.13 [775.44–798.00] |

Across all cold tasks, 56 samples in 10 task/input combinations were below their timer resolution. They remain in the raw report with their zero reading and a censoring flag; they are not interpreted as zero execution cost. No cold ratios are claimed.


The 1 KiB reader remains slower than Rust, although both validate complete TOML. Ours retains the source and spans, owns stable nodes and keys, accounts capacities and explicitly destroys the document. The profiles do not isolate each operation's contribution, so the remaining gap is not claimed to be irreducible or entirely caused by validation.

The 10 MiB prepared writer emits 11,494,989 bytes, versus 9,476,537 for the community library and 10,485,763 for Rust; every byte is consumed. It also validates the current raw value tree, UTF-8, scalar fields, depth, node counts and duplicate keys, then checks each output append. Existing profiles show ordinary emission, current-value checks and conversion after redundant staging was removed. These observed operations do not explain every remaining microsecond, and parse-task teardown differences do not explain prepared-write results.

The earlier subset formatter has narrower behavior; the previous general formatter is the matching semantic control. Current configuration processing additionally validates unrelated sections and projects ordered source declarations before copying surviving settings. The historical heap/chunk comparison establishes retained-memory tradeoffs, not latency superiority. No competitor gap is attributed solely to strictness.

## Reproducing the comparison

The native benchmark uses the existing `tools/bench` sandbox, process measurements and statistics. The measured community writer preset is scientific with 16 fractional digits (17 significant digits) and no zero removal; it passes the complete valid/encoder inventory and avoids the padding generated by zero removal in fixed-width display formatting. Output byte counts remain part of every writer result. The community comparison harness currently supports macOS/Linux, where its compiler/runtime subprocesses have a supported sandbox launcher. Windows core correctness remains required; community comparison is not run there because these subprocess wrappers do not use the Windows AppContainer launcher.

Prepare the pinned community source opaquely, without opening its implementation. Build the locked Rust example with a source/binary receipt. Build timing executables separately from execution:

```sh
export PYTHONPATH="$PWD/tools"
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
