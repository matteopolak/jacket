#import "Basic";

main :: () {
    carry := S128.{ 0xffffffffffffffff, 7 } + S128.{ 1, 9 };

    assert(carry.low == 0 && carry.high == 17);
    borrow := S128.{ 0, 7 } - S128.{ 1, 2 };

    assert(borrow.low == 0xffffffffffffffff && borrow.high == 4);
    negative := to_s128(-7) + to_s128(3);

    assert(negative.low == 0xfffffffffffffffc && negative.high == -1);
    wrapped := S128.{ 0xffffffffffffffff, 0x7fffffffffffffff } + to_s128(1);

    assert(wrapped.low == 0 && wrapped.high == -9223372036854775808);
    unsigned := U128.{ 0, 0 } - U128.{ 1, 0 };

    assert(unsigned.low == 0xffffffffffffffff && unsigned.high == 0xffffffffffffffff);
    start := seconds_to_apollo(5);
    end := start + milliseconds_to_apollo(250);
    milliseconds, ok := to_milliseconds(end - start);

    assert(ok && milliseconds == 250);
    negative_ms, negative_ok := to_milliseconds(start - end);

    assert(negative_ok && negative_ms == -250);
    current := current_time_monotonic();

    sleep_milliseconds(2);
    elapsed, elapsed_ok := to_nanoseconds(current_time_monotonic() - current);

    assert(elapsed_ok && elapsed >= 0);

    print("ok\n");
}
