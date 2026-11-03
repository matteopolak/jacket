#import "Basic";

parse :: (text: string) -> int, bool {
    if text == "7" return 7, true;
    return 0, false;
}

main :: () {
    value := parse("x");
    print("%\n", value);
}
