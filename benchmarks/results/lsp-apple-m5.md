# Language server latency and memory benchmark (2026-10-09)

Machine: Apple M5, 10 cores, 16.0 GiB, Darwin 27.0.  
jailsp: jailsp 0.6.0, checkout `e3f2bc3b32ef` (dirty), rustc 1.100.0-nightly (17fd5b8a3 2026-08-28).  
Sessions per workload: 3 (cold = first session, warm = median of the rest). A cell is a status when there is no number: `limit` (the server refused or died on a size limit), `timeout`, `memory` (RSS cap), `crash`, `error`, `none` (no diagnostics pushed), `unsupported`, `skipped`.

## Warm wall time, ms (median of the sessions after the first)

| workload | doc KiB | startup | first_diagnostics | hover_first | hover_warm | completion_member | references | document_symbols | semantic_tokens | edit_hover | typing_first | typing_median | completion_member_typed | total CPU s | peak RSS MiB |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| jailsp/focus/pull | 200 | 4.9 | 537 | 2.1 | 2.2 | 1.1 | 3.8 | 7.5 | 17 | 584 | 20 | 14 | 653 | 2.47 | 243 |
| jailsp/focus/push | 200 | 4.6 | 686 | 2.0 | 2.1 | 1.1 | 3.8 | 6.2 | 15 | 1121 | 1074 | 746 | 2148 | 5.76 | 274 |
| jailsp/focus-main/pull | 200 | 4.6 | 515 | 2.4 | 2.3 | 1.1 | 3.2 | 6.4 | 18 | 614 | 17 | 14 | 612 | 2.49 | 243 |
| jailsp/focus-main/push | 200 | 4.8 | 1482 | 2.4 | 2.4 | 1.1 | 3.8 | 6.0 | 20 | 834 | 886 | 114 | 1677 | 4.92 | 268 |
| jailsp/jails/pull | 56 | 5.0 | 83 | 1.2 | 1.4 | 0.7 | 1.4 | 2.2 | 5.6 | 88 | 9.6 | 6.8 | 128 | 0.54 | 88 |
| jailsp/jails/push | 56 | 5.2 | 163 | 1.4 | 1.4 | 0.9 | 1.3 | 2.2 | 5.2 | 137 | 206 | 97 | 374 | 1.22 | 120 |
| jailsp/chess-jai/pull | 78 | 4.2 | 34 | 0.8 | 1.2 | 1.0 | 1.0 | 4.8 | 11 | 28 | 7.5 | 6.3 | 43 | 0.31 | 63 |
| jailsp/chess-jai/push | 78 | 4.2 | 89 | 1.0 | 1.2 | 0.9 | 1.0 | 4.7 | 12 | 66 | 108 | 74 | 186 | 0.97 | 85 |
| jailsp/gen-60k/pull | 163 | 3.6 | 365 | 2.0 | 1.4 | 0.6 | 5.9 | 7.8 | 22 | 295 | 13 | 13 | 279 | 1.65 | 210 |
| jailsp/gen-60k/push | 163 | 3.6 | 638 | 1.3 | 1.1 | 0.5 | 4.1 | 5.5 | 17 | 332 | 424 | 163 | 933 | 3.54 | 261 |
| jailsp/gen-240k/pull | 206 | 3.3 | 1888 | 2.8 | 2.5 | 0.8 | 13 | 6.1 | 22 | 1744 | 22 | 17 | 1726 | 9.02 | 638 |
| jailsp/gen-240k/push | 206 | 3.3 | 3712 | 2.4 | 2.4 | 0.7 | 13 | 5.1 | 21 | 1852 | 2576 | 763 | 4229 | 23.08 | 948 |
| jailsp/large-25k/pull | 805 | 3.3 | 206 | 3.2 | 2.4 | 1.0 | 2.2 | 25 | 78 | 98 | 50 | 32 | 138 | 0.98 | 200 |
| jailsp/large-25k/push | 805 | 3.7 | 201 | 2.4 | 2.3 | 1.1 | 2.1 | 26 | 78 | 205 | 227 | 32 | 474 | 1.96 | 192 |
| jailsp/large-100k/pull | 3295 | 3.7 | 846 | 11 | 8.4 | 3.8 | 7.9 | 155 | 351 | 379 | 216 | 194 | 213 | 3.08 | 649 |
| jailsp/large-100k/push | 3295 | 3.6 | 818 | 8.6 | 8.3 | 3.8 | 7.9 | 175 | 342 | 847 | 945 | 194 | 949 | 5.88 | 626 |

## Cold wall time, ms (first session)

