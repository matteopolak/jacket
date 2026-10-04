// A parameter type may name an earlier parameter (`p: cache.Proc`).
#import "Basic";

Cache :: struct (T: Type) {
    Proc :: #type (*T) -> int;
    proc: Proc;
}

init :: (cache: *$C/Cache, proc: cache.Proc) { cache.proc = proc; }
init_plain :: (cache: *Cache(int), proc: cache.Proc) { cache.proc = proc; }

main :: () {
    c: Cache(int);
    init(*c, (x: *int) -> int { return x.* + 1; });
    n := 41;
    if c.proc(*n) != 42 exit(1);
    init_plain(*c, (x: *int) -> int { return x.* + 2; });
    if c.proc(*n) != 43 exit(2);
    print("ok\n");
}
