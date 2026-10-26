// Variants in JSON: how Extensions/Variant's tagged unions are encoded and decoded, and the small
// run-time helpers the generated decoders call. The forms, as serde calls them:
//
//     externally tagged   {"Circle": {"r": 1}}                  the default, unless every case is a scalar
//     internally tagged   {"kind": "Circle", "r": 1}            @json(tag=kind), every case a struct
//     adjacently tagged   {"t": "Circle", "c": {"r": 1}}        @json(tag=t, content=c)
//     untagged            {"r": 1}                              @json(untagged); the default for scalar cases
//
// `@json(tags=circle,rect)` names the tags instead of using the case types' names.

#scope_module

Variant_Lib :: #import "Extensions/Variant";

// Where the temporaries inside the attempt procedures of an untagged variant are numbered from, apart
// from the numbering of the procedure around them.
ATTEMPT_IDS :: 1000;
ATTEMPT_IDS_PER_CASE :: 50;

Variant_Mode :: enum u8 {
    EXTERNAL;
    INTERNAL;
    ADJACENT;
    UNTAGGED;
}

is_variant :: (info: *Type_Info) -> bool {
    return Variant_Lib.is_variant_type(info);
}

variant_cases :: (info: *Type_Info) -> [] *Type_Info {
    return Variant_Lib.variant_case_infos(info);
}

// The name a case has by default as a tag: the type as written. A variant inside a variant is
// spelled out from its own cases.
case_name :: (info: *Type_Info) -> string {
    if !is_variant(info) return tprint("%", cast(Type) info);

    builder: String_Builder;

    append(*builder, "Variant(");

    for variant_cases(info) {
        if it_index append(*builder, ", ");
        append(*builder, case_name(it));
    }

    append(*builder, ")");

    return builder_to_string(*builder,, temp);
}

// Whether a case is written as a bare number, string or bool, so that a variant of such cases can
// do without tags.
is_scalar_case :: (info: *Type_Info) -> bool {
    return info.type == .INTEGER || info.type == .FLOAT || info.type == .BOOL || info.type == .STRING || info.type == .ENUM;
}

// A struct that is written as an object of its fields (not a Json_Value, Hash_Table, Json_Maybe
// or Variant), which an internal tag can be added to.
is_plain_struct :: (info: *Type_Info) -> bool {
    return info.type == .STRUCT && !is_json_value(info) && !is_table(info) && !is_maybe(info) && !is_variant(info);
}

// Whether a field's type has a Variant in it that a tagging note would reach: the type itself, or
// the element of an array, pointer, Json_Maybe or Hash_Table.
reaches_variant :: (info: *Type_Info) -> bool {
    if info.type == {
        case .ARRAY;
            return reaches_variant((cast(*Type_Info_Array) info).element_type);
        case .POINTER;
            return reaches_variant((cast(*Type_Info_Pointer) info).pointer_to);
        case .STRUCT;
            if is_variant(info) return true;
            if is_maybe(info) return reaches_variant(maybe_value(info));

            if is_table(info) {
                _, element := table_entry(info);

                return reaches_variant(element);
            }
    }

    return false;
}

// The notes on a field that say how a variant is tagged must agree with each other and name a
// field that has a variant.
check_variant_notes :: (plan: *Plan, member: *Type_Info_Struct_Member, style: Enum_Style) -> bool {
    named := style.tag.count > 0 || style.content.count > 0 || style.untagged || style.tags.count > 0;

    if !named return true;

    if !reaches_variant(member.type) {
        complain(plan.type_name, member.name, "`tag=`, `content=`, `untagged` and `tags=` are for a field that holds a Variant");

        return false;
    }

    if style.untagged && (style.tag.count > 0 || style.content.count > 0) {
        complain(plan.type_name, member.name, "`untagged` cannot be combined with `tag=` or `content=`");

        return false;
    }

    if style.content.count > 0 && style.tag.count == 0 {
        complain(plan.type_name, member.name, "`content=` needs `tag=`, as in `@json(tag=t,content=c)`");

        return false;
    }

    if style.content.count > 0 && style.content == style.tag {
        complain(plan.type_name, member.name, "`tag=` and `content=` must be different keys");

        return false;
    }

    return true;
}

