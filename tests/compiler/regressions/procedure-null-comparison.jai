// rules: ops.18
// A procedure name compares with `null` as the procedure's own address, so a foreign or local
// procedure is never null.
#import "Basic";

local :: () {}

main :: () {
    assert(local != null);
    assert(null != local);
    assert(!(local == null));
    loaded := local != null;

    assert(loaded);
}
