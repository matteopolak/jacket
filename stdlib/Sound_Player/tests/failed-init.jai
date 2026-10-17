#import "Basic";
#import "Sound_Player";

main :: () {
    config: Sound_Player_Config;
    config.update_from_a_thread = false;
    config.seconds_to_fill_ahead = 0;
    assert(!sound_player_init(config));
    assert(get_audio_sampling_rate() == 0);
    assert(decode_queue_initted, "reviewed initialization creates decoder queues before requesting a device");
    sound_player_shutdown();
    assert(!decode_queue_initted && get_audio_sampling_rate() == 0);
    print("ok\n");
}
