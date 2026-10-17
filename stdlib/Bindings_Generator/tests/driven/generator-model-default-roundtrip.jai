#import "Basic";
G :: #import "generator-model-defaults-generated";

native :: #foreign_library "model_defaults";
model_default_fill :: (record: *G.ModelWideRecord) #foreign native;

main :: () {
    first: G.IndependentGeneric();
    second: G.IndependentGeneric(u8, 7);

    first.data[2] = 42;
    second.data[6] = 17;
    assert(first.data.count == 3 && first.data[2] == 42 && second.data.count == 7 && second.data[6] == 17);
    assert(size_of(G.IndependentGeneric()) == 12 && size_of(G.IndependentGeneric(u8, 7)) == 7);
    assert(size_of(G.ModelWideRecord) == 32 && align_of(G.ModelWideRecord) == 16);
    assert(type_of(G.ModelWideRecord.signed_value) == S128 && type_of(G.ModelWideRecord.unsigned_value) == U128);
    wide: G.ModelWideRecord;

    model_default_fill(*wide);
    assert(wide.signed_value.low == 0xfffffffffffffff9 && wide.signed_value.high == -1);
    assert(wide.unsigned_value.low == 17 && wide.unsigned_value.high == 9);
    print("ok\n");
}
