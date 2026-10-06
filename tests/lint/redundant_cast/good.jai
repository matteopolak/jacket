#import "Basic";

Handle :: #type,distinct u32;

widen :: (value: $T) -> s64 {
    // In a polymorphic body the cast does something for other `T`s.
    return cast(s64) value;
}

main :: () {
    x: s32 = 5;
    y := cast(s64) x;
    f := cast(float) x;
    t := cast,trunc(s32) x;
    raw: u32 = 7;
    h := cast(Handle) raw;
    print("% % % % %\n", y, f, t, h, widen(3));
}
