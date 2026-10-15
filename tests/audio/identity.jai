#import "Basic";
#import "Sound_Player";
Again :: #import "Sound_Player";

main :: () {
    assert(type_info(Sound_Stream).name == "Sound_Stream");
    assert(type_info(Cached_Ogg_Decoder).name == "Cached_Ogg_Decoder");
    assert(type_info(Sound_Data).name == "Sound_Data");
    assert(type_info(Sound_Stream) == type_info(Again.Sound_Stream));
    assert(*arcs_6ch == *Again.arcs_6ch);
    assert(*the_accumulator == *Again.the_accumulator);
    assert(*decode_queue_initted == *Again.decode_queue_initted);
    for direction: vbap_speaker_directions assert(direction.x == 0 && direction.y == 0);
    for direction: vbap_source_directions assert(direction.x == 0 && direction.y == 0);

    config: Sound_Player_Config;

    assert(config.update_from_a_thread && config.keep_sounds_alive_by_default);
    assert(!config.audible_when_window_is_not_in_focus && config.seconds_to_fill_ahead == 2.4 / 60);
    stream: Sound_Stream;

    assert(stream.repeat_end_position == -1 && stream.num_channels == 2);
    assert(stream.inner_radius == 0.5 && stream.outer_radius == 10);
    data: Sound_Data;

    assert(data.nchannels == 1 && data.sampling_rate == 44100 && data.volume_scale == 1);
    print("ok\n");
}
