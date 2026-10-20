// An `xx` operand of an arithmetic operator takes the type of the other operand, not the type
// the whole expression is converted to.
#import "Basic";

Collision :: enum s64 {
    NO :: 0;
    BORDER_LEFT :: 1;
    BORDER_TOP :: 2;
}

main :: () {
    index: int = 2;
    target: Collision;

    target = xx (xx Collision.NO + index);
    assert(target == .BORDER_TOP);

    declared: Collision = xx (xx Collision.BORDER_LEFT + index);

    assert(cast(s64) declared == 3);

    total := xx Collision.BORDER_TOP + index;

    assert(type_of(total) == int && total == 4);

    shown: float32 = 3.5;
    pixel: s64 = xx shown + 1;

    assert(pixel == 4);

    print("ok\n");
}
