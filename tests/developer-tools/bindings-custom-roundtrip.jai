#import "Basic";
G :: #import "developer-tools-custom-generated";

double_value :: (value: s32) -> s32 #c_call {
    return value * 2;
}

main :: () {
    sample: G.Sample_Record;
    sample.x = 8;
    sample.y = 2;
    sample.lanes[2] = 6;
    assert(G.sample_apply(sample = *sample, mode = .ADD) == 23);
    assert(sample.lanes[2] == 10);
    assert(G.Mode.MODE_ADD == G.Mode.ADD);
    assert(G.call_callback(callback = double_value, number = .SUBTRACT) == 16);
    alias: G.Sample;

    assert(type_of(alias) == G.Sample_Record);
    print("ok\n");
}
