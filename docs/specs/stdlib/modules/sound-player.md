# Sound_Player


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `Audio_Channel :: enum u16`
- `ACHANNEL_MAX :: #run enum_highest_value(Audio_Channel) + 1`
- `OUTPUT_CHANNELS_MAX :: ACHANNEL_MAX`
- `MAX_CHANNELS_PER_SOUND :: 8`
- `Sound_Category :: enum u32`
- `Source_Direction :: enum u16`
- `NUM_SOURCE_DIRECTIONS :: #run enum_highest_value(Source_Direction) + 1`
- `SOUND_INNER_RADIUS_DEFAULT :: 10.0`
- `SOUND_OUTER_RADIUS_DEFAULT :: 150.0`
- `CACHED_OGG_DECODE_THREADED :: true`
- `Scale_Mapping :: struct`
- `Sound_Stream :: struct`
- `Backend_Properties :: struct`
- `perceptual_to_linear :: (x: float) -> float`
- `retired_decoders: [..] *Cached_Decoder`
- `SECONDS_TO_FETCH_OGG :: 0.4`
- `SECONDS_TO_FETCH_ADPCM:: 0.4`
- `VERY_CLOSE :: 4.0`
- `vbap_speaker_directions: [ACHANNEL_MAX] Vector2`
- `vbap_source_directions: [NUM_SOURCE_DIRECTIONS] Vector2`
- `ROOT_2_OVER_2 : float : 0.7071067`
- `Vbap_Arc :: struct`
- `arcs_6ch := Vbap_Arc.[ .{ .CENTER, .FRONT_LEFT, .{} }, .{ .FRONT_LEFT, .REAR_LEFT, .{} }, .{ .REAR_LEFT, .REAR_RIGHT, .{} }, .{ .REAR_RIGHT, .FRONT_RIGHT, .{} }, .{ .FRONT_RIGHT, .CENTER, .{} } ]`
- `arcs_8ch := Vbap_Arc.[ .{ .CENTER, .FRONT_LEFT, .{} }, .{ .FRONT_LEFT, .REAR_LEFT, .{} }, .{ .REAR_LEFT, .EXTEND_LEFT, .{} }, .{ .EXTEND_LEFT, .EXTEND_RIGHT, .{} }, .{ .EXTEND_RIGHT, .REAR_RIGHT, .{} }, .{ .REAR_RIGHT, .FRONT_RIGHT, .{} }, .{ .FRONT_RIGHT, .CENTER, .{} } ]`
- `arcs: [] Vbap_Arc`
- `count_retired_decoders :: () -> s64`
- `invert :: (using m: *Matrix2, result: *Matrix2)`
- `clear_scale_accumulation :: (stream: *Sound_Stream, index: s64)`
- `get_target_2_closeness :: (stream: *Sound_Stream) -> float`
- `adjust_stereo_dots_by_closeness :: (dot_left: *float, dot_right: *float, closeness: float)`
- `accum_scale_target_2_source_mono :: (stream: *Sound_Stream, plane_dir: Vector2, gain_scale: float, target: *Scale_Mapping)`
- `accum_scale_target_2_source_n :: (stream: *Sound_Stream, plane_dir: Vector2, n: s64, gain_scale: float)`
- `add_vbap_gain_for_direction :: (target: *Scale_Mapping, plane_dir: Vector2, gain_scale: float)`
- `skip_this_output_channel :: (index: Audio_Channel, stream: *Sound_Stream) -> bool`
- `accum_scale_target_all_source_mono :: (stream: *Sound_Stream, plane_dir: Vector2, gain_scale: float, target: *Scale_Mapping)`
- `accum_scale_target_all_source_n :: (stream: *Sound_Stream, plane_dir: Vector2, n: s64, gain_scale: float)`
- `get_plane_dir :: (stream: *Sound_Stream, position: Vector3) -> Vector2`
- `source_dir_to_target_dir :: (source_dir_index: Source_Direction) -> Audio_Channel, success := true`
- `update_panning_helper_for_many_channel_music :: (stream: *Sound_Stream, gain_scale: float)`
- `update_panning_helper_for_locked_output :: (stream: *Sound_Stream, gain_scale: float)`
- `update_panning_helper :: (stream: *Sound_Stream, gain_scale: float)`
- `update_panning :: (stream: *Sound_Stream)`
- `Sound_Player_Config :: struct`
- `Output_Device :: struct`
- `get_devices :: () -> [] Output_Device`
- `sound_player_init :: (given_config: Sound_Player_Config) -> bool`
- `sound_player_shutdown :: ()`
- `samples_to_bytes :: (stream: *Sound_Stream, num_samples: s64) -> s64`
- `set_master_volume :: (value: float)`
- `find :: (entity_id: s64, manager_id: s64) -> *Sound_Stream`
- `update_desired_rate :: (stream: *Sound_Stream)`
- `make_stream :: (data: *Sound_Data, category := Sound_Category.GENERAL_SFX) -> *Sound_Stream`
- `start_playing :: (stream: *Sound_Stream)`
- `stop_stream_abruptly :: (id: s64)`
- `stop_stream_abruptly :: (data: *Sound_Data)`
- `stop_all_sounds_abruptly :: (manager_id: s64)`
- `get_desired_volume_perceptual :: (stream: *Sound_Stream, position: Vector3) -> float`
- `update_listener :: (camera_position: Vector3, camera_orientation: Quaternion)`
- `get_audio_sampling_rate :: () -> s64`
- `decode_queue_initted := false`
- [CACHED_OGG_DECODE_THREADED] `decode_queue: [..] *Cached_Decoder_Page`
- [CACHED_OGG_DECODE_THREADED] `decode_queue_semaphore: Semaphore`
- [CACHED_OGG_DECODE_THREADED] `decode_queue_lock: Mutex`
- [CACHED_OGG_DECODE_THREADED] `decode_queue_thread: Thread`
- `completed_queue: [..] *Cached_Decoder_Page`
- `completed_queue_lock: Mutex`
- `init_sound_player_decode_queue :: ()`
- `process_one_page :: (page: *Cached_Decoder_Page)`
- `add_decode_queue_item :: (page: *Cached_Decoder_Page)`
- `get_next_completed_decode_queue_item :: () -> *Cached_Decoder_Page`
- `pre_entity_update :: ()`
- `post_entity_update :: (dt: float)`
- `update :: (dt := FLOAT32_INFINITY)`
- `set_repeating :: (stream: *Sound_Stream, repeating: bool)`
- `load_audio_file :: (filename: string, optional := false) -> Sound_Data`
- `load_audio_data :: (name_for_error_reporting: string, data: string) -> Sound_Data`
- `release_sound_data :: (data: *Sound_Data)`
- `replace_sound_data :: (data: *Sound_Data, replacement: Sound_Data) -> bool`
- `get_magic4 :: (data: string, base: s64) -> u32`
- `TEST_K :: 1.0`
- `DELTA_RATE_MAX :: 18.0 * (1.0 / 40000.0) * TEST_K`
- `DELTA_RATE_MIN :: -DELTA_RATE_MAX`
- `the_accumulator: [48000] Audio_Dest_Sample`
- `accumulator_element_size :: size_of(Audio_Dest_Sample)`
- `Audio_Sample_Base :: struct`
- `Fill_Region_Result :: struct`
- `Audio_Source_Sample :: struct`
- `Audio_Dest_Sample :: struct`
- `Sound_Data :: struct`
- `Buffer_Bounds :: struct`
- `get_buffer_bounds :: (stream: *Sound_Stream, data: *Sound_Data, index: s64, result: *Buffer_Bounds) -> success: bool , no_overflow: bool`
- `add :: (v0: *Audio_Sample_Base, v1: *Audio_Sample_Base, nchannels: s64) -> Audio_Sample_Base`
- `plus_equals :: (v0: *Audio_Sample_Base, v1: *Audio_Sample_Base, nchannels: s64)`
- `multiply :: (v0: *Audio_Sample_Base, v1: *Audio_Sample_Base) -> Audio_Sample_Base`
- `multiply :: (v0: *Audio_Sample_Base, k: float) -> Audio_Sample_Base`
- `multiply :: (k: float, v0: *Audio_Sample_Base) -> Audio_Sample_Base`
- `times_equals :: (v0: *Audio_Sample_Base, k: float)`
- `get_icursor_and_remainder :: (source_cursor: float64) -> icursor: s64, remainder: float64`
- `sample_n_channels :: (stream: *Sound_Stream, samples: *s16, page_start: s64, page_end: s64, source_cursor: float64, samples_return: [] float, no_overflow: bool) #no_abc`
- `mix_sample_into_output_streams :: (stream: *Sound_Stream, source: *Audio_Source_Sample, dest: *Audio_Dest_Sample) #no_abc`
- `maybe_wrap_play_cursor :: (stream: *Sound_Stream)`
- `catch_up_volumes :: (stream: *Sound_Stream)`
- `accumulate_buffer_f :: (accumulator: *Audio_Dest_Sample, num_samples: s64, source_cursor: float64, page: *void, page_start: s64, page_end: s64, stream: *Sound_Stream, no_overflow: bool) -> (cursor: float64, samples_written: s64)`
- `accumulate_stream :: (stream: *Sound_Stream, accumulator: [] Audio_Dest_Sample, num_samples: s64) -> bool`
- `advance_history_cursor :: ()`
- `fill_sample_buffer :: (buffer: string)`
- `update_from_async_thread :: (actually_async: bool)`
- `async_audio_update :: (thread: *Thread) -> s64`
- `Decoder_Type :: enum`
- `Cached_Decoder_Page :: struct`
- `Cached_Decoder :: struct`
- `Cached_Ogg_Decoder :: struct`
- `CACHED_OGG_DECODE_NUM_THREADS :: 1`
- `get_base_address :: (decoder: *Cached_Decoder, play_cursor: s64) -> s64`
- `unmark_all_pages :: (using decoder: *Cached_Decoder)`
- `mark_relevant_pages :: (using decoder: *Cached_Decoder, start: s64, end: s64)`
- `free_unmarked_pages :: (using decoder: *Cached_Decoder)`
- `find_page_by_base_address :: (using decoder: *Cached_Decoder, start_address: s64) -> *Cached_Decoder_Page`
- `find_page_containing_cursor :: (using decoder: *Cached_Decoder, play_cursor: s64) -> (found: bool, start_address: s64, end_address: s64, data: *void)`
- `deinit :: (using decoder: *Cached_Decoder)`
- `create_ogg_decoder :: (data: *Sound_Data, _page_size_in_samples: s64) -> *Cached_Decoder`
- `deinit_proc_ogg :: (decoder: *Cached_Decoder)`
- `make_page_ogg :: (using decoder: *Cached_Decoder) -> *Cached_Decoder_Page`
- `recompute_page_ogg :: (using decoder: *Cached_Decoder, _page: *Cached_Decoder_Page)`
- `start_vorbis_file :: (using decoder: *Cached_Ogg_Decoder) -> bool`
- `get_compressed_rate_channels_and_size :: (s: string) -> (num_samples: s64, num_channels: s64, sampling_rate: s64)`
- `ADPCM_BLOCKS_PER_PAGE :: 3`
- `Adpcm_Decoder :: struct`
- `create_adpcm_decoder :: (data: *Sound_Data) -> *Adpcm_Decoder`
- `make_page_adpcm :: (using decoder: *Cached_Decoder) -> *Cached_Decoder_Page`
- `recompute_page_adpcm :: (using decoder: *Cached_Decoder, page: *Cached_Decoder_Page)`


