#import "Basic";

classify :: (code: int, n: s32, x: float, other: int) {
    if code != 404 && code != 410 print("kept\n");
    if n > 5 && n < 10 print("between\n");
    if n < 3 || n > 7 print("outside\n");
    if code == 1 || code == 2 print("small\n");
    if x > 1 && x < 2 print("fraction\n");
    if code > 10 && other < 5 print("two values\n");
    if n > 4 && n < 6 print("five\n");
}

main :: () {
    classify(1, 2, 1.5, 3);
}
