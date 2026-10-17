// Unix select must accept the canonical POSIX timeout record.
#import "Basic";
S :: #import "Socket";
P :: #import "POSIX";

main :: () {
    timeout: P.timeval;
    assert(S.select(0, null, null, null, *timeout) == 0);
    print("ok\n");
}
