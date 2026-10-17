#import "Basic";
G :: #import "developer-tools-custom-generated";

double_value :: (value: s32) -> s32 #c_call {
    return value * 2;
}

main :: () {
    assert(size_of(G.Mode) == 4 && cast(u32) G.Mode.MODE_ADD == 3 && cast(u32) G.Mode.MODE_SUBTRACT == 7);
    sample: G.Sample_Record;
    sample.x = 8;
    sample.y = 2;
    sample.lanes[2] = 6;
    assert(G.sample_apply(*sample, .MODE_ADD) == 23);
    assert(sample.lanes[2] == 10);
    assert(G.call_callback(double_value, .MODE_SUBTRACT) == 16);
    print("ok\n");
}