| workload | doc KiB | startup | first_diagnostics | hover_first | hover_warm | completion_member | references | document_symbols | semantic_tokens | edit_hover | typing_first | typing_median | completion_member_typed |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| jailsp/focus/pull | 200 | 4.2 | 841 | 10 | 2.0 | 1.1 | 3.8 | 6.3 | 20 | 913 | 33 | 31 | 985 |
| jailsp/focus/push | 200 | 5.6 | 643 | 2.4 | 2.1 | 1.4 | 3.7 | 6.3 | 15 | 659 | 769 | 702 | 2359 |
| jailsp/focus-main/pull | 200 | 5.2 | 571 | 2.4 | 2.4 | 1.1 | 3.8 | 7.0 | 16 | 582 | 20 | 18 | 679 |
| jailsp/focus-main/push | 200 | 4.2 | 1009 | 2.6 | 2.6 | 1.4 | 5.6 | 6.6 | 22 | 483 | 761 | 118 | 1828 |
| jailsp/jails/pull | 56 | 4.8 | 79 | 1.4 | 1.4 | 0.9 | 0.8 | 2.1 | 6.2 | 71 | 9.1 | 7.0 | 125 |
| jailsp/jails/push | 56 | 5.0 | 158 | 1.3 | 1.4 | 0.8 | 1.3 | 2.1 | 6.2 | 105 | 184 | 95 | 239 |
| jailsp/chess-jai/pull | 78 | 4.3 | 42 | 0.8 | 1.2 | 0.9 | 0.9 | 4.9 | 12 | 27 | 8.7 | 6.5 | 41 |
| jailsp/chess-jai/push | 78 | 5.5 | 85 | 0.9 | 1.3 | 0.9 | 1.0 | 4.6 | 9.9 | 63 | 131 | 82 | 201 |
| jailsp/gen-60k/pull | 163 | 22 | 476 | 1.6 | 1.9 | 1.9 | 6.6 | 8.8 | 22 | 303 | 13 | 13 | 309 |
| jailsp/gen-60k/push | 163 | 3.7 | 705 | 1.1 | 1.1 | 0.5 | 5.9 | 6.0 | 20 | 379 | 458 | 156 | 932 |
| jailsp/gen-240k/pull | 206 | 3.8 | 1894 | 2.9 | 2.5 | 0.7 | 14 | 5.6 | 23 | 1758 | 23 | 17 | 1757 |
| jailsp/gen-240k/push | 206 | 3.5 | 3906 | 2.5 | 2.7 | 0.8 | 13 | 5.4 | 24 | 1976 | 2759 | 819 | 4554 |
| jailsp/large-25k/pull | 805 | 3.8 | 211 | 3.2 | 2.3 | 1.0 | 2.0 | 24 | 77 | 99 | 51 | 32 | 136 |
| jailsp/large-25k/push | 805 | 3.5 | 203 | 2.4 | 2.3 | 1.1 | 1.9 | 29 | 77 | 201 | 221 | 32 | 486 |
| jailsp/large-100k/pull | 3295 | 4.2 | 848 | 11 | 8.3 | 3.8 | 7.7 | 161 | 346 | 370 | 218 | 193 | 211 |
| jailsp/large-100k/push | 3295 | 3.7 | 840 | 8.3 | 8.5 | 3.5 | 7.9 | 166 | 343 | 856 | 945 | 197 | 951 |

## Warm CPU time, ms (user + system of the server during the step)

