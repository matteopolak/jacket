#import "Basic";

Shape :: struct {
    kind: enum {
        CIRCLE;
        SQUARE;
    };
    union {
        radius: float;
        side: float;
    }
}

circle_area :: (s: Shape) -> float {
    r := s.radius;
    return 3.14159 * r * r;
}

main :: () {
    print("%\n", circle_area(.{ kind = .CIRCLE, radius = 2 }));
}
