#import "Basic";
A :: #import "Sound_Player";
B :: #import "Sound_Player";

main :: () {
    left: A.Audio_Sample_Base;
    right: A.Audio_Sample_Base;

    for 0..7 {
        left.channels[it] = cast(float) (it + 1);
        right.channels[it] = cast(float) (8 - it);
    }

    sum := A.add(*left, *right, 8);
    for sum.channels assert(it == 9);

    A.plus_equals(*left, *right, 3);
    for 0..2 assert(left.channels[it] == 9);

    assert(left.channels[3] == 4 && left.channels[7] == 8);
    product := A.multiply(*left, *right);

    assert(product.channels[0] == 72 && product.channels[7] == 8);
    scale := A.multiply(*left, cast(float) 0.5);
    reverse := A.multiply(cast(float) 0.5, *left);

    for 0..7 assert(scale.channels[it] == reverse.channels[it] && scale.channels[it] == left.channels[it] * 0.5);

    A.times_equals(*left, 2);
    for 0..7 assert(left.channels[it] == scale.channels[it] * 4);

    source: A.Audio_Source_Sample;
    dest: A.Audio_Dest_Sample;

    source.channels[2] = 0.25;
    dest.channels[2] = 0.75;
    combined := A.add(*source.base, *dest.base, 8);

    assert(combined.channels[2] == 1);
    assert(size_of(A.Audio_Source_Sample) == 32 && size_of(A.Audio_Dest_Sample) == 32);
    assert((cast(*Type_Info_Struct) type_info(A.Audio_Sample_Base)).name == "Audio_Sample_Base");
    assert((cast(*Type_Info_Struct) type_info(A.Audio_Dest_Sample)).name == "Audio_Dest_Sample");
    assert(*A.the_accumulator == *B.the_accumulator);
    A.the_accumulator[47999].channels[7] = 0.625;
    assert(B.the_accumulator[47999].channels[7] == 0.625);
    B.the_accumulator[47999].channels[7] = 0;
    assert(A.get_magic4("xRIFF", 1) == 0x46464952);
    assert(A.arcs_6ch.count == 5 && A.arcs_8ch.count == 7);
    assert(A.arcs_6ch[0].speaker1 == .CENTER && A.arcs_6ch[2].speaker2 == .REAR_RIGHT);
    assert(A.arcs_8ch[3].speaker1 == .EXTEND_LEFT && A.arcs_8ch[3].speaker2 == .EXTEND_RIGHT);
    assert(*A.arcs_6ch == *B.arcs_6ch && *A.arcs_8ch == *B.arcs_8ch);
    A.arcs_6ch[0].l_inverse._11 = 0.75;
    assert(B.arcs_6ch[0].l_inverse._11 == 0.75);
    B.arcs_6ch[0].l_inverse._11 = 0;
    A.arcs = A.arcs_8ch;
    assert(B.arcs.data == A.arcs_8ch.data && B.arcs.count == 7);
    A.arcs = .{};
    decoder: A.Cached_Decoder;

    array_add(*A.retired_decoders, *decoder);
    assert(B.count_retired_decoders() == 1 && B.retired_decoders[0] == *decoder);
    array_free(A.retired_decoders);
    A.retired_decoders = .{};
    assert(B.count_retired_decoders() == 0);
    data: A.Sound_Data;
    stream: A.Sound_Stream;

    assert(data.nchannels == 1 && data.sampling_rate == 44100 && data.volume_scale == 1 && !data.loaded);
    assert(stream.num_channels == 2 && stream.repeat_end_position == -1 && stream.current_rate == 1);
    assert(decoder.page_size_in_samples == -1 && decoder.do_not_queue);
    assert((cast(*Type_Info_Struct) type_info(A.Sound_Stream)).name == "Sound_Stream");
    assert((cast(*Type_Info_Struct) type_info(A.Sound_Data)).name == "Sound_Data");
    print("ok\n");
}
