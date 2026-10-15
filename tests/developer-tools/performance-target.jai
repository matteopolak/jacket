#import "Basic";

work :: (x: $T) -> T {
    return x + 1;
}

answer :: #run work(40);

main :: () {
    assert(answer == 41);
    assert(work(4) == 5);
    print("ok\n");
}
