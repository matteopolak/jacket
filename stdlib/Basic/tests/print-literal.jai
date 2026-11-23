#import "Basic";

// A format with no `%` and no values writes the text as it is; anything else goes through the
// formatter.
main :: () {
    count := print("plain\n");

    print("bytes %\n", count);
    print("100\%\n");
    print("% and %\n", 1, "two");
    print("to stderr\n", to_standard_error = true);
    format := "from a variable\n";

    print(format);
}
