// Values a `#run` computes reach the program unchanged: numbers, strings, arrays, enums and
// structs holding them. When this file is compiled for another OS or CPU than the compiler's
// (`jaic -check -os linux`), the `#run` code is compiled for the machine running the compiler
// and the results are matched with the program's own declarations by name and layout.
#import "Basic";

Level :: enum u8 {
    LOW;
    MID;
    HIGH;
}

Entry :: struct {
    label: string;
    level: Level;
    weight: float64;
    tags: [] string;
}

Table :: struct {
    entries: [] Entry;
    total: s64;
    origin: *u8;
}

build_table :: () -> Table {
    table: Table;
    entries := NewArray(3, Entry);

    for *entry, index: entries {
        entry.label = tprint("entry-%", index);
        entry.level = cast(Level) index;
        entry.weight = 0.5 * cast(float64) (index + 1);
        tags := NewArray(index + 1, string);
        for *tag, tag_index: tags tag.* = tprint("t%", tag_index);

        entry.tags = tags;
        table.total += index * 10;
    }

    table.entries = entries;

    return table;
}

host_sees_itself :: () -> bool {
    return OS == BUILD_OS && CPU == BUILD_CPU && !IS_CROSS_COMPILING;
}

TABLE :: #run build_table();
SEES_ITSELF :: #run host_sees_itself();
SQUARES :: #run squares();

squares :: () -> [4] s32 {
    result: [4] s32;
    for 0..3 result[it] = cast(s32) (it * it);
    return result;
}

Operation :: struct {
    name: string;
    apply: (s64) -> s64;
}

double :: (value: s64) -> s64 {
    return value * 2;
}

pick_operation :: () -> Operation {
    return .{ "double", double };
}

OPERATION :: #run pick_operation();

#assert SEES_ITSELF;
#assert TABLE.entries.count == 3;
#assert TABLE.total == 30;

main :: () {
    assert(TABLE.entries[2].label == "entry-2");
    assert(TABLE.entries[2].level == .HIGH);
    assert(TABLE.entries[1].weight == 1.0);
    assert(TABLE.entries[2].tags.count == 3);
    assert(TABLE.entries[2].tags[2] == "t2");
    assert(SQUARES[3] == 9);
    assert(OPERATION.name == "double");
    assert(OPERATION.apply(21) == 42);
    print("ok\n");
}
