#import "Basic";

HANDLE_LIKE :: cast(*void) 32512;
INVALID_LIKE :: cast(*void) -1;

main :: () {
    assert(cast(u64) HANDLE_LIKE == 32512);
    assert(cast(s64) INVALID_LIKE == -1);
}
