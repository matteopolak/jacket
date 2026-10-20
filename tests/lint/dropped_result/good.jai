#import "Basic";

parse :: (text: string) -> int, bool {
    if text == "7" return 7, true;
    return 0, false;
}

counted :: (text: string) -> string, int {
    return text, text.count;
}

// The `bool` comes after another result: a status extra, not the flag for the first value.
wide :: (text: string) -> int, string, bool {
    return text.count, text, true;
}

main :: () {
    value, ok := parse("7");

    // Ignored on purpose.
    other, _ := parse("x");
    _ := parse("y");

    // A dropped value that is not a `bool`.
    kept := counted("abc");

    // The first value left over is not a `bool`.
    size := wide("abc");

    // Nothing receives the values.
    parse("w");

    print("% % % % %\n", value, ok, other, kept, size);
}
