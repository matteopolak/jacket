#import "Basic";
Verbose :: #import "Sound_Player"(VERBOSE = true);
Silent :: #import "Sound_Player";

notices: int;

notice :: (message: string, data: *void, info: Log_Info) {
    assert(message.count > 0);
    notices += 1;
}

main :: () {
    context.logger = notice;
    verbose_config: Verbose.Sound_Player_Config;
    verbose_config.seconds_to_fill_ahead = 0;
    assert(!Verbose.sound_player_init(verbose_config));
    Verbose.sound_player_shutdown();
    assert(notices == 1, "VERBOSE must report unavailable output");
    silent_config: Silent.Sound_Player_Config;
    silent_config.seconds_to_fill_ahead = 0;
    assert(!Silent.sound_player_init(silent_config));
    Silent.sound_player_shutdown();
    assert(notices == 1, "default configuration must not emit verbose notices");
    print("ok\n");
}
