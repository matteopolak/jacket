TOKENS_TO_REPLACE :: #insert #run generate_replacements_array();

generate_replacements_array :: () -> string {
    builder: String_Builder;

    defer free_buffers(*builder);
    append(*builder, "[2] string.[");
    for pair: common_replacements print_to_builder(*builder, "string.[\"%\", \"%\"],", pair[0], pair[1]);

    pairs := ifx TARGET_OS == .WINDOWS then windows_replacements else other_replacements;
    for pair: pairs print_to_builder(*builder, "string.[\"%\", \"%\"],", pair[0], pair[1]);

    append(*builder, "]");

    return builder_to_string(*builder);
}

#scope_file
#import "Basic";

common_replacements :: [2] string.[string.["FLT_MIN", "FLOAT32_MIN"], string.["FLT_MAX", "FLOAT32_MAX"], string.["DBL_MIN", "FLOAT64_MIN"], string.["DBL_MAX", "FLOAT64_MAX"], string.["SCHAR_MIN", "S8_MIN"], string.["SCHAR_MAX", "S8_MAX"], string.["UCHAR_MIN", "0"], string.["UCHAR_MAX", "U8_MAX"], string.["SHRT_MIN", "S16_MIN"], string.["SHRT_MAX", "S16_MAX"], string.["USHRT_MIN", "0"], string.["USHRT_MAX", "U16_MAX"], string.["INT_MIN", "S32_MIN"], string.["INT_MAX", "S32_MAX"], string.["UINT_MIN", "0"], string.["UINT_MAX", "U32_MAX"], string.["LLONG_MIN", "S64_MIN"], string.["LLONG_MAX", "S64_MAX"], string.["ULLONG_MIN", "0"], string.["ULLONG_MAX", "U64_MAX"], string.["INT8_MIN", "S8_MIN"], string.["INT8_MAX", "S8_MAX"], string.["UINT8_MAX", "U8_MAX"], string.["INT16_MIN", "S16_MIN"], string.["INT16_MAX", "S16_MAX"], string.["UINT16_MAX", "U16_MAX"], string.["INT32_MIN", "S32_MIN"], string.["INT32_MAX", "S32_MAX"], string.["UINT32_MAX", "U32_MAX"], string.["INT64_MIN", "S64_MIN"], string.["INT64_MAX", "S64_MAX"], string.["UINT64_MAX", "U64_MAX"]];

windows_replacements :: [2] string.[string.["LONG_MIN", "S32_MIN"], string.["LONG_MAX", "S32_MAX"], string.["ULONG_MIN", "0"], string.["ULONG_MAX", "U32_MAX"]];

other_replacements :: [2] string.[string.["LONG_MIN", "S64_MIN"], string.["LONG_MAX", "S64_MAX"], string.["ULONG_MIN", "0"], string.["ULONG_MAX", "U64_MAX"]];
