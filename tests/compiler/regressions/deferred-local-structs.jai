#import "Basic";

probe :: () -> bool {
    Box :: struct(N: Later) {
        #insert #run make_field(N);
    }

    Later :: int;

    make_field :: (N: int) -> string {
        return tprint("values:[2]int; values[1]=%;", N + 41);
    }

    first: Box(1);
    second: Box(2);

    Plain :: struct {
        cells: [count] int;
        value: int = number;
    }

    count :: 3;
    number :: 17;
    plain: Plain;

    return first.values[1] == 42 && second.values[1] == 43 && plain.cells.count == 3 && plain.value == 17;
}

#assert probe();

main :: () {
    assert(probe());
}
