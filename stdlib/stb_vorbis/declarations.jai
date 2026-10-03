// Generated from docs/specs/stdlib/modules/stb-vorbis.md.
// Regenerate with tools/native_bindings/generate.py; do not hand-edit.

#scope_file
FILE :: struct {};
lib_stb_vorbis :: #library,system "stb_vorbis";
#scope_export

// Types and layouts

stb_vorbis :: struct {}

stb_vorbis_alloc :: struct {
    alloc_buffer: *u8;
    alloc_buffer_length_in_bytes: s32;
}

stb_vorbis_comment :: struct {
    vendor: *u8;

    comment_list_length: s32;
    comment_list: **u8;
}

stb_vorbis_info :: struct {
    sample_rate: u32;
    channels: s32;

    setup_memory_required: u32;
    setup_temp_memory_required: u32;
    temp_memory_required: u32;

    max_frame_size: s32;
}

STBVorbisError :: enum u32 {
    no_error :: 0;

    need_more_data :: 1;

    invalid_api_mixing :: 2;
    outofmem :: 3;
    feature_not_supported :: 4;
    too_many_channels :: 5;
    file_open_failure :: 6;
    seek_without_length :: 7;

    unexpected_eof :: 10;
    seek_invalid :: 11;

    invalid_setup :: 20;
    invalid_stream :: 21;

    missing_capture_pattern :: 30;
    invalid_stream_structure_version :: 31;
    continued_packet_flag_invalid :: 32;
    incorrect_stream_serial_number :: 33;
    invalid_first_page :: 34;
    bad_packet_type :: 35;
    cant_find_last_page :: 36;
    seek_failed :: 37;
    ogg_skeleton_not_supported :: 38;

    VORBIS__no_error :: no_error;

    VORBIS_need_more_data :: need_more_data;

    VORBIS_invalid_api_mixing :: invalid_api_mixing;
    VORBIS_outofmem :: outofmem;
    VORBIS_feature_not_supported :: feature_not_supported;
    VORBIS_too_many_channels :: too_many_channels;
    VORBIS_file_open_failure :: file_open_failure;
    VORBIS_seek_without_length :: seek_without_length;

    VORBIS_unexpected_eof :: unexpected_eof;
    VORBIS_seek_invalid :: seek_invalid;

    VORBIS_invalid_setup :: invalid_setup;
    VORBIS_invalid_stream :: invalid_stream;

    VORBIS_missing_capture_pattern :: missing_capture_pattern;
    VORBIS_invalid_stream_structure_version :: invalid_stream_structure_version;
    VORBIS_continued_packet_flag_invalid :: continued_packet_flag_invalid;
    VORBIS_incorrect_stream_serial_number :: incorrect_stream_serial_number;
    VORBIS_invalid_first_page :: invalid_first_page;
    VORBIS_bad_packet_type :: bad_packet_type;
    VORBIS_cant_find_last_page :: cant_find_last_page;
    VORBIS_seek_failed :: seek_failed;
    VORBIS_ogg_skeleton_not_supported :: ogg_skeleton_not_supported;
}

// Native entry points

stb_vorbis_close :: (f: *stb_vorbis) -> void #foreign lib_stb_vorbis;

stb_vorbis_decode_filename :: (filename: *u8, channels: *s32, sample_rate: *s32, output: **s16) -> s32 #foreign lib_stb_vorbis;

stb_vorbis_decode_frame_pushdata :: (f: *stb_vorbis, datablock: *u8, datablock_length_in_bytes: s32, channels: *s32, output: ***float, samples: *s32) -> s32 #foreign lib_stb_vorbis;

stb_vorbis_decode_memory :: (mem: *u8, len: s32, channels: *s32, sample_rate: *s32, output: **s16) -> s32 #foreign lib_stb_vorbis;

stb_vorbis_flush_pushdata :: (f: *stb_vorbis) -> void #foreign lib_stb_vorbis;

stb_vorbis_get_comment :: (f: *stb_vorbis) -> stb_vorbis_comment #foreign lib_stb_vorbis;

stb_vorbis_get_error :: (f: *stb_vorbis) -> s32 #foreign lib_stb_vorbis;

stb_vorbis_get_file_offset :: (f: *stb_vorbis) -> u32 #foreign lib_stb_vorbis;

stb_vorbis_get_frame_float :: (f: *stb_vorbis, channels: *s32, output: ***float) -> s32 #foreign lib_stb_vorbis;

stb_vorbis_get_frame_short :: (f: *stb_vorbis, num_c: s32, buffer: **s16, num_samples: s32) -> s32 #foreign lib_stb_vorbis;

stb_vorbis_get_frame_short_interleaved :: (f: *stb_vorbis, num_c: s32, buffer: *s16, num_shorts: s32) -> s32 #foreign lib_stb_vorbis;

stb_vorbis_get_info :: (f: *stb_vorbis) -> stb_vorbis_info #foreign lib_stb_vorbis;

stb_vorbis_get_sample_offset :: (f: *stb_vorbis) -> s32 #foreign lib_stb_vorbis;

stb_vorbis_get_samples_float :: (f: *stb_vorbis, channels: s32, buffer: **float, num_samples: s32) -> s32 #foreign lib_stb_vorbis;

stb_vorbis_get_samples_float_interleaved :: (f: *stb_vorbis, channels: s32, buffer: *float, num_floats: s32) -> s32 #foreign lib_stb_vorbis;

stb_vorbis_get_samples_short :: (f: *stb_vorbis, channels: s32, buffer: **s16, num_samples: s32) -> s32 #foreign lib_stb_vorbis;

stb_vorbis_get_samples_short_interleaved :: (f: *stb_vorbis, channels: s32, buffer: *s16, num_shorts: s32) -> s32 #foreign lib_stb_vorbis;

stb_vorbis_open_file :: (f: *FILE, close_handle_on_close: s32, error: *s32, alloc_buffer: *stb_vorbis_alloc) -> *stb_vorbis #foreign lib_stb_vorbis;

stb_vorbis_open_file_section :: (f: *FILE, close_handle_on_close: s32, error: *s32, alloc_buffer: *stb_vorbis_alloc, len: u32) -> *stb_vorbis #foreign lib_stb_vorbis;

stb_vorbis_open_filename :: (filename: *u8, error: *s32, alloc_buffer: *stb_vorbis_alloc) -> *stb_vorbis #foreign lib_stb_vorbis;

stb_vorbis_open_memory :: (data: *u8, len: s32, error: *s32, alloc_buffer: *stb_vorbis_alloc) -> *stb_vorbis #foreign lib_stb_vorbis;

stb_vorbis_open_pushdata :: (datablock: *u8, datablock_length_in_bytes: s32, datablock_memory_consumed_in_bytes: *s32, error: *s32, alloc_buffer: *stb_vorbis_alloc) -> *stb_vorbis #foreign lib_stb_vorbis;

stb_vorbis_seek :: (f: *stb_vorbis, sample_number: u32) -> s32 #foreign lib_stb_vorbis;

stb_vorbis_seek_frame :: (f: *stb_vorbis, sample_number: u32) -> s32 #foreign lib_stb_vorbis;

stb_vorbis_seek_start :: (f: *stb_vorbis) -> s32 #foreign lib_stb_vorbis;

stb_vorbis_stream_length_in_samples :: (f: *stb_vorbis) -> u32 #foreign lib_stb_vorbis;

stb_vorbis_stream_length_in_seconds :: (f: *stb_vorbis) -> float #foreign lib_stb_vorbis;
