#import "Basic";

Mode :: enum {
    QUIET;
    VERBOSE;
    DEBUG;
}

announce :: (mode: Mode) {
    if mode == {
        case .VERBOSE;
            print("loud\n");
    }
}

main :: () {
    announce(.VERBOSE);
}
