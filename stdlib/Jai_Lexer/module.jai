// Generated from the Jai_Lexer public contract by tools/generate-jai-lexer-contract.py.

LEXER_SIMD_WIDTH :: 16;

MAX_CONCURRENT_TOKENS :: 8;

MAX_IDENTIFIER_LENGTH :: 512;

USE_SIMD :: false;

Table_String :: #type,isa string;

Hex_Digit_Type :: enum u8 {
    ASCII_8_BIT :: 0;
    UNICODE_16 :: 1;
    UNICODE_32 :: 2;
}

Note_Flags :: enum_flags u8 {
    IS_SYSTEM_LEVEL :: 0x1;
}

Token :: struct {
    type: Token_Type = .ERROR;
    l0, c0: s32;
    l1, c1: s32 = -1;
    union {
        ident_value: struct {
            name: Table_String;
            hash: u32;
        };
        integer_value: u64;
        float64_value: float64;
        string_value: string;
    };
    value_flags: Value_Flags;
    ident_is_backticked := false;
}

Token_Type :: enum s16 {
    BITWISE_XOR :: #char "^";
    BITWISE_NOT :: #char "~";
    BITWISE_AND :: #char "&";
    BITWISE_OR :: #char "|";
    IDENT :: 256;
    NUMBER :: 257;
    STRING :: 258;
    PLUSEQUALS :: 259;
    MINUSEQUALS :: 260;
    TIMESEQUALS :: 261;
    DIVEQUALS :: 262;
    MODEQUALS :: 263;
    ISEQUAL :: 264;
    ISNOTEQUAL :: 265;
    LOGICAL_AND :: 266;
    LOGICAL_OR :: 267;
    LESSEQUALS :: 268;
    GREATEREQUALS :: 269;
    RIGHT_ARROW :: 270;
    DOUBLE_DOT :: 271;
    DOUBLE_DOLLAR :: 272;
    SHIFT_LEFT :: 273;
    SHIFT_RIGHT :: 274;
    ROTATE_LEFT :: 275;
    ROTATE_RIGHT :: 276;
    TRIPLE_EQUALS :: 277;
    SHIFT_LEFT_EQUALS :: 290;
    SHIFT_RIGHT_EQUALS :: 291;
    ROTATE_LEFT_EQUALS :: 292;
    ROTATE_RIGHT_EQUALS :: 293;
    BITWISE_AND_EQUALS :: 294;
    BITWISE_OR_EQUALS :: 295;
    BITWISE_XOR_EQUALS :: 296;
    LOGICAL_AND_EQUALS :: 297;
    LOGICAL_OR_EQUALS :: 298;
    POINTER_DEREFERENCE :: 310;
    POSTFIX_DEREFERENCE :: 311;
    ISEQUAL_FOR_SWITCH_STATEMENT :: 312;
    DOUBLE_MINUS :: 313;
    TRIPLE_MINUS :: 314;
    DOUBLE_COMMA :: 315;
    BEGIN_STRUCT_LITERAL :: 316;
    BEGIN_ARRAY_LITERAL :: 317;
    KEYWORD_FOR :: 318;
    __FIRST_KEYWORD :: KEYWORD_FOR;
    KEYWORD_IF :: 319;
    KEYWORD_IFX :: 320;
    KEYWORD_THEN :: 321;
    KEYWORD_ELSE :: 322;
    KEYWORD_CASE :: 323;
    KEYWORD_RETURN :: 324;
    KEYWORD_STRUCT :: 325;
    KEYWORD_WHILE :: 326;
    KEYWORD_BREAK :: 327;
    KEYWORD_CONTINUE :: 328;
    KEYWORD_REMOVE :: 329;
    KEYWORD_USING :: 330;
    KEYWORD_DEFER :: 331;
    KEYWORD_SIZE_OF :: 332;
    KEYWORD_TYPE_OF :: 333;
    KEYWORD_CODE_OF :: 334;
    KEYWORD_INITIALIZER_OF :: 335;
    KEYWORD_TYPE_INFO :: 336;
    KEYWORD_NULL :: 337;
    KEYWORD_ENUM :: 338;
    KEYWORD_TRUE :: 339;
    KEYWORD_FALSE :: 340;
    KEYWORD_INLINE :: 341;
    KEYWORD_NO_INLINE :: 342;
    KEYWORD_CAST :: 343;
    KEYWORD_AUTO_CAST :: 344;
    KEYWORD_CONTEXT :: 345;
    KEYWORD_PUSH_CONTEXT :: 346;
    KEYWORD_OPERATOR :: 347;
    KEYWORD_IS_CONSTANT :: 348;
    KEYWORD_ENUM_FLAGS :: 349;
    KEYWORD_UNION :: 350;
    KEYWORD_INTERFACE :: 351;
    __LAST_KEYWORD :: KEYWORD_INTERFACE;
    QUICK_LAMBDA :: 352;
    NOTE :: 353;
    END_OF_INPUT :: 354;
    POINTER_DEREFERENCE_OR_SHIFT_LEFT :: 355;
    OPERATOR_ARRAY_SUBSCRIPT :: 500;
    OPERATOR_ASSIGNMENT_TO_ARRAY_SUBSCRIPT :: 501;
    ERROR :: 502;
}

