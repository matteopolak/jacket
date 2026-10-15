#import "Basic";
I :: #import "Iprof"(IMPORT_MODE = .CLIENT, USE_GRAPHICS = false);
No_History :: #import "Iprof"(IMPORT_MODE = .CLIENT, USE_GRAPHICS = false, DO_HISTORY = false);

main :: () {
    scalar: I.History_Scalar;
    for *sample: scalar.history sample.* = 99;

    I.history_index = 7;
    I.eternity_set(*scalar, 5);
    for value: scalar.values assert(value == 5);
    for moment: scalar.variances assert(moment == 25);
    for value, i: scalar.history assert(value == (ifx i == 7 then 5 else 99));

    factors: [3] float64 = .[0, 0.5, 1];

    I.update(*scalar, 9, factors);
    assert(scalar.values[0] == 9 && scalar.values[1] == 7 && scalar.values[2] == 5);
    assert(scalar.variances[0] == 81 && scalar.variances[1] == 53 && scalar.variances[2] == 25);
    assert(scalar.history[7] == 9);
    heat := I.compute_heat(53, 7);

    assert(heat > 0.285 && heat < 0.286);
    assert(I.compute_heat(0, 7) == 0 && I.compute_heat(10, 0) == 0 && I.compute_heat(100, 1) == 1);
    I.clear(*scalar);
    for value: scalar.values assert(value == 0);
    for moment: scalar.variances assert(moment == 0);

    assert(scalar.history[7] == 9 && scalar.history[8] == 99);
    no_history: No_History.History_Scalar;
    no_history.history[0] = 99;
    No_History.eternity_set(*no_history, 5);
    No_History.update(*no_history, 9, factors);
    assert(no_history.history[0] == 99 && no_history.values[1] == 7 && no_history.variances[1] == 53);
    print("ok\n");
}