// A struct that gets an internal tag must not use the tag's key itself and must be written as an
// object.
check_tagged_plan :: (plan: *Plan, tag_key: string) -> bool {
    ok := true;

    if plan.transparent {
        complain(plan.type_name, "", tprint("a `transparent` struct cannot be a case of a Variant tagged with `tag=%`", tag_key));

        return false;
    }

    for plan.leaves {
        clash := it.key == tag_key;

        for alias: it.aliases if alias == tag_key clash = true;

        if clash && !(it.skip_serializing && it.skip_deserializing) {
            complain(plan.type_name, it.member, tprint("the key `%` is also the tag of the Variant it is a case of", tag_key));
            ok = false;
        }
    }

    return ok;
}

// The style a case's payload is written in: its enums follow the field's rule, but the tagging
// notes were about the variant and do not go further down.
payload_style :: (style: Enum_Style) -> Enum_Style {
    inner := style;

    inner.tag = "";
    inner.content = "";
    inner.untagged = false;
    inner.tags = .[];

    return inner;
}

variant_mode :: (g: *Gen, cases: [] *Type_Info, style: Enum_Style) -> Variant_Mode {
    if style.untagged return .UNTAGGED;
    if style.tag.count && style.content.count return .ADJACENT;

    if style.tag.count {
        for cases {
            if !is_plain_struct(it) {
                fail(g, tprint("`tag=%` needs every case to be a struct, and `%` is %. Use `tag=%,content=c` to tag a case that is not a struct", style.tag, case_name(it), type_label(it), style.tag));

                break;
            }
        }

        return .INTERNAL;
    }

    for cases if !is_scalar_case(it) return .EXTERNAL;

    return .UNTAGGED;
}

variant_tags :: (g: *Gen, cases: [] *Type_Info, style: Enum_Style) -> [..] string {
    tags: [..] string;

    if style.tags.count {
        if style.tags.count != cases.count {
            fail(g, tprint("`tags=` must give one tag per case: it has % and the Variant has % cases", style.tags.count, cases.count));

            return tags;
        }

        for style.tags array_add(*tags, it);
    } else {
        for cases array_add(*tags, apply_rule(case_name(it), style.variant_rule));
    }

    for tag, i: tags {
        for j: i + 1..tags.count - 1 {
            if tags[j] == tag {
                fail(g, tprint("the cases `%` and `%` are both tagged `%`. Give them `tags=` of their own", case_name(cases[i]), case_name(cases[j]), tag));

                return tags;
            }
        }
    }

    return tags;
}

// `a, b, c`, for the message of an unknown tag.
tag_list :: (tags: [] string) -> string {
    builder: String_Builder;

    for tags {
        if it_index append(*builder, ", ");
        append(*builder, it);
    }

    return builder_to_string(*builder,, temp);
}

// The pointer to case `k` of the variant `value` as a declaration: `name := cast(*Case) ...;`.
case_pointer :: (g: *Gen, name: string, value: string, k: int) -> string {
    return tprint("% := cast(*type_of(%).case_type_%) %.payload.data;\n", name, value, k, value);
}

// ---- encoding ----

emit_key :: (g: *Gen, first: string, key: string) {
    quoted: String_Builder;

    append_json_string(*quoted, key);
    quoted_key := builder_to_string(*quoted,, temp);

    print(*g.out, "json_key_fixed(w, *%, %, %, %);\n", first, jai_quote(quoted_key), jai_quote(tprint("%:", quoted_key)), jai_quote(tprint(",%:", quoted_key)));
}

