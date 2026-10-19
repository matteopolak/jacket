#import "Basic";

DEBUG :: false;

fine :: (count: s64, flag: bool) {
    if count > 10 {
        if count < 20 print("between\n");
    }
    if DEBUG print("debug\n");
    while true {
        if flag break;
    }
    if count < 5 {
        return;
    }
    if count < 9 print("after the early return\n");
}

unsigned :: (size: u32) -> bool {
    return size >= 0;
}

main :: () {
    fine(12, true);
    print("%\n", unsigned(1));
}
