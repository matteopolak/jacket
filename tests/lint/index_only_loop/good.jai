#import "Basic";

// Two arrays walked in step.
dot :: (a: [] float, b: [] float) -> float {
    total := 0.0;
    for i: 0..a.count - 1 total += a[i] * b[i];
    return total;
}

// Neighbours: the index is computed with.
pairs :: (xs: [] int) {
    for i: 0..xs.count - 2 print("%\n", xs[i] + xs[i + 1]);
}

// Elements filled from their index.
fill :: (xs: [] int) {
    for i: 0..xs.count - 1 xs[i] = i * i;
}

// A grid, by row and column.
clear :: (grid: [][3] int) {
    for r: 0..grid.count - 1 for c : 0..grid[r].count - 1 grid[r][c] = 0;
}

// Not from the start.
tail :: (xs: [] int) {
    for i: 1..xs.count - 1 print("%\n", xs[i]);
}

// The array changes size in the loop.
grow :: (xs: *[..] int) {
    for i: 0..xs.count - 1 array_add(xs, xs.*[i]);
}

main :: () {
    a := float.[1, 2];
    xs: [3] int;
    fill(xs);
    pairs(xs);
    tail(xs);
    grid: [2][3] int;
    clear(grid);
    more: [..] int;
    array_add(*more, 1);
    grow(*more);
    print("%\n", dot(a, a));
}