emit_encode_variant :: (g: *Gen, value: string, info: *Type_Info, style: Enum_Style) {
    out := *g.out;
    cases := variant_cases(info);
    mode := variant_mode(g, cases, style);
    tags := variant_tags(g, cases, style);

    if !g.ok return;

    id := next_id(g);
    inner := payload_style(style);

    print(out, "{\n");

    for case_info, k: cases {
        pointer := tprint("case_%_%", id, k);
        first := tprint("first_%_%", id, k);

        print(out, "%if %.index == % {\n", ifx k then "else " else "", value, k);
        append(out, case_pointer(g, pointer, value, k));

        if mode == {
            case .UNTAGGED;
                emit_encode(g, tprint("%.*", pointer), case_info, inner);
            case .EXTERNAL;
                print(out, "json_begin_object(w);\nfirst_%_% := true;\n", id, k);
                emit_key(g, first, tags[k]);
                emit_encode(g, tprint("%.*", pointer), case_info, inner);
                print(out, "json_end_object(w, %);\n", first);
            case .ADJACENT;
                print(out, "json_begin_object(w);\nfirst_%_% := true;\n", id, k);
                emit_key(g, first, style.tag);
                print(out, "json_write_raw(w, %);\n", json_literal(tags[k]));
                emit_key(g, first, style.content);
                emit_encode(g, tprint("%.*", pointer), case_info, inner);
                print(out, "json_end_object(w, %);\n", first);
            case .INTERNAL;
                print(out, "json_encode_tagged(w, %, %, %);\n", pointer, jai_quote(style.tag), jai_quote(tags[k]));
        }

        print(out, "}\n");
    }

    print(out, "else json_write_null(w);\n}\n");
}

// ---- freeing ----

emit_free_variant :: (g: *Gen, out: *String_Builder, target: string, info: *Type_Info, default: string) {
    cases := variant_cases(info);
    owning := false;

    for cases if owns_memory(it) owning = true;

    // A variant that still holds its declared default is not the decoder's to free.
    if default.count print(out, "if !json_variant_unchanged(%, %) {\n", address_of(target), address_of(default));
    else print(out, "{\n");

    if owning {
        id := next_id(g);

        for case_info, k: cases {
            if !owns_memory(case_info) continue;

            pointer := tprint("case_%_%", id, k);

            print(out, "if %.index == % {\n", target, k);
            append(out, case_pointer(g, pointer, target, k));
            emit_free(g, out, tprint("%.*", pointer), case_info, "");
            print(out, "}\n");
        }
    }

    print(out, "%.index = 0;\nmemset(%.payload.data, 0, %.payload.count);\n}\n", target, target, target);
}

// ---- decoding ----

// Reads the tag from the object ahead and goes back to its start: `which_N` is the case, or the
// statements in `on_fail` ran. Needs the object's keys to come in any order, so it looks before it
// reads.
emit_find_tag :: (g: *Gen, id: int, tag_key: string, tags: [] string, on_fail: string) {
    out := *g.out;

    print(out, "mark_% := json_mark(r);\n", id);
    print(out, "tag_%, found_%, ok_% := json_find_tag(r, %);\nif !ok_% { % }\n", id, id, id, jai_quote(tag_key), id, on_fail);
    print(out, "if !found_% { json_missing_field(r, %); % }\n", id, jai_quote(tag_key), on_fail);
    print(out, "which_% := -1;\n", id);

    for tags {
        print(out, "%if tag_% == % { which_% = %; }\n", ifx it_index then "else " else "", id, jai_quote(it), id, it_index);
    }

    print(out, "if which_% < 0 { json_fail_at(r, .UNKNOWN_VARIANT, tprint(\"unknown variant `%`, expected one of: %\", tag_%), r.key_start); % }\n", id, "%", tag_list(tags), id, on_fail);
    print(out, "json_rewind(r, mark_%);\n", id);
}

// Starts case `k` from its declared defaults and decodes it in place; on success the variant holds
// it.
emit_decode_case :: (g: *Gen, value: string, id: int, k: int, case_info: *Type_Info, style: Enum_Style, on_fail: string) {
    out := *g.out;
    slot := tprint("slot_%_%", id, k);

    print(out, "{\n");
    append(out, case_pointer(g, slot, value, k));
    print(out, "init_%_%: type_of(%.*);\n%.* = init_%_%;\n", id, k, slot, slot, id, k);
    emit_decode(g, tprint("%.*", slot), case_info, style, on_fail);
    print(out, "%.index = %;\n}\n", value, k);
}

