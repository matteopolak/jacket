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

main :: () {
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
