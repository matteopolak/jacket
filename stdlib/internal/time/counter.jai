counter_to_words :: (counter: s64, frequency: s64) -> (low: u64, high: s64, success: bool) {
    if counter < 0 || frequency <= 0 return 0, 0, false;

    high, low := B.multiply_64bits_with_overflow(cast(u64) counter, B.FEMTOSECONDS);
    quotient, _remainder, error := B.unsigned_divide_with_remainder(B.U128.{ low, high }, B.U128.{ cast(u64) frequency, 0 });

    if error return 0, 0, false;

    return quotient.low, cast(s64) quotient.high, true;
}

#scope_file
B :: #import "Basic";
