#import "Basic";

V :: struct {
    x: int;
    y: int;
}

same :: (a: V, b: V) -> bool {
    return a.x == b.x && a.y == b.y;
}

N :: 3;

main :: () {
    f := 0.5;
    is_nan := f != f; // the NaN test
    finite := f - f == 0;
    print("% % % %\n", same(.{}, .{}), is_nan, finite, N == N);
}
