# Utf8

Strict UTF-8 decoding, encoding and validation that never allocate. The modules that read or write UTF-8 (`Json`, `Regex`, `Postgres`, `Protobuf`, `Archive`, `UI`) share this one implementation.

```jai
Utf8 :: #import "Extensions/Utf8";

ok := Utf8.is_valid(text);                    // strict: no overlong forms, surrogates or > U+10FFFF
code, size := Utf8.decode(text, at);          // size 0 when the bytes at `at` are invalid or cut short
buffer: [4] u8;
count := Utf8.encode(code, buffer.data);      // 1 to 4 bytes
```

## How it works

- `decode` reads one character at an index and returns `0, 0` for anything the standard rejects, so the caller picks the policy: `Regex` treats the byte as itself, `Postgres` rejects the password.
- `encode` writes into caller memory (up to four bytes) and does not check the value; `is_scalar` is the check.
- `is_valid` skips ASCII eight bytes at a time and validates the rest with the lead-byte ranges of the standard (`E0` needs `A0..BF` next, `ED` needs `80..9F`, `F0` needs `90..BF`, `F4` needs `80..8F`).

## How to change it

- Keep the three procedures in agreement: a byte string is valid exactly when `decode` never returns size 0 along it. `tests/utf8.jai` pins the edge cases (overlongs, surrogates, the last code point, truncation).
- A new caller with its own policy for invalid bytes wraps `decode` where it is used (see `decode_utf8` in `Extensions/Regex/syntax.jai`); do not add the policy here.
- The `internal/numerics` text helpers have their own `encode_utf8` and are not on this module.

Dependencies: `Basic` only.
