#import "Basic";
F :: #import "Flat_Pool";
P :: #import "POSIX";

main :: () {
    page := P.sysconf(P._SC_PAGESIZE);

    assert(page > 0);
    pool: F.Flat_Pool;

    F.init(*pool, page);
    F.get(*pool, page + 1);
}