emit_decode_variant :: (g: *Gen, target: string, info: *Type_Info, style: Enum_Style, on_fail: string) {
    out := *g.out;
    cases := variant_cases(info);
    mode := variant_mode(g, cases, style);
    tags := variant_tags(g, cases, style);

    if !g.ok return;

    id := next_id(g);
    inner := payload_style(style);

    // Whatever the variant held before (its declared default) is replaced, not released.
    print(out, "{\n%.index = 0;\nmemset(%.payload.data, 0, %.payload.count);\n", target, target, target);

    // What a failure after a case was read releases.
    cleanup: String_Builder;

    emit_free_variant(g, *cleanup, target, info, "");
    release := builder_to_string(*cleanup,, temp);
    fail := tprint("% %", release, on_fail);

    if mode == {
        case .UNTAGGED;
            emit_decode_untagged(g, target, id, cases, inner, on_fail);
        case .EXTERNAL;
            print(out, "first_% := true;\nif !json_begin_object(r) { % }\n", id, on_fail);
            print(out, "key_%, status_% := json_object_next(r, *first_%);\nif status_% == .ERROR { % }\n", id, id, id, id, on_fail);
            print(out, "if status_% == .END { json_fail(r, .TYPE, \"expected an object with one key naming the variant\"); % }\n", id, on_fail);
            print(out, "which_% := -1;\n", id);

            for tags {
                print(out, "%if key_% == % { which_% = %; }\n", ifx it_index then "else " else "", id, jai_quote(it), id, it_index);
            }

            print(out, "if which_% < 0 { json_fail_at(r, .UNKNOWN_VARIANT, tprint(\"unknown variant `%`, expected one of: %\", key_%), r.key_start); % }\n", id, "%", tag_list(tags), id, on_fail);

            for case_info, k: cases {
                print(out, "if which_% == % {\n", id, k);
                emit_decode_case(g, target, id, k, case_info, inner, tprint("json_path_key(r, %); %", jai_quote(tags[k]), on_fail));
                print(out, "}\n");
            }

            print(out, "_, status2_% := json_object_next(r, *first_%);\nif status2_% == .ERROR { % }\n", id, id, id, fail);
            print(out, "if status2_% != .END { json_fail(r, .TYPE, \"expected an object with exactly one key naming the variant\"); % }\n", id, fail);
        case .INTERNAL;
            emit_find_tag(g, id, style.tag, tags, on_fail);

            for case_info, k: cases {
                slot := tprint("slot_%_%", id, k);

                print(out, "if which_% == % {\n", id, k);
                append(out, case_pointer(g, slot, target, k));
                print(out, "init_%_%: type_of(%.*);\n%.* = init_%_%;\n", id, k, slot, slot, id, k);
                print(out, "if !json_decode_tagged(r, %, %) { % }\n%.index = %;\n}\n", slot, jai_quote(style.tag), on_fail, target, k);
            }
        case .ADJACENT;
            emit_find_tag(g, id, style.tag, tags, on_fail);

            print(out, "if !json_begin_object(r) { % }\nfirst_% := true;\nread_% := false;\n", on_fail, id, id);
            print(out, "while true {\nkey_%, status_% := json_object_next(r, *first_%);\nif status_% == .ERROR { % }\nif status_% == .END break;\n", id, id, id, id, fail, id);
            print(out, "if % { if !json_skip_value(r) { % } }\n", key_match(style.tag, tprint("key_%", id)), fail);
            print(out, "else if % {\n", key_match(style.content, tprint("key_%", id)));
            print(out, "if read_% { if !r.duplicate_last { json_duplicate_field(r, key_%); % }\n", id, id, fail);
            append(out, release);
            print(out, "}\nread_% = true;\n", id);

            for case_info, k: cases {
                print(out, "if which_% == % {\n", id, k);
                emit_decode_case(g, target, id, k, case_info, inner, tprint("json_path_key(r, %); %", jai_quote(style.content), fail));
                print(out, "}\n");
            }

            print(out, "} else if r.deny_unknown { json_unknown_field(r, key_%); % }\n", id, fail);
            print(out, "else if !json_skip_value(r) { % }\n}\n", fail);
            print(out, "if !read_% { json_missing_field(r, %); % }\n", id, jai_quote(style.content), fail);
    }

    print(out, "}\n");
}

