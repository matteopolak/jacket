#import "Basic";
#import "Math";

MY_PI :: 3.14159265;

area :: (r: float) -> float {
    return PI * r * r;
}

main :: () {
    print("%\n", area(2));
    rough := 3.14;
    other := 3.1428;
    small := 1.4;

    print("% % % %\n", rough, other, small, MY_PI);
}
