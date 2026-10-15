#import "Basic";
BG :: #import "Bindings_Generator";

check_operators :: () {
    inputs := string.["<=>", "->*", "->", "++", "--", "==", "!=", ">=", "<=", "<<", ">>", "&&", "||", "+", "-", "*", "/", "%", "^", "&", "|", "~", "<", ">", "[]", "!", "=", ",", "()", "\"\""];
    suffixes := string.["_wtf", "_wtf2", "_arrow", "_plus_plus", "_minus_minus", "_equals", "_not_equals", "_greater_equals", "_less_equals", "_lshift", "_rshift", "_and_and", "_or_or", "_plus", "_minus", "_times", "_divide", "_modulo", "_xor", "_and", "_or", "_tilde", "_less", "_greater", "_brackets", "_not", "_assign", "_comma", "_parens", "_literal"];

    for input, index: inputs {
        name := sprint("operator %", input);

        defer free(name);
        expected := sprint("operator%", suffixes[index]);

        defer free(expected);
        assert(BG.replace_operator_name(name) == expected);
        kept := BG.replace_operator_name(name, keep_operators_allowed_in_jai = true);
        if index < 5 || index >= 25 assert(kept == expected);
        else assert(kept == sprint("operator%", input));
    }

    assert(BG.replace_operator_name("operator +=") == "operator_plus_assign");
    assert(BG.replace_operator_name("operator +=", true) == "operator+=");
    assert(BG.replace_operator_name("operator new []") == "operatornew_brackets");
    assert(BG.avoid_reserved_names("operator ==") == "operator==");
    assert(BG.avoid_reserved_names("operator!") == "operator_not");
    assert(BG.avoid_reserved_names("operator=") == "operator_assign");
    assert(BG.replace_operator_name("operator\t+") == "operator\t_plus");
    assert(BG.replace_operator_name("") == "");
}

check_identifiers :: () {
    assert(BG.avoid_cpp_keywords("this") == "_this");
    assert(BG.avoid_cpp_keywords("class") == "class");
    assert(BG.avoid_cpp_keywords("This") == "This");

    reserved := string.["cast", "context", "false", "is_constant", "null", "remove", "true", "type_info", "xx", "interface", "Any", "bool", "float", "float32", "float64", "int", "s16", "s32", "s64", "s8", "string", "u16", "u32", "u64", "u8", "v128", "void"];

    assert(BG.RESERVED_NAMES.count == reserved.count);
    assert(BG.RESERVED_TYPE_NAMES.count == 1 && BG.RESERVED_TYPE_NAMES[0] == "Type");

    for name, index: reserved {
        assert(BG.RESERVED_NAMES[index] == name);
        assert(BG.avoid_reserved_names(name) == sprint("_%", name));
    }

    assert(BG.avoid_reserved_names("Type") == "Type");
    assert(BG.avoid_reserved_names("Type", true) == "_Type");
    assert(BG.avoid_reserved_names("Int", true) == "Int");
    assert(BG.avoid_reserved_names("") == "");
    mutable := copy_string("9-a.b !");

    defer free(mutable);
    BG.make_valid_identifer(mutable);
    assert(mutable == "9_a_b !");
    BG.make_valid_identifer("");

    selector := "setValue:forKey:";
    original := selector;

    BG.convert_objc_selector_name_to_function_identifier(*selector);
    assert(selector == "setValue" && selector.data == original.data);
    assert(original == "setValue:forKey:");
    builder: String_Builder;

    defer free_buffers(*builder);
    append(*builder, "prefix:");
    BG.print_objc_selector_identifier(*builder, original);
    BG.print_objc_selector_identifier(*builder, "");
    rendered := builder_to_string(*builder);

    defer free(rendered);
    assert(rendered == "prefix:setValue_forKey_");
}

check_enum_lookup :: () {
    options: BG.Generate_Bindings_Options;
    saved := context._bindings_generator_options;

    defer context._bindings_generator_options = saved;
    context._bindings_generator_options = *options;
    options = .{};
    enumeration: BG.Enum;
    enumeration.name = "Color";
    assert(BG.get_enum_value_name(*enumeration, "Color_Red") == "Red");
    assert(BG.get_enum_value_name(*enumeration, "ColorRed") == "Red");
    assert(BG.get_enum_value_name(*enumeration, "Color__Red") == "_Red");
    assert(BG.get_enum_value_name(*enumeration, "Color_bool") == "bool");
    spelling := "Color_Red";
    sliced := BG.get_enum_value_name(*enumeration, spelling);

    assert(sliced.data == spelling.data + 6);
    options.strip_enum_values_with_suffix = "_MAX";
    assert(BG.get_enum_value_name(*enumeration, "Color_MAX") == "");
    assert(BG.get_enum_value_name(*enumeration, "Color_MAXimum") == "MAXimum");
    pairs: [..][2] string;

    defer array_free(pairs);
    array_add(*pairs, string.["Other", "C_"]);
    array_add(*pairs, string.["Color", "C_"]);
    array_add(*pairs, string.["Color", "C_Color_"]);
    options.strip_enum_value_prefixes = pairs;
    assert(BG.get_enum_value_name(*enumeration, "C_Color_Red") == "Red");
    options.strip_enum_prefixes = false;
    assert(BG.get_enum_value_name(*enumeration, "C_Color_Red") == "Color_Red");
    assert(BG.get_enum_value_name(*enumeration, spelling).data == spelling.data);
    enumeration.name = "";
    assert(BG.get_enum_value_name(*enumeration, "_Red") == "_Red");
}

