// Plays sounds: it mixes streams of WAV (PCM or ADPCM) and Ogg Vorbis audio and sends the result
// to the system's audio output.
//
// Load a sound with `load_audio_file` or `load_audio_data`, start the player with `sound_player_init`,
// then make a stream with `make_stream`, set it up and `start_playing` it. Streams with a position
// are panned and softened by distance from the listener (`update_listener`). Call `update` once per
// frame; `sound_player_shutdown` ends it all.
//
// ```jai,no_run,skip_on(wasm)
// #import "Sound_Player";
//
// sound := load_audio_file("jump.wav");
// assert(sound.loaded);
// assert(sound_player_init(.{}));
//
// stream := make_stream(*sound, .GENERAL_SFX);
// start_playing(stream);
//
// while true {
//     update();
//     // ... run a frame ...
// }
// ```

#module_parameters (MAX_SOUND_CATEGORIES := DEFAULT_MAX_SOUND_CATEGORIES, VERBOSE := false) {
    DEFAULT_MAX_SOUND_CATEGORIES :: 64;
}

#load "../internal/audio/records.jai";
#load "../internal/audio/samples.jai";
#load "../internal/audio/topology.jai";

#load "../internal/audio/mixer.jai";
#load "../internal/audio/panning.jai";
#load "../internal/audio/cache.jai";
#load "../internal/audio/adpcm.jai";
#load "../internal/audio/decode-queue.jai";
#load "../internal/audio/vorbis.jai";
#load "../internal/audio/loading.jai";
#load "../internal/audio/backend.jai";
#load "../internal/audio/history.jai";
#load "../internal/audio/player.jai";
#if OS == .WINDOWS #load "../internal/audio/windows-abi.jai";

#scope_module
#import "Basic";
#import "Math";
#import "Thread";
V :: #import "stb_vorbis";
A :: #import "Adpcm";
F :: #import "File";
Wav :: #import "Wav_File";