- [OS == .WINDOWS] `CONFIG :: (speaker_config: s32) -> DSSPEAKER`
- [OS == .WINDOWS] `DSBCAPS_CTRLVOLUME :: 0x80`
- [OS == .WINDOWS] `DSBCAPS_GETCURRENTPOSITION2:: 0x10000`
- [OS == .WINDOWS] `DSBCAPS_GLOBALFOCUS :: 0x08000`
- [OS == .WINDOWS] `DSBCAPS_PRIMARYBUFFER :: 0x1`
- [OS == .WINDOWS] `DSBPLAY_LOOPING :: 0x01`
- [OS == .WINDOWS] `DSBSTATUS_BUFFERLOST :: 0x002`
- [OS == .WINDOWS] `DSBSTATUS_LOCHARDWARE :: 0x008`
- [OS == .WINDOWS] `DSBSTATUS_LOCSOFTWARE :: 0x010`
- [OS == .WINDOWS] `DSBSTATUS_LOOPING :: 0x004`
- [OS == .WINDOWS] `DSBSTATUS_PLAYING :: 0x001`
- [OS == .WINDOWS] `DSBSTATUS_TERMINATED :: 0x020`
- [OS == .WINDOWS] `DSBUFFERCAPS :: struct`
- [OS == .WINDOWS] `DSBUFFERDESC :: struct`
- [OS == .WINDOWS] `DSSPEAKER :: enum`
- [OS == .WINDOWS] `DS_OK :: 0`
- [OS == .WINDOWS] `Enumeration_Callback_Data :: struct`
- [OS == .WINDOWS] `FILL_BUFFER_SIZE_IN_SAMPLES :: 24000`
- [OS == .MACOS] `FILL_BUFFER_SIZE_IN_SAMPLES :: OUTPUT_SAMPLING_RATE / 2`
- [OS == .WINDOWS] `GEOMETRY :: (speaker_config: s32) -> u8`
- [OS == .WINDOWS] `IDirectSound8 :: struct`
- [OS == .WINDOWS] `IDirectSound8Vtable :: struct`
- [OS == .WINDOWS] `IDirectSoundBuffer8 :: struct`
- [OS == .WINDOWS] `IDirectSoundBuffer8Vtable :: struct`
- [OS == .WINDOWS] `KSAUDIO_SPEAKER_5POINT1 :: SPEAKER_FRONT_LEFT | SPEAKER_FRONT_RIGHT | SPEAKER_FRONT_CENTER | SPEAKER_LOW_FREQUENCY | SPEAKER_BACK_LEFT | SPEAKER_BACK_RIGHT`
- [OS == .WINDOWS] `KSAUDIO_SPEAKER_5POINT1_SURROUND :: SPEAKER_FRONT_LEFT | SPEAKER_FRONT_RIGHT | SPEAKER_FRONT_CENTER | SPEAKER_LOW_FREQUENCY | SPEAKER_SIDE_LEFT | SPEAKER_SIDE_RIGHT`
- [OS == .WINDOWS] `KSAUDIO_SPEAKER_7POINT1        :: SPEAKER_FRONT_LEFT | SPEAKER_FRONT_RIGHT | SPEAKER_FRONT_CENTER | SPEAKER_LOW_FREQUENCY | SPEAKER_BACK_LEFT | SPEAKER_BACK_RIGHT | SPEAKER_FRONT_LEFT_OF_CENTER | SPEAKER_FRONT_RIGHT_OF_CENTER`
- [OS == .WINDOWS] `KSAUDIO_SPEAKER_7POINT1_SURROUND :: SPEAKER_FRONT_LEFT | SPEAKER_FRONT_RIGHT | SPEAKER_FRONT_CENTER | SPEAKER_LOW_FREQUENCY | SPEAKER_BACK_LEFT | SPEAKER_BACK_RIGHT | SPEAKER_SIDE_LEFT | SPEAKER_SIDE_RIGHT`
- [OS == .WINDOWS] `KSAUDIO_SPEAKER_MONO :: SPEAKER_FRONT_CENTER`
- [OS == .WINDOWS] `KSAUDIO_SPEAKER_QUAD :: SPEAKER_FRONT_LEFT | SPEAKER_FRONT_RIGHT | SPEAKER_BACK_LEFT | SPEAKER_BACK_RIGHT`
- [OS == .WINDOWS] `KSAUDIO_SPEAKER_STEREO :: SPEAKER_FRONT_LEFT | SPEAKER_FRONT_RIGHT`
- [OS == .WINDOWS] `KSAUDIO_SPEAKER_SURROUND :: SPEAKER_FRONT_LEFT | SPEAKER_FRONT_RIGHT | SPEAKER_FRONT_CENTER | SPEAKER_BACK_CENTER`
- [OS == .LINUX] `MY_SAMPLING_RATE :: 48000`
- [OS == .WINDOWS] `OUTPUT_SAMPLING_RATE :: 44100`
- [OS == .MACOS] `OUTPUT_SAMPLING_RATE :: 44100`
- [OS == .WINDOWS] `PRIMARY_BUFFER_SIZE_IN_BYTES :: 0`
- [OS == .WINDOWS] `SPEAKER_BACK_CENTER :: 0x100`
- [OS == .WINDOWS] `SPEAKER_BACK_LEFT :: 0x10`
- [OS == .WINDOWS] `SPEAKER_BACK_RIGHT :: 0x20`
- [OS == .WINDOWS] `SPEAKER_FRONT_CENTER :: 0x4`
- [OS == .WINDOWS] `SPEAKER_FRONT_LEFT :: 0x1`
- [OS == .WINDOWS] `SPEAKER_FRONT_LEFT_OF_CENTER :: 0x40`
- [OS == .WINDOWS] `SPEAKER_FRONT_RIGHT :: 0x2`
- [OS == .WINDOWS] `SPEAKER_FRONT_RIGHT_OF_CENTER :: 0x80`
- [OS == .WINDOWS] `SPEAKER_LOW_FREQUENCY :: 0x8`
- [OS == .WINDOWS] `SPEAKER_SIDE_LEFT :: 0x200`
- [OS == .WINDOWS] `SPEAKER_SIDE_RIGHT :: 0x400`
- [OS == .WINDOWS] `SPEAKER_TOP_BACK_CENTER :: 0x10000`
- [OS == .WINDOWS] `SPEAKER_TOP_BACK_LEFT :: 0x8000`
- [OS == .WINDOWS] `SPEAKER_TOP_BACK_RIGHT :: 0x20000`
- [OS == .WINDOWS] `SPEAKER_TOP_CENTER :: 0x800`
- [OS == .WINDOWS] `SPEAKER_TOP_FRONT_CENTER :: 0x2000`
- [OS == .WINDOWS] `SPEAKER_TOP_FRONT_LEFT :: 0x1000`
- [OS == .WINDOWS] `SPEAKER_TOP_FRONT_RIGHT :: 0x4000`
- [OS == .WINDOWS] `WAVEFORMATEXTENSIBLE :: struct`
- [OS == .WINDOWS] `WAVE_FORMAT_EXTENSIBLE :: 0xFFFE`
- [OS == .WINDOWS] `backend_count_buffered_bytes :: () -> (currently_buffered: s64, minimum_prebuffered: s64)`
- [OS == .LINUX] `backend_count_buffered_bytes :: () -> (buffered_bytes: s64, minimum_prebuffered_bytes: s64)`
- [OS == .MACOS] `backend_count_buffered_bytes :: () -> (currently_buffered: s64, minimum_prebuffered_bytes: s64)`
- [OS == .WINDOWS] `backend_get_devices :: () -> [] Output_Device`
- [OS == .LINUX] `backend_get_devices :: () -> [] Output_Device`
- [OS == .MACOS] `backend_get_devices :: () -> [] Output_Device`
- [OS == .WINDOWS] `backend_init :: (config: Sound_Player_Config) -> Backend_Properties`
- [OS == .LINUX] `backend_init :: (config: Sound_Player_Config) -> Backend_Properties`
- [OS == .MACOS] `backend_init :: (config: Sound_Player_Config) -> Backend_Properties`
- [OS == .WINDOWS] `backend_lock_fill_regions :: (bytes_to_lock: s64) -> Fill_Region_Result, bool`
- [OS == .LINUX] `backend_lock_fill_regions :: (bytes_to_lock: s64) -> Fill_Region_Result, bool`
- [OS == .MACOS] `backend_lock_fill_regions :: (bytes_to_lock: s64) -> Fill_Region_Result, bool`
- [OS == .WINDOWS] `backend_needs_async_update_from_main_thread :: () -> bool`
- [OS == .LINUX] `backend_needs_async_update_from_main_thread :: () -> bool`
- [OS == .MACOS] `backend_needs_async_update_from_main_thread :: () -> bool`
- [OS == .WINDOWS] `backend_play :: ()`
- [OS == .LINUX] `backend_play :: ()`
- [OS == .MACOS] `backend_play :: ()`
- [OS == .WINDOWS] `backend_release_fill_regions :: (result: Fill_Region_Result)`
- [OS == .LINUX] `backend_release_fill_regions :: (result: Fill_Region_Result)`
- [OS == .MACOS] `backend_release_fill_regions :: (result: Fill_Region_Result)`
- [OS == .WINDOWS] `backend_shutdown :: ()`
- [OS == .LINUX] `backend_shutdown :: ()`
- [OS == .MACOS] `backend_shutdown :: ()`
- [OS == .WINDOWS] `get_bytes_between :: (front: $T, rear: T, buffer_len: T) -> T`
- [OS == .MACOS] `get_bytes_between :: (front: $T, rear: T, buffer_len: T) -> T`
- [OS == .WINDOWS] `init_KSDATAFORMAT_SUBTYPE_PCM :: (dest: *GUID)`
- [OS == .WINDOWS] `init_wave_format :: (using w: *Waveformatex, nchannels: s16)`

