X :: #import "Xar";

main :: () {
    array: X.Xar(s32, 1, 2);
    for i: 0..4 X.xar_add(*array, cast(s32) i);
}
