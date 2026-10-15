// Independent safety requirement: negative extents cannot align as byte sizes.
S :: #import "Socket";

main :: () {
    S.CMSG_ALIGN(-1);
}
