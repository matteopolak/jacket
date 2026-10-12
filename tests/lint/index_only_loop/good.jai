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

// A string compared with its neighbour, walked from the middle, or in step with another.
runs :: (str: string, other: string) -> int {
    n := 0;
    for i: 0..str.count - 2 if str[i] == str[i + 1] n += 1;
    for i: 1..str.count - 1 n += str[i];
    for i: 0..str.count - 1 if str[i] != other[i] n += 1;

    return n;
}

// Every pair of elements of one array: both loops keep their indices.
spread :: (ms: [] int) {
    for i: 0..ms.count - 1 {
        for j: 0..ms.count - 1 {
            r := ms[i] - ms[j];
            print("%\n", r);
        }
    }
}

main :: () {
    spread(.[1, 2, 3]);
    print("%\n", runs("aab", "abb"));
    a := float.[1, 2];
    xs: [3] int;

    pairs(xs);
    tail(xs);
    grid: [2][3] int;

    clear(grid);
    more: [..] int;

    array_add(*more, 1);
    grow(*more);
    print("%\n", dot(a, a));
}
