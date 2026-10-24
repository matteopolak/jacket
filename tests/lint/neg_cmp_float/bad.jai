#import "Basic";

inside :: (x: float, limit: float, lo: float64) -> bool {
    if !(x < limit) return false;

    ok := !(x >= lo * 2) && true;

    wide := (!(lo <= 1.5)) == ok;

    return ok || wide;
}

main :: () {
    print("%\n", inside(1, 2, 3));
}
