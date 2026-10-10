#import "Basic";

// The other platforms' `set_title` takes `(window, title)`, not this: no variant, so `flags` is
// reported.
set_title :: (handle: int, flags: int) {
    print("linux %\n", handle);
}

main :: () {
    set_title(1, 0);
}
