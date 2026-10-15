// Reviewed smoothing selection and public scalar history records; no timer samples fabricated.
#import "Basic";
I :: #import "Iprof"(USE_GRAPHICS = false);
Other :: #import "Iprof"(USE_GRAPHICS = false, DO_HISTORY = false);

main :: () {
    assert(I.NUM_SMOOTHING_SLOTS == 3 && I.NUM_FRAME_SLOTS == 128);
    assert(I.slot == 1 && Other.slot == 1);
    scalar: I.History_Scalar;
    scalar.values = .[7, 19, 43];
    assert(I.get_value(*scalar) == 19);
    I.set_smoothing(-100);
    assert(I.slot == 0 && I.get_value(*scalar) == 7);
    I.set_smoothing(100);
    assert(I.slot == 2 && I.get_value(*scalar) == 43);
    I.set_smoothing(1);
    assert(I.slot == 1 && Other.slot == 1);
    Other.set_smoothing(0);
    assert(Other.slot == 0 && I.slot == 1);
    for *variance: scalar.variances variance.* = 88;
    for *sample: scalar.history sample.* = 99;

    I.clear(*scalar);
    for value: scalar.values assert(value == 0);
    for variance: scalar.variances assert(variance == 0);
    for sample: scalar.history assert(sample == 99);

    assert(I.get_value(*scalar) == 0);
    I.slot = -1;
    assert(I.get_value(*scalar) == 0);
    I.slot = 3;
    assert(I.get_value(*scalar) == 0);
    I.set_smoothing(1);
    print("ok\n");
}
