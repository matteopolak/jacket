#import "Basic";

area :: (r: float) -> float {
    return 3.14159 * r * r;
}

turns :: (angle: float64) -> float64 {
    return angle / 6.2832;
}

main :: () {
    print("%\n", area(2));
    print("%\n", turns(1));
    diagonal := 1.4142;
    quarter := 0.7854;

    print("% %\n", diagonal, quarter);
}
