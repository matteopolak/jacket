#import "Basic";

open :: (path: string, read: bool, write: bool, create: bool, truncate: bool) -> int {
    return path.count;
}

configure :: (verbose := false, quiet := false, color := true, strict := false, name := "x") -> int {
    return name.count;
}

main :: () {
    print("% %\n", open("a", true, false, true, false), configure());
}
