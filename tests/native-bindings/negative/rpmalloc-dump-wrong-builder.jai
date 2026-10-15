// Reviewed enabled builder form rejects a different pointee type at compile time.
RPM :: #import "rpmalloc";

main :: () {
    wrong: s64;
    RPM.dump_statistics(*wrong);
}