check_enum_compute :: () {
    options: BG.Generate_Bindings_Options;
    saved := context._bindings_generator_options;

    defer context._bindings_generator_options = saved;
    context._bindings_generator_options = *options;
    options = .{};
    enumeration: BG.Enum;

    defer array_free(enumeration.enumerates);
    red: BG.Declaration;
    green: BG.Declaration;
    omitted: BG.Declaration;

    red.name = "API_Red";
    red.output_name = red.name;
    green.name = "API_Green";
    green.output_name = green.name;
    omitted.name = "Other";
    omitted.output_name = "Other";
    omitted.decl_flags = .OMIT_FROM_OUTPUT;
    array_add(*enumeration.enumerates, *red);
    array_add(*enumeration.enumerates, *omitted);
    array_add(*enumeration.enumerates, *green);
    BG.compute_enum_names(*enumeration);
    assert(enumeration.output_name == "API");
    assert(red.output_name == "Red" && green.output_name == "Green");
    assert(omitted.output_name == "Other");

    enumeration.name = "Color";
    enumeration.output_name = "Color";
    red.name = "Color_Red";
    red.output_name = "Red";
    green.name = "Color_Rose";
    green.output_name = "Rose";
    options.enum_prefix_delimiter = "";
    BG.compute_enum_names(*enumeration);
    assert(red.output_name == "Red" && green.output_name == "Rose");

    red.output_name = "PRE_9";
    green.output_name = "PRE_bool";
    red.name = "OriginalA";
    green.name = "OriginalB";
    options.enum_prefix_delimiter = "_";
    BG.compute_enum_names(*enumeration);
    assert(red.output_name == "_9" && green.output_name == "_bool");

    red.output_name = "_";
    green.output_name = "7";
    BG.compute_enum_names(*enumeration, already_stripped = true);
    assert(red.output_name == "_" && green.output_name == "_7");

    red.output_name = "PRE_A";
    green.output_name = "PRE_B";
    BG.compute_enum_names(*enumeration, already_stripped = true);
    assert(red.output_name == "PRE_A" && green.output_name == "PRE_B");

    red.output_name = "Same";
    green.output_name = "Same";
    options.enum_prefix_delimiter = "";
    BG.compute_enum_names(*enumeration);
    assert(red.output_name == "_" && green.output_name == "_");

    red.output_name = "NoPrefixA";
    green.output_name = "NoPrefixB";
    options.enum_prefix_delimiter = "_";
    BG.compute_enum_names(*enumeration);
    assert(red.output_name == "NoPrefixA" && green.output_name == "NoPrefixB");

    options.auto_detect_enum_prefixes = false;
    red.output_name = "PRE_A";
    green.output_name = "PRE_B";
    BG.compute_enum_names(*enumeration);
    assert(red.output_name == "PRE_A" && green.output_name == "PRE_B");

    red.output_name = "__bool";
    green.output_name = "__1";
    BG.compute_enum_names(*enumeration);
    assert(red.output_name == "_bool" && green.output_name == "_1");

    empty: BG.Enum;

    BG.compute_enum_names(*empty, true);
    assert(empty.output_name == "anon_enum");

    options = .{};
    enumeration.name = "";
    enumeration.output_name = "";
    red.name = "A";
    green.name = "B";
    red.output_name = "LEFT__one";
    green.output_name = "LEFT__two";
    BG.compute_enum_names(*enumeration);
    assert(enumeration.output_name == "LEFT");
    assert(red.output_name == "one" && green.output_name == "two");

    enumeration.output_name = "Keep";
    red.output_name = "A";
    green.output_name = "B";
    BG.compute_enum_names(*enumeration);
    assert(red.output_name == "A" && green.output_name == "B");

    options.enum_prefix_delimiter = "::";
    red.output_name = "Root::Child::one";
    green.output_name = "Root::Child::two";
    BG.compute_enum_names(*enumeration);
    assert(red.output_name == "one" && green.output_name == "two");

    options.strip_enum_prefixes = false;
    options.enum_prefix_delimiter = "_";
    enumeration.name = "Color";
    red.name = "Color_Red";
    green.name = "Color_Green";
    red.output_name = red.name;
    green.output_name = green.name;
    BG.compute_enum_names(*enumeration);
    assert(red.output_name == "Red" && green.output_name == "Green");

    red.output_name = "ONLY_Red";
    green.output_name = "OMITTED_Green";
    green.decl_flags = .OMIT_FROM_OUTPUT;
    BG.compute_enum_names(*enumeration);
    assert(red.output_name == "ONLY_Red");
    assert(green.output_name == "OMITTED_Green");
    red.decl_flags = .OMIT_FROM_OUTPUT;
    enumeration.output_name = "";
    BG.compute_enum_names(*enumeration);
    assert(enumeration.output_name == "anon_enum");
    assert(red.output_name == "ONLY_Red" && omitted.output_name == "Other");
}

main :: () {
    check_identifiers();
    check_operators();
    check_enum_lookup();
    check_enum_compute();
    print("generator naming contracts passed\n");
}
