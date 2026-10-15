#import "Basic";
#import "Math";
#import "Sound_Player";

main :: () {
    assert(abs(perceptual_to_linear(0.5) - 0.31622777) < 0.000001);
    assert(abs(perceptual_to_linear(2) - 3.16227766) < 0.00001);
    data: Sound_Data;
    data.nsamples_times_nchannels = 40;
    data.nchannels = 2;
    stream: Sound_Stream;
    stream.sound_data = *data;
    stream.repeat_end_position = 3;
    set_repeating(*stream, true);
    assert(stream.repeat_end_position == 20);
    set_repeating(*stream, false);
    assert(stream.repeat_end_position == 20 && !(stream.user_flags & .REPEATING));
    set_master_volume(2);
    assert(get_desired_volume_perceptual(*stream, .{}) == 2);
    set_master_volume(-1);
    assert(get_desired_volume_perceptual(*stream, .{}) == -1);
    set_master_volume(1);
    print("ok\n");
}