## Public aggregate shapes and enum values

### Windows DirectSound ABI shapes

#### `DSSPEAKER` (enum, Windows)

```jai
DSSPEAKER :: enum {
    HEADPHONE        :: 1;
    MONO             :: 2;
    QUAD             :: 3;
    STEREO           :: 4;
    SURROUND         :: 5;
    _5POINT1         :: 6;
    _5POINT1_BACK    :: _5POINT1;
    _5POINT1_SURROUND :: 9;
    _7POINT1         :: 7;
    _7POINT1_SURROUND :: 8;
    _7POINT1_WIDE    :: _7POINT1;
}
```
#### `DSBUFFERCAPS` (struct, Windows)

```jai
DSBUFFERCAPS :: struct {}
```
#### `WAVEFORMATEXTENSIBLE` (struct, Windows)

```jai
WAVEFORMATEXTENSIBLE :: struct {
    Format: Waveformatex;
    wValidBitsPerSample: s16;
    dwChannelMask: s32;
    SubFormat: GUID;
}
```
#### `DSBUFFERDESC` (struct, Windows)

```jai
DSBUFFERDESC :: struct {
    dwSize: s32;
    dwFlags: s32;
    dwBufferBytes: s32;
    dwReserved: s32;
    lpwfxFormat: *Waveformatex;
    guid3DAlgorithm: GUID;
}
```
#### `Enumeration_Callback_Data` (struct, Windows)

