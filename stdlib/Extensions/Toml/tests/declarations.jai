#import "Basic";
#import "String";
#import "Extensions/Toml";

main :: () {
    valid := string.[
        "[a.b.c]\nx=1\n[a]\nb.y=2\n",
        "[a.b.c]\nx=1\n[a]\n[a.b]\nz=3\n",
        "[[items]]\n[items.a.b.c]\nx=1\n[items.a]\nb.y=2\n[[items]]\n[items.a.b]\nz=3\n",
    ];
    for valid {
        doc, error := parse_toml(it);
        assert(error.kind == .NONE, error.message);
        free_toml(*doc);
    }

    invalid := string.[
        "[a.b.c]\nx=1\n[a]\nb.y=2\n[a.b]\nz=3\n",
        "[[items]]\n[items.a.b.c]\nx=1\n[items.a]\nb.y=2\n[items.a.b]\nz=3\n",
    ];
    for invalid {
        doc, error := parse_toml(it);
        assert(error.kind == .DUPLICATE, it);
        assert(error.related.start == find_index_from_left(it, "b.y"));
        free_toml(*doc);
    }

    _, duplicate := parse_toml("[a.b]\n[a]\n[a]\n");

    assert(duplicate.kind == .DUPLICATE && duplicate.related.start == 6);
    print("ok\n");
}
