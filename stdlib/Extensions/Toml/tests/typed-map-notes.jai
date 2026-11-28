#import "Basic";
#import "String";
#import "Hash_Table";
#import "Extensions/Toml";

// `@toml(range=, message=, as_number)` on a `Table(string, V)` member applies to every value.
Level :: enum u8 {
    LOW :: 1;
    HIGH :: 2;
}

Config :: struct {
    ports: Table(string, int); @toml(default, range=1..9, message="a port from 1 to 9")
    levels: Table(string, Level); @toml(default, as_number)
    nested: Table(string, Table(string, int)); @toml(default, range=0..3)
}

main :: () {
    value, error := from_toml(Config, "ports.a = 5\nlevels.x = 2\n[nested.n]\nk = 3\n");

    assert(error.kind == .NONE, error.message);
    assert(table_find_pointer(*value.ports, "a").* == 5);
    assert(table_find_pointer(*value.levels, "x").* == .HIGH);
    assert(table_find_pointer(*table_find_pointer(*value.nested, "n").*, "k").* == 3);

    text, write_error := to_toml(value);

    assert(write_error.kind == .NONE, write_error.message);
    assert(contains(text, "x = 2"), text);

    for source: string.["ports.a = 10", "ports = { a = 0 }", "[ports]\na = 99", "nested.n.k = 4", "[nested.n]\nk = -1"] {
        _, bad := from_toml(Config, source);

        assert(bad.kind == .RANGE, tprint("% gave %", source, bad.kind));
    }

    _, bad := from_toml(Config, "ports.a = 10");

    assert(bad.message == "a port from 1 to 9", bad.message);
    print("ok\n");
}