```jai
Enumeration_Callback_Data :: struct {
    outer_context: #Context;
    devices: [..] Output_Device;
}
```
#### `IDirectSound8` (struct, Windows)

```jai
IDirectSound8 :: struct {
    #as unknown: IUnknown;
    #overlay (unknown.vtable) using vtable: *IDirectSound8Vtable;
}
```
#### `IDirectSoundBuffer8` (struct, Windows)

```jai
IDirectSoundBuffer8 :: struct {
    #as unknown: IUnknown;
    #overlay (unknown.vtable) using vtable: *IDirectSoundBuffer8Vtable;
}
```
#### `IDirectSoundBuffer8Vtable` (struct, Windows)

```jai
IDirectSoundBuffer8Vtable :: struct {
    #as using base: IUnknown_VTable;
    GetCaps: (buffer: *IDirectSoundBuffer8, caps: *DSBUFFERCAPS) #c_call;
    GetCurrentPosition: (buffer: *IDirectSoundBuffer8, pdwCurrentPlayCursor: *s32, pdwCurrentWriteCursor: *s32) -> s32 #c_call;
    GetFormat: *void;
    GetVolume: *void;
    GetPan: *void;
    GetFrequency: *void;
    GetStatus: (buffer: *IDirectSoundBuffer8, pdwStatus: *s32) -> s32 #c_call;
    Initialize: *void;
    Lock: (buffer: *IDirectSoundBuffer8, dwOffset: s32, dwBytes: s32, pvAudioPtr1: **void, dwAudioBytes1: *s32, pvAudioPtr2: **void, dwAudioBytes2: *s32, dwFlags: s32) -> s32 #c_call;
    Play: (buffer: *IDirectSoundBuffer8, dwReserved1: s32, dwPriority: s32, dwFlags: s32) -> s32 #c_call;
    SetCurrentPosition: *void;
    SetFormat: (buffer: *IDirectSoundBuffer8, format: *Waveformatex) -> s32 #c_call;
    SetVolume: *void;
    SetPan: *void;
    SetFrequency: *void;
    Stop: (buffer: *IDirectSoundBuffer8) -> s32 #c_call;
    Unlock: (buffer: *IDirectSoundBuffer8, pvAudioPtr1: *void, dwAudioBytes1: s32, pvAudioPtr2: *void, dwAudioBytes2: s32) -> s32 #c_call;
    Restore: (buffer: *IDirectSoundBuffer8) -> s32 #c_call;
    SetFX: *void;
    AcquireResources: *void;
    GetObjectInPath: *void;
}
```
#### `IDirectSound8Vtable` (struct, Windows)

