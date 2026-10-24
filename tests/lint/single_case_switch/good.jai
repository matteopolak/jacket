#import "Basic";

Mode :: enum {
    QUIET;
    VERBOSE;
}

announce :: (mode: Mode) {
    if mode == .VERBOSE print("loud\n");
}

// Every member must be covered: adding one is an error here.
describe :: (mode: Mode) -> string {
    if #complete mode == {
        case .QUIET;
            return "quiet";
        case .VERBOSE;
            return "verbose";
    }

    return "";
}

pair :: (n: int) {
    if n == {
        case 1, 2;
            print("small\n");
    }
}

three :: (n: int) {
    if n == {
        case 1;
            print("one\n");
        case 2;
            print("two\n");
        case;
            print("other\n");
    }
}

through :: (n: int) {
    if n == {
        case 1;
            print("one\n");
            #through;
        case;
            print("any\n");
    }
}

main :: () {
    announce(.VERBOSE);
    pair(1);
    three(2);
    through(1);
    print("%\n", describe(.QUIET));
}
