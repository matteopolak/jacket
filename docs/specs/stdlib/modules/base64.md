# Base64

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `base64_encode :: (input: string, table: string = BASE64_ENCODER_TABLE, pad := true) -> string`
- `base64_decode :: (input: string, table: [256]u8 = BASE64_DECODER_TABLE) -> string, success: bool`
- `base64url_encode :: (input: string) -> string`
- `base64url_decode :: (input: string) -> string, success: bool`
- `BASE64_ENCODER_TABLE :: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"`
- `BASE64_DECODER_TABLE :: [256]u8`
- `BASE64URL_ENCODER_TABLE :: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_"`
- `BASE64URL_DECODER_TABLE :: [256]u8`
- `test_base64 :: ()`
- `test_base64_url :: ()`
- `test_base64_module :: ()`

## Public type members

No public aggregate fields or enum members were identified.

## Behavioral contract and constraints

`base64_encode` accepts a custom alphabet string and pads by default; URL encoding uses `-` and `_` in place of `+` and `/` and omits padding. The default decoder tables map `A`–`Z` to 0–25, `a`–`z` to 26–51, and `0`–`9` to 52–61. The standard table maps `+` to 62 and `/` to 63; the URL table maps `-` to 62 and `_` to 63. Both map line feed (`0x0A`) to the whitespace sentinel 64, `=` (`0x3D`) to the padding sentinel 65, and every other byte to invalid sentinel 66. The sentinel constants themselves are private, but the initialized `[256]u8` tables are exported compile-time constants and cannot be edited as mutable globals. A caller-supplied decoder table can define different byte mappings. `base64_decode` ignores bytes mapped to whitespace, stops at the first padding marker, and returns partial output with `success = false` at the first invalid value. Two- or three-symbol trailing groups emit one or two bytes; a one-symbol tail emits none and still returns success. Padding placement and bytes after the first padding marker are not validated. Results are returned as Jai strings; allocator choice is not documented.
