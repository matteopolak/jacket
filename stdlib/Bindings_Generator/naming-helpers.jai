avoid_cpp_keywords :: (name: string) -> string {
    if name == "this" return "_this";
    return name;
}

avoid_reserved_names :: (name: string, avoid_type_names := false) -> string {
    if generator_name_begins(name, "operator") return replace_operator_name(name, true);
    for reserved: RESERVED_NAMES if name == reserved return sprint("_%", name);

    if avoid_type_names {
        for reserved: RESERVED_TYPE_NAMES if name == reserved return sprint("_%", name);
    }

    return name;
}

make_valid_identifer :: (_struct: string) {
    for byte, index: _struct {
        if byte == #char "-" || byte == #char "." _struct.data[index] = #char "_";
    }
}

replace_operator_name :: (name: string, keep_operators_allowed_in_jai := false) -> string {
    return generator_name_replace_operator(name, keep_operators_allowed_in_jai);
}

convert_objc_selector_name_to_function_identifier :: (s: *string) {
    s.count = generator_name_selector_stem_length(s.*);
}

print_objc_selector_identifier :: (builder: *String_Builder, ident: string) {
    for byte: ident append(builder, ifx byte == #char ":" then #char "_" else byte);
}

get_enum_value_name :: (_enum: *Enum, name: string) -> string {
    options: Generate_Bindings_Options;
    configured := gm_active_options();

    if configured options = configured.*;

    suffix := options.strip_enum_values_with_suffix;

    if suffix.count > 0 && generator_name_ends(name, suffix) return "";

    result := name;

    for pair: options.strip_enum_value_prefixes {
        if pair[0] == _enum.name && generator_name_begins(result, pair[1]) {
            result = generator_name_drop(result, pair[1].count);
            break;
        }
    }

    if options.strip_enum_prefixes && _enum.name.count > 0 && generator_name_begins(result, _enum.name) {
        result = generator_name_drop(result, _enum.name.count);
        if result.count > 0 && result.data[0] == #char "_" result = generator_name_drop(result, 1);
    }

    return result;
}

compute_enum_names :: (_enum: *Enum, already_stripped := false) {
    options: Generate_Bindings_Options;
    configured := gm_active_options();

    if configured options = configured.*;

    prefix: string;
    count := 0;
    suppress_detection := false;

    for value: _enum.enumerates {
        if value.decl_flags & .OMIT_FROM_OUTPUT continue;
        if count == 0 prefix = value.output_name;
        else prefix.count = generator_name_common_length(prefix, value.output_name);

        count += 1;
        if options.strip_enum_prefixes && _enum.name.count > 0 && generator_name_begins(value.name, _enum.name) suppress_detection = true;
    }

    if already_stripped || !options.auto_detect_enum_prefixes || count < 2 || suppress_detection prefix.count = 0;
    else if options.enum_prefix_delimiter.count > 0 {
        prefix.count = generator_name_delimited_length(prefix, options.enum_prefix_delimiter);
    }

    if _enum.output_name.count == 0 {
        _enum.output_name = prefix;
        while _enum.output_name.count > 0 && _enum.output_name.data[_enum.output_name.count - 1] == #char "_" _enum.output_name.count -= 1;
        if _enum.output_name.count == 0 _enum.output_name = "anon_enum";
    }

    for value: _enum.enumerates {
        if value.decl_flags & .OMIT_FROM_OUTPUT continue;

        result := generator_name_drop(value.output_name, prefix.count);
        while result.count > 0 && result.data[0] == #char "_" result = generator_name_drop(result, 1);

        result = avoid_reserved_names(result);
        if result.count == 0 || (result.data[0] >= #char "0" && result.data[0] <= #char "9") result = sprint("_%", result);

        value.output_name = result;
    }
}

#scope_file
#import "Basic";
#load "../internal/developer-tools/generator-names-private.jai";
