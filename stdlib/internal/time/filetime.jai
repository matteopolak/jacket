ticks_to_words :: (ticks: u64) -> (low: u64, high: s64) #no_aoc {
    epoch_low, epoch_high := filetime_epoch();
    high, low := B.multiply_64bits_with_overflow(ticks, 100000000);
    result_low := epoch_low + low;
    result_high := cast(u64) epoch_high + high;

    if result_low < epoch_low result_high += 1;

    return result_low, cast(s64) result_high;
}

words_to_ticks :: (input_low: u64, input_high: s64) -> (ticks: u64, success: bool) #no_aoc {
    epoch_low, epoch_high := filetime_epoch();
    if input_high < epoch_high || (input_high == epoch_high && input_low < epoch_low) return 0, false;

    low := input_low - epoch_low;
    high := cast(u64) input_high - cast(u64) epoch_high;

    if input_low < epoch_low high -= 1;

    quotient, _remainder, error := B.unsigned_divide_with_remainder(B.U128.{ low, high }, B.U128.{ 100000000, 0 });
    if error || quotient.high != 0 return 0, false;

    return quotient.low, true;
}

#scope_file
B :: #import "Basic";

filetime_epoch :: () -> (low: u64, high: s64) #no_aoc {
    // Gregorian UTC elapsed seconds between 1601-01-01 and 1970-01-01.
    offset := B.seconds_to_apollo(11644473600);
    unix := B.APOLLO_TIME_FOR_JAN_1_1970;
    low := unix.low - offset.low;
    high := cast(u64) unix.high - cast(u64) offset.high;

    if unix.low < offset.low high -= 1;

    return low, cast(s64) high;
}
