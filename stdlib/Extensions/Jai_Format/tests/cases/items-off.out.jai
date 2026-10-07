// Blank lines between file-level items (blank_lines_between_items).
#import "Basic";
#import,file "helpers.jai";
Math :: #import "Math";
#load "other.jai";
Vec2 :: struct {
    x, y: float;
}
Vec3 :: struct {
    x, y, z: float;
}
COUNT :: 4;
counter := 0;
Mode :: enum {
    A;
    B;
}
twice :: (x: int) -> int {
    return x * 2;
}
// Attached to `main`: the blank line goes above it.
// Still attached.
main :: () {
    print("%\n", twice(COUNT));
}
/* A block comment above an item. */
helper :: () {}
// A comment that already has a blank line under it stays where it is.

later :: () {}
TEXT :: #string END
two
lines
END;
NAME :: "single";
#run {
    print("compile time\n");
}
#if OS == .WINDOWS {
    #import "Windows";
    #load "win.jai";
    win_only :: () {}
    WIN_FLAG :: true;
} else {
    other_only :: () {}
    #scope_file
    detail :: () {}
}
#if OS == .MACOS #load "mac.jai";
#if OS == {
    case .LINUX;
        x :: 1;
    case;
        x :: 2;
}
#scope_file
private :: () {}
#scope_export
#import "String";
struct_then_semicolon :: struct {
    a: int;
};
LAST :: 1; // trailing comment
