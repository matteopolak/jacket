#import "Basic";

show :: (names: [] string) {
    n := 0;

    for names {
        print("% %\n", n, it);
        n += 1;
    }
}

main :: () {
    show(.["ada", "grace"]);
}
