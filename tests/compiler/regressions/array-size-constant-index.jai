// An element of a constant array can be an array size.
#import "Basic";

SIZES :: int.[1, 4, 9];

Table :: struct {
    cells: [SIZES[2]] u8;
}

main :: () {
    a: [SIZES[1]] int;

    assert(a.count == 4);

    table: Table;

    assert(table.cells.count == 9);

    b: [SIZES[0] + SIZES[1]] s16;

    assert(b.count == 5);

    print("ok\n");
}