```jai
IDirectSound8Vtable :: struct {
    #as using base: IUnknown_VTable;
    CreateSoundBuffer: (dsound: *IDirectSound8, desc: *DSBUFFERDESC, buffer: **IDirectSoundBuffer8, outer: **void = null) -> s32 #c_call;
    GetCaps: *void;
    DuplicateSoundBuffer: *void;
    SetCooperativeLevel: (dsound: *IDirectSound8, hwnd: HWND, level: s32) -> s32 #c_call;
    Compact: *void;
    GetSpeakerConfig: (dsound: *IDirectSound8, config: *s32) -> s32 #c_call;
    SetSpeakerConfig: *void;
    Initialize: *void;
}
```
Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `Adpcm_Decoder`

```jai
Adpcm_Decoder :: struct {
    using base: Cached_Decoder;
}
```
### `Audio_Channel`

```jai
Audio_Channel :: enum u16 {
    FRONT_LEFT  :: 0;
    FRONT_RIGHT :: 1;
    CENTER      :: 2;
    SUBWOOFER   :: 3;
    REAR_LEFT   :: 4;
    REAR_RIGHT  :: 5;
    EXTEND_LEFT :: 6;
    EXTEND_RIGHT:: 7;
}
```
### `Audio_Dest_Sample`

```jai
Audio_Dest_Sample :: struct {
    using base: Audio_Sample_Base;
}
```
### `Audio_Sample_Base`

```jai
Audio_Sample_Base :: struct {
    channels: [MAX_CHANNELS_PER_SOUND] float;
}
```
### `Audio_Source_Sample`

```jai
Audio_Source_Sample :: struct {
    using base: Audio_Sample_Base;
}
```
### `Backend_Properties`

```jai
Backend_Properties :: struct {
    initted := false;

    num_channels: s32;
    channel_names: [] string;

    output_sampling_rate: s32;
}
```
### `Buffer_Bounds`

```jai
Buffer_Bounds :: struct {
    start, end: s64;
    memory: *void;
}
```
### `Cached_Decoder`

```jai
Cached_Decoder :: struct {
    type: Decoder_Type;

    pages: [..] *Cached_Decoder_Page;
    issued_start_addresses: [..] s64;

    page_size_in_samples:           s64 = -1;
    uncompressed_length_in_samples: s64 = 0;

    num_channels  : s64;
    sampling_rate : s64;

    sound_data: *Sound_Data;
    stream:     *Sound_Stream;

    do_not_queue := true;

    deinit_proc: (decoder: *Cached_Decoder) -> ();

    add_decode_queue_item: (page: *Cached_Decoder_Page) -> ();

    recompute_page : (decoder: *Cached_Decoder, page: *Cached_Decoder_Page) -> ();
    make_page:       (decoder: *Cached_Decoder) -> *Cached_Decoder_Page;
}
```
### `Cached_Decoder_Page`

```jai
Cached_Decoder_Page :: struct {
    type:  Decoder_Type;
    owner: *Cached_Decoder;

    start_address: s64 = -1;
    end_address:   s64 = -1;
    page_index:    s64 = -1;

    data: *void;

    buffer_length_in_bytes: s64;

    num_samples_contained:  s64;

    marked := false;
}
```
### `Cached_Ogg_Decoder`

```jai
Cached_Ogg_Decoder :: struct {
    using base: Cached_Decoder;

    vorbis_file: *stb_vorbis;

    seek_base     : s64;
    opus_pre_skip : s64;

    bytes_per_mono_sample : s64;
    uncompressed_length_in_bytes : s64;

    bitrate_low     : s64;
    bitrate_nominal : s64;
    bitrate_high    : s64;
}
```
### `Decoder_Type`

```jai
Decoder_Type :: enum {
    OGG   :: 0;
    ADPCM :: 1;
}
```
### `Fill_Region_Result`

```jai
Fill_Region_Result :: struct {
    buffer0: string;
    buffer1: string;
}
```
### `Output_Device`

```jai
Output_Device :: struct {
    name: string;
    hints: [] string;
    device_index: s64;
}
```
### `Scale_Mapping`

```jai
Scale_Mapping :: struct {

    source_scale_for_this_output_index:              [OUTPUT_CHANNELS_MAX] float;
    interpolated_source_scale_for_this_output_index: [OUTPUT_CHANNELS_MAX] float;
    dvolume_dsample:                                 [OUTPUT_CHANNELS_MAX] float;
}
```
### `Sound_Category`

```jai
Sound_Category :: enum u32 {
    GENERAL_SFX :: 0;
    AMBIENCES   :: 1;
    FOOTSTEPS   :: 2;
    UI          :: 3;
    MUSIC       :: 4;
}
```
### `Sound_Data`

```jai
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

    nsamples_times_nchannels : s64;

    nchannels: u16 = 1;
    sampling_rate: u32 = 44100;

    buffer: string;
    samples: *s16;

    volume_scale : float = 1;
    silence_before_repeat : float = 0;

	filename_to_stream_from : string;
    source_directions: [MAX_CHANNELS_PER_SOUND] Source_Direction;

    nBlockAlign : s32 = 0;
    wSamplesPerBlock : s32 = 0;
}
```
### `Sound_Player_Config`

```jai
Sound_Player_Config :: struct {
    update_from_a_thread := true;
    audible_when_window_is_not_in_focus := false;
    keep_sounds_alive_by_default := true;

    seconds_to_fill_ahead := 2.4 / 60.0;

    set_async_thread_priority: () -> ();

    release_asset: (stream: *Sound_Stream, data: *Sound_Data);
}
```
### `Sound_Stream`

