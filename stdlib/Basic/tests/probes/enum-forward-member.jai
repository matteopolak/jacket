// Original general reproducer for explicit dependency expressions in enums.
#import "Basic";

Options :: enum_flags u32 {
    READ :: 1;
    ALL :: READ | WRITE;
    WRITE :: 2;
}

main :: () {
    #assert cast(u32) Options.ALL == 3;
    assert((Options.ALL & .WRITE) != 0);
    print("ok\n");
}
