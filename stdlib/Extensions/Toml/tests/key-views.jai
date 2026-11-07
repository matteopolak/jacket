#import "Basic";
#import "Extensions/Toml";
#load "../../test-support/counting-allocator.jai";
#load "../key-storage.jai";

MIXED :: "alpha = 1\n\"quoted\" = 2\n'lit' = 3\n\"esc\\u0061ped\" = 4\n\"\" = 5\n\"nul\\u0000key\" = 6\nvalue = 'payload'\n# beta\n";

main :: () {
    tally: Tally;
    doc: Toml_Document;

    {
        push_allocator(tally_proc, *tally);
        input := copy_string(MIXED);
        error: Toml_Error;
        doc, error = parse_toml(input);
        assert(error.kind == .NONE, error.message);
        for *input it.* = 0;

        free(input);
    }

    // The caller allocation is dead; these exact slices belong to the retained copy.
    assert(doc.root.members[0].key == "alpha");
    assert(doc.root.members[0].key.data == doc.source.data);
    assert(doc.root.members[1].key.data == doc.source.data + 11);
    assert(doc.root.members[2].key.data == doc.source.data + 24);
    assert(toml_get(doc.root, "escaped").integer == 4);
    assert(toml_get(doc.root, "").integer == 5);
    assert(toml_get(doc.root, "nul\0key").integer == 6);
    assert(toml_get(doc.root, "value").text == "payload");
    assert(!toml_key_uses_source(doc.source, toml_get(doc.root, "value").text));
    for i: 3..5 assert(!toml_key_uses_source(doc.source, doc.root.members[i].key));

    saved := doc.root.members[0].key;

    for 0..31 {
        value, error := toml_new_value(*doc, .INTEGER);
        assert(error.kind == .NONE);
        value.integer = it;
        assert(toml_insert(*doc, doc.root, tprint("added_%", it), value).kind == .NONE);
    }

    assert(doc.root.members[0].key.data == saved.data && saved == "alpha");
    beta := string.{ 4, doc.source.data + doc.source.count - 5 };

    assert(beta == "beta");
    value, _ := toml_new_value(*doc, .INTEGER);

    assert(toml_insert(*doc, doc.root, beta, value).kind == .NONE);
    assert(doc.root.members[doc.root.members.count - 1].key.data != beta.data);
    free_toml(*doc); // captured allocator after the ambient allocator changed
    free_toml(*doc);
    assert(tally.outstanding == 0);

    for limit: 1..2 {
        unicode, error := parse_toml("\"é\"=1", .{ max_string_bytes = limit });
        assert((error.kind == .NONE) == (limit == 2));
        free_toml(*unicode);
    }
}
