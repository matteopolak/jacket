// Generated reviewed public layouts; independent dependency grouping.
MAX_CHANNELS_PER_SOUND :: 8;
ACHANNEL_MAX :: 8;
OUTPUT_CHANNELS_MAX :: ACHANNEL_MAX;
NUM_SOURCE_DIRECTIONS :: 11;

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

Sound_Category :: enum u32 {
    GENERAL_SFX :: 0;
    AMBIENCES :: 1;
    FOOTSTEPS :: 2;
    UI :: 3;
    MUSIC :: 4;
}

Audio_Sample_Base :: struct {
    channels: [MAX_CHANNELS_PER_SOUND] float;
}

Audio_Source_Sample :: struct {
    using base: Audio_Sample_Base;
}

Audio_Dest_Sample :: struct {
    using base: Audio_Sample_Base;
}

Scale_Mapping :: struct {

    source_scale_for_this_output_index: [OUTPUT_CHANNELS_MAX] float;
    interpolated_source_scale_for_this_output_index: [OUTPUT_CHANNELS_MAX] float;
    dvolume_dsample: [OUTPUT_CHANNELS_MAX] float;
}

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

Decoder_Type :: enum {
    OGG :: 0;
    ADPCM :: 1;
}

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

Adpcm_Decoder :: struct {
    using base: Cached_Decoder;
}

Buffer_Bounds :: struct {
    start, end: s64;
    memory: *void;
}

Fill_Region_Result :: struct {
    buffer0: string;
    buffer1: string;
}

Backend_Properties :: struct {
    initted := false;

    num_channels: s32;
    channel_names: [] string;

    output_sampling_rate: s32;
}

Output_Device :: struct {
    name: string;
    hints: [] string;
    device_index: s64;
}

Sound_Player_Config :: struct {
    update_from_a_thread := true;
    audible_when_window_is_not_in_focus := false;
    keep_sounds_alive_by_default := true;

    seconds_to_fill_ahead := 2.4 / 60.0;

    set_async_thread_priority: () -> ();

    release_asset: (stream: *Sound_Stream, data: *Sound_Data);
}

Vbap_Arc :: struct {
    speaker1: Audio_Channel;
    speaker2: Audio_Channel;

    l_inverse: Matrix2;
}

#scope_file
#import "Math";
V :: #import "stb_vorbis";

stb_vorbis :: V.stb_vorbis;
