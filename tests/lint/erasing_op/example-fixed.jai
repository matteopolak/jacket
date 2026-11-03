#import "Basic";

offset :: (row: int, col: int) -> int {
    return row * 8 + col;
}

main :: () {
    print("%\n", offset(2, 3));
}
