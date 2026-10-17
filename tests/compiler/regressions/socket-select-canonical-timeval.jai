#import "Basic";
S :: #import "Socket";
P :: #import "POSIX";

#assert OS_IS_UNIX;
#assert !#exists(S.TIMEVAL);

main :: () {
    packet: struct {
        before: u64 = 0x0123456789abcdef;
        timeout: P.timeval;
        after: u64 = 0xfedcba9876543210;
    }

    assert(S.select(0, null, null, null, *packet.timeout) == 0);
    assert(packet.before == 0x0123456789abcdef);
    assert(packet.after == 0xfedcba9876543210);
    assert(packet.timeout.tv_sec == 0 && packet.timeout.tv_usec == 0);
    print("ok\n");
}
