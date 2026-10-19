#import "Basic";

narrowed :: (count: s64) {
    if count > 10 {
        if count < 5 {
            print("never\n");
        }
    }
}

else_never :: (count: s64) {
    if count > 10 {
        if count > 5 {
            print("always\n");
        } else {
            print("never\n");
        }
    }
}

after_exit :: (count: s64) {
    if count < 5 {
        return;
    }
    if count < 3 print("never\n");
}

main :: () {
    narrowed(12);
    else_never(12);
    after_exit(7);
}
