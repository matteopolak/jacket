# Language server latency and memory benchmark (2026-10-09)

Machine: Apple M5, 10 cores, 16.0 GiB, Darwin 27.0.  
jailsp: jailsp 0.6.0, checkout `e15e3678c981` (dirty), rustc 1.100.0-nightly (17fd5b8a3 2026-08-28).  
Sessions per workload: 3 (cold = first session, warm = median of the rest). A cell is a status when there is no number: `limit` (the server refused or died on a size limit), `timeout`, `memory` (RSS cap), `crash`, `error`, `none` (no diagnostics pushed), `unsupported`, `skipped`.

## Warm wall time, ms (median of the sessions after the first)

| workload | doc KiB | startup | first_diagnostics | hover_first | hover_warm | completion_member | references | document_symbols | semantic_tokens | edit_hover | typing_first | typing_median | completion_member_typed | total CPU s | peak RSS MiB |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| jailsp/focus/pull | 200 | 3.0 | 293 | 1.3 | 1.2 | 0.6 | 1.8 | 4.1 | 10 | 274 | 9.8 | 7.0 | 280 | 1.55 | 241 |
| jailsp/focus/push | 200 | 3.1 | 522 | 1.2 | 1.2 | 0.6 | 1.9 | 3.1 | 10 | 280 | 10 | 7.4 | 289 | 1.92 | 273 |
| jailsp/focus-main/pull | 200 | 3.1 | 298 | 1.4 | 1.3 | 0.7 | 2.0 | 3.3 | 10 | 282 | 11 | 7.9 | 288 | 1.59 | 241 |
| jailsp/focus-main/push | 200 | 3.3 | 525 | 1.4 | 1.3 | 0.7 | 2.1 | 3.7 | 11 | 283 | 12 | 8.2 | 291 | 1.94 | 245 |
| jailsp/jails/pull | 56 | 3.2 | 55 | 0.8 | 0.7 | 0.5 | 0.7 | 1.2 | 3.8 | 51 | 5.0 | 4.0 | 55 | 0.36 | 88 |
| jailsp/jails/push | 56 | 3.0 | 262 | 0.9 | 0.7 | 0.6 | 0.7 | 1.0 | 3.4 | 49 | 5.2 | 4.0 | 51 | 0.42 | 87 |
| jailsp/chess-jai/pull | 78 | 2.9 | 25 | 0.5 | 0.6 | 0.5 | 0.4 | 2.3 | 6.3 | 18 | 5.1 | 3.4 | 21 | 0.21 | 62 |
| jailsp/chess-jai/push | 78 | 3.2 | 241 | 0.5 | 0.6 | 0.5 | 0.5 | 2.5 | 6.3 | 18 | 4.9 | 3.4 | 21 | 0.26 | 82 |
| jailsp/gen-60k/pull | 163 | 3.1 | 265 | 1.3 | 1.1 | 0.5 | 3.8 | 4.6 | 15 | 223 | 12 | 12 | 225 | 1.27 | 209 |
| jailsp/gen-60k/push | 163 | 3.1 | 504 | 1.0 | 1.1 | 0.5 | 3.8 | 5.8 | 16 | 222 | 13 | 13 | 227 | 1.58 | 217 |
| jailsp/gen-240k/pull | 206 | 3.2 | 1706 | 2.9 | 2.4 | 0.8 | 13 | 5.5 | 20 | 1567 | 21 | 16 | 1555 | 8.19 | 635 |
| jailsp/gen-240k/push | 206 | 3.3 | 2046 | 2.5 | 2.4 | 0.7 | 12 | 5.7 | 20 | 1565 | 20 | 16 | 1553 | 10.10 | 646 |
| jailsp/large-25k/pull | 805 | 3.3 | 205 | 2.9 | 2.2 | 1.1 | 2.1 | 26 | 76 | 97 | 52 | 33 | 137 | 0.99 | 192 |
| jailsp/large-25k/push | 805 | 3.4 | 458 | 2.3 | 2.2 | 1.0 | 2.0 | 26 | 78 | 111 | 56 | 33 | 138 | 1.24 | 190 |
| jailsp/large-100k/pull | 3295 | 4.5 | 1109 | 15 | 9.5 | 4.2 | 9.3 | 194 | 437 | 497 | 272 | 246 | 265 | 3.94 | 623 |
| jailsp/large-100k/push | 3295 | 3.9 | 1102 | 9.9 | 8.9 | 3.9 | 8.4 | 188 | 360 | 400 | 239 | 228 | 234 | 3.79 | 617 |

## Cold wall time, ms (first session)