```jai
Sound_Stream :: struct {

    user_data_u64:      u64;
    user_data_pointer: *void;

    manager_id: s64;
    entity_id:  s64;

    sound_name: string;

    user_flags: enum u32 {
        REPEATING              :: 0x1;
        SPATIALIZED            :: 0x2;
        MUSIC                  :: 0x10;
        VOICEOVER              :: 0x20;
        IS_FOOTSTEP            :: 0x40;
        DO_NOT_KILL            :: 0x80;
        PAUSED_DUE_TO_MENU     :: 0x100;
        WAS_PAUSED_DUE_TO_MENU :: 0x200;
        AMBIENT                :: 0x400;
    }

    internal_flags: enum u32 {
        FIRST_VOLUME_UPDATE               :: 0x2;
        WAITING_FOR_INITIAL_DECODER_PAGES :: 0x4;
        FADING_OUT                        :: 0x8;
    }

    num_channels : s32 = 2;
    sound_data:  *Sound_Data;
    decoder:     *Cached_Decoder;

    category: Sound_Category;

    repeat_start_position: s64;
    repeat_end_position:   s64 = -1;

    universe_volume_scale := 1.0;
    user_volume_scale := 1.0;

    play_cursor: float64 = 0;
    silent_samples_at_start_of_play: s64 = 0;

    marked              := false;
    inaudible           := false;

    position:    Vector3;
    orientation: Quaternion;
    inner_radius := 0.5;
    outer_radius := 10.0;

    rate_scale   := 1.0;

    samples_streamed_since_entity_update: s64;

    max_gain: float;
    last_plane_dir: Vector2;
    debug_display_volume: float = 0;

    silent_this_frame  := false;
    debug_watch        := false;

    current_rate: float = 1;
    desired_rate: float = 1;

    input_scale_mappings: [MAX_CHANNELS_PER_SOUND] Scale_Mapping;

    initial_samples_fetched: s64;
    samples_needed_to_start_playing: s64;
}
```
### `Source_Direction`

```jai
Source_Direction :: enum u16 {
    MONO                      :: 0;
    LEFT                      :: 1;
    RIGHT                     :: 2;
    FRONT_LEFT                :: 3;
    FRONT_RIGHT               :: 4;
    REAR_LEFT                 :: 5;
    REAR_RIGHT                :: 6;
    CENTER_FORWARD            :: 7;
    SUBWOOFER_CENTER_FORWARD  :: 8;
    EXTEND_LEFT               :: 9;
    EXTEND_RIGHT              :: 10;
}
```
### `Vbap_Arc`

```jai
Vbap_Arc :: struct {
    speaker1: Audio_Channel;
    speaker2: Audio_Channel;

    l_inverse: Matrix2;
}
```
## Contracts and constraints

This is a cross-platform audio playback system. Initialize it before loading/playing audio and shut it down before process exit; `Sound_Data` and its backing buffer must remain valid while any stream refers to them. Configure a stream before `start_playing`; once it is active, mutate its public fields only where an exported procedure documents that operation as synchronized. The entity-update calls do not provide a synchronization window for caller edits. The module supports device enumeration, multiple channels, categories, repeat/rate/position controls, and spatial panning; output devices, backends, thread behavior, and supported formats depend on OS/build configuration. The module parameters tune category capacity and verbose diagnostics.

## Public behavior and lifecycle

### Initialization, devices, and shutdown

`sound_player_init(config)` stores the supplied configuration, initializes its private state and decoder queues, asks the selected platform backend to create an output device, initializes panning for the selected output channel count, and starts the mixer thread when `update_from_a_thread` is true. It returns false when the backend reports that it did not initialize; callers should treat playback as unavailable in that state. The selected output sample rate and channel count are backend properties; `get_audio_sampling_rate()` returns the actual backend output rate. `get_devices()` returns the selected backend's device list. Each `Output_Device` has a name, string hints, and backend-specific `device_index`.

Call `sound_player_shutdown()` after use. It asks the mixer thread to exit, shuts down the backend, releases all live streams and their assets, then releases retired decoders. The configured `release_asset(stream, data)` callback is called once for each stream as that stream is freed, including streams sharing one `Sound_Data`; it is suitable for per-stream reference counting and does not mean no other stream uses the data. Do not assume that shutdown can cancel an in-flight decoder operation; source code retains decoders until their issued work is drained.

`Sound_Player_Config` defaults are `update_from_a_thread = true`, `audible_when_window_is_not_in_focus = false`, `keep_sounds_alive_by_default = true`, and `seconds_to_fill_ahead = 2.4/60` seconds. The optional `set_async_thread_priority` callback runs when the mixer thread begins. Focus-independent playback is backend-dependent; Android logs a warning when requested, and it does not provide background-service behavior. The module uses signed 16-bit output samples and requests 44.1 kHz on Windows, macOS, and Android; Linux requests 48 kHz. Backend channel counts and device negotiation remain platform-specific.

The exported decode-queue declarations are public names because of their declaration scope, but are low-level implementation-facing state. The live stream array, backend state, master volume storage, listener storage, and mix-level storage are not exported. The `retired_decoders` array and the exported decode queue types/functions are visible, but callers should not mutate internal queue state while playback is active. No caller-accessible synchronization primitive for direct live-stream field edits is exported.

### Stream ownership, start, stop, and entity updates

`make_stream(data, category)` allocates a stream associated with the supplied `Sound_Data`; `category` defaults to `GENERAL_SFX`. It does not start playback. Configure public stream fields before calling `start_playing(stream)`. Starting appends the pointer to the live-stream collection; the module does not reject duplicate starts, enforce a stream limit, or take a copy of the stream or `Sound_Data`. Callers must not free or reuse a stream after starting it; the module frees it when stopped or when entity updates remove it.

`stop_stream_abruptly(id)` removes every live stream whose `entity_id` equals `id`; it does not filter by `manager_id`. `stop_stream_abruptly(data)` removes every live stream pointing to that exact `Sound_Data`. `stop_all_sounds_abruptly(manager_id)` removes every live stream with that manager ID. Removal happens before the release callback, so the callback may safely perform another stop operation. Release callbacks and stream resource disposal are performed outside the mixer lock to avoid blocking audio mixing. There is no public natural-completion callback.

`pre_entity_update()` sets each stream's `marked` field to the configured default and flags a stream for fade-out once its play cursor reaches its repeat end. Callers using `keep_sounds_alive_by_default = false` must mark streams they want to retain during their entity update. `post_entity_update(dt)` updates rate and panning, gathers completed decoder pages, and frees unmarked streams unless `DO_NOT_KILL` is set. A fading stream is unmarked after the mixer reports it silent for a frame, so normal entity marking does not keep it alive indefinitely. `update(dt)` is the convenience sequence of these two calls; omitted `dt` is represented by positive float32 infinity and causes elapsed wall time to be measured internally.

