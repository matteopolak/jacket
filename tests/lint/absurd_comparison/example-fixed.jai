#import "Basic";

countdown :: (n: u32) {
    i := n;
    while i > 0 {
        print("%\n", i);
        i -= 1;
    }
}

main :: () {
    countdown(3);
}
