// The compile-time side: patterns written as constants are parsed and compiled while the program
// builds, and their named groups become typed fields. Everything here runs inside #run and
// #insert; what reaches the program is constant data and a few lines of field assignments.

// Bigger than this and the DFA is built lazily at run time instead of stored in the program.
STATIC_STATE_LIMIT :: 256;

// The engine for a constant pattern, built at compile time. A bad pattern is a compile error.
static_engine :: (pattern: string, full: bool) -> Engine {
    engine: Engine;
    engine.pattern = pattern;
    source := pattern;
    if full source = tprint("\\A(?:%)\\z", pattern);
    parsed := parse_pattern(source, .{});
    if !parsed.ok {
        offset := ifx full then max(parsed.offset - 5, 0) else parsed.offset;
        report_pattern_error(pattern, parsed.error, offset);
        return engine;
    }

    ok: bool;
    message: string;
    engine.program, ok, message = compile_program(*parsed, false);
    if ok engine.reverse, ok, message = compile_program(*parsed, true);

    if !ok {
        report_pattern_error(pattern, message, 0);
        return engine;
    }

    forward, backward: Dfa;

    dfa_init(*forward, *engine.program, false, STATIC_STATE_LIMIT);
    dfa_init(*backward, *engine.reverse, true, STATIC_STATE_LIMIT);

    if dfa_build_all(*forward) && dfa_build_all(*backward) {
        engine.forward = dfa_freeze(*forward);
        engine.backward = dfa_freeze(*backward);
        engine.frozen = true;
        // Freezing is optional: a pattern with no table just captures the slower way.
        engine.onepass = onepass_build(*engine.program, ONEPASS_STATIC_STATE_LIMIT);
    }

    engine.fixed_length = fixed_length(*engine.program);
    dfa_free(*forward);
    dfa_free(*backward);
    return engine;
}

// Static's check that its pattern parses; returns no code.
check_pattern :: (pattern: string) -> string {
    parsed := parse_pattern(pattern, .{});
    if !parsed.ok report_pattern_error(pattern, parsed.error, parsed.offset);
    return "";
}

// The #modify of every procedure that takes a Static type.
is_static :: (R: Type) -> bool, string {
    info := cast(*Type_Info) R;
    if info.type == .STRUCT && (cast(*Type_Info_Struct) info).polymorph_source_struct == type_info(Static) {
        return true, "";
    }

    return false, tprint("expected a Regex.Static(\"...\") type, not %; for a pattern made at run time, pass *re from Regex.compile", R);
}

