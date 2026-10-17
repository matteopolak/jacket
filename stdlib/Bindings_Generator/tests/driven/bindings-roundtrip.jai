#import "Basic";
G :: #import "developer-tools-generated";

double_value :: (value: s32) -> s32 #c_call {
    return value * 2;
}

main :: () {
    sample: G.Sample;
    sample.x = 8;
    sample.y = 2;
    sample.lanes[2] = 6;
    assert(G.sample_apply(*sample, .MODE_ADD) == 23);
    assert(sample.lanes[2] == 10);
    assert(G.callback_apply(double_value, 5) == 12);
    print("ok\n");
}
