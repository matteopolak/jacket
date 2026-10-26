// test: compile_fail
B :: #import "Base64";

main :: () {
    B.BASE64_DECODER_TABLE[0] = 0;
}
