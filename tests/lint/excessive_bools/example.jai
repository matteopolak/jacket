#import "Basic";

open :: (path: string, read: bool, write: bool, create: bool, truncate: bool) -> int {
    return path.count;
}

main :: () {
    print("%\n", open("a.txt", true, false, true, false));
}
