#import "Basic";
#import "Math";

Point :: struct {
    x: int;
    y: int;
}

sum :: (xs: [] int) -> int {
    total := 0;
    for i: 0..xs.count - 1 {
        total += xs[i];
    }
    return total;
}

bump :: (points: [] Point) {
    for i: 0..points.count - 1 points[i].x += 1;
}

list :: (names: [] string) {
    for i: 0..names.count - 1 {
        if i > 0 print(", ");
        print("%", names[i]);
    }
    print("\n");
}

// Elements filled from their index.
fill :: (table: [] float) {
    for i: 0..table.count - 1 {
        table[i] = sin(cast(float) i * 0.5);
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
    for i: 0..stops.count - 1 {
        print("% %\n", i + 1, stops[i].title);
        stops[i].run();
    }
}
