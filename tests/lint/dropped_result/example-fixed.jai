#import "Basic";

parse :: (text: string) -> int, bool {
    if text == "7" return 7, true;
    return 0, false;
}

main :: () {
    value, ok := parse("x");
    if !ok print("not a number\n");
    else print("%\n", value);
}