Value_Flags :: enum_flags u32 {
    HERE_STRING :: 0x1;
    NUMBER :: 0x2;
    HEX :: 0x4;
    BINARY :: 0x8;
    FLOAT :: 0x10;
    REQUIRES_FLOAT64 :: 0x10_0000;
    DEFAULTS_TO_FLOAT64 :: 0x20_0000;
    REQUIRES_FLOAT64_DUE_TO_SIGNIFICANT_DIGITS :: 0x40_0000;
    OVERFLOWED :: 0x1000_0000;
}

Lexer :: struct {
    current_line_number: s32;
    current_external_file_error_report_line_number: s32;
    current_character_index: s32;
    atom_pool: P.Pool;
    tokens: [MAX_CONCURRENT_TOKENS] Token;
    token_array_cursor: s32;
    num_incoming_tokens: s32;
    input: string;
    input_cursor: s64;
    should_free_input := false;
    total_lines_processed: s64;
    nonblank_lines_processed: s64;
    previous_token_line_number: s64;
    eof_token: Token;
    user_data: *void;
    scratch_buffer: [MAX_IDENTIFIER_LENGTH] u8;
    reported_error := false;
}

cached_make_atom :: Impl.cached_make_atom;

check_for_equals :: Impl.check_for_equals;

check_for_keyword :: Impl.check_for_keyword;

compose_new_token :: Impl.compose_new_token;

continues_identifier :: Impl.continues_identifier;

count_herestring_lines :: Impl.count_herestring_lines;

eat_character :: Impl.eat_character;

eat_input_due_to_block_comment :: Impl.eat_input_due_to_block_comment;

eat_token :: Impl.eat_token;

eat_until_newline :: Impl.eat_until_newline;

get_decimal_digit :: Impl.get_decimal_digit;

get_hex_digit :: Impl.get_hex_digit;

get_last_token :: Impl.get_last_token;

get_unused_token :: Impl.get_unused_token;

has_a_big_exponent :: Impl.has_a_big_exponent;

make_hexfloat :: Impl.make_hexfloat;

make_ident_or_keyword :: Impl.make_ident_or_keyword;

make_note :: Impl.make_note;

make_number :: Impl.make_number;

make_one_character_token :: Impl.make_one_character_token;

make_string_constant :: Impl.make_string_constant;

parse_here_string :: Impl.parse_here_string;

parse_ident :: Impl.parse_ident;

parse_note :: Impl.parse_note;

peek_next_character :: Impl.peek_next_character;

peek_next_token :: Impl.peek_next_token;

peek_token :: Impl.peek_token;

report_detail :: Impl.report_detail;

report_parse_error :: Impl.report_parse_error;

set_end_of_token :: Impl.set_end_of_token;

set_input_from_file :: Impl.set_input_from_file;

set_input_from_string :: Impl.set_input_from_string;

starts_identifier :: Impl.starts_identifier;

starts_number :: Impl.starts_number;

unwind_one_character :: Impl.unwind_one_character;

#scope_file
P :: #import "Pool";
Impl :: #import "internal/jai-lexer";
