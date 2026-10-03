# Jai_Lexer

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `USE_SIMD :: false`
- `MAX_IDENTIFIER_LENGTH :: 512`
- `MAX_CONCURRENT_TOKENS :: 8`
- `LEXER_SIMD_WIDTH :: 16`
- `Token_Type :: enum s16`
- `Note_Flags :: enum_flags u8`
- `Table_String :: #type,isa string`
- `Value_Flags :: enum_flags u32`
- `Token :: struct`
- `Lexer :: struct`
- `peek_next_token :: (using lexer: *Lexer) -> *Token`
- `get_last_token :: (lexer: *Lexer) -> Token`
- `starts_identifier :: inline (c: u8) -> bool`
- `Hex_Digit_Type :: enum u8`
- `cached_make_atom :: (lexer: *Lexer, name: string, token_to_fill: *Token)`
- `eat_character :: (using lexer: *Lexer)`
- `peek_next_character :: (lexer: *Lexer) -> s16`
- `report_parse_error :: (lexer: *Lexer, format: string, arguments: .. Any)`
- `report_parse_error :: (lexer: *Lexer, token: *Token, format: string, arguments: .. Any)`
- `report_detail :: (lexer: *Lexer, format: string, arguments: .. Any)`
- `get_hex_digit :: (lexer: *Lexer, type: Hex_Digit_Type) -> (value: u8, success: bool)`
- `get_decimal_digit :: (lexer: *Lexer) -> s32`
- `set_end_of_token :: (lexer: *Lexer, token: *Token)`
- `unwind_one_character :: (lexer: *Lexer)`
- `continues_identifier :: (c: s32) -> bool`
- `starts_number :: (c: s32) -> bool`
- `get_unused_token :: (lexer: *Lexer) -> *Token`
- `parse_ident :: (lexer: *Lexer, result: *Token) -> string`
- `parse_here_string :: (lexer: *Lexer) -> *Token`
- `make_note :: (lexer: *Lexer) -> *Token`
- `parse_note :: (lexer: *Lexer) -> string`
- `check_for_keyword :: (token: *Token) -> Token_Type`
- `make_ident_or_keyword :: (lexer: *Lexer) -> *Token`
- `make_hexfloat :: (lexer: *Lexer, result: *Token) -> *Token`
- `make_number :: (lexer: *Lexer) -> *Token`
- `make_string_constant :: (lexer: *Lexer) -> *Token`
- `make_one_character_token :: (lexer: *Lexer, c: Token_Type) -> *Token`
- `count_herestring_lines :: (lexer: *Lexer, token: *Token) -> s64`
- `eat_token :: (lexer: *Lexer)`
- `check_for_equals :: (lexer: *Lexer, c: Token_Type, augmented: Token_Type, eat := true, subtractor: s32 = 0) -> *Token`
- `eat_input_due_to_block_comment :: (lexer: *Lexer)`
- `eat_until_newline :: (lexer: *Lexer)`
- `compose_new_token :: (lexer: *Lexer) -> *Token`

- `has_a_big_exponent :: inline (value: float64) -> bool`
- `peek_token :: (lexer: *Lexer, lookahead_index: s32) -> *Token`
- `set_input_from_file :: (lexer: *Lexer, file_path: string) -> bool`
- `set_input_from_string :: (lexer: *Lexer, input: string)`

## Public type members

#### `Token_Type` (enum)

