#import "Basic";

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

main :: () {
    xs := int.[1, 2, 3];
    points: [2] Point;
    bump(points);
    list(.["a", "b"]);
    print("% %\n", sum(xs), points[0].x);
}
