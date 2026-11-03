#import "Basic";

Mode :: enum {
    QUIET;
    VERBOSE;
    DEBUG;
}

announce :: (mode: Mode) {
    if mode == .VERBOSE print("loud\n");
}

main :: () {
    announce(.VERBOSE);
}
