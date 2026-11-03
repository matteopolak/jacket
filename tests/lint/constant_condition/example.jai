#import "Basic";

check :: (count: s64) {
    if count > 10 {
        if count < 5 {
            print("never\n");
        }
    }
}

main :: () {
    check(20);
}
