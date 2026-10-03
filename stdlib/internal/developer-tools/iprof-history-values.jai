NUM_SMOOTHING_SLOTS :: 3;

History_Scalar :: struct {
    values: [NUM_SMOOTHING_SLOTS] float64;
    variances: [NUM_SMOOTHING_SLOTS] float64;
    history: [NUM_FRAME_SLOTS] float;
}

slot: s64 = 1;

set_smoothing :: (x: s64) {
    slot = history_slot(x);
}

clear :: (using s: *History_Scalar) {
    assert(s != null);
    for *value: s.values value.* = 0;
    for *moment: s.variances moment.* = 0;
}

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