These functions each synchronize their own internal work, but they do not hold synchronization across the caller's entity-update interval: `pre_entity_update()` returns before the caller edits/marks streams, and `post_entity_update()` begins a separate synchronized phase. They are lifecycle-ordering helpers, not a bracketed critical section. When mixing can occur concurrently, direct writes to active `Sound_Stream` fields between these calls can race with the mixer. Set fields before starting a stream, or use exported operations such as `set_repeating` and the stop procedures where applicable; this module exports no general setter or caller synchronization handle for every mutable field. The safe behavior of unsynchronized active-field writes is unspecified.

`set_repeating(stream, true)` sets `REPEATING` and resets `repeat_end_position` to the full data length in mono-frame units; false clears the flag but leaves the end position unchanged. The repeat interval is `[repeat_start_position, repeat_end_position)`. Cursor wrapping occurs when the integer cursor reaches or passes the end, retaining overshoot through a cycling modulo and adding the repeat start. `silence_before_repeat` is a public data field but the playback path does not apply it. Invalid or empty intervals produce no samples; callers should provide a positive interval. A stream beginning before sample zero or continuing beyond the nonrepeating bounds advances through silence until it enters valid source data or is retired.

### Gain, panning, and rate

`perceptual_to_linear(x)` computes `x^(1/(2*log10(2)))` (approximately `x^1.66096`), so `0.5` maps to about `0.3162`, and doubling the perceptual value corresponds to twice the nominal loudness. No input clamp is applied. Negative values are outside the meaningful gain range and may yield a non-finite result; values above one are accepted and amplify.

The listener basis is +X forward, +Y left, +Z up. `update_listener(position, orientation)` stores the camera position and rotates those basis vectors by the supplied quaternion; panning is updated during stream/entity updates. The source computes spatial distance from listener to stream position, then linearly scales perceptual gain from zero at `outer_radius` to full at `inner_radius`, clamping the interpolation factor to [0,1]. The defaults declared in the public inventory are the compile-time constants; `Sound_Stream`'s own per-instance radius fields have their own aggregate defaults and those instance values drive attenuation. `SPATIALIZED` gates distance attenuation. Non-spatial streams use their configured perceptual gains, category mix level, and master volume.

The module supports mono and multichannel source mappings and stereo, 5.1, or 7.1 output layouts. Two-channel output uses left/right panning; six- and eight-channel output uses the public VBAP direction/arc data. Other initialized output channel counts are not accepted by the mixing path. Gain changes are interpolated over samples, with a maximum output-scale change of 1/30 per second; rate changes are also slewed. Menu pause flags stop the source cursor immediately and resume without a rate warm-up. `set_master_volume` stores its argument without clamping.

### Loading and source data

`load_audio_file(filename, optional)` reads the entire file and passes its storage to `load_audio_data`; an unreadable file logs an error unless `optional` is true and returns an uninitialized value. `load_audio_data(name, bytes)` recognizes RIFF/WAVE and Ogg container signatures. It accepts PCM WAV only at 16 bits per sample and DVI ADPCM WAV; Ogg is accepted for compressed decoding. Unsupported formats or malformed headers log errors and may return a partially populated but `loaded == false` record. The Ogg signature check does not guarantee that the stream is decodable; decoder initialization supplies that result.

`Sound_Data.buffer` records the supplied byte string, and `samples` points into that buffer for linear PCM. `load_audio_data` does not copy its input, so caller-provided bytes must remain stable and alive for the full data/stream lifetime. `load_audio_file` uses the allocation returned by the file-reading operation. `nsamples_times_nchannels` counts interleaved scalar samples; duration/frame count is that value divided by channel count. `nchannels` defaults to 1 and `sampling_rate` to 44100. For Ogg, `make_stream` opens the decoder and replaces the channel/sample metadata with values read from the compressed stream.

`release_sound_data(data)` stops every stream that plays `data` (as `stop_stream_abruptly(data)`, so the release callback runs for each), frees `data.buffer` with `context.allocator` and resets `data` to an empty record. `replace_sound_data(data, replacement)` does the same stop and free and then stores `replacement` in `data`, so pointers to the `Sound_Data` stay valid; a `replacement` that is not `loaded` is refused with `false` and nothing is stopped. Both are how a hot-reloaded sound is freed or swapped. A stream that was made but not started is not known to the player and must be started or dropped first.

### Compressed pages and observable limits

`Cached_Decoder_Page` addresses are sample-frame indices, with `end_address` exclusive. `mark_relevant_pages(start,end)` treats the requested interval as half-open, ignores wholly negative ranges, clamps a negative start to zero, and limits work to the known decoded length. Existing pages are marked for retention; already-issued page addresses are not queued twice. `find_page_containing_cursor` succeeds only when a cached page contains that cursor. Unmarked cached pages are freed when page retention is refreshed. There is no public cancellation API; retired decoders remain alive until their issued-address list is empty.

Ogg decoder pages are 8,000 sample frames in `make_stream`; source metadata and length are obtained at decoder creation. The decoder attempts a seek to each requested page start, decodes interleaved signed 16-bit samples, and treats a nonpositive decode/seek result as an error. `seek_base` and `opus_pre_skip` exist as public fields on `Cached_Ogg_Decoder,` but the decoder initializes `seek_base` to zero and does not apply Opus pre-skip. The decoder opens through the Vorbis interface; general Opus support is not guaranteed. With the exported threaded setting enabled, the page-processing function currently returns without decoding or completing pages, so threaded Ogg playback can remain waiting for its initial data and issued work can remain outstanding. ADPCM differs: its decoder opts out of queueing and decodes pages synchronously through the ADPCM decoder; page content is limited by the compressed input length. Do not infer an asynchronous ADPCM execution path from the queue callback field.

### Platform coverage and limits

Backend declarations are available for Windows, Linux, macOS, and Android in the supplied module. Windows and macOS request 44.1 kHz; Linux uses 48 kHz; Android requests 44.1 kHz signed 16-bit audio and receives its actual channel count from AAudio. The mixer writes interleaved signed 16-bit PCM frames. The fill-region API returns up to two backend-owned writable byte regions; the mixer fills both and the backend release operation commits/unlocks them. A successful/failed lock result is returned by the backend, but the common mixer path does not branch on that result before using the regions, so backend errors are not a documented recoverable playback signal.

