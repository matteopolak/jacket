// Sound through Sound_Player. When no output device can be opened the game stays silent instead of
// failing, so a machine without audio (or the smoke test) still runs.

Audio :: struct {
    ready: bool;
    muted: bool;
    music: *SP.Sound_Stream;
    played: s64;
    music_wanted: bool; // set by `audio_start_music`; a reloaded music file starts again
}

// The player calls this for every stream that ends. A stopped music stream is a dead pointer from then on.
audio_stream_released :: (stream: *SP.Sound_Stream, data: *SP.Sound_Data) {
    if current_audio && current_audio.music == stream current_audio.music = null;
}

current_audio: *Audio;

audio_init :: (audio: *Audio, assets: *Assets) {
    config: SP.Sound_Player_Config;

    config.update_from_a_thread = false;
    config.release_asset = audio_stream_released;
    current_audio = audio;

    audio.ready = SP.sound_player_init(config);

    if !audio.ready return;

    for Sound_Id.SHOOT..Sound_Id.MUSIC {
        if !load_sound(assets, it) print("could not load %\n", it);
    }
}

audio_shutdown :: (audio: *Audio) {
    if audio.ready SP.sound_player_shutdown();

    audio.ready = false;
    current_audio = null;
}

// Muting goes through the master volume so the music, which is already playing, goes quiet too.
audio_set_muted :: (audio: *Audio, muted: bool) {
    audio.muted = muted;

    if audio.ready SP.set_master_volume(ifx muted then 0.0 else 1.0);
}

audio_play :: (audio: *Audio, assets: *Assets, id: Sound_Id, volume := 0.5, rate := 1.0) {
    data := assets.sounds[cast(s64) id];

    if !audio.ready || audio.muted || !data return;

    stream := SP.make_stream(data);

    if !stream return;

    stream.user_volume_scale = volume;
    stream.rate_scale = rate;
    SP.start_playing(stream);
    audio.played += 1;
}

audio_start_music :: (audio: *Audio, assets: *Assets) {
    data := assets.sounds[cast(s64) Sound_Id.MUSIC];

    audio.music_wanted = true;

    if !audio.ready || !data || audio.music return;

    audio.music = SP.make_stream(data, .MUSIC);

    if !audio.music return;

    audio.music.user_volume_scale = 0.25;
    SP.set_repeating(audio.music, true);
    SP.start_playing(audio.music);
}

// Plays what the simulation reported this tick, with a little pitch variation.
audio_events :: (audio: *Audio, assets: *Assets, world: *World, time: float) {
    for world.events {
        wobble := 1.0 + 0.08 * sin(time * 13 + cast(float) it_index);

        if #complete it == {
            case .SHOT; audio_play(audio, assets, .SHOOT, 0.25, wobble);
            case .HIT; audio_play(audio, assets, .HIT, 0.3, wobble);
            case .EXPLODE; audio_play(audio, assets, .EXPLODE, 0.5, wobble);
            case .PICKUP; audio_play(audio, assets, .PICKUP, 0.5);
            case .PLAYER_HIT; audio_play(audio, assets, .EXPLODE, 0.8, 0.6);
            case .WAVE; audio_play(audio, assets, .PICKUP, 0.3, 0.7);
            case .GAME_OVER; audio_play(audio, assets, .EXPLODE, 0.9, 0.4);
        }
    }
}

audio_update :: (audio: *Audio, assets: *Assets, dt: float) {
    if audio.ready && audio.music_wanted && !audio.music audio_start_music(audio, assets);

    if audio.ready SP.update(dt);
}
