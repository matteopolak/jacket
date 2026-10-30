# Unicode


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `Unicode_Result :: enum`
- `UNI_REPLACEMENT_CHAR :: 0x0000FFFD`
- `UNI_MAX_BMP :: 0x0000FFFF`
- `UNI_MAX_UTF16 :: 0x0010FFFF`
- `UNI_MAX_UTF32 :: 0x7FFFFFFF`
- `character_utf8_to_utf32 :: (s: *u8, source_length: s64, $strict := false) -> u32, s64, Unicode_Result`
- `character_utf32_to_utf8 :: (ch: u32, preallocated_string: *string)`
- `character_utf32_to_utf8 :: (ch: u32) -> string`
- `unicode_next_character :: (s: *u8) -> *u8`
- `utf8_next_character :: (s: *string) -> code: u32, success: Unicode_Result`
- `utf8_iter :: (s: *string, body: Code, flags: For_Flags) #expand`
- `trailingBytesForUTF8 :: [256]u8`
- `utf8_inital_byte_mask :: [6]u8`
- `offsetsFromUTF8 : [6] u32 : .[ 0x00000000, 0x00003080, 0x000E2080, 0x03C82080, 0xFA082080, 0x82082080 ]`
- `firstByteMark : [7] u8 : .[ 0x00, 0x00, 0xC0, 0xE0, 0xF0, 0xF8, 0xFC ]`


## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `Unicode_Result`

```jai
Unicode_Result :: enum {
    CONVERSION_OK;
    SOURCE_EXHAUSTED;
    SOURCE_ILLEGAL;
}
```

## Public array values and mutability

`trailingBytesForUTF8` is an immutable `[256]u8` constant: indices `0x00..0xBF` contain 0; `0xC0..0xDF` contain 1; `0xE0..0xEF` contain 2; `0xF0..0xF7` contain 3; `0xF8..0xFB` contain 4; and `0xFC..0xFF` contain 5. `utf8_inital_byte_mask` is the immutable `[6]u8` value `[0x7F, 0x1F, 0x0F, 0x07, 0x03, 0x01]`.

`offsetsFromUTF8` and `firstByteMark` are typed initialized constants (the declarations use `: Type : initializer`), with values `[0x00000000, 0x00003080, 0x000E2080, 0x03C82080, 0xFA082080, 0x82082080]` and `[0x00, 0x00, 0xC0, 0xE0, 0xF0, 0xF8, 0xFC]` respectively. Encoding consults `firstByteMark`; decoding keeps only the payload bits of each byte (the lead byte masked by its sequence length, each continuation byte's low six bits), so `offsetsFromUTF8` is informational. These lookup tables are not caller-mutable storage.
## Behavioral contract

UTF-8 decoding returns a code point, consumed byte count, and `Unicode_Result`. A broken byte structure (a continuation byte where a character should start, or a non-continuation byte inside a character) is `SOURCE_ILLEGAL` with U+FFFD in both modes; the consumed count is the lead byte plus the good continuation bytes before the bad one, so it is never 0 and decoding can resume at the bad byte (a stray continuation byte consumes just itself). Strict mode additionally reports overlong encodings, surrogates, and scalar values above U+10FFFF as `SOURCE_ILLEGAL`, consuming the complete sequence; non-strict mode decodes such well-formed sequences as they are (values above `UNI_MAX_UTF32` become U+FFFD). If the input ends before the expected sequence length and the bytes present are fine, the decoder returns `SOURCE_EXHAUSTED`, U+FFFD, and consumes all remaining input bytes.

`utf8_next_character` calls the decoder in its non-strict default mode, then advances `s.data` and reduces `s.count` by exactly the returned byte count regardless of status. Consequently an incomplete trailing sequence is replaced and consumes the entire remainder. This helper returns zero consumed bytes only for an empty string; callers must use the status.

`utf8_iter` explicitly decodes strictly: on any non-OK status it logs an error and stops without yielding a replacement element. It supports forward and reverse traversal, but rejects pointer iteration. Forward `it_index` starts at zero; reverse iteration does not compute an index and sets it to -1. A stray continuation byte is invalid like any other malformation; going backwards, stray bytes after the last character are reported first. For valid text, reverse iteration gives the forward elements last first. Encoding writes U+FFFD in place of a surrogate or a value above U+10FFFF. It writes into caller storage with at least four bytes available or allocates a four-byte buffer and returns a shorter view. Depends on `Basic` allocation and compiler custom-iterator expansion.
