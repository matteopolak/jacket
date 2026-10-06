#import "Basic";

V :: struct {
    x: int;
    y: int;
}

same :: (a: V, b: V) -> bool {
    return a.x == b.x && a.y == a.y;
}

main :: () {
    n := 5;
    zero := n - n;
    ok := n > 0;
    print("% % %\n", same(.{}, .{}), zero, ok || ok);
}
