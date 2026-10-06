#import "Basic";

lookup :: (key: string) -> int, bool {
    return key.count, key.count > 0;
}

main :: () {
    leftover := 5;
    a, b := 1, 2;
    print("%\n", a);
    value, found := lookup("key");
    print("%\n", value);
    first, second: string;
    first = "x";
    print("%\n", first);
}
