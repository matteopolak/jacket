#import "Basic";

// One platform's file, linted alone: `windows/window.jai`, one directory over, defines
// `set_title` with the same parameter names, so the ignored `title` here is the other's.
set_title :: (window: int, title: string) {
    print("macos %\n", window);
}

main :: () {
    set_title(1, "x");
}
