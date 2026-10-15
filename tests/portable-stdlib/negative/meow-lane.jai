M :: #import "meow_hash";

main :: () {
    #asm { pxor value:, value; }
    M.MeowU64From(value, 2);
}
