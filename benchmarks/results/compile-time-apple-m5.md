# jaic compile-time benchmark (2026-10-06)

Machine: Apple M5, 10 cores, 16.0 GiB, Darwin 27.0, LLVM 22.1.1.  
jaic: `cfd30421ef74` (dirty), rustc 1.100.0-nightly (17fd5b8a3 2026-08-28). Runs per workload: 5 (cold = first run, warm = median of the rest).

| workload | mode | cold s | warm s | min-max s | peak RSS MiB | front end s | codegen s | link s |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| focus | check | 4.19 | 2.73 | 2.56-4.19 | 846 | 2.69 | 0.00 | 0.00 |
| focus | build-O0 | 3.92 | 4.12 | 3.92-4.84 | 1272 | 2.93 | 0.86 | 0.18 |
| focus | build-O2 | 23.39 | 23.65 | 19.28-30.81 | 1030 | 8.99 | 14.28 | 0.16 |
| jails | check | 0.36 | 0.34 | 0.32-0.39 | 156 | 0.31 | 0.00 | 0.00 |
| jails | build-O0 | 0.71 | 0.74 | 0.67-0.77 | 257 | 0.39 | 0.18 | 0.06 |
| jails | build-O2 | 9.46 | 4.31 | 4.24-10.38 | 360 | 0.31 | 3.87 | 0.05 |
| sgpu-examples | check | 0.94 | 0.84 | 0.82-0.94 | 203 | 0.81 | 0.00 | 0.00 |
| jaison-tests | check | 0.06 | 0.06 | 0.06-0.07 | 51 | 0.03 | 0.00 | 0.00 |
| jaison-tests | build-O0 | 0.21 | 0.20 | 0.20-0.21 | 109 | 0.03 | 0.07 | 0.04 |
| jaison-tests | build-O2 | 1.47 | 1.33 | 1.23-2.29 | 154 | 0.04 | 1.15 | 0.07 |
| open-jai-getrect | check | 0.13 | 0.15 | 0.12-0.20 | 88 | 0.11 | 0.00 | 0.00 |
| open-jai-getrect | build-O0 | 0.51 | 0.43 | 0.43-0.51 | 165 | 0.11 | 0.11 | 0.14 |
| open-jai-getrect | build-O2 | 1.94 | 3.24 | 1.94-4.41 | 226 | 0.11 | 2.86 | 0.14 |
| chess-jai | check | 2.58 | 2.73 | 2.58-3.09 | 243 | 2.71 | 0.00 | 0.00 |
| chess-jai | build-O0 | 4.28 | 5.34 | 3.80-8.40 | 716 | 3.90 | 0.91 | 0.23 |
| chess-jai | build-O2 | 8.61 | 11.95 | 8.54-19.79 | 697 | 3.14 | 8.45 | 0.21 |
| forbear | check | 0.27 | 0.28 | 0.27-0.38 | 143 | 0.26 | 0.00 | 0.00 |
| forbear | build-O0 | 0.88 | 0.79 | 0.77-0.91 | 192 | 0.30 | 0.12 | 0.20 |

Notes: "dirty" was uncommitted documentation only; the compiler was built from `cfd30421ef74`. Other builds shared the machine during this run, which is why some min-max ranges are wide (focus build-O2, chess-jai). Re-run on an idle machine before comparing small changes against this baseline.
