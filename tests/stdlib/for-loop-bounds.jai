// Range and array loops at their edges: bounds at the type's limits (where stepping past the
// end would wrap), empty and one-element ranges, reverse ranges, a body that moves `it` or
// changes the array, and `continue`. Checked at run time and, through #run, by the interpreter
// at compile time.
#import "Basic";

check :: () -> bool {
    // Up to the largest u8: the step must stop before 255 wraps to 0.
    count := 0;
    last: u8;
    for cast(u8) 250..cast(u8) 255 {
        count += 1;
        last = it;
    }
    if count != 6 || last != 255 return false;

    // Down to the smallest s8.
    count = 0;
    for < cast(s8) -128..cast(s8) -126 count += 1;
    if count != 3 return false;

    // Empty, single and reverse ranges.
    count = 0;
    high := 4;
    for 5..high count += 1;
    if count != 0 return false;
    for 7..7 count += 1;
    if count != 1 return false;
    total := 0;
    for < 1..4 total = total * 10 + it;
    if total != 4321 return false;

    // A body that moves `it` past the end stops the loop.
    count = 0;
    for 0..100 {
        count += 1;
        if it == 3 it = 1000;
    }
    if count != 4 return false;

    // A body that moves `it` back repeats values.
    seen := 0;
    for 0..3 {
        seen += 1;
        if seen == 2 it = 0;
    }
    if seen != 5 return false;

    // `continue` still steps.
    odd := 0;
    for 0..9 {
        if it % 2 == 0 continue;
        odd += it;
    }
    if odd != 25 return false;

    // Arrays: the count is read again after each step, so growing or shrinking takes effect.
    values: [..] int;
    array_add(*values, 1, 2, 3);
    visited := 0;
    for values {
        visited += 1;
        if it == 1 array_add(*values, 4);
    }
    if visited != 4 return false;
    sum := 0;
    for values {
        if it == 2 remove it;
        else sum += it;
    }
    if values.count != 3 || sum != 8 return false;

    // A loop that copies through pointers with an index (constant steps and element sizes).
    source: [8] s16;
    target: [8] s16;
    for 0..7 source[it] = cast(s16) (it * 3 - 7);
    data := cast(*u8) source.data;
    for 0..7 {
        target[it] = (cast(*s16) data).*;
        data += 2;
    }
    for 0..7 if target[it] != source[it] return false;

    empty: [] int;
    for empty return false;
    for < empty return false;
    return true;
}

#run assert(check());

main :: () {
    assert(check());
}
