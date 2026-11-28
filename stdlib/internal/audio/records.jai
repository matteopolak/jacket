MAX_CHANNELS_PER_SOUND :: 8;
ACHANNEL_MAX :: 8;
// The most channels the player mixes to the output.
OUTPUT_CHANNELS_MAX :: ACHANNEL_MAX;
NUM_SOURCE_DIRECTIONS :: 11;

// A speaker of the output, as laid out by the player.
Audio_Channel :: enum u16 {
    FRONT_LEFT :: 0;
    FRONT_RIGHT :: 1;
    CENTER :: 2;
    SUBWOOFER :: 3;
    REAR_LEFT :: 4;
    REAR_RIGHT :: 5;
    EXTEND_LEFT :: 6;
    EXTEND_RIGHT :: 7;
}

// Where a channel of a sound belongs in the room. `load_audio_file` sets these from the channel count; change
// `Sound_Data.source_directions` before starting a stream to override them.
Source_Direction :: enum u16 {
    MONO :: 0;
    LEFT :: 1;
    RIGHT :: 2;
    FRONT_LEFT :: 3;
    FRONT_RIGHT :: 4;
    REAR_LEFT :: 5;
    REAR_RIGHT :: 6;
    CENTER_FORWARD :: 7;
    SUBWOOFER_CENTER_FORWARD :: 8;
    EXTEND_LEFT :: 9;
    EXTEND_RIGHT :: 10;
}

// A group of sounds. A stream's category sets its default flags (a `.MUSIC` stream is music).
Sound_Category :: enum u32 {
    GENERAL_SFX :: 0;
    AMBIENCES :: 1;
    FOOTSTEPS :: 2;
    UI :: 3;
    MUSIC :: 4;
}

// One frame of audio, a float of -1 to 1 for each of up to `MAX_CHANNELS_PER_SOUND` channels.
Audio_Sample_Base :: struct {
    channels: [MAX_CHANNELS_PER_SOUND] float;
}

// One frame of a sound before mixing, with a sample for each of its channels.
Audio_Source_Sample :: struct {
    using base: Audio_Sample_Base;
}

// One frame of the mixed output, with a sample for each output channel.
Audio_Dest_Sample :: struct {
    using base: Audio_Sample_Base;
}

// How one input channel of a stream feeds each output channel, as gain factors from 0 to 1.
//
// `source_scale_for_this_output_index` is the target, `interpolated_source_scale_for_this_output_index`
// the gain in use, and `dvolume_dsample` the most it moves toward the target each sample.
Scale_Mapping :: struct {

    source_scale_for_this_output_index: [OUTPUT_CHANNELS_MAX] float;
    interpolated_source_scale_for_this_output_index: [OUTPUT_CHANNELS_MAX] float;
    dvolume_dsample: [OUTPUT_CHANNELS_MAX] float;
}

// A loaded sound. `loaded` says whether loading worked. `volume_scale` scales every stream of it, and
// `buffer` holds the bytes the sound was read from. Make a `Sound_Stream` from it to play it.
//
// ```jai,skip_on(wasm)
// samples: [8] s16;
// sound := Sound_Data.{ loaded = true, type = .LINEAR_SAMPLE_ARRAY, nchannels = 1, samples = samples.data, nsamples_times_nchannels = samples.count };
// assert(sound.sampling_rate == 44100);
// ```
Sound_Data :: struct {
    using resource: struct {
        name: string;
        full_path: string;
        loaded := false;
        skip_load := false;
    };

    Kind :: enum {
        UNINITIALIZED;
        LINEAR_SAMPLE_ARRAY;
        OGG_COMPRESSED;
        ADPCM_COMPRESSED;
    }

    type := Kind.UNINITIALIZED;

    nsamples_times_nchannels: s64;

    nchannels: u16 = 1;
    sampling_rate: u32 = 44100;

    buffer: string;
    samples: *s16;

    volume_scale: float = 1;
    silence_before_repeat: float = 0;

    filename_to_stream_from: string;
    source_directions: [MAX_CHANNELS_PER_SOUND] Source_Direction;

    nBlockAlign: s32 = 0;
    wSamplesPerBlock: s32 = 0;
}

// One playing instance of a sound. `make_stream` creates it and `start_playing` hands it to the player,
// which owns it from then on. Set `user_volume_scale`, `rate_scale`, `position` and the `user_flags` before
// starting it, or any time while it plays.
//
// ```jai,no_run,skip_on(wasm)
// sound := load_audio_file("jump.wav");
// stream := make_stream(*sound);
// stream.user_volume_scale = 0.5;
// stream.rate_scale = 1.2; // a little higher in pitch
// start_playing(stream);
// ```
Sound_Stream :: struct {

    user_data_u64: u64;
    user_data_pointer: *void;

    manager_id: s64;
    entity_id: s64;

    sound_name: string;

    user_flags: enum u32 {
        REPEATING :: 0x1;
        SPATIALIZED :: 0x2;
        MUSIC :: 0x10;
        VOICEOVER :: 0x20;
        IS_FOOTSTEP :: 0x40;
        DO_NOT_KILL :: 0x80;
        PAUSED_DUE_TO_MENU :: 0x100;
        WAS_PAUSED_DUE_TO_MENU :: 0x200;
        AMBIENT :: 0x400;
    }

    internal_flags: enum u32 {
        FIRST_VOLUME_UPDATE :: 0x2;
        WAITING_FOR_INITIAL_DECODER_PAGES :: 0x4;
        FADING_OUT :: 0x8;
    }

    num_channels: s32 = 2;
    sound_data: *Sound_Data;
    decoder: *Cached_Decoder;

    category: Sound_Category;

    repeat_start_position: s64;
    repeat_end_position: s64 = -1;

    universe_volume_scale := 1.0;
    user_volume_scale := 1.0;

    play_cursor: float64 = 0;
    silent_samples_at_start_of_play: s64 = 0;

    marked := false;
    inaudible := false;

    position: Vector3;
    orientation: Quaternion;
    inner_radius := 0.5;
    outer_radius := 10.0;

    rate_scale := 1.0;

    samples_streamed_since_entity_update: s64;

    max_gain: float;
    last_plane_dir: Vector2;
    debug_display_volume: float = 0;

    silent_this_frame := false;
    debug_watch := false;

    current_rate: float = 1;
    desired_rate: float = 1;

    input_scale_mappings: [MAX_CHANNELS_PER_SOUND] Scale_Mapping;

    initial_samples_fetched: s64;
    samples_needed_to_start_playing: s64;
}

