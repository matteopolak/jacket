#import "Basic";
G :: #import "developer-tools-public-generated";

double_value :: (value: s32) -> s32 #c_call {
    return value * 2;
}

main :: () {
    sample: G.Sample;
    sample.x = 8;
    sample.y = 2;
    sample.lanes[2] = 6;
    assert(G.sample_apply(sample = *sample, mode = .ADD) == 23);
    assert(sample.lanes[2] == 10);
    assert(G.Mode.MODE_ADD == G.Mode.ADD);
    assert(G.callback_apply(callback = double_value, value = 5) == 12);
    print("ok\n");
}