PS5 and Nintendo Switch 2 backends are referenced by conditional hooks, but their signatures, output formats, and lifecycle behavior are not specified. The Android backend is loaded for Android targets; lifecycle recovery when backgrounding is incomplete. Do not infer unsupported platform behavior from the listed conditional backend hooks.

### Output-history cursor

`advance_history_cursor()` advances audio-history diagnostics to the next recording interval, resetting that interval's extrema and partial-sample progress. It returns no value and accepts no arguments. Direct calls are not guarded by the history recording enable/pause state. The history data and counters are private, with no exported accessor; their internal organization and capacity are not compatibility requirements. Callers must serialize this operation with the audio mixer.

`arcs_6ch` and `arcs_8ch` are public inferred-initialized variables (`:=`), not constants. Their arrays contain the default speaker-pair topology used by the panning data; callers can edit the arrays, which changes later processing that consults them.

### Spatial panning details

For spatialized sources, `get_plane_dir(stream, position)` projects the source-to-listener vector onto the listener's horizontal plane, normalizes it, and returns its dot products against listener-forward and listener-left. With no listener or without `SPATIALIZED`, it returns `(1, 0)`. A source exactly on the listener's vertical axis has a zero projected vector; the module does not define a special fallback for normalization of that degenerate input.

`add_vbap_gain_for_direction` evaluates the configured adjacent speaker pairs and selects the pair whose two unconstrained gains have the greatest minimum component. It adds positive components only; if there are no arcs it makes no change. This selection also applies when the direction lies outside the speaker coverage, so a negative component is suppressed rather than producing negative output gain. The public 6-channel and 8-channel arc arrays are therefore behaviorally significant and must have valid speaker indices and invertible speaker-pair bases when edited.

Stereo output is a separate equal-power-style approximation. Mono panning uses clamped nonnegative left/right directional dots, adds a shared stereo base, and normalizes the pair before applying gain. Multichannel source mapping to stereo treats each source channel's configured direction independently; the public closeness helpers compute planar distance and move each stereo dot toward unity inside `min(VERY_CLOSE, inner_radius * 0.75)`, with the interpolation clamped to [0,1]. In multichannel output, mono spatial sources use the selected VBAP pair; near the listener, non-subwoofer speakers are blended toward equal contribution and normalized. `skip_this_output_channel` returns true only for rear-left and rear-right when `IS_FOOTSTEP` is set; the near-source fill respects that rule. Subwoofer is excluded from this fill.

Stereo-source positional separation is not part of the active contract: the current stereo source path uses one stream position, and the left/right positional override is not implemented. `AMBIENT`, UI, music, non-spatial streams, and streams without a listener use locked mappings. For mono music on 5.1/7.1 output, the channel gains are front 0.8, center/subwoofer 1.0, and rear 0.5 times the supplied gain. Stereo music uses front 1.0, center/subwoofer 0.7, and rear 0.5; left input maps to left-side speakers, right input to right-side speakers, and both contribute to center/subwoofer. On stereo output, mono locked sources are split equally and stereo locked sources remain left/right. Other multichannel locked sources on stereo output are assigned left or right according to the sign of each configured source direction's left coordinate.

The direct source-direction mapping for six/eight-channel output maps mono and center-forward to center, left/right to front-left/front-right, rear directions to rear speakers, subwoofer-center-forward to subwoofer, and extension directions to extension speakers only on eight-channel output (otherwise the corresponding rear speaker). Other enum values assert and report an unsuccessful mapping. The supported output counts in the common mixer are 2, 6, and 8. Seven-channel source data fits the public eight-source-channel capacity, but the module provides no distinct center-rear source direction; do not infer a dedicated seven-channel mapping. The music-specific multi-channel helper accepts only one- or two-channel source data. The `Sound_Data.filename_to_stream_from` string is public metadata, but no playback or decoder behavior in this module reads it.

### Buffer and cursor edge behavior

`get_buffer_bounds` returns `(false, false)` when a compressed page containing the requested frame is unavailable. On success it fills a half-open frame range and returns `(true, true)` for both in-memory and compressed data; a nonnegative repeat end truncates the range. `no_overflow` therefore does not signal a second failure mode in the implementation. When interpolation reaches the final frame of a successful page with `no_overflow == true`, the next interpolation sample is the same final frame, so the value is held through the fractional boundary instead of reading beyond that page.

`accumulate_stream` returns `false` on every path; its boolean is not a completion or success indication. A null data pointer and a stream waiting for initial compressed pages leave the accumulator untouched. An empty or nonpositive repeat interval also produces no source samples. Otherwise the function mixes up to the requested output frame count, inserting silence while the source cursor lies before frame zero or beyond the available nonrepeating/repeat end. The source cursor advances through this silence. A missing compressed page changes the stream to silent processing for the affected update rather than returning a true status. Source samples are linearly interpolated between adjacent frames using the fractional cursor. Cursor rate can be positive or negative and is slewed toward the desired rate; a zero rate holds the cursor. The public return value does not report how many frames were written.

For repeating streams, the cursor wraps after reaching the configured repeat end and includes overshoot in the next position; the repeat interval must have positive length. `get_bytes_between(front, rear, buffer_len)` on Windows and macOS returns the forward circular distance from `rear` to `front`; equal cursors return zero. Its generic integer type is shared by all three arguments and the result.

### Platform-specific public behavior

Android does not add a caller-facing variant of the exported `Sound_Player` API; the AAudio routines are module-internal backend hooks selected for Android builds. That backend requests 44.1 kHz signed 16-bit output from an unspecified device with low-latency preference and uses the channel count returned by AAudio. Channel names are populated only when the platform channel mask is available and recognized. Requesting playback while the window is unfocused logs a warning: background playback is not supported by this module's Android backend. These hooks are internal implementation details, not caller contracts.

## Module parameters

```jai
#module_parameters (MAX_SOUND_CATEGORIES := DEFAULT_MAX_SOUND_CATEGORIES, VERBOSE := false) {
    DEFAULT_MAX_SOUND_CATEGORIES :: 64;
}
```
Platform coverage note: Nintendo, PS5 branches exist, but their declarations are not specified here; exports for those targets are unspecified.
