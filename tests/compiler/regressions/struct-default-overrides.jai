#import "Basic";

Cell :: struct {
    text: string = "old";
    value: int = 5;
}

Record :: struct {
    cells: [2] Cell;
    grid: [2][2] int;
    x, y: int;
    #insert "cells[1].text = \"new\"; cells[1].value = 17;";
    #if true {
        grid[1][0] = 29;
    } else {
        missing = bad;
    }
    x, y = 31, 37;
}

probe :: () -> bool {
    a: Record;
    b := Record.{};

    initializer_of(Record)(*b);

    return a.cells[0].text == "old" && a.cells[1].text == "new"
        && a.cells[0].value == 5 && a.cells[1].value == 17
        && a.grid[1][0] == 29 && a.grid[0][0] == 0
        && a.x == 31 && a.y == 37 && b.cells[1].value == 17;
}

#assert probe();

main :: () {
    assert(probe());
}
