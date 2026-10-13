// A fiber that recurses until its stack is gone. Natively it hits the guard page and the program
// stops with a message that names the fiber; under `jaic -interp` the interpreter's own depth limit
// stops it first.
#import "Basic";
Io :: #import "Extensions/Io";

recurse :: (n: int) -> int {
    buffer: [64] int;
    buffer[n % 64] = n;
    if n < 0 return 0;

    return recurse(n + 1) + buffer[n % 64];
}

main :: () {
    Io.io_run(() {
        print("start\n");
        print("%\n", recurse(0));
    });
}
