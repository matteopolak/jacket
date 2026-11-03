#import "Basic";

check :: (count: s64) {
    if count > 10 {
        if count < 50 {
            print("between\n");
        }
    }
}

main :: () {
    check(20);
}
