#import "Basic";

parse :: (text: string) -> int, bool {
    if text == "7" return 7, true;
    return 0, false;
}

lookup :: (key: int) -> string, int, bool {
    if key == 1 return "one", 1, true;
    return "", 0, false;
}

main :: () {
    // The parse failure and the zero it leaves look the same.
    value := parse("x");
    total: int;
    total = parse("7");

    // Two of three values kept; the flag is still dropped.
    name, number := lookup(2);

    callback := parse;
    other := callback("1");

    print("% % % % %\n", value, total, name, number, other);
}
