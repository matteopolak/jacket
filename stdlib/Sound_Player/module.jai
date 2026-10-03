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
