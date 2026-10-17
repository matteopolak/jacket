#import "Basic";
B :: #import "Bindings_Generator";
B2 :: #import "Bindings_Generator";
BW :: #import "Bindings_Generator"(TARGET_OS = .WINDOWS);

#run {
    storage := cast(*Type_Info_Enum) type_info(B.CX_StorageClass);

    #if OS == .WINDOWS {
        assert(storage.internal_type.signed);
    } else {
        assert(!storage.internal_type.signed);
    }
}

main :: () {
    assert(B.TOKENS_TO_REPLACE.count == 36);
    assert(B.TOKENS_TO_REPLACE[0][0] == "FLT_MIN" && B.TOKENS_TO_REPLACE[0][1] == "FLOAT32_MIN");
    assert(B.TOKENS_TO_REPLACE[0][0] == B2.TOKENS_TO_REPLACE[0][0]);
    assert(type_info(B.CType) == type_info(B2.CType));
    assert(BW.TARGET_OS == .WINDOWS && BW.TOKENS_TO_REPLACE[32][1] == "S32_MIN");
    assert(B.is_hex_digit(#char "a") && !B.is_hex_digit(#char "g"));
    assert(B.is_valid_int_literal_suffix("ULL"));
    assert(B.is_valid_int_literal_suffix("uz"));
    assert(!B.is_valid_int_literal_suffix("lL"));
    assert(!B.is_valid_float_literal_suffix("L"));
    value, ok, string_value := B.convert_literal_value("077u");

    assert(ok && !string_value && value == "0x3f");
    binary, binary_ok, was_binary_string := B.convert_literal_value("0b101U");

    assert(binary_ok && !was_binary_string && binary == "0b101");
    floating, float_ok := B.convert_float_literal("12e-3F");

    assert(float_ok && floating == "12.0e-3");
    invalid, invalid_ok := B.convert_float_literal("0x1p2");

    assert(!invalid_ok && invalid == "");
    quoted, quoted_ok, was_string := B.convert_literal_value("\"hello\"");

    assert(quoted_ok && was_string && quoted == "\"hello\"");
    rejected, rejected_ok, rejected_string := B.convert_literal_value("\"a\"b\"");

    assert(!rejected_ok && !rejected_string && rejected == "");
    overflow, overflow_ok, overflow_string := B.convert_literal_value("0777777777777777777777777");

    assert(!overflow_ok && !overflow_string && overflow == "");
    print("ok\n");
}
