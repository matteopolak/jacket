#scope_module

generator_name_selector_stem_length :: (selector: string) -> int {
    for byte, index: selector {
        if byte == #char ":" return index;
    }

    return selector.count;
}

generator_name_drop :: (name: string, count: int) -> string {
    assert(count >= 0 && count <= name.count);
    if count == 0 return name;

    result := name;
    result.data += count;
    result.count -= count;

    return result;
}

generator_name_begins :: (name: string, prefix: string) -> bool {
    if prefix.count > name.count return false;
    for byte, index: prefix if name.data[index] != byte return false;
    return true;
}

generator_name_ends :: (name: string, suffix: string) -> bool {
    if suffix.count > name.count return false;
    return generator_name_begins(generator_name_drop(name, name.count - suffix.count), suffix);
}

generator_name_common_length :: (a: string, b: string) -> int {
    count := 0;
    while count < a.count && count < b.count && a.data[count] == b.data[count] count += 1;
    return count;
}

generator_name_delimited_length :: (prefix: string, delimiter: string) -> int {
    last := 0;

    for index: 0..prefix.count - 1 {
        if generator_name_begins(generator_name_drop(prefix, index), delimiter) last = index + delimiter.count;
    }

    return last;
}

Generator_Name_Replacement :: struct {
    source: string;
    replacement: string;
}

GENERATOR_NAME_ALWAYS :: Generator_Name_Replacement.[
    .{ "<=>", "_wtf" }, .{ "->*", "_wtf2" }, .{ "->", "_arrow" },
    .{ "++", "_plus_plus" }, .{ "--", "_minus_minus" },
    .{ ",", "_comma" }, .{ "()", "_parens" }, .{ "\"\"", "_literal" }
];

GENERATOR_NAME_ORDINARY :: Generator_Name_Replacement.[
    .{ "==", "_equals" }, .{ "!=", "_not_equals" }, .{ ">=", "_greater_equals" },
    .{ "<=", "_less_equals" }, .{ "<<", "_lshift" }, .{ ">>", "_rshift" },
    .{ "&&", "_and_and" }, .{ "||", "_or_or" },
    .{ "+", "_plus" }, .{ "-", "_minus" }, .{ "*", "_times" },
    .{ "/", "_divide" }, .{ "%", "_modulo" }, .{ "^", "_xor" },
    .{ "&", "_and" }, .{ "|", "_or" }, .{ "~", "_tilde" },
    .{ "<", "_less" }, .{ ">", "_greater" }, .{ "[]", "_brackets" },
    .{ "!", "_not" }, .{ "=", "_assign" }
];

generator_name_operator_fragment :: (name: string, keep: bool) -> string, int {
    for entry: GENERATOR_NAME_ALWAYS {
        if generator_name_begins(name, entry.source) return entry.replacement, entry.source.count;
    }

    if !keep {
        for entry: GENERATOR_NAME_ORDINARY {
            if generator_name_begins(name, entry.source) return entry.replacement, entry.source.count;
        }
    }

    return "", 0;
}

generator_name_replace_operator :: (name: string, keep: bool) -> string {
    compact_builder: String_Builder;

    defer free_buffers(*compact_builder);
    for byte: name if byte != #char " " append(*compact_builder, byte);

    compact := builder_to_string(*compact_builder);

    defer free(compact);
    if keep && compact == "operator!" return "operator_not";
    if keep && compact == "operator=" return "operator_assign";

    builder: String_Builder;

    defer free_buffers(*builder);
    index := 0;
    while index < compact.count {
        fragment, consumed := generator_name_operator_fragment(generator_name_drop(compact, index), keep);
        if consumed > 0 {
            append(*builder, fragment);
            index += consumed;
        } else {
            append(*builder, compact.data[index]);
            index += 1;
        }
    }

    return builder_to_string(*builder);
}

#scope_file
#import "Basic";