// The codec a [Cached_Decoder] reads.
Decoder_Type :: enum {
    OGG :: 0;
    ADPCM :: 1;
}

// A run of decoded frames of a compressed sound, as 16-bit interleaved samples in `data`.
//
// `start_address` and `end_address` are frame indexes (end exclusive), and `num_samples_contained` is
// the count of frames decoded so far. The decoder that made it owns it.
Cached_Decoder_Page :: struct {
    type: Decoder_Type;
    owner: *Cached_Decoder;

    start_address: s64 = -1;
    end_address: s64 = -1;
    page_index: s64 = -1;

    data: *void;

    buffer_length_in_bytes: s64;

    num_samples_contained: s64;

    marked := false;
}

// Decodes a compressed sound page by page and keeps the pages near the play cursor.
//
// Lengths are in frames (one sample per channel). `issued_start_addresses` lists the first frames of the
// pages queued and not yet decoded. With `do_not_queue` set, pages are decoded when requested instead of
// on the decode thread. The procedure fields are set by the codec that creates the decoder.
Cached_Decoder :: struct {
    type: Decoder_Type;

    pages: [..] *Cached_Decoder_Page;
    issued_start_addresses: [..] s64;

    page_size_in_samples: s64 = -1;
    uncompressed_length_in_samples: s64 = 0;

    num_channels: s64;
    sampling_rate: s64;

    sound_data: *Sound_Data;
    stream: *Sound_Stream;

    do_not_queue := true;

    deinit_proc: (decoder: *Cached_Decoder) -> ();

    add_decode_queue_item: (page: *Cached_Decoder_Page) -> ();

    recompute_page: (decoder: *Cached_Decoder, page: *Cached_Decoder_Page) -> ();
    make_page: (decoder: *Cached_Decoder) -> *Cached_Decoder_Page;
}

// A [Cached_Decoder] for Ogg Vorbis, with the open stream in `vorbis_file`.
Cached_Ogg_Decoder :: struct {
    using base: Cached_Decoder;

    vorbis_file: *stb_vorbis;

    seek_base: s64;
    opus_pre_skip: s64;

    bytes_per_mono_sample: s64;
    uncompressed_length_in_bytes: s64;

    bitrate_low: s64;
    bitrate_nominal: s64;
    bitrate_high: s64;
}

// A [Cached_Decoder] for IMA ADPCM; it needs no state beyond the base.
Adpcm_Decoder :: struct {
    using base: Cached_Decoder;
}

// A range of frames, `start` up to `end`, and the memory holding them (16-bit interleaved samples,
// starting at frame `start`).
Buffer_Bounds :: struct {
    start, end: s64;
    memory: *void;
}

// The memory the mixer fills for the output. This backend uses only `buffer0`; `buffer1` stays empty.
Fill_Region_Result :: struct {
    buffer0: string;
    buffer1: string;
}

// What the audio backend settled on: the channel count and the output sampling rate.
Backend_Properties :: struct {
    initted := false;

    num_channels: s32;
    channel_names: [] string;

    output_sampling_rate: s32;
}

// An audio output that `get_devices` found.
Output_Device :: struct {
    name: string;
    hints: [] string;
    device_index: s64;
}

// Settings for `sound_player_init`. The default works: it mixes on its own thread and keeps sounds
// playing when the window loses focus only if `audible_when_window_is_not_in_focus` is set.
// `release_asset` is called when a stream ends, so the program can free what the stream used.
//
// ```jai,no_run,skip_on(wasm)
// config: Sound_Player_Config;
// config.update_from_a_thread = false; // mix inside update() instead
// config.audible_when_window_is_not_in_focus = true;
// assert(sound_player_init(config));
// ```
Sound_Player_Config :: struct {
    update_from_a_thread := true;
    audible_when_window_is_not_in_focus := false;
    keep_sounds_alive_by_default := true;

    seconds_to_fill_ahead := 2.4 / 60.0;

    set_async_thread_priority: () -> ();

    release_asset: (stream: *Sound_Stream, data: *Sound_Data);
}

// Two neighbouring speakers that sound can be panned between, and the inverse of the matrix of their
// directions.
Vbap_Arc :: struct {
    speaker1: Audio_Channel;
    speaker2: Audio_Channel;

    l_inverse: Matrix2;
}

#scope_file
#import "Math";
V :: #import "stb_vorbis";

stb_vorbis :: V.stb_vorbis;
