// A loop value named `it_index` hides the loop's default index, in a plain loop and in a
// for_expansion that declares it for the caller with a backtick.
#import "Basic";

Countdown :: struct {
    from: int;
}

for_expansion :: (c: Countdown, body: Code, flags: For_Flags) #expand {
    for < `it_index: 0..c.from {
        `it := it_index;
        #insert body;
    }
}

main :: () {
    seen: [..] int;
    for value: Countdown.{ 3 } array_add(*seen, value);
    assert(seen.count == 4 && seen[0] == 3 && seen[3] == 0);

    total := 0;
    for < it_index: 0..3 total += it_index;
    assert(total == 6);
}
