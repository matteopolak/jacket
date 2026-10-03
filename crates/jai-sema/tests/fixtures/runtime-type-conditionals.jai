choose :: (condition: bool, first: Type, second: Type) -> Type {
    return ifx condition then first else second;
}
main :: () -> int {
    first: Type = ifx true then s32 else float64;
    second := ifx false then First else s32;
    omitted := ifx false then float32;
    if first != second return 1;
    if omitted != null return 2;
    chosen := choose(false, float64, s32);
    if chosen != first return 3;
    return 42;
}
First :: struct { value: int; }
