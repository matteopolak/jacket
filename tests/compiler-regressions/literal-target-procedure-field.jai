// A struct literal can assign a procedure-typed field through a nested member path.
#import "Basic";

Handlers :: struct {
    on_start: (value: int) -> int;
    count: int;
}

Widget :: struct {
    #as base: Handlers;
    extra: int;
}

double :: (value: int) -> int {
    return value * 2;
}

main :: () {
    widget := Widget.{ base.on_start = double, base.count = 3, extra = 1 };
    if widget.base.on_start(21) != 42 exit(1);
    if widget.base.count != 3 || widget.extra != 1 exit(2);
    print("ok\n");
}
