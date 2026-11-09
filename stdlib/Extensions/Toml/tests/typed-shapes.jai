#import "Basic";
#import "Extensions/Toml";

// Fixed-size arrays take an inline array of exactly their length; a `[] T` view takes `[[x]]`
// headers like a `[..] T`.
Point :: struct {
    x: int;
    y: int; @toml(default)
}

Item :: struct {
    name: string;
    points: [] Point; @toml(default)
}

Shapes :: struct {
    rgb: [3] u8; @toml(default)
    names: [2] string; @toml(default)
    items: [] Item; @toml(default)
}

Stats :: struct {
    live: int;
}

count_live :: (mode: Allocator_Mode, size: s64, old_size: s64, memory: *void, data: *void) -> *void {
    stats := cast(*Stats) data;
    backing := context.default_allocator;

    if mode == .ALLOCATE || (mode == .RESIZE && !memory) stats.live += 1;
    if mode == .FREE && memory stats.live -= 1;

    return backing.proc(mode, size, old_size, memory, backing.data);
}

// Success or failure, nothing stays allocated once the value is freed.
check :: (text: string, kind: Toml_Error_Kind, path := "") {
    stats: Stats;

    {
        push_allocator(count_live, *stats);
        value, error := from_toml(Shapes, text);

        assert(error.kind == kind, tprint("% gave %: %", text, error.kind, error.message));
        if path.count assert(toml_error_path(*error) == path, tprint("% path %", text, toml_error_path(*error)));

        free_toml_value(*value);
    }

    assert(stats.live == 0, tprint("% leaked % allocations", text, stats.live));
}

main :: () {
    source := "rgb = [1, 2, 3]\nnames = ['a', 'b']\n\n[[items]]\nname = 'first'\n\n[[items.points]]\nx = 1\n[[items.points]]\nx = 2\ny = 3\n\n[[items]]\nname = 'second'\n\n[[items]]\nname = 'third'\n[[items.points]]\nx = 9\n";
    value, error := from_toml(Shapes, source);

    assert(error.kind == .NONE, tprint("%: %", error.line, error.message));
    assert(value.rgb[0] == 1 && value.rgb[1] == 2 && value.rgb[2] == 3);
    assert(value.names[0] == "a" && value.names[1] == "b");
    assert(value.items.count == 3 && value.items[0].name == "first" && value.items[2].name == "third");
    assert(value.items[0].points.count == 2 && value.items[0].points[1].y == 3);
    assert(value.items[1].points.count == 0 && value.items[2].points.count == 1 && value.items[2].points[0].x == 9);

    text, write_error := to_toml(value);

    assert(write_error.kind == .NONE, write_error.message);
    defer free(text);

    again, again_error := from_toml(Shapes, text);

    assert(again_error.kind == .NONE, tprint("%: %", again_error.message, text));
    assert(again.rgb[2] == 3 && again.names[1] == "b" && again.items.count == 3 && again.items[0].points[1].x == 2);
    free_toml_value(*again);
    free_toml_value(*value);

    check(source, .NONE);
    check("rgb = [1, 2]\n", .TYPE, "$[\"rgb\"]");
    check("rgb = [1, 2, 3, 4]\n", .TYPE, "$[\"rgb\"]");
    check("names = ['a', 'b', 'c']\n", .TYPE);
    check("names = ['a']\n", .TYPE);
    check("names = 'a'\n", .TYPE);
    check("[[items]]\nname = 'a'\n[[items]]\nname = 'b'\nname = 'c'\n", .DUPLICATE, "$[\"items\"][1][\"name\"]");
    check("[[items]]\nname = 'a'\n[[items]]\n", .TYPE, "$[\"items\"][1][\"name\"]");
    check("[[items]]\nname = 'a'\n[[items.points]]\nx = 1\n[[items.points]]\nx = 'z'\n", .TYPE, "$[\"items\"][0][\"points\"][1][\"x\"]");
    check("items = [{ name = 'a', points = [{ x = 1 }] }]\n", .NONE);
    check("[items]\nname = 'a'\n", .TYPE);
    print("ok\n");
}
