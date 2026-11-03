#import "Basic";
#import "Math";

area :: (r: float) -> float {
    return PI * r * r;
}

main :: () {
    print("%\n", area(2));
}
