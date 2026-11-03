#import "Basic";

first :: (items: [] int) -> int {
    if items.count == 0 return 0;
    return items[0];
}

main :: () {
    print("%\n", first(.[1, 2]));
}
