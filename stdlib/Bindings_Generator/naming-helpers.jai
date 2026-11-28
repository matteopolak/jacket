// Returns `name`, except that the C++ keyword `this` becomes `_this`. No other keyword is changed. The result is
// `name` or a constant.
avoid_cpp_keywords :: (name: string) -> string {
    if name == "this" return "_this";
    return name;
}

// Returns `name` changed so that it does not clash with a reserved Jai name.
//
// An `operator` name is converted with [replace_operator_name], keeping the operators Jai allows. A name in
// `RESERVED_NAMES`, and with `avoid_type_names` also in `RESERVED_TYPE_NAMES`, gets a `_` in front. Any other
// name is returned as it is.
//
// The result is never freed by the caller: it is `name` itself, a constant, or a string in temporary storage that
// stays valid until the next `reset_temporary_storage`. Copy it to keep it.
avoid_reserved_names :: (name: string, avoid_type_names := false) -> string {
    if generator_name_begins(name, "operator") return replace_operator_name(name, true);
    for reserved: RESERVED_NAMES if name == reserved return tprint("_%", name);

    if avoid_type_names {
        for reserved: RESERVED_TYPE_NAMES if name == reserved return tprint("_%", name);
    }

    return name;
}

// Replaces every `-` and `.` in `_struct` with `_`, in place.
//
// The bytes of the string are modified, so it must not point at a literal.
make_valid_identifer :: (_struct: string) {
    for byte, index: _struct {
        if byte == #char "-" || byte == #char "." _struct.data[index] = #char "_";
    }
}

// Returns the C++ operator name `name` with its operator symbols spelled as words, such as `operator_equals`.
//
// Spaces are removed first. With `keep_operators_allowed_in_jai`, symbols that Jai accepts in an operator
// name (`==`, `+`, `[]` and similar) stay as they are; only the others (`->`, `++`, `,`, `()`) are replaced,
// and `operator!` and `operator=` become `operator_not` and `operator_assign`.
//
// The result is never freed by the caller: it is a constant or a string in temporary storage that stays valid
// until the next `reset_temporary_storage`. Copy it to keep it.
replace_operator_name :: (name: string, keep_operators_allowed_in_jai := false) -> string {
    return generator_name_replace_operator(name, keep_operators_allowed_in_jai);
}

// Shortens the Objective-C selector `s` in place to the part before its first `:`.
//
// A selector without `:` is left whole.
convert_objc_selector_name_to_function_identifier :: (s: *string) {
    s.count = generator_name_selector_stem_length(s.*);
}

// Appends `ident` to `builder` with every `:` written as `_`.
print_objc_selector_identifier :: (builder: *String_Builder, ident: string) {
    for byte: ident append(builder, ifx byte == #char ":" then #char "_" else byte);
}

// Returns the name of the enum value `name` of `_enum` with the configured prefixes removed.
//
// Returns an empty string for a name that ends with the options' `strip_enum_values_with_suffix`. Otherwise
// removes the prefix given for this enum in `strip_enum_value_prefixes`, and with `strip_enum_prefixes` the
// enum's own name and one `_` after it. Uses default options when none are active.
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

// Sets the output name of `_enum` and of each of its values that is not omitted.
//
// Removes the prefix shared by all the values, cut after its last `enum_prefix_delimiter`, when
// `auto_detect_enum_prefixes` is on, there are at least two values, `already_stripped` is false and no value
// name already starts with the enum name under `strip_enum_prefixes`. Leading `_` are dropped, reserved names
// are escaped, and a name that is empty or starts with a digit gets a `_` in front. An enum without an output
// name takes the shared prefix without trailing `_`, or `anon_enum` when that is empty. Uses default options
// when none are active.
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
