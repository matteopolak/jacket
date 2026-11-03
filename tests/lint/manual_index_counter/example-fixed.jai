#import "Basic";

show :: (names: [] string) {
    for names {
        print("% %\n", it_index, it);
    }
}

main :: () {
    show(.["ada", "grace"]);
}
