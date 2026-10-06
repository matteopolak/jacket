#import "Basic";

// A string and values that are not a format.
greet :: (name: string, extras: ..Any) {
    print("hello %\n", name);
    for extras print("%\n", it);
}

main :: () {
    print("% and %\n", 1, 2);
    print("100\% sure\n");
    print("%2 then %1\n", "second", "first");
    greet("you", 1, 2, 3);
    // Spread arguments and computed formats are not counted.
    values := Any.[1, 2];
    print("% %\n", ..values);
    format := "%\n";
    print(format, 3);
}