| workload | doc KiB | startup | first_diagnostics | hover_first | hover_warm | completion_member | references | document_symbols | semantic_tokens | edit_hover | typing_first | typing_median | completion_member_typed |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| jailsp/focus/pull | 200 | 2.7 | 297 | 1.3 | 1.1 | 0.6 | 2.1 | 3.2 | 10 | 280 | 10 | 7.0 | 282 |
| jailsp/focus/push | 200 | 3.0 | 521 | 1.3 | 1.2 | 0.6 | 1.9 | 3.1 | 10 | 280 | 9.6 | 6.9 | 281 |
| jailsp/focus-main/pull | 200 | 2.9 | 306 | 1.9 | 1.3 | 0.9 | 1.9 | 3.5 | 11 | 282 | 12 | 8.5 | 288 |
| jailsp/focus-main/push | 200 | 2.8 | 519 | 1.6 | 1.2 | 0.7 | 1.8 | 3.4 | 10 | 287 | 12 | 8.2 | 287 |
| jailsp/jails/pull | 56 | 3.0 | 59 | 0.8 | 0.7 | 0.4 | 0.7 | 1.1 | 3.4 | 49 | 5.6 | 3.9 | 54 |
| jailsp/jails/push | 56 | 3.1 | 259 | 0.8 | 0.7 | 0.4 | 0.6 | 1.1 | 3.3 | 48 | 5.0 | 3.7 | 54 |
| jailsp/chess-jai/pull | 78 | 3.4 | 26 | 0.6 | 0.6 | 0.5 | 0.5 | 2.3 | 6.6 | 18 | 5.0 | 3.3 | 21 |
| jailsp/chess-jai/push | 78 | 2.9 | 242 | 0.5 | 0.6 | 0.4 | 0.4 | 2.4 | 5.9 | 17 | 4.4 | 3.3 | 20 |
| jailsp/gen-60k/pull | 163 | 3.2 | 268 | 1.3 | 1.1 | 0.5 | 3.9 | 4.9 | 17 | 229 | 13 | 13 | 224 |
| jailsp/gen-60k/push | 163 | 3.1 | 504 | 1.4 | 1.1 | 0.5 | 3.6 | 4.6 | 17 | 224 | 12 | 12 | 228 |
| jailsp/gen-240k/pull | 206 | 4.0 | 1800 | 2.7 | 2.5 | 0.8 | 13 | 5.1 | 21 | 1628 | 22 | 17 | 1567 |
| jailsp/gen-240k/push | 206 | 3.1 | 2028 | 2.5 | 2.5 | 0.8 | 12 | 4.6 | 19 | 1568 | 21 | 16 | 1553 |
| jailsp/large-25k/pull | 805 | 3.1 | 200 | 3.0 | 2.2 | 1.1 | 2.1 | 26 | 78 | 97 | 53 | 32 | 132 |
| jailsp/large-25k/push | 805 | 3.4 | 428 | 2.5 | 2.2 | 0.9 | 1.9 | 26 | 76 | 101 | 53 | 33 | 137 |
| jailsp/large-100k/pull | 3295 | 3.6 | 881 | 11 | 8.2 | 3.8 | 7.4 | 174 | 365 | 385 | 242 | 233 | 266 |
| jailsp/large-100k/push | 3295 | 4.4 | 1307 | 12 | 11 | 3.7 | 8.4 | 208 | 440 | 489 | 282 | 230 | 231 |

## Warm CPU time, ms (user + system of the server during the step)

