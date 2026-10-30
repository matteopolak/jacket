// A `using #as name :: Base(T).{};` constant makes the struct count as an instance of
// `Base` for a `$A/Base` restriction, and `A.data_type` reaches the constants of `Base(T)`.
#import "Basic";

Plane :: struct(data_type: Type) {}
Strip :: struct(data_type: Type) {}

put :: inline (a: *$A/Plane, x: int, y: int, value: A.data_type) {
    set(a, x - 1, y - 1, value);
}

put :: inline (a: *$A/Strip, x: int, value: A.data_type) {
    set(a, x - 1, value);
}

Grid :: struct(W: int, H: int, T: Type) {
    using #as _base :: Plane(T).{};
    cells: [W * H] T;
}

Line :: struct(N: int, T: Type) {
    using #as _base :: Strip(T).{};
    cells: [N] T;
}

set :: inline (a: *Grid, x: int, y: int, value: a.data_type) {
    a.cells[y * a.W + x] = value;
}

set :: inline (a: *Line, x: int, value: a.data_type) {
    a.cells[x] = value;
}

fill :: ($T: Type) {
    grid: Grid(4, 3, T);
    line: Line(5, T);

    put(*grid, 2, 3, cast(T) 7);
    put(*line, 4, cast(T) 9);

    assert(grid.cells[2 * 4 + 1] == 7);
    assert(line.cells[3] == 9);
}

main :: () {
    fill(float32);
    fill(s64);

    print("ok\n");
}
