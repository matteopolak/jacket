// Stateful ownership APIs need lifecycle-aware hooks rather than generated unpaired calls.
toml_semantic_equal :: (a: *Target.Toml_Value, b: *Target.Toml_Value) -> bool {
    if a.kind != b.kind return false;

    if a.kind == .TABLE {
        if a.members.count != b.members.count return false;

        for a.members {
            other := Target.toml_get(b, it.key);
            if !other || !toml_semantic_equal(it.value, other) return false;
        }

        return true;
    }

    if a.kind == .ARRAY {
        if a.items.count != b.items.count return false;
        for a.items if !toml_semantic_equal(it, b.items[it_index]) return false;
        return true;
    }

    if a.kind == .STRING return a.text == b.text;
    if a.kind == .INTEGER return a.integer == b.integer;
    if a.kind == .BOOL return a.boolean == b.boolean;

    if a.kind == .FLOAT {
        x := cast,force(u64) a.real;
        y := cast,force(u64) b.real;
        if (x & 0x7fffffffffffffff) > 0x7ff0000000000000 return (y & 0x7fffffffffffffff) > 0x7ff0000000000000;
        return x == y;
    }

    x := a.temporal;
    y := b.temporal;

    return x.year == y.year && x.month == y.month && x.day == y.day && x.hour == y.hour && x.minute == y.minute && x.second == y.second && x.fraction == y.fraction && x.offset_minutes == y.offset_minutes && x.negative_zero_offset == y.negative_zero_offset;
}

// Not a fuzz_* name: this helper has document preconditions and is not a standalone fuzz case.
toml_round_trip :: (doc: Target.Toml_Document) {
    text, error := Target.write_toml(doc.root);

    assert(error.kind == .NONE, error.message);
    defer free(text);
    again, parse_error := Target.parse_toml(text);

    assert(parse_error.kind == .NONE, parse_error.message);
    defer Target.free_toml(*again);
    assert(toml_semantic_equal(doc.root, again.root), "TOML semantic round trip");
    text2, error2 := Target.write_toml(again.root);

    assert(error2.kind == .NONE && text2 == text, "TOML writer determinism");
    free(text2);
}

fuzz_toml_random_bytes :: (r: *Rng) {
    input := gen(r, string);
    doc, error := Target.parse_toml(input);

    defer Target.free_toml(*doc);
    if error.kind == .NONE toml_round_trip(doc);
    else assert(doc.root == null, "parse failure is atomic");
}

fuzz_toml_mutated_documents :: (r: *Rng) {
    seeds := string.[
        "[format]\nindent_width = 2\nignore = ['vendor/**']\n[doc.format]\nmax_width = 80\n",
        "[[records]]\nat = 2000-02-29T12:34:56.123456789012-00:00\nvalue = { key = [1, true, 'x'], }\n",
        "a.b.c = 0x7fff_ffff_ffff_ffff\ntext = \"\\u03bb\\e\\x7f\"\n",
    ];
    input := copy_string(seeds[gen(r, u8) % seeds.count]);

    defer free(input);
    edits := gen(r, u8) % 5;
    for 0..cast(int) edits - 1 {
        at := cast(int) (gen(r, u32) % cast(u32) input.count);
        input[at] = gen(r, u8);
    }

    input.count = cast(int) (gen(r, u32) % cast(u32) (input.count + 1));
    doc, error := Target.parse_toml(input);

    defer Target.free_toml(*doc);
    if error.kind == .NONE toml_round_trip(doc);
    else assert(doc.root == null, "mutated parse failure is atomic");
}

fuzz_toml_constructed_values :: (r: *Rng) {
    doc, error := Target.new_toml_document();

    assert(error.kind == .NONE);
    defer Target.free_toml(*doc);
    integer, integer_error := Target.toml_new_value(*doc, .INTEGER);

    assert(integer_error.kind == .NONE);
    integer.integer = gen(r, s64);
    assert(Target.toml_insert(*doc, doc.root, "integer", integer).kind == .NONE);
    real, real_error := Target.toml_new_value(*doc, .FLOAT);

    assert(real_error.kind == .NONE);
    bits := gen(r, u64);

    real.real = cast,force(float64) bits;
    assert(Target.toml_insert(*doc, doc.root, "float", real).kind == .NONE);
    text, text_error := Target.toml_new_value(*doc, .STRING);

    assert(text_error.kind == .NONE);
    text_error = Target.toml_set_text(*doc, text, gen(r, string));
    if text_error.kind == .ENCODING return;

    assert(text_error.kind == .NONE);
    assert(Target.toml_insert(*doc, doc.root, "string", text).kind == .NONE);
    toml_round_trip(doc);
}
