#import "Basic";
#import "Math";

Point :: struct {
    x: int;
    y: int;
}

sum :: (xs: [] int) -> int {
    total := 0;
    for xs {
        total += it;
    }
    return total;
}

bump :: (points: [] Point) {
    for *points it.x += 1;
}

list :: (names: [] string) {
    for names {
        if it_index > 0 print(", ");
        print("%", it);
    }
    print("\n");
}

// Elements filled from their index.
fill :: (table: [] float) {
    for *table {
        it.* = sin(cast(float) it_index * 0.5);
    }
}

// A string iterates its bytes as `u8`, as indexing reads them.
shout :: (str: string) {
    for str {
        cur := it;
        print("%\n", cur);
    }
}

// A parenthesized bound (the fix takes the parentheses along).
bytes :: (str: string) -> int {
    total := 0;
    for str total += it;
    return total;
}

// Writes go through `for *str`.
shift :: (str: string) {
    for *str it.* += 1;
}

Named :: struct {
    name: string;
}

dump :: (n: Named) {
    for n.name print("%: %\n", it_index, it);
}

// The inner loop is rewritten to bind `it`, so the outer one cannot use `it` as well.
find :: (str: string, to_find: string) -> int {
    for i: 0..str.count - 1 {
        for to_find {
            if str[i] == it return i;
        }
    }
    return -1;
}

// An inner `for` that already binds `it`.
cross :: (xs: [] int, ys: [] int) {
    for i: 0..xs.count - 1 {
        for ys {
            print("%\n", xs[i] * it);
        }
    }
}

main :: () {
    print("%\n", find("hello", "lo"));
    cross(.[1, 2], .[3, 4]);
    s := copy_string("abc");
    shout(s);
    print("%\n", bytes(s));
    shift(s);
    dump(.{ s });
    xs := int.[1, 2, 3];
    points: [2] Point;
    bump(points);
    list(.["a", "b"]);
    print("% %\n", sum(xs), points[0].x);
    tour();
    t: [4] float;
    fill(t);
}

Stop :: struct {
    title: string;
    run: () -> ();
}

stops :: Stop.[.{ "one", one }, .{ "two", two }];

one :: () {
    print("1\n");
}

two :: () {
    print("2\n");
}

tour :: () {
    for stops {
        print("% %\n", it_index + 1, it.title);
        it.run();
    }
}