report_pattern_error :: (pattern: string, message: string, offset: int) {
    pointer := tprint("%^", repeat_char(#char " ", offset));
    compiler_report(tprint("Regex: %\n    %\n    %", message, pattern, pointer));
}

repeat_char :: (c: u8, count: int) -> string {
    text := alloc_string(count,, temp);
    for 0..count - 1 text[it] = c;
    return text;
}

// What a named group's field holds.
Field_Kind :: enum u8 {
    STRING;
    INTEGER;
    FLOAT;
    ENUM;
}

Field_Plan :: struct {
    name: string;
    group: int;
    kind: Field_Kind;
    type_name: string; // u16, s64, float64, or the enum's name
    members: [..] string; // ENUM: the member names, in pattern order
    words: [..] string; // ENUM: each word of the alternation...
    word_members: [..] int; // ...and the member it becomes
    optional: bool; // may not take part in a match
}

// One plan per named group, in group order. ok is false if the pattern does not parse (the
// error is reported where the engine is built).
plan_groups :: (pattern: string) -> [..] Field_Plan, ok: bool {
    plans: [..] Field_Plan;
    parsed := parse_pattern(pattern, .{});

    if !parsed.ok return plans, false;
    collect_groups(parsed.root, false, *plans);
    used: [..] string;
    for plans array_add(*used, it.name);

    for *plans {
        if it.kind != .ENUM continue;
        name := copy_string(it.name);
        if name[0] >= #char "a" && name[0] <= #char "z" name[0] -= 32;
        for other: used if other == name name = tprint("%_Value", name);
        it.type_name = name;
        array_add(*used, name);
    }

    return plans, true;
}

collect_groups :: (node: *Node, optional: bool, plans: *[..] Field_Plan) {
    if node.kind == {
        case .CAPTURE;
            if node.name.count {
                plan: Field_Plan;
                plan.name = node.name;
                plan.group = node.index;
                plan.optional = optional;
                infer_kind(node.children[0], *plan);
                array_add(plans, plan);
            }

            collect_groups(node.children[0], optional, plans);
        case .REPEAT;
            collect_groups(node.children[0], optional || node.min == 0, plans);
        case .CONCAT;
            for node.children collect_groups(it, optional, plans);
        case .ALTERNATE;
            for node.children collect_groups(it, true, plans);
    }
}

JAI_KEYWORDS :: string.["if", "else", "then", "case", "for", "while", "break", "continue", "return", "defer", "using", "cast", "xx", "ifx", "struct", "union", "enum", "enum_flags", "inline", "no_inline", "true", "false", "null", "it", "it_index", "remove", "context", "push_context", "size_of", "type_of", "type_info", "is_constant", "initializer_of", "operator", "interface", "string", "bool", "int", "float", "float32", "float64", "void", "u8", "u16", "u32", "u64", "s8", "s16", "s32", "s64", "Type", "Code", "Any"];

infer_kind :: (node: *Node, plan: *Field_Plan) {
    plan.kind = .STRING;
    plan.type_name = "string";

    // A few fixed words: an enum, with each word written as a member name (`dog`, `Dog` and
    // `DOG` all become DOG; `hotDog` becomes HOT_DOG).
    if node.kind == .ALTERNATE && node.children.count >= 2 && node.children.count <= 64 {
        members, words: [..] string;
        word_members: [..] int;

        for node.children {
            word, ok := literal_text(it);
            if !ok || !is_identifier(word) return;
            name := member_name(word);
            index := index_of_name(members, name);

            if index < 0 {
                index = members.count;
                array_add(*members, name);
            }

            array_add(*words, word);
            array_add(*word_members, index);
        }

        if members.count < 2 return;
        plan.kind = .ENUM;
        plan.members = members;
        plan.words = words;
        plan.word_members = word_members;
        return;
    }

    signed, digits, is_float, ok := number_shape(node);
    if !ok return;

    if is_float {
        plan.kind = .FLOAT;
        plan.type_name = "float64";
        return;
    }

    plan.kind = .INTEGER;

    if signed {
        plan.type_name = ifx digits >= 0 && digits <= 2 then "s8" else ifx digits >= 0 && digits <= 4 then "s16" else ifx digits >= 0 && digits <= 9 then "s32" else "s64";
    } else {
        plan.type_name = ifx digits >= 0 && digits <= 2 then "u8" else ifx digits >= 0 && digits <= 4 then "u16" else ifx digits >= 0 && digits <= 9 then "u32" else "u64";
    }
}

// The letter a case-insensitive class stands for, written in lower case: (?i)y matches Y and y. Other
// characters in the class must be outside ASCII (like the Kelvin sign for k).
folded_letter :: (node: *Node) -> string, bool {
    letter: u8;
    seen := 0;

    for node.ranges {
        if it.lo > 0x7F continue;
        if it.hi > 0x7F return "", false;

        for c: it.lo..it.hi {
            lower := ifx c >= #char "A" && c <= #char "Z" then c + 32 else c;
            if lower < #char "a" || lower > #char "z" return "", false;
            if seen && lower != letter return "", false;
            letter = cast(u8) lower;
            seen += 1;
        }
    }

    if seen != 2 return "", false;
    text := alloc_string(1);
    text[0] = letter;
    return text, true;
}

// An identifier in upper snake case, the way enum members are written.
member_name :: (word: string) -> string {
    builder: String_Builder;

    for word {
        c := it;
        lower_before := it_index > 0 && ((word[it_index - 1] >= #char "a" && word[it_index - 1] <= #char "z") || (word[it_index - 1] >= #char "0" && word[it_index - 1] <= #char "9"));

        if c >= #char "A" && c <= #char "Z" && lower_before append(*builder, #char "_");
        if c >= #char "a" && c <= #char "z" c -= 32;
        append(*builder, c);
    }

    return builder_to_string(*builder);
}

index_of_name :: (array: [] string, value: string) -> int {
    for array if it == value return it_index;
    return -1;
}

// The text a node always matches exactly, if it is a plain string.
literal_text :: (node: *Node) -> string, bool {
    if node.kind == .LITERAL return node.literal, true;
    if node.kind == .CLASS return folded_letter(node);
    if node.kind == .CAPTURE return literal_text(node.children[0]);

    if node.kind == .CONCAT {
        builder: String_Builder;

        for node.children {
            piece, ok := literal_text(it);
            if !ok return "", false;
            append(*builder, piece);
        }

        return builder_to_string(*builder), true;
    }

    return "", false;
}

is_identifier :: (word: string) -> bool {
    if !word.count return false;

    for word {
        c := it;
        letter := (c >= #char "a" && c <= #char "z") || (c >= #char "A" && c <= #char "Z") || c == #char "_";
        if !letter && !(it_index > 0 && c >= #char "0" && c <= #char "9") return false;
    }

    return true;
}

// Whether a node only matches numbers: an optional sign, digits, and for a float a `.` and more
// digits. `digits` is the most integer digits it can match, -1 for no limit.
number_shape :: (node: *Node) -> signed: bool, digits: int, is_float: bool, ok: bool {
    items: [..] *Node;

    flatten(node, *items);
    i := 0;
    signed := false;

    if i < items.count && is_sign(items[i]) {
        signed = true;
        i += 1;
    }

    min_digits, max_digits := 0, 0;

    while i < items.count {
        ok, lo, hi := digit_run(items[i]);
        if !ok break;
        min_digits += lo;
        if hi < 0 || max_digits < 0 max_digits = -1;
        else max_digits += hi;
        i += 1;
    }

    if min_digits < 1 return false, 0, false, false;
    if i == items.count return signed, max_digits, false, true;
    // A fraction: `.` then digits, either inline or as an optional group.
    rest: [..] *Node;
    for i..items.count - 1 array_add(*rest, items[it]);

    if rest.count == 1 && rest[0].kind == .REPEAT && rest[0].min == 0 && rest[0].max == 1 {
        inner: [..] *Node;
        flatten(rest[0].children[0], *inner);
        rest = inner;
    }

    if rest.count < 2 return false, 0, false, false;
    dot, dot_ok := literal_text(rest[0]);
    if !dot_ok || dot != "." return false, 0, false, false;
    fraction := 0;

    for 1..rest.count - 1 {
        ok, lo := digit_run(rest[it]);
        if !ok return false, 0, false, false;
        fraction += lo;
    }

    if fraction < 1 return false, 0, false, false;
    return signed, max_digits, true, true;
}

flatten :: (node: *Node, items: *[..] *Node) {
    if node.kind == .CONCAT {
        for node.children flatten(it, items);
    } else if node.kind == .CAPTURE {
        flatten(node.children[0], items);
    } else if node.kind == .LITERAL && node.literal.count > 1 {
        // Each byte on its own, so "12" and "-" split the same way as classes do.
        for 0..node.literal.count - 1 {
            piece := New(Node);
            piece.kind = .LITERAL;
            piece.literal = slice(node.literal, it, 1);
            array_add(items, piece);
        }
    } else if node.kind != .EMPTY {
        array_add(items, node);
    }
}

is_sign :: (node: *Node) -> bool {
    if node.kind == .REPEAT && node.min == 0 && node.max == 1 return is_sign(node.children[0]);
    if node.kind == .CAPTURE return is_sign(node.children[0]);
    if node.kind == .LITERAL return node.literal == "-" || node.literal == "+";

    if node.kind == .CLASS {
        for node.ranges if it.lo != #char "-" && it.lo != #char "+" || it.hi != it.lo return false;
        return node.ranges.count > 0;
    }

    return false;
}

// Matches only decimal digits; how many (hi -1: no limit).
digit_run :: (node: *Node) -> ok: bool, lo: int, hi: int {
    if node.kind == {
        case .LITERAL;
            for node.literal if it < #char "0" || it > #char "9" return false, 0, 0;
            return true, node.literal.count, node.literal.count;
        case .CLASS;
            if !node.ranges.count return false, 0, 0;
            for node.ranges if !is_digit_range(it, node.bytes) return false, 0, 0;
            return true, 1, 1;
        case .CAPTURE;
            return digit_run(node.children[0]);
        case .REPEAT;
            ok, lo, hi := digit_run(node.children[0]);
            if !ok return false, 0, 0;
            if node.max < 0 || hi < 0 return true, lo * node.min, -1;
            return true, lo * node.min, hi * node.max;
        case .CONCAT;
            lo, hi := 0, 0;

            for node.children {
                ok, a, b := digit_run(it);
                if !ok return false, 0, 0;
                lo += a;
                if b < 0 || hi < 0 hi = -1;
                else hi += b;
            }

            return true, lo, hi;
        case .ALTERNATE;
            lo, hi := -1, 0;

            for node.children {
                ok, a, b := digit_run(it);
                if !ok return false, 0, 0;
                if lo < 0 || a < lo lo = a;
                if b < 0 || hi < 0 hi = -1;
                else hi = max(hi, b);
            }

            return true, lo, hi;
    }

    return false, 0, 0;
}

is_digit_range :: (r: Char_Range, bytes: bool) -> bool {
    if r.lo >= #char "0" && r.hi <= #char "9" return true;
    if bytes return false;
    for unicode_perl_digit() if r.lo >= it.lo && r.hi <= it.hi return true;
    return false;
}

// The members of Groups(pattern).
groups_members :: (pattern: string) -> string {
    plans, ok := plan_groups(pattern);
    if !ok return check_pattern(pattern);
    builder: String_Builder;
    any_optional := false;

    for plans {
        reserved := it.name == "match" || it.name == "has" || array_find(JAI_KEYWORDS, it.name);
        if reserved || !is_identifier(it.name) {
            compiler_report(tprint("Regex: group `%` in % can't be a field of Groups; rename it, or use captures_into with your own struct", it.name, pattern));
            return "";
        }

        if it.kind == .ENUM {
            print_to_builder(*builder, "% :: enum u8 {", it.type_name);
            for word: it.members print_to_builder(*builder, " %;", word);
            append(*builder, " }\n");
        }

        print_to_builder(*builder, "%: %;\n", it.name, it.type_name);
        if it.optional any_optional = true;
    }

    if any_optional {
        append(*builder, "has: struct {\n");
        for plans if it.optional print_to_builder(*builder, "    %: bool;\n", it.name);
        append(*builder, "};\n");
    }

    return builder_to_string(*builder);
}

// Statements that fill `out` (a *Groups(pattern)) from `text` and `slots`; `return false` when a
// number does not fit its field.
groups_fill :: (pattern: string) -> string {
    plans, ok := plan_groups(pattern);
    if !ok return "";
    builder: String_Builder;

    for plans {
        print_to_builder(*builder, "if slots[%] >= 0 {\n", 2 * it.group);
        print_to_builder(*builder, "    piece := slice(text, slots[%], slots[%] - slots[%]);\n", 2 * it.group, 2 * it.group + 1, 2 * it.group);
        if it.optional print_to_builder(*builder, "    out.has.% = true;\n", it.name);

        if it.kind == {
            case .STRING;
                print_to_builder(*builder, "    out.% = piece;\n", it.name);
            case .INTEGER;
                print_to_builder(*builder, "    value, fits := parse_integer(piece, %);\n    if !fits return false;\n    out.% = value;\n", it.type_name, it.name);
            case .FLOAT;
                print_to_builder(*builder, "    value, fits := parse_decimal(piece);\n    if !fits return false;\n    out.% = value;\n", it.name);
            case .ENUM;
                // The exact word first, then any case (for a case-insensitive pattern).
                plan := it;
                for word: plan.words {
                    print_to_builder(*builder, "    %if piece == \"%\" out.% = .%;\n", ifx it_index then "else " else "", word, plan.name, plan.members[plan.word_members[it_index]]);
                }
                for word: plan.words {
                    print_to_builder(*builder, "    else if equal_nocase(piece, \"%\") out.% = .%;\n", word, plan.name, plan.members[plan.word_members[it_index]]);
                }

                // Only a non-ASCII fold (the Kelvin sign for k) gets here.
                append(*builder, "    else return false;\n");
        }

        append(*builder, "}\n");
    }

    return builder_to_string(*builder);
}

// Statements that fill `out` (a *T the caller declared) from `text` and `slots`. Every field
// must have a named group and every named group a field; anything else is a compile error.
into_fill :: (pattern: string, T: Type) -> string {
    plans, ok := plan_groups(pattern);
    if !ok return "";
    info := cast(*Type_Info) T;
    if info.type != .STRUCT {
        compiler_report(tprint("Regex: can only match into a struct, not %", T));
        return "";
    }

    st := cast(*Type_Info_Struct) info;
    builder: String_Builder;
    seen: [..] string;

    for member: st.members {
        if member.flags & .CONSTANT continue;
        group := -1;
        for plans if it.name == member.name group = it.group;

        if group < 0 {
            compiler_report(tprint("Regex: field `%` of % has no group `(?<%>...)` in %", member.name, T, member.name, pattern));
            return "";
        }

        array_add(*seen, member.name);
        print_to_builder(*builder, "if slots[%] >= 0 {\n", 2 * group);
        print_to_builder(*builder, "    piece := slice(text, slots[%], slots[%] - slots[%]);\n", 2 * group, 2 * group + 1, 2 * group);
        type := member.type;
        if type.type == {
            case .STRING;
                print_to_builder(*builder, "    out.% = piece;\n", member.name);
            case .INTEGER;
                print_to_builder(*builder, "    value, fits := parse_integer(piece, type_of(out.%));\n    if !fits return false;\n    out.% = value;\n", member.name, member.name);
            case .FLOAT;
                print_to_builder(*builder, "    value, fits := parse_decimal(piece);\n    if !fits return false;\n    out.% = cast(type_of(out.%)) value;\n", member.name, member.name);
            case .BOOL;
                print_to_builder(*builder, "    if equal_nocase(piece, \"true\") out.% = true;\n    else if equal_nocase(piece, \"false\") out.% = false;\n    else return false;\n", member.name, member.name);
            case .ENUM;
                info_enum := cast(*Type_Info_Enum) type;
                // The exact name first, then any case.
                for info_enum.names print_to_builder(*builder, "    %if piece == \"%\" out.% = .%;\n", ifx it_index then "else " else "", it, member.name, it);
                for info_enum.names print_to_builder(*builder, "    else if equal_nocase(piece, \"%\") out.% = .%;\n", it, member.name, it);
                append(*builder, "    else return false;\n");
            case;
                compiler_report(tprint("Regex: field `%` of % has a type a group can't be parsed into; use a string, integer, float, bool or enum", member.name, T));
                return "";
        }

        append(*builder, "}\n");
    }

    for plans {
        found := false;
        for name: seen if name == it.name found = true;

        if !found {
            compiler_report(tprint("Regex: group `%` in % has no field in %", it.name, pattern, T));
            return "";
        }
    }

    return builder_to_string(*builder);
}

// Decimal digits (any script's) with an optional sign, into an integer of type T. False when it
// does not fit.
parse_integer :: (piece: string, $T: Type) -> T, bool {
    i := 0;
    negative := false;

    if i < piece.count && (piece[i] == #char "-" || piece[i] == #char "+") {
        negative = piece[i] == #char "-";
        i += 1;
    }

    magnitude: u64 = 0;
    LIMIT :: 0xFFFF_FFFF_FFFF_FFFF;

    while i < piece.count {
        digit, size := digit_at(piece, i);
        if digit < 0 return 0, false;
        if magnitude > (LIMIT - cast(u64) digit) / 10 return 0, false;
        magnitude = magnitude * 10 + cast(u64) digit;
        i += size;
    }

    #if #run type_info(T).signed {
        BITS :: size_of(T) * 8;
        MAX_POSITIVE :: (cast(u64) 1 << (BITS - 1)) - 1;

        if negative {
            if magnitude > MAX_POSITIVE + 1 return 0, false;
            if magnitude == MAX_POSITIVE + 1 return cast,no_check(T) (cast(u64) 1 << (BITS - 1)), true;
            return cast(T) -cast(s64) magnitude, true;
        }

        if magnitude > MAX_POSITIVE return 0, false;
        return cast(T) magnitude, true;
    } else {
        if negative && magnitude != 0 return 0, false;

        #if size_of(T) < 8 {
            if magnitude > (cast(u64) 1 << (size_of(T) * 8)) - 1 return 0, false;
        }

        return cast(T) magnitude, true;
    }
}

// A number like 12.5 or -3 (digits from any script) as a float64.
parse_decimal :: (piece: string) -> float64, bool {
    ascii: [..] u8;
    ascii.allocator = temp;
    i := 0;
    while i < piece.count {
        c := piece[i];
        if c == #char "-" || c == #char "+" || c == #char "." {
            array_add(*ascii, c);
            i += 1;
            continue;
        }

        digit, size := digit_at(piece, i);
        if digit < 0 return 0, false;
        array_add(*ascii, cast(u8) (#char "0" + digit));
        i += size;
    }

    text: string;
    text.data = ascii.data;
    text.count = ascii.count;
    value, ok, rest := string_to_float64(text);

    return value, ok && rest.count == 0;
}

// The value of the decimal digit at `index` and its size in bytes; -1 if it is not one.
digit_at :: (text: string, index: int) -> int, int {
    c := text[index];
    if c >= #char "0" && c <= #char "9" return cast(int) (c - #char "0"), 1;
    if c < 0x80 return -1, 1;
    code, size := decode_utf8(text, index);
    if size == 1 return -1, 1;

    if !digit_table_ready {
        digit_table = unicode_perl_digit();
        digit_table_ready = true;
    }

    // Decimal digits come in runs of ten, zero first.
    for digit_table if code >= it.lo && code <= it.hi return cast(int) ((code - it.lo) % 10), size;
    return -1, size;
}

digit_table: [..] Char_Range;
digit_table_ready := false;
