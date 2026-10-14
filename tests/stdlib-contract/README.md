# Standard-library public-contract fixtures

These original, self-checking cases exercise caller-visible behavior documented in `docs/specs/stdlib`, checked through compatibility-spec review and independent contract review. They are intended to run against a locally source-built compiler and an explicitly chosen standard-library root. They do not treat passing these fixtures as evidence that any compiler implementation is complete.

## Run

```sh
python3 tests/stdlib-contract/run.py \
  --jaic /absolute/path/to/repository/target/debug/jaic \
  --stdlib-root /absolute/path/to/stdlib
```

The compiler must reside under this repository's `target/` directory. Each manifest case has a timeout. Use `--mode interp`, `--mode native`, or `--mode check` to override the positive fixture mode globally. Native mode compiles into a temporary directory and executes only the resulting local build. This runner must not be pointed at a supplied or otherwise untrusted compiler binary.

## Coverage

The current fixtures include: Basic dynamic-array behavior; String byte/token operations and mutation on writable storage; overlapping Memory copies; PCG state, seeded sequence and bounds; Base64 table mappings and partial decoding; CRC-64/WE; Bit_Array fields, padding, logical iteration and selected indexes; Hash_Table public shape and duplicate keys; Reflection type names and boolean-prefix parsing; Sort callback forms; Math and Float16 edge cases; Unicode incomplete-tail replacement/cursor behavior and immutable tables; Adpcm constants, saturation and clamping; Pool defaults, alignment, accounting and reset; file offset reads and paths; Process output/status; command-line argument handling; Bucket_Array locators and pointer iteration; allocator overwrite behavior; Thread synchronization; Socket address formatting; Compiler options; Code_Visit traversal; Program_Print escaping; and Jai_Lexer token/EOF handling.

Cases name public module imports and avoid private layouts and implementation helpers. Platform-specific operations are conditional or explicitly reported unavailable; pure contract checks use fixed inputs and expected outputs. The manifest is the index of runnable cases; add one focused source file and manifest row for each new contract edge.

This is a substantial acceptance set, not exhaustive coverage of every declaration in every module. The fixtures do not yet cover all Math/geometry operations, file watcher and shared-memory APIs, process/thread/socket error variants, all allocator families, every compiler API/message shape, or every portable collection and codec edge. Those remain follow-up coverage areas.

## Known differences from legacy tests

These fixtures and the listed legacy migrations encode the documented public contract, including behavior established by compatibility-specification review. Legacy expectations are updated only where they contradicted that contract; the public corpus and upstream examples remain untouched.

- `hash-digest-vectors.jai` and `allocation-memory.jai` now use the documented CRC-64/WE contract for `crc64`; they no longer expect CRC-64/ECMA or call the nonpublic `crc64_we` helper. The standard check value for `"123456789"` is `0x62ec59e3f1a4f00a`.
- `base64-codec.jai` and `base64-round-trips.jai` now accept incomplete tails, stop at the first `=`, ignore line feed, and reject other whitespace/invalid bytes with partial output. Their strict-padding and broad-whitespace expectations were replaced with those documented cases.
- `reflection-values-and-types.jai` checks that the boolean parser accepts the `true` prefix of `truest`, consuming four bytes; it is case-sensitive. The old token-boundary expectation was corrected.
- `Bit_Array.set_all_bits` sets the unused high bits in its final storage word; logical iteration still stops at `count`.
- `unicode-iteration.jai` checks that `Unicode.utf8_next_character` reports `SOURCE_EXHAUSTED`, returns U+FFFD, and consumes an incomplete trailing sequence.
- `random-pcg-sequences.jai` checks the documented pre-advance XSH-RR sequence after seed 42 (`0x75830bbd` first), replacing an expected value from a different sequence.
- `socket-loopback.jai` checks that `Socket.to_string(sockaddr_in6)` emits eight uncompressed hexadecimal groups followed by `:` and the port, with no brackets, in pure formatting and loopback cases.
- `basic-context-identity.jai` uses the language-level `#Context` type. `basic-protocol-aliases.jai` checks protocol structures through `Preload`; neither assumes extra `Basic.*` aliases.
- `rules-compiler-records.jai` still checks import-before-file events and the presence of file messages, but no longer assumes the `Basic` module consists of multiple source files.

The suite does not settle underspecified behavior such as allocator-specific failure diagnostics, malformed strict Unicode variants beyond the documented cursor case, or invalid Hash_Table generic parameters.

Compiler worker additions cover real-prelude atomic scalars/procedure identity, caller-owned context cleanup and AST flags, evaluated generic default reflection, and measured performance messages with nested records. These original fixtures run in both interpreter/native modes and pass repository jaifmt/jailint.
