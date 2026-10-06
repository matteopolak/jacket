# jaic compile-time benchmark (2026-10-06)

Machine: Apple M5, 10 cores, 16.0 GiB, Darwin 27.0, LLVM 23.1.2.  
jaic: `f6e3231ce802`, rustc 1.100.0-nightly (17fd5b8a3 2026-08-28). Runs per workload: 4 (cold = first run, warm = median of the rest).

| workload | mode | cold s | warm s | min-max s | peak RSS MiB | front end s | codegen s | link s |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| focus | check | 1.75 | 1.69 | 1.59-1.81 | 705 | 1.67 | 0.00 | 0.00 |
| focus | build-O0 | 2.38 | 2.26 | 2.25-2.48 | 1138 | 1.55 | 0.52 | 0.10 |
| focus | build-O2 | 13.17 | 15.11 | 11.86-16.02 | 1278 | 6.77 | 8.13 | 0.11 |
| jails | check | 0.21 | 0.22 | 0.20-0.22 | 130 | 0.20 | 0.00 | 0.00 |
| jails | build-O0 | 0.36 | 0.43 | 0.36-0.45 | 236 | 0.20 | 0.12 | 0.04 |
| jails | build-O2 | 2.61 | 2.19 | 1.99-2.61 | 368 | 0.20 | 1.90 | 0.04 |
| sgpu-examples | check | 0.52 | 0.49 | 0.47-0.52 | 204 | 0.47 | 0.00 | 0.00 |
| jaison-tests | check | 0.04 | 0.04 | 0.04-0.05 | 47 | 0.02 | 0.00 | 0.00 |
| jaison-tests | build-O0 | 0.16 | 0.17 | 0.16-0.17 | 99 | 0.02 | 0.06 | 0.03 |
| jaison-tests | build-O2 | 0.83 | 0.81 | 0.77-0.83 | 164 | 0.03 | 0.69 | 0.03 |
| open-jai-getrect | check | 0.08 | 0.08 | 0.07-0.11 | 82 | 0.05 | 0.00 | 0.00 |
| open-jai-getrect | build-O0 | 0.35 | 0.29 | 0.28-0.36 | 154 | 0.06 | 0.06 | 0.11 |
| open-jai-getrect | build-O2 | 1.08 | 0.94 | 0.92-1.17 | 243 | 0.06 | 0.73 | 0.10 |
| chess-jai | check | 0.66 | 0.66 | 0.61-0.67 | 197 | 0.63 | 0.00 | 0.00 |
| chess-jai | build-O0 | 1.91 | 2.03 | 1.91-2.14 | 764 | 0.97 | 0.75 | 0.20 |
| chess-jai | build-O2 | 4.71 | 4.90 | 4.61-5.15 | 836 | 0.60 | 4.10 | 0.13 |
| forbear | check | 0.15 | 0.14 | 0.14-0.15 | 126 | 0.12 | 0.00 | 0.00 |
| forbear | build-O0 | 0.35 | 0.33 | 0.33-0.35 | 177 | 0.12 | 0.05 | 0.09 |

Notes: the first baseline on LLVM 23 (Homebrew 23.1.2, linked dynamically). For the move from LLVM 22, `main` at `7039ad3a` built against Homebrew's LLVM 22.1.8 ran alternately with this build, two rounds of 4 runs each. Pooled warm medians (codegen is the median of the two rounds):

| workload/mode | LLVM 22 warm s | LLVM 23 warm s | change | 22 codegen s | 23 codegen s | 22 RSS MiB | 23 RSS MiB |
|---|---:|---:|---:|---:|---:|---:|---:|
| focus/check | 1.98 | 1.64 | -16.8% | 0.00 | 0.00 | 705 | 705 |
| focus/build-O0 | 2.77 | 2.46 | -11.1% | 0.77 | 0.52 | 1144 | 1138 |
| focus/build-O2 | 16.99 | 13.59 | -20.0% | 10.02 | 7.37 | 1220 | 1278 |
| jails/check | 0.20 | 0.20 | -1.8% | 0.00 | 0.00 | 130 | 130 |
| jails/build-O0 | 0.41 | 0.36 | -13.1% | 0.13 | 0.10 | 231 | 236 |
| jails/build-O2 | 3.17 | 1.85 | -41.7% | 2.87 | 1.68 | 346 | 368 |
| sgpu-examples/check | 0.47 | 0.46 | -1.5% | 0.00 | 0.00 | 209 | 205 |
| jaison-tests/check | 0.04 | 0.04 | -7.0% | 0.00 | 0.00 | 46 | 47 |
| jaison-tests/build-O0 | 0.18 | 0.15 | -17.1% | 0.07 | 0.05 | 97 | 100 |
| jaison-tests/build-O2 | 1.00 | 0.70 | -30.3% | 0.89 | 0.61 | 148 | 165 |
| open-jai-getrect/check | 0.06 | 0.07 | +7.9% | 0.00 | 0.00 | 80 | 82 |
| open-jai-getrect/build-O0 | 0.30 | 0.28 | -8.1% | 0.11 | 0.06 | 150 | 154 |
| open-jai-getrect/build-O2 | 1.25 | 0.91 | -27.0% | 1.13 | 0.70 | 228 | 243 |
| chess-jai/check | 0.57 | 0.60 | +6.8% | 0.00 | 0.00 | 194 | 197 |
| chess-jai/build-O0 | 1.73 | 1.80 | +4.0% | 0.79 | 0.67 | 763 | 767 |
| chess-jai/build-O2 | 7.09 | 4.83 | -31.9% | 6.16 | 4.01 | 824 | 836 |
| forbear/check | 0.15 | 0.17 | +15.5% | 0.00 | 0.00 | 125 | 126 |
| forbear/build-O0 | 0.41 | 0.37 | -8.5% | 0.09 | 0.06 | 173 | 177 |

`-O2` codegen got 26-41% faster on every project and `-O0` codegen 15-45%; `-O2` peak RSS grew 1-12%. `check` rows do not touch LLVM, so their spread (up to ±17%) is the noise of a machine shared with other builds. The two LLVMs are separate Homebrew builds, so part of the difference may come from how each was compiled rather than from LLVM's code.
