#import "Basic";

// TODO: handle the empty list
first :: (items: [] int) -> int {
    return items[0];
}

main :: () {
    print("%\n", first(.[1, 2]));
}