- `BITWISE_XOR :: #char "^"`
- `BITWISE_NOT :: #char "~"`
- `BITWISE_AND :: #char "&"`
- `BITWISE_OR  :: #char "|"`
- `IDENT :: 256`
- `NUMBER :: 257`
- `STRING :: 258`
- `PLUSEQUALS :: 259`
- `MINUSEQUALS :: 260`
- `TIMESEQUALS :: 261`
- `DIVEQUALS :: 262`
- `MODEQUALS :: 263`
- `ISEQUAL :: 264`
- `ISNOTEQUAL :: 265`
- `LOGICAL_AND :: 266`
- `LOGICAL_OR :: 267`
- `LESSEQUALS :: 268`
- `GREATEREQUALS :: 269`
- `RIGHT_ARROW :: 270`
- `DOUBLE_DOT :: 271`
- `DOUBLE_DOLLAR :: 272`
- `SHIFT_LEFT  :: 273`
- `SHIFT_RIGHT :: 274`
- `ROTATE_LEFT         :: 275`
- `ROTATE_RIGHT        :: 276`
- `TRIPLE_EQUALS       :: 277`
- `SHIFT_LEFT_EQUALS   :: 290`
- `SHIFT_RIGHT_EQUALS  :: 291`
- `ROTATE_LEFT_EQUALS  :: 292`
- `ROTATE_RIGHT_EQUALS :: 293`
- `BITWISE_AND_EQUALS  :: 294`
- `BITWISE_OR_EQUALS   :: 295`
- `BITWISE_XOR_EQUALS  :: 296`
- `LOGICAL_AND_EQUALS  :: 297`
- `LOGICAL_OR_EQUALS   :: 298`
- `POINTER_DEREFERENCE :: 310`
- `POSTFIX_DEREFERENCE :: 311`
- `ISEQUAL_FOR_SWITCH_STATEMENT :: 312`
- `DOUBLE_MINUS :: 313`
- `TRIPLE_MINUS :: 314`
- `DOUBLE_COMMA :: 315`
- `BEGIN_STRUCT_LITERAL :: 316`
- `BEGIN_ARRAY_LITERAL :: 317`
- `KEYWORD_FOR :: 318`
- `__FIRST_KEYWORD :: KEYWORD_FOR`
- `KEYWORD_IF :: 319`
- `KEYWORD_IFX :: 320`
- `KEYWORD_THEN :: 321`
- `KEYWORD_ELSE :: 322`
- `KEYWORD_CASE :: 323`
- `KEYWORD_RETURN :: 324`
- `KEYWORD_STRUCT :: 325`
- `KEYWORD_WHILE :: 326`
- `KEYWORD_BREAK :: 327`
- `KEYWORD_CONTINUE :: 328`
- `KEYWORD_REMOVE :: 329`
- `KEYWORD_USING :: 330`
- `KEYWORD_DEFER :: 331`
- `KEYWORD_SIZE_OF :: 332`
- `KEYWORD_TYPE_OF :: 333`
- `KEYWORD_CODE_OF :: 334`
- `KEYWORD_INITIALIZER_OF :: 335`
- `KEYWORD_TYPE_INFO :: 336`
- `KEYWORD_NULL :: 337`
- `KEYWORD_ENUM :: 338`
- `KEYWORD_TRUE :: 339`
- `KEYWORD_FALSE :: 340`
- `KEYWORD_INLINE :: 341`
- `KEYWORD_NO_INLINE :: 342`
- `KEYWORD_CAST :: 343`
- `KEYWORD_AUTO_CAST :: 344`
- `KEYWORD_CONTEXT :: 345`
- `KEYWORD_PUSH_CONTEXT :: 346`
- `KEYWORD_OPERATOR :: 347`
- `KEYWORD_IS_CONSTANT :: 348`
- `KEYWORD_ENUM_FLAGS :: 349`
- `KEYWORD_UNION :: 350`
- `KEYWORD_INTERFACE :: 351`
- `__LAST_KEYWORD :: KEYWORD_INTERFACE`
- `QUICK_LAMBDA :: 352`
- `NOTE :: 353`
- `END_OF_INPUT :: 354`
- `POINTER_DEREFERENCE_OR_SHIFT_LEFT :: 355`
- `OPERATOR_ARRAY_SUBSCRIPT :: 500`
- `OPERATOR_ASSIGNMENT_TO_ARRAY_SUBSCRIPT :: 501`
- `ERROR :: 502`

#### `Note_Flags` (enum_flags)

- `IS_SYSTEM_LEVEL :: 0x1`

#### `Value_Flags` (enum_flags)

- `HERE_STRING      :: 0x1`
- `NUMBER           :: 0x2`
- `HEX              :: 0x4`
- `BINARY           :: 0x8`
- `FLOAT            :: 0x10`
- `REQUIRES_FLOAT64    :: 0x10_0000`
- `DEFAULTS_TO_FLOAT64 :: 0x20_0000`
- `REQUIRES_FLOAT64_DUE_TO_SIGNIFICANT_DIGITS :: 0x40_0000`
- `OVERFLOWED       :: 0x1000_0000`

#### `Token` (struct)

```jai
Token :: struct {
    type: Token_Type = .ERROR;
    l0, c0: s32;
    l1, c1: s32 = -1;
    union {
        ident\ _value: struct { name: Table_String; hash: u32; };
        integer_value: u64;
        float64_value: float64;
        string\_value: string;
    };
    value_flags: Value_Flags;
    ident_is_backticked := false;
}
```
#### `Lexer` (struct)

