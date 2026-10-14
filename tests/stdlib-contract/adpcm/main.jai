// Public constants, nominal types, saturation, and documented delayed index clamp.
#import "Basic";
A :: #import "Adpcm";

#assert(is_constant(A.step_table));
#assert(is_constant(A.index_table));
#assert(A.step_table.count == 89);
#assert(A.index_table.count == 16);

main :: () {
    assert(type_info(A.ADPCM_State).name == "ADPCM_State");
    assert(type_info(A.Adpcm_Info).name == "Adpcm_Info");
    assert(A.ADPCM_CHANNELS_MAX == 9);
    assert(A.step_table[0] == 7 && A.step_table[88] == 32767);
    assert(A.index_table[0] == -1 && A.index_table[7] == 8);

    positive: A.ADPCM_State;
    positive.sample = 32760;
    positive.index = 88;
    assert(A.IMA_ADPCM_nibble(*positive, 7) == 32767);
    assert(positive.index == 96);
    assert(A.IMA_ADPCM_nibble(*positive, 0) == 32767);
    assert(positive.index == 87);

    negative: A.ADPCM_State;
    negative.sample = -32760;
    negative.index = 88;
    assert(A.IMA_ADPCM_nibble(*negative, 15) == -32768);
    assert(negative.index == 96);
    assert(A.IMA_ADPCM_nibble(*negative, 8) == -32768);
    assert(negative.index == 87);
    print("ok\n");
}
