// Sound_Player through its offline output (no audio device, the mix is pulled with render_offline):
// loading a WAV file, master volume, the entity-update protocol that stops unmarked streams, the
// listener orientation, loudness queries and the small helpers. Mixed samples are checked exactly
// enough to catch wrong gains. Sound_Player needs stb_vorbis, which has no wasm build, so the wasm
// targets skip it (see the skip list).
#import "Basic";

#import "Math";
#import "File";
#import "Sound_Player"(OFFLINE = true);

RATE :: 48000;
SCRATCH :: ".scratch/sound-player-offline-api";

released: [..] *Sound_Stream;
note_release :: (stream: *Sound_Stream, data: *Sound_Data) {
    array_add(*released, stream);
}
was_released :: (stream: *Sound_Stream) -> bool {
    for released if it == stream return true;
    return false;
}

// A mono 16-bit WAV holding `frames` copies of `value`.
constant_wav :: (frames: int, value: s16) -> string {
    builder: String_Builder;
    u16le :: (b: *String_Builder, v: int) {
        append(b, cast(u8) (v & 0xff));
        append(b, cast(u8) ((v >> 8) & 0xff));
    }
    u32le :: (b: *String_Builder, v: int) {
        u16le(b, v & 0xffff);
        u16le(b, (v >> 16) & 0xffff);
    }
    append(*builder, "RIFF");
    u32le(*builder, 36 + frames * 2);
    append(*builder, "WAVEfmt ");
    u32le(*builder, 16);
    u16le(*builder, 1); // PCM
    u16le(*builder, 1); // mono
    u32le(*builder, RATE);
    u32le(*builder, RATE * 2);
    u16le(*builder, 2);
    u16le(*builder, 16);
    append(*builder, "data");
    u32le(*builder, frames * 2);
    for 1..frames u16le(*builder, cast(u16) value);
    return builder_to_string(*builder);
}

render :: (frames: int) -> [] s16 {
    out := NewArray(frames * 2, s16, initialized = false);
    got := render_offline(out);
    assert(got == frames, "render_offline mixed % frames, wanted %", got, frames);
    return out;
}

near :: (sample: s16, expected: float, slack := 3.0) -> bool {
    return abs(cast(float) sample - expected) <= slack;
}

