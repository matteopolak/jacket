// rules: opov.12
// `g[i] = .{...}` builds the literal as the `operator []=` value parameter, here a type taken
// from the container's own parameters.
#import "Basic";
#import "Soa";

Point :: struct {
    x, y: int;
}

Grid :: struct {
    cells: [4] Point;
}

operator []= :: (grid: *Grid, index: int, value: Point) {
    grid.cells[index] = value;
}

main :: () {
    soa: Soa(Point, 4);

    for 0..3 {
        soa[it] = .{ it, it * 2 };
    }

    point := soa[3];

    assert(point.x == 3 && point.y == 6);

    grid: Grid;
    grid[1] = .{ 5, 6 };
    assert(grid.cells[1].y == 6);
}
