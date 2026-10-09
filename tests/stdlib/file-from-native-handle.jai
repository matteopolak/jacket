// A `File` wraps the platform's own handle: `File.{stdin}` (the C stream) on macOS and Linux,
// and on Windows what `GetStdHandle` returns (jai-format reads standard input this way).
#import "Basic";
#import "File";

#if OS == .WINDOWS {
    #import "Windows";
} else {
    #import "POSIX";
}

main :: () {
    #if OS == .WINDOWS {
        handle := GetStdHandle(STD_INPUT_HANDLE);
        input := File.{ handle };
    } else {
        input := File.{ stdin };
    }
    assert(is_valid(input), "the standard input handle is not valid");
    print("ok\n");
}
