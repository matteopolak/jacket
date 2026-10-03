// Canonical reviewed shapes retain ABI field order.
Video_Motion :: struct {
    full_px: s32;
    is_set: s32;
    r_size: s32;
    h: s32;
    v: s32;
}

Video :: struct {
    framerate: float64;
    time: float64;
    frames_decoded: s32;
    width: s32;
    height: s32;
    mb_width: s32;
    mb_height: s32;
    mb_size: s32;

    luma_width: s32;
    luma_height: s32;

    chroma_width: s32;
    chroma_height: s32;

    start_code: Start_Code;
    picture_type: Picture_Type;

    motion_forward: Video_Motion;
    motion_backward: Video_Motion;

    has_sequence_header := false;

    quantizer_scale: s32;
    slice_begin := false;
    macroblock_address: s32;

    mb_row: s32;
    mb_col: s32;

    macroblock_type: s32;
    macroblock_intra: s32;

    dc_predictor: [3] s32;

    buffer: *Buffer;
    destroy_buffer_when_done: bool;

    frame_current: Frame;
    frame_forward: Frame;
    frame_backward: Frame;

    frames_data: *u8;

    block_data: [64] s32;
    intra_quant_matrix: [64] u8;
    non_intra_quant_matrix: [64] u8;

    has_reference_frame := false;
    assume_no_b_frames := false;
}

video_create_with_buffer :: (buffer: *Buffer, destroy_when_done: bool) -> *Video {
    if !buffer return null;

    video := B.New(Video);
    video.buffer = buffer;
    video.destroy_buffer_when_done = destroy_when_done;
    video.start_code = .NOT_FOUND;
    video_has_header(video);

    return video;
}

video_destroy :: (video: *Video) {
    if !video return;
    if video.destroy_buffer_when_done buffer_destroy(video.buffer);
    B.free(video.frames_data);
    B.free(video);
}

video_has_header :: (video: *Video) -> bool {
    B.assert(video != null && video.buffer != null);
    if video.has_sequence_header return true;

    if video.start_code != .SEQUENCE {
        if !buffer_find_start_code(video.buffer, .SEQUENCE) return false;
        video.start_code = .SEQUENCE;
    }

    return video_decode_sequence_header(video);
}

video_get_framerate :: (video: *Video) -> float64 {
    if video_has_header(video) return video.framerate;
    return 0;
}

video_set_no_delay :: (video: *Video, no_delay: bool) {
    B.assert(video != null);
    video.assume_no_b_frames = no_delay;
}

video_set_time :: (video: *Video, time: float64) {
    B.assert(video != null);
    frames := video.framerate * time;

    B.assert(frames >= -2147483648 && frames <= 2147483647, "MPEG video time out of range");
    video.time = time;
    video.frames_decoded = cast(s32) frames;
}

video_has_ended :: (video: *Video) -> bool {
    B.assert(video != null && video.buffer != null);
    return video.buffer.has_ended;
}

video_rewind :: (video: *Video) {
    B.assert(video != null && video.buffer != null);
    buffer_rewind(video.buffer);
    video.time = 0;
    video.frames_decoded = 0;
    video.has_reference_frame = false;
    video.start_code = .NOT_FOUND;
}

// Transactional variable-length parsing: incomplete custom matrices retain the
// header cursor for a later callback/append, without exposing partial metadata.
video_decode_sequence_header :: (video: *Video) -> bool {
    B.assert(video != null && video.buffer != null);
    buffer := video.buffer;
    discard := buffer.discard_read_bytes;

    buffer.discard_read_bytes = false;
    saved := buffer.bit_index;
    committed := false;

    defer {
        buffer.discard_read_bytes = discard;
        if !committed buffer.bit_index = saved;
    }

    if !buffer_has(buffer, 64) return false;

    width := buffer_read(buffer, 12);
    height := buffer_read(buffer, 12);

    buffer_skip(buffer, 4); // Aspect ratio has no field in the reviewed shape.
    rate := buffer_read(buffer, 4);

    buffer_skip(buffer, 18);
    marker := buffer_read(buffer, 1);

    buffer_skip(buffer, 11);

    if width == 0 || height == 0 || PICTURE_RATE[rate] == 0 || marker != 1 {
        context.logger("invalid MPEG sequence header\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
        return false;
    }

    intra: [64] u8;
    non_intra: [64] u8;

    for 0..63 {
        intra[it] = INTRA_QUANT_MATRIX[it];
        non_intra[it] = NON_INTRA_QUANT_MATRIX[it];
    }

    if buffer_read(buffer, 1) != 0 {
        if !buffer_has(buffer, 513) return false;
        for 0..63 intra[ZIG_ZAG[it]] = cast(u8) buffer_read(buffer, 8);
    }

    if buffer_read(buffer, 1) != 0 {
        if !buffer_has(buffer, 512) return false;
        for 0..63 non_intra[ZIG_ZAG[it]] = cast(u8) buffer_read(buffer, 8);
    }

    if video.has_sequence_header {
        B.assert(video.width == width && video.height == height && video.framerate == cast(float64) PICTURE_RATE[rate],
            "MPEG sequence dimension/rate changes are unsupported");
        video.intra_quant_matrix = intra;
        video.non_intra_quant_matrix = non_intra;
        video.start_code = .NOT_FOUND;
        committed = true;

        return true;
    }

    mb_width := (width + 15) >> 4;
    mb_height := (height + 15) >> 4;
    frame_bytes := cast(s64) mb_width * mb_height * 384;
    storage := cast(*u8) B.alloc(frame_bytes * 3);

    if !storage return false;

    memset(storage, 0, frame_bytes * 3);
    B.free(video.frames_data);
    video.frames_data = storage;
    video.width = width;
    video.height = height;
    video.framerate = cast(float64) PICTURE_RATE[rate];
    video.mb_width = mb_width;
    video.mb_height = mb_height;
    video.mb_size = mb_width * mb_height;
    video.luma_width = mb_width * 16;
    video.luma_height = mb_height * 16;
    video.chroma_width = mb_width * 8;
    video.chroma_height = mb_height * 8;
    video.intra_quant_matrix = intra;
    video.non_intra_quant_matrix = non_intra;
    video_init_frame(video, *video.frame_current, storage);
    video_init_frame(video, *video.frame_forward, storage + frame_bytes);
    video_init_frame(video, *video.frame_backward, storage + frame_bytes * 2);
    video.has_sequence_header = true;
    video.has_reference_frame = false;
    video.start_code = .NOT_FOUND;
    committed = true;

    return true;
}

video_init_frame :: (video: *Video, frame: *Frame, base: *u8) {
    B.assert(video != null && frame != null && base != null);
    B.assert(video.luma_width > 0 && video.luma_height > 0 && video.chroma_width > 0 && video.chroma_height > 0);
    luma_bytes := cast(s64) video.luma_width * video.luma_height;
    chroma_bytes := cast(s64) video.chroma_width * video.chroma_height;

    frame.width = video.width;
    frame.height = video.height;
    frame.y = .{ video.luma_width, video.luma_height, base };
    frame.cr = .{ video.chroma_width, video.chroma_height, base + luma_bytes };
    frame.cb = .{ video.chroma_width, video.chroma_height, base + luma_bytes + chroma_bytes };
}

#scope_file
B :: #import "Basic";
