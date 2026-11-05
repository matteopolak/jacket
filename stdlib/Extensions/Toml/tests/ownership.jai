#import "Basic";
#import "String";
#import "Extensions/Toml";
#load "../../test-support/counting-allocator.jai";

DOCUMENT :: "title = 'tool'\n[[records]]\nname = \"a\\nb\"\nvalues = [1, {x = 2}]\nat = 2000-02-29T12:34:56.123456789012Z\n";

main :: () {
    for 0..DOCUMENT.count {
        tally: Tally;

        {
            push_allocator(tally_proc, *tally);
            doc := parse_toml(slice(DOCUMENT, 0, it));
            free_toml(*doc);
        }

        assert(tally.outstanding == 0, tprint("leak at cut %", it));
    }

    for limit: 0..2048 {
        tally: Tally;

        {
            push_allocator(tally_proc, *tally);
            limited := parse_toml("", .{ max_retained_bytes = limit });
            free_toml(*limited);
        }

        assert(tally.outstanding == 0, tprint("budget rollback leak at %", limit));
    }

    tally: Tally;
    doc: Toml_Document;

    {
        push_allocator(tally_proc, *tally);
        input := copy_string(DOCUMENT);
        error: Toml_Error;
        doc, error = parse_toml(input);
        assert(error.kind == .NONE, error.message);
        free(input);
    }

    assert(toml_get(doc.root, "title").text == "tool");
    free_toml(*doc); // uses captured allocator after context has changed
    assert(tally.outstanding == 0);
    print("ok\n");
}
