// A `#no_reset` global keeps what a plain `#run` stores in it, also when the build is a cross
// build: the program starts from the value the compile-time code left.
#import "Basic";

Table :: struct {
    values: [4] s32;
    label: string;
    first: *s32;
}

#no_reset table: Table;
#no_reset count := 1;
plain := 5;

fill :: () {
    for 0..3 table.values[it] = cast(s32) ((it + 1) * 10);

    table.label = copy_string("kept");
    table.first = *table.values[2];
    count = 99;
    plain = 77;
}

#run fill();

main :: () {
    assert(table.values[3] == 40);
    assert(table.label == "kept");
    assert(table.first.* == 30);
    assert(count == 99);
    assert(plain == 5);
    print("ok\n");
}
