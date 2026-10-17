#import "Basic";
S :: #import "Socket";

main :: () {
    set: S.fd_set;
    capacity := cast(s32) (size_of(S.fd_set) * 8);

    assert(S.check_fd_set(0, *set));
    assert(S.check_fd_set(capacity - 1, *set));
    assert(!S.check_fd_set(-1, *set));
    assert(!S.check_fd_set(capacity, *set));
    assert(!S.check_fd_set(0, null));
    assert(S.FD_ISSET(0, null) == 0);
    S.FD_SET(capacity - 1, *set);
    assert(S.FD_ISSET(capacity - 1, *set) == 1);
    S.FD_CLR(capacity - 1, *set);
    assert(S.FD_ISSET(capacity - 1, *set) == 0);
    print("ok\n");
}