// Tries each case in order on the same text: a case that fails leaves nothing behind (its decoder
// freed what it allocated), the reader goes back, and the next case gets its turn. Each attempt is
// a procedure of its own so that its failures can simply return.
emit_decode_untagged :: (g: *Gen, target: string, id: int, cases: [] *Type_Info, style: Enum_Style, on_fail: string) {
    out := *g.out;
    names: String_Builder;

    for cases {
        if it_index append(*names, ", ");
        append(*names, case_name(it));
    }

    name := tprint("Variant(%)", builder_to_string(*names,, temp));

    print(out, "mark_% := json_mark(r);\nwhy_%: String_Builder;\nwhy_%.allocator = temp;\ndone_% := false;\n", id, id, id, id);

    for case_info, k: cases {
        print(out, "CT_%_% :: type_of(%).case_type_%;\n", id, k, target, k);
        print(out, "attempt_%_% :: (r: *Json_Reader, out: *CT_%_%) -> bool {\n", id, k, id, k);

        // The attempt is a procedure of its own; its ids may not collide with the enclosing ones.
        inside: Gen;
        inside.uid = id * ATTEMPT_IDS + k * ATTEMPT_IDS_PER_CASE;
        inside.type_name = g.type_name;
        inside.field = g.field;

        emit_decode(*inside, "out.*", case_info, style, "return false;");

        if !inside.ok g.ok = false;

        append(out, builder_to_string(*inside.out,, temp));
        print(out, "return true;\n}\n");

        slot := tprint("slot_%_%", id, k);

        print(out, "if !done_% {\n", id);
        print(out, "memset(%.payload.data, 0, %.payload.count);\n", target, target);
        append(out, case_pointer(g, slot, target, k));
        print(out, "init_%_%: CT_%_%;\n%.* = init_%_%;\n", id, k, id, k, slot, id, k);
        print(out, "if attempt_%_%(r, %) {\n%.index = %;\ndone_% = true;\n} else json_note_attempt(r, *why_%, %, mark_%);\n}\n", id, k, slot, target, k, id, id, jai_quote(case_name(case_info)), id);
    }

    print(out, "if !done_% {\njson_fail_at(r, .TYPE, tprint(\"no variant of `%` matched: %\", builder_to_string(*why_%,, temp)), mark_%.pos);\n%\n}\n", id, name, "%", id, id, on_fail);
}

// ---- run time ----

// A place in the document, to come back to.
Json_Mark :: struct {
    pos: int;
    ip: int;
    depth: int;
    key_start: int;
    path: int;
}

json_mark :: (r: *Json_Reader) -> Json_Mark {
    return .{ pos = r.pos, ip = r.ip, depth = r.depth, key_start = r.key_start, path = r.path.count };
}

json_rewind :: (r: *Json_Reader, mark: Json_Mark) {
    r.pos = mark.pos;
    r.ip = mark.ip;
    r.depth = mark.depth;
    r.key_start = mark.key_start;
    r.path.count = mark.path;
}

// Looks in the object at the reader for the member `key`, which must be a string, and returns it
// (valid until the next read). The reader is left wherever the search ended: rewind it. `found` is
// false when the object has no such member, and `ok` false after an error was recorded.
json_find_tag :: (r: *Json_Reader, key: string) -> tag: string, found: bool, ok: bool {
    if !json_begin_object(r) return "", false, false;

    first := true;

    while true {
        name, status := json_object_next(r, *first);

        if status == .ERROR return "", false, false;
        if status == .END return "", false, true;

        if name == key {
            tag, ok := json_read_name(r);

            return tag, ok, ok;
        }

        if !json_skip_value(r) return "", false, false;
    }

    return "", false, false;
}

// An attempt at one case of an untagged variant failed: note why in `why`, forget the error and
// go back to where the attempt began.
json_note_attempt :: (r: *Json_Reader, why: *String_Builder, name: string, mark: Json_Mark) {
    error := finish_error(r);

    if why.initialized && builder_string_length(why) > 0 append(why, "; ");

    print(why, "`%`: %", name, error.message);
    if error.path.count print(why, " (at %)", error.path);

    free(error.path);
    free(r.error.message);

    r.error = .{};
    json_rewind(r, mark);
}

// Whether two variants of the same type hold the same bytes.
json_variant_unchanged :: (a: *$V, b: *V) -> bool {
    if a.index != b.index return false;

    return memcmp(a.payload.data, b.payload.data, a.payload.count) == 0;
}
