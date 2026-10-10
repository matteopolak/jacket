#import "Basic";

set_title :: (window: int, title: string) {
    print("windows % %\n", window, title);
}

main :: () {
    set_title(1, "x");
}
