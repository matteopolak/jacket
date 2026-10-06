// A traced call made from an inline (untraced) procedure records the line of the inline call
// in the caller's node, not a line of the inline procedure's source.
#import "Basic";

caller_line :: () -> s64 { return context.stack_trace.next.line_number; }
through_inline :: inline () -> s64 {
    x := 1;
    return caller_line() * x;
}

main :: () {
    // The location is taken on the next line, so this holds whatever the layout.
    line := through_inline();
    here := #location();
    if line != here.line_number - 1 { print("FAIL: % vs %\n", line, here.line_number - 1); exit(1); }
    print("ok\n");
}