| workload | doc KiB | startup | first_diagnostics | hover_first | hover_warm | completion_member | references | document_symbols | semantic_tokens | edit_hover | typing_first | typing_median | completion_member_typed |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| jailsp/focus/pull | 200 | 1.9 | 290 | 1.0 | 5.0 | 0.0 | 1.5 | 1.0 | 5.0 | 823 | 118 | - | 278 |
| jailsp/focus/push | 200 | 2.1 | 314 | 1.0 | 5.0 | 0.0 | 2.0 | 1.0 | 5.5 | 840 | 118 | - | 284 |
| jailsp/focus-main/pull | 200 | 2.1 | 296 | 1.0 | 5.5 | 0.0 | 2.0 | 1.0 | 5.5 | 842 | 122 | - | 284 |
| jailsp/focus-main/push | 200 | 2.3 | 316 | 1.0 | 7.0 | 0.0 | 2.0 | 1.0 | 6.0 | 848 | 126 | - | 288 |
| jailsp/jails/pull | 56 | 2.1 | 55 | 0.0 | 3.5 | 0.0 | 0.0 | 0.0 | 2.0 | 152 | 86 | - | 54 |
| jailsp/jails/push | 56 | 1.9 | 58 | 0.0 | 3.5 | 0.0 | 0.0 | 0.0 | 2.0 | 146 | 88 | - | 50 |
| jailsp/chess-jai/pull | 78 | 1.9 | 24 | 1.0 | 3.0 | 0.0 | 0.0 | 1.0 | 3.0 | 52 | 94 | - | 20 |
| jailsp/chess-jai/push | 78 | 2.1 | 40 | 1.0 | 3.0 | 0.0 | 0.0 | 0.0 | 3.0 | 52 | 93 | - | 20 |
| jailsp/gen-60k/pull | 163 | 2.0 | 262 | 1.0 | 4.0 | 0.0 | 3.0 | 1.0 | 8.0 | 654 | 94 | - | 222 |
| jailsp/gen-60k/push | 163 | 1.9 | 294 | 1.0 | 4.0 | 0.0 | 3.5 | 1.0 | 8.5 | 656 | 92 | - | 222 |
| jailsp/gen-240k/pull | 206 | 1.9 | 1694 | 3.0 | 11 | 0.0 | 12 | 1.0 | 10 | 4667 | 195 | - | 1546 |
| jailsp/gen-240k/push | 206 | 1.9 | 1829 | 3.0 | 11 | 0.0 | 12 | 1.0 | 10 | 4694 | 194 | - | 1544 |
| jailsp/large-25k/pull | 805 | 1.9 | 200 | 2.0 | 10 | 1.0 | 2.0 | 5.0 | 38 | 290 | 273 | - | 135 |
| jailsp/large-25k/push | 805 | 1.9 | 220 | 2.0 | 11 | 1.0 | 2.0 | 5.5 | 40 | 329 | 281 | - | 135 |
| jailsp/large-100k/pull | 3295 | 2.6 | 1066 | 12 | 48 | 4.0 | 9.5 | 29 | 226 | 1448 | 672 | - | 260 |
| jailsp/large-100k/push | 3295 | 1.9 | 861 | 10 | 43 | 3.5 | 8.0 | 26 | 188 | 1182 | 600 | - | 230 |

## Warm resident set after the step, MiB

| workload | doc KiB | startup | first_diagnostics | hover_first | hover_warm | completion_member | references | document_symbols | semantic_tokens | edit_hover | typing_first | typing_median | completion_member_typed |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| jailsp/focus/pull | 200 | 3 | 205 | 205 | 205 | 205 | 205 | 205 | 205 | 206 | 237 | - | 238 |
| jailsp/focus/push | 200 | 3 | 235 | 235 | 235 | 235 | 235 | 235 | 235 | 236 | 266 | - | 269 |
| jailsp/focus-main/pull | 200 | 3 | 205 | 205 | 205 | 205 | 205 | 206 | 206 | 206 | 237 | - | 238 |
| jailsp/focus-main/push | 200 | 3 | 208 | 208 | 208 | 208 | 208 | 208 | 208 | 209 | 238 | - | 240 |
| jailsp/jails/pull | 56 | 3 | 56 | 56 | 56 | 56 | 56 | 56 | 56 | 57 | 86 | - | 87 |
| jailsp/jails/push | 56 | 3 | 56 | 56 | 56 | 56 | 56 | 56 | 56 | 57 | 86 | - | 87 |
| jailsp/chess-jai/pull | 78 | 3 | 30 | 30 | 31 | 31 | 31 | 31 | 31 | 31 | 60 | - | 61 |
| jailsp/chess-jai/push | 78 | 3 | 50 | 50 | 50 | 51 | 51 | 51 | 51 | 51 | 80 | - | 81 |
| jailsp/gen-60k/pull | 163 | 3 | 175 | 175 | 175 | 175 | 175 | 176 | 176 | 177 | 204 | - | 204 |
| jailsp/gen-60k/push | 163 | 3 | 175 | 175 | 175 | 175 | 175 | 175 | 176 | 177 | 204 | - | 204 |
| jailsp/gen-240k/pull | 206 | 3 | 584 | 584 | 584 | 584 | 584 | 584 | 585 | 588 | 616 | - | 618 |
| jailsp/gen-240k/push | 206 | 3 | 584 | 584 | 584 | 584 | 584 | 584 | 584 | 588 | 616 | - | 618 |
| jailsp/large-25k/pull | 805 | 3 | 140 | 140 | 140 | 140 | 140 | 142 | 142 | 143 | 172 | - | 168 |
| jailsp/large-25k/push | 805 | 3 | 138 | 138 | 138 | 139 | 139 | 140 | 141 | 141 | 170 | - | 167 |
| jailsp/large-100k/pull | 3295 | 3 | 483 | 483 | 483 | 483 | 483 | 490 | 494 | 494 | 535 | - | 537 |
| jailsp/large-100k/push | 3295 | 3 | 477 | 477 | 477 | 477 | 477 | 484 | 490 | 489 | 531 | - | 533 |

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
