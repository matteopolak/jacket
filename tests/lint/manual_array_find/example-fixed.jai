#import "Basic";

has :: (names: [] string, wanted: string) -> bool {
    found := array_find(names, wanted);
    return found;
}

main :: () {
    print("%\n", has(.["ada", "grace"], "grace"));
}
