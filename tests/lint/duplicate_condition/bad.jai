#import "Basic";

Mode :: enum {
    READ;
    WRITE;
    APPEND;
}

describe :: (key: int, mode: Mode) {
    if key == 1 print("one\n");
    else if key == 2 print("two\n");
    else if key == 1 print("uno\n");

    if mode == {
        case .READ; print("read\n");
        case .WRITE; print("write\n");
        case .READ; print("append\n");
    }
}

main :: () {
    describe(1, .READ);
}