| workload | doc KiB | startup | first_diagnostics | hover_first | hover_warm | completion_member | references | document_symbols | semantic_tokens | edit_hover | typing_first | typing_median | completion_member_typed |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| jailsp/focus/pull | 200 | 3.1 | 454 | 2.0 | 10 | 1.0 | 3.0 | 1.0 | 8.5 | 1321 | 179 | - | 447 |
| jailsp/focus/push | 200 | 3.0 | 562 | 1.5 | 10 | 1.0 | 3.0 | 1.0 | 8.5 | 1495 | 2187 | - | 964 |
| jailsp/focus-main/pull | 200 | 3.1 | 442 | 3.0 | 11 | 1.0 | 2.5 | 1.0 | 9.5 | 1340 | 186 | - | 450 |
| jailsp/focus-main/push | 200 | 3.1 | 948 | 3.0 | 11 | 1.0 | 3.0 | 1.0 | 10 | 1472 | 965 | - | 996 |
| jailsp/jails/pull | 56 | 3.1 | 82 | 1.0 | 6.0 | 0.0 | 1.0 | 0.0 | 3.0 | 224 | 130 | - | 75 |
| jailsp/jails/push | 56 | 3.3 | 155 | 2.0 | 6.0 | 0.0 | 1.0 | 0.0 | 2.0 | 282 | 434 | - | 224 |
| jailsp/chess-jai/pull | 78 | 2.9 | 35 | 2.0 | 5.5 | 0.0 | 0.0 | 1.0 | 5.0 | 78 | 142 | - | 30 |
| jailsp/chess-jai/push | 78 | 2.9 | 88 | 2.0 | 5.0 | 0.0 | 0.5 | 1.0 | 6.0 | 183 | 484 | - | 115 |
| jailsp/gen-60k/pull | 163 | 2.4 | 356 | 1.5 | 7.0 | 0.0 | 5.0 | 1.5 | 11 | 854 | 112 | - | 272 |
| jailsp/gen-60k/push | 163 | 2.4 | 624 | 1.0 | 5.5 | 0.0 | 4.0 | 1.0 | 9.0 | 974 | 666 | - | 914 |
| jailsp/gen-240k/pull | 206 | 1.9 | 1858 | 3.0 | 13 | 0.0 | 14 | 1.0 | 11 | 5153 | 210 | - | 1702 |
| jailsp/gen-240k/push | 206 | 1.9 | 3667 | 3.0 | 12 | 0.0 | 13 | 1.0 | 10 | 5468 | 7873 | - | 4184 |
| jailsp/large-25k/pull | 805 | 1.8 | 200 | 2.0 | 12 | 1.0 | 2.0 | 6.0 | 39 | 290 | 268 | - | 134 |
| jailsp/large-25k/push | 805 | 2.1 | 194 | 2.0 | 11 | 1.0 | 2.0 | 5.5 | 40 | 596 | 442 | - | 466 |
| jailsp/large-100k/pull | 3295 | 2.1 | 819 | 8.5 | 41 | 3.0 | 8.0 | 24 | 180 | 1138 | 534 | - | 210 |
| jailsp/large-100k/push | 3295 | 1.9 | 784 | 8.0 | 40 | 3.5 | 8.0 | 24 | 178 | 2455 | 1252 | - | 935 |

## Warm resident set after the step, MiB

| workload | doc KiB | startup | first_diagnostics | hover_first | hover_warm | completion_member | references | document_symbols | semantic_tokens | edit_hover | typing_first | typing_median | completion_member_typed |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| jailsp/focus/pull | 200 | 3 | 205 | 205 | 205 | 205 | 205 | 206 | 206 | 208 | 237 | - | 239 |
| jailsp/focus/push | 200 | 3 | 235 | 235 | 235 | 235 | 235 | 235 | 235 | 238 | 269 | - | 270 |
| jailsp/focus-main/pull | 200 | 3 | 205 | 205 | 205 | 205 | 205 | 206 | 206 | 208 | 238 | - | 239 |
| jailsp/focus-main/push | 200 | 3 | 209 | 209 | 209 | 209 | 209 | 209 | 209 | 211 | 263 | - | 265 |
| jailsp/jails/pull | 56 | 3 | 56 | 56 | 56 | 56 | 56 | 56 | 56 | 58 | 86 | - | 87 |
| jailsp/jails/push | 56 | 3 | 57 | 57 | 57 | 57 | 57 | 57 | 57 | 58 | 118 | - | 118 |
| jailsp/chess-jai/pull | 78 | 3 | 30 | 30 | 31 | 31 | 31 | 31 | 31 | 32 | 60 | - | 61 |
| jailsp/chess-jai/push | 78 | 3 | 51 | 51 | 51 | 51 | 51 | 51 | 51 | 52 | 83 | - | 84 |
| jailsp/gen-60k/pull | 163 | 3 | 177 | 177 | 177 | 177 | 177 | 178 | 178 | 180 | 206 | - | 207 |
| jailsp/gen-60k/push | 163 | 3 | 176 | 176 | 176 | 176 | 176 | 176 | 176 | 177 | 251 | - | 245 |
| jailsp/gen-240k/pull | 206 | 3 | 586 | 586 | 586 | 586 | 586 | 587 | 587 | 590 | 619 | - | 621 |
| jailsp/gen-240k/push | 206 | 3 | 587 | 587 | 587 | 587 | 587 | 587 | 587 | 589 | 927 | - | 847 |
| jailsp/large-25k/pull | 805 | 3 | 148 | 148 | 148 | 148 | 148 | 150 | 152 | 152 | 180 | - | 177 |
| jailsp/large-25k/push | 805 | 3 | 139 | 139 | 139 | 139 | 139 | 141 | 141 | 142 | 171 | - | 170 |
| jailsp/large-100k/pull | 3295 | 3 | 513 | 513 | 513 | 513 | 513 | 521 | 523 | 522 | 564 | - | 566 |
| jailsp/large-100k/push | 3295 | 3 | 477 | 478 | 478 | 478 | 478 | 484 | 489 | 490 | 533 | - | 541 |

Metrics:

