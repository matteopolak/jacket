#import "Basic";

lookup :: (key: string) -> int, bool {
    return key.count, key.count > 0;
}

main :: () {
    a, _ := 1, 2;
    print("%\n", a);
    value := lookup("key");
    print("%\n", value);
    first: string;
    first = "x";
    print("%\n", first);
}
