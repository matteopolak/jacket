#import "Basic";

classify :: (code: int, n: s32, b: u8) {
    if code != 404 || code != 410 print("gone\n");
    if n > 10 && n < 5 print("never\n");
    if b == 1 && b == 2 print("never\n");
    if 3 > code && code > 7 print("never\n");
    if n >= 0 || n < 0 print("always\n");
}

main :: () {
    classify(1, 2, 3);
}