- `startup`: launch to initialize response
- `first_diagnostics`: didOpen to first diagnostics
- `hover_first`: first hover
- `hover_warm`: hover again, no edit (median of 5)
- `completion_member`: completion after `.` (existing text)
- `references`: references of the procedure
- `document_symbols`: document symbols
- `semantic_tokens`: semantic tokens (full)
- `edit_hover`: edit, then hover (median of 3)
- `typing_first`: typing: first character completion
- `typing_median`: typing: median completion
- `completion_member_typed`: edit typing `name.`, then completion

What the first diagnostics said (diagnostics by code; `jai-limit` = the syntax layer gave up on the document, `jai-check` = type-checker errors, so later facts may be incomplete; hover null = no fact):

- `jailsp/focus/pull`: 65 diagnostics {'jai-check': 64, 'unused_variable': 1}, hover answered
- `jailsp/focus/push`: 65 diagnostics {'jai-check': 64, 'unused_variable': 1}, hover answered
- `jailsp/focus-main/pull`: 65 diagnostics {'jai-check': 64, 'unused_variable': 1}, hover answered
- `jailsp/focus-main/push`: 65 diagnostics {'jai-check': 64, 'unused_variable': 1}, hover answered
- `jailsp/jails/pull`: 3 diagnostics {'unused_variable': 1, 'unused_parameter': 1, 'defer_in_loop': 1}, hover answered
- `jailsp/jails/push`: 3 diagnostics {'unused_variable': 1, 'unused_parameter': 1, 'defer_in_loop': 1}, hover answered
- `jailsp/chess-jai/pull`: 6 diagnostics {'jai-check': 2, 'unused_variable': 3, 'bitwise_precedence': 1}, hover answered after 1 other position(s)
- `jailsp/chess-jai/push`: 6 diagnostics {'jai-check': 2, 'unused_variable': 3, 'bitwise_precedence': 1}, hover answered after 1 other position(s)
- `jailsp/gen-60k/pull`: 585 diagnostics {'unused_import': 1, 'unused_variable': 158, 'unused_parameter': 118, 'identical_operands': 192, 'manual_assign_op': 12, 'self_assignment': 18, 'redundant_cast': 77, 'bitwise_precedence': 8, 'erasing_op': 1}, hover answered
- `jailsp/gen-60k/push`: 585 diagnostics {'unused_import': 1, 'unused_variable': 158, 'unused_parameter': 118, 'identical_operands': 192, 'manual_assign_op': 12, 'self_assignment': 18, 'redundant_cast': 77, 'bitwise_precedence': 8, 'erasing_op': 1}, hover answered
- `jailsp/gen-240k/pull`: 691 diagnostics {'unused_import': 1, 'identical_operands': 247, 'self_assignment': 11, 'manual_assign_op': 18, 'unused_parameter': 135, 'unused_variable': 198, 'bitwise_precedence': 14, 'redundant_cast': 62, 'identity_op': 5}, hover answered
- `jailsp/gen-240k/push`: 691 diagnostics {'unused_import': 1, 'identical_operands': 247, 'self_assignment': 11, 'manual_assign_op': 18, 'unused_parameter': 135, 'unused_variable': 198, 'bitwise_precedence': 14, 'redundant_cast': 62, 'identity_op': 5}, hover answered
- `jailsp/large-25k/pull`: 2678 diagnostics {'unused_parameter': 550, 'self_assignment': 66, 'identical_operands': 952, 'unused_variable': 717, 'bitwise_precedence': 35, 'manual_assign_op': 60, 'redundant_cast': 286, 'identity_op': 6, 'erasing_op': 5, 'almost_swapped': 1}, hover answered
- `jailsp/large-25k/push`: 2678 diagnostics {'unused_parameter': 550, 'self_assignment': 66, 'identical_operands': 952, 'unused_variable': 717, 'bitwise_precedence': 35, 'manual_assign_op': 60, 'redundant_cast': 286, 'identity_op': 6, 'erasing_op': 5, 'almost_swapped': 1}, hover answered
- `jailsp/large-100k/pull`: 10706 diagnostics {'unused_parameter': 2410, 'self_assignment': 267, 'identical_operands': 3730, 'unused_variable': 2917, 'bitwise_precedence': 160, 'manual_assign_op': 282, 'redundant_cast': 870, 'identity_op': 41, 'erasing_op': 26, 'almost_swapped': 3}, hover answered
- `jailsp/large-100k/push`: 10706 diagnostics {'unused_parameter': 2410, 'self_assignment': 267, 'identical_operands': 3730, 'unused_variable': 2917, 'bitwise_precedence': 160, 'manual_assign_op': 282, 'redundant_cast': 870, 'identity_op': 41, 'erasing_op': 26, 'almost_swapped': 3}, hover answered
