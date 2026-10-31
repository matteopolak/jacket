NUM_SMOOTHING_SLOTS :: 3;

// A smoothed value with its variance and a short history. It holds the same measure at several smoothing speeds.
History_Scalar :: struct {
    values: [NUM_SMOOTHING_SLOTS] float64;
    variances: [NUM_SMOOTHING_SLOTS] float64;
    history: [NUM_FRAME_SLOTS] float;
}

slot: s64 = 1;

// Chooses which smoothing speed `get_value` reads: 0 reacts fastest, higher numbers average over more frames.
set_smoothing :: (x: s64) {
    slot = history_slot(x);
}

// Resets the smoothed values and variances to zero.
clear :: (using s: *History_Scalar) {
    assert(s != null);
    for *value: s.values value.* = 0;
    for *moment: s.variances moment.* = 0;
}

// The smoothed value at the speed set with `set_smoothing`.
get_value :: (using s: *History_Scalar) -> float64 {
    assert(s != null);
    return s.values[history_slot(slot)];
}

#scope_file
#import "Basic";

history_slot :: (requested: s64) -> s64 {
    if requested < 0 return 0;
    if requested >= NUM_SMOOTHING_SLOTS return NUM_SMOOTHING_SLOTS - 1;
    return requested;
}
