// `#no_reset` globals keep what `#run` stored in them, in the interpreter and in a compiled
// executable alike (chess-jai loads its network weights at compile time this way).
// rules: ctdata.10
#import "Basic";

Weights :: struct {
    biases: [8] s16 #align 64;
    table: [64] s16 #align 64;
    name: string;
    steps: [..] int;
    first: *s16;
}

#no_reset weights: Weights #align 64;
#no_reset count := 1;
untouched_reset := 5;

#run {
    for i: 0..7 weights.biases[i] = cast(s16) (i + 1);
    for i: 0..63 weights.table[i] = cast(s16) (i * 2);
    weights.name = copy_string("network");
    array_add(*weights.steps, 3, 5, 8);
    weights.first = *weights.table[10];
    count = 99;
    untouched_reset = 77;
}

main :: () {
    assert(weights.biases[3] == 4);
    assert(weights.table[63] == 126);
    assert(weights.name == "network");
    assert(weights.steps.count == 3 && weights.steps[2] == 8);
    assert(weights.first.* == 20);
    assert(count == 99);
    assert(untouched_reset == 5);
    print("ok\n");
}
