#import "Basic";
#import "Extensions/Toml";

// An error inside `[[x]]` names the element: `$["items"][1]["name"]`.
Point :: struct {
    x: int;
    y: int; @toml(default)
}

Item :: struct {
    name: string;
    points: [..] Point; @toml(default)
}

Config :: struct {
    items: [..] Item; @toml(default)
}

check :: (text: string, kind: Toml_Error_Kind, path: string) {
    config, error := from_toml(Config, text);

    assert(error.kind == kind, tprint("% gave %: %", text, error.kind, error.message));
    assert(toml_error_path(*error) == path, tprint("% path %", text, toml_error_path(*error)));
    free_toml_value(*config);
}

main :: () {
    check("[[items]]\nname = 'a'\n[[items]]\nname = 5\n", .TYPE, "$[\"items\"][1][\"name\"]");
    check("[[items]]\nname = 'a'\n[[items]]\nname = 'b'\nname = 'c'\n", .DUPLICATE, "$[\"items\"][1][\"name\"]");
    check("[[items]]\nname = 'a'\n[[items]]\n", .TYPE, "$[\"items\"][1][\"name\"]");
    check("[[items]]\nname = 'a'\n[[items]]\nname = 'b'\n[[items.points]]\nx = 1\n[[items.points]]\nx = 'z'\n", .TYPE, "$[\"items\"][1][\"points\"][1][\"x\"]");
    check("[[items]]\nname = 'a'\n[[items.points]]\ny = 1\n", .TYPE, "$[\"items\"][0][\"points\"][0][\"x\"]");
    check("[[items]]\nname = 'a'\n[[items]]\nname = 'b'\n[items.points]\nx = 1\n", .TYPE, "$[\"items\"][1][\"points\"]");

    // Keys the type does not know obey the same rules and name the same elements.
    check("[[items]]\nname = 'a'\n[[more]]\nk = 1\n[[more]]\nk = 1\nk = 2\n", .DUPLICATE, "$[\"more\"][1][\"k\"]");
    print("ok\n");
}
