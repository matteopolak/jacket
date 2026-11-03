#import "Basic";

offset :: (row: int, col: int) -> int {
    return row * 0 + col;
}

main :: () {
    print("%\n", offset(2, 3));
}
