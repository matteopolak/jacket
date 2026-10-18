#import "Basic";
#import "Math";
#load "lib.jai";

total_area :: (bodies: []Body) -> float {
    total := 0.0;
    for bodies total += area(it);
    return total;
}

largest :: (bodies: []Body) -> Body {
    best: Body;
    for bodies {
        if it.size > best.size best = it;
    }
    return best;
}

main :: () {
    bodies: [3]Body;
    bodies[0] = .{shape = .CIRCLE, size = 2, name = "disc"};
    bodies[1] = .{shape = .SQUARE, size = 3, name = "tile"};
    bodies[2] = scaled(bodies[1], 2);

    count := 0;
    while count < bodies.count {
        print("%\n", describe(bodies[count]));
        count += 1;
    }

    big := largest(bodies);
    print("largest: %, total area %\n", big.name, total_area(bodies));
    assert(big.size >= 2);
}
