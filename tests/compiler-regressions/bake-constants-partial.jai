// `#bake_constants` that binds some of a procedure's `$` variables leaves a procedure that is
// still polymorphic over the rest.
#import "Basic";

Grid :: struct(T: Type, W: int, H: int) {
    cells: [H][W] T;
}

diagonal :: (g: Grid($T, $W, $H)) -> T {
    #assert W == H;
    sum: T;
    for i: 0..W - 1 sum += g.cells[i][i];
    return sum;
}

diagonal_f :: #bake_constants diagonal(T = float32);

main :: () {
    small: Grid(float32, 2, 2);
    small.cells[0][0] = 1.5;
    small.cells[1][1] = 2;
    assert(diagonal_f(small) == 3.5);

    big: Grid(float32, 3, 3);
    big.cells[2][2] = 4;
    assert(diagonal_f(big) == 4);
}
