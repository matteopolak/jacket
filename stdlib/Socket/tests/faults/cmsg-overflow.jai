// Independent safety requirement: the reviewed u32 result cannot hold this size.
S :: #import "Socket";

main :: () {
    S.CMSG_LEN(0x100000000);
}