- `current_line_number: s32`
- `current_external_file_error_report_line_number: s32`
- `current_character_index: s32`
- `atom_pool:  Pool`
- `tokens: [MAX_CONCURRENT_TOKENS] Token`
- `token_array_cursor:  s32`
- `num_incoming_tokens: s32`
- `input: string`
- `input_cursor: s64`
- `should_free_input := false`
- `total_lines_processed:    s64`
- `nonblank_lines_processed: s64`
- `previous_token_line_number: s64`
- `eof_token: Token`
- `user_data: *void`
- `scratch_buffer: [MAX_IDENTIFIER_LENGTH] u8`
- `reported_error := false`

#### `Hex_Digit_Type` (enum)

- `ASCII_8_BIT :: 0`
- `UNICODE_16  :: 1`
- `UNICODE_32  :: 2`

## Behavioral contract and constraints

The lexer tokenizes source into tokens with start/end line and character coordinates. `Token_Type` values 312–355 are sequential in declaration order; `__FIRST_KEYWORD` and `__LAST_KEYWORD` alias the first and last keyword values, respectively. `POINTER_DEREFERENCE_OR_SHIFT_LEFT` is 355; it is a disambiguation token used while parsing `<`, not an alias of `SHIFT_LEFT` (273). The lexer emits `ROTATE_LEFT` for `<<<`, `ROTATE_RIGHT` for `>>>`, `ROTATE_LEFT_EQUALS` for `<<<=`, and `ROTATE_RIGHT_EQUALS` for `>>>=`. `OPERATOR_ARRAY_SUBSCRIPT` and `OPERATOR_ASSIGNMENT_TO_ARRAY_SUBSCRIPT` are operator identifiers rather than input tokens.

Identifiers and note names are copied into strings by the active atom helper. The module guarantees no canonical interned pointer identity or a stable/populated `Token.ident_value.hash`; callers should compare identifier text and treat the hash value as unspecified. `Table_String :: #type,isa string` declares a distinct type variant based on `string`, rather than an ordinary alias; it retains its own type identity and has the language's implicit conversion relationship with its base type. String token contents are owned/copied separately from the input; source coordinates refer to the lexer's current input. The input string must remain valid while lexing.

Floating literals set `REQUIRES_FLOAT64` when their parsed magnitude has a normal binary exponent outside float32's range (greater than 127 or less than -126), or when the lexer counts more than eight significant digits after the decimal point. More than eight counted digits also sets `REQUIRES_FLOAT64_DUE_TO_SIGNIFICANT_DIGITS`; exactly eight or more counted digits that do not otherwise require float64 set `DEFAULTS_TO_FLOAT64`. This significant-digit count is a heuristic: digits before the decimal point are not included by the current rule, and trailing zeros after the decimal are ignored. `has_a_big_exponent` returns false for zero, subnormal, infinity, and NaN bit patterns. Hex-float spellings `0h` require 4, 8, or 16 hex digits; 4/8 digits are interpreted as float32 bit patterns and 16 as float64 bits.

`#string IDENT` consumes the rest of its header line as whitespace, then takes lines through a line whose leading whitespace is ignored and whose first identifier is exactly `IDENT`; a following identifier-continuation character prevents a match. Content indentation is retained, terminator indentation and line are excluded, and the content includes the preceding line ending. The optional `,cr` modifier appears before the terminator identifier: both `#string,cr END` and `#string , cr END` are accepted, as are spaces before the comma, between comma and `cr`, and between `cr` and `END`. The modifier inserts CR before each LF that lacks one; without it, CR in CRLF pairs is removed if the content contains carriage returns. Invalid modifiers, non-whitespace after the header identifier, or EOF before a terminator report an error; the sublexer may return null on invalid header/unterminated input. Here-string parsing requires no buffered lookahead tokens.

`get_hex_digit` consumes one ASCII hex digit and returns its 0–15 value with `success=true`; on failure it reports an error, leaves the non-hex character unconsumed, and returns `(0,false)`. Its `Hex_Digit_Type` argument selects the diagnostic for `\\x` (two digits), `\\u` (four), or `\\U` (eight); it does not itself consume or validate a complete escape. `get_decimal_digit` similarly consumes one decimal digit, or reports the three-digit `\\d` diagnostic and returns -1 without consuming the invalid character.

Number parsing returns the prefix recognized before an invalid character and normally leaves that character for the next token. Invalid binary digits and malformed decimal/exponent forms set the lexer error state and may return a partial numeric token; integer overflow sets `OVERFLOWED` and reports an error but still returns the accumulated `u64` value. A second decimal point is consumed before reporting the error. Unterminated strings/comments and other truncated sublexers have edge cases not specified uniformly; callers should check `reported_error` and not assume recovery preserves the whole malformed spelling.