main :: () {
    // Helpers that need no running player.
    assert(perceptual_to_linear(0) == 0 && perceptual_to_linear(1) == 1);
    half_loudness := perceptual_to_linear(0.5);
    assert(abs(half_loudness - 0.3162) < 0.001, "half perceived loudness is amplitude %", half_loudness);
    assert(perceptual_to_linear(0.25) < half_loudness, "perceptual_to_linear is not increasing");
    assert(get_devices().count == 0, "the offline output lists devices");
    nothing: [8] s16;
    assert(render_offline(nothing) == 0, "rendered without a running player");

    // Loading from a file; a missing optional file is quietly not loaded.
    make_directory_if_it_does_not_exist(".scratch");
    make_directory_if_it_does_not_exist(SCRATCH);
    path := tprint("%/constant.wav", SCRATCH);
    assert(write_entire_file(path, constant_wav(2000, 16384)), "could not write %", path);
    data := load_audio_file(path);
    assert(data.loaded && data.type == .LINEAR_SAMPLE_ARRAY, "load_audio_file(%) did not load", path);
    assert(data.full_path == path && data.name == path, "loaded names % / %", data.full_path, data.name);
    assert(data.nchannels == 1 && data.sampling_rate == RATE && data.nsamples_times_nchannels == 2000);
    missing := load_audio_file(tprint("%/missing.wav", SCRATCH), optional = true);
    assert(!missing.loaded, "a missing file loaded");
    file_delete(path);

    config: Sound_Player_Config;
    config.update_from_a_thread = false;
    config.keep_sounds_alive_by_default = false;
    config.release_asset = note_release;
    assert(sound_player_init(config), "offline init failed");
    assert(sound_player_init(config), "a second init must report the running player");
    assert(get_audio_sampling_rate() == 48000);

    // A centred mono sound reaches both speakers at constant power, scaled by the master volume
    // through the perceptual curve.
    full := 0.5 * 32767.0 * sqrt(0.5);
    set_master_volume(0.5);
    first := make_stream(*data);
    assert(first != null);
    assert(samples_to_bytes(first, 10) == 20, "10 mono samples are % bytes", samples_to_bytes(first, 10));
    first.entity_id = 1;
    start_playing(first);
    start_playing(first); // Already playing: not added twice.
    out := render(4);
    assert(near(out[0], full * half_loudness) && near(out[1], full * half_loudness), "half master volume gave % %", out[0], out[1]);
    assert(get_desired_volume_perceptual(first, .{}) == 0.5);
    set_master_volume(1);
    update();
    out = render(600); // Gains glide for 10 ms (480 frames).
    assert(near(out[1198], full) && near(out[1199], full), "full master volume gave % %", out[1198], out[1199]);
    assert(get_desired_volume_perceptual(first, .{}) == 1);

    // Loudness of a positioned stream falls off between its radii.
    probe := make_stream(*data);
    probe.user_flags |= .SPATIALIZED;
    probe.inner_radius = 1;
    probe.outer_radius = 11;
    assert(get_desired_volume_perceptual(probe, .{ 6, 0, 0 }) == 0.5, "halfway between the radii: %", get_desired_volume_perceptual(probe, .{ 6, 0, 0 }));
    assert(get_desired_volume_perceptual(probe, .{ 0.5, 0, 0 }) == 1);
    assert(get_desired_volume_perceptual(probe, .{ 0, 20, 0 }) == 0);
    probe.user_volume_scale = 0.5;
    assert(get_desired_volume_perceptual(probe, .{}) == 0.5);
    assert(get_desired_volume_perceptual(null, .{}) == 0);
    free(probe); // Never started, so it is still ours.

    // Entity protocol: streams nobody marks are stopped, unless they ask to be kept.
    kept := make_stream(*data);
    kept.entity_id = 2;
    kept.user_flags |= .DO_NOT_KILL;
    set_repeating(kept, true);
    start_playing(kept);
    marked := make_stream(*data);
    marked.entity_id = 3;
    set_repeating(marked, true);
    start_playing(marked);
    pre_entity_update();
    assert(!first.marked && !kept.marked && !marked.marked, "pre_entity_update left marks set");
    marked.marked = true;
    post_entity_update(1.0 / 60);
    assert(was_released(first) && find(1, 0) == null, "an unmarked stream kept playing");
    assert(find(2, 0) == kept && find(3, 0) == marked, "a marked or DO_NOT_KILL stream was stopped");
    stop_stream_abruptly(3);
    assert(was_released(marked) && find(3, 0) == null);
    set_repeating(kept, false);
    assert(!(kept.user_flags & .REPEATING));
    stop_stream_abruptly(*data);
    assert(was_released(kept) && find(2, 0) == null);

    // The listener turns: a source on its left inside the inner radius moves to the right ear.
    side := make_stream(*data);
    set_repeating(side, true);
    side.user_flags |= .SPATIALIZED;
    side.position = .{ 0, 20, 0 };
    side.inner_radius = 30;
    side.outer_radius = 60;
    side.manager_id = 9;
    start_playing(side);
    out = render(4);
    assert(near(out[0], 0.5 * 32767.0) && near(out[1], 0), "left source gave % %", out[0], out[1]);
    turned: Quaternion;
    set_from_axis_and_angle(*turned, .{ 0, 0, 1 }, PI);
    update_listener(.{}, turned);
    update();
    out = render(600);
    assert(near(out[1198], 0) && near(out[1199], 0.5 * 32767.0), "after turning around % %", out[1198], out[1199]);
    // Moving the listener far away makes it inaudible.
    update_listener(.{ 1000, 0, 0 }, Quaternion_Identity);
    update();
    assert(side.inaudible, "a source 1000 units away is audible");
    stop_all_sounds_abruptly(9);
    assert(was_released(side));
    update_listener(.{}, Quaternion_Identity);

    sound_player_shutdown();
    assert(render_offline(nothing) == 0, "rendered after shutdown");
    print("ok\n");
}
