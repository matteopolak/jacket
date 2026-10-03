// Entropy decoding, dequantization and frame ordering adapted from licensed
// public C pl_mpeg. Reviewed Jai records remain the actual live decoder state.
video_decode_block :: (video: *Video, block: s32) {
    B.assert(video != null && block >= 0 && block < 6 && video.has_sequence_header);
    B.assert(video.mb_row >= 0 && video.mb_row < video.mb_height && video.mb_col >= 0 && video.mb_col < video.mb_width);
    B.assert(video.quantizer_scale > 0 && video.quantizer_scale <= 31);
    n: s32;
    matrix := video.non_intra_quant_matrix.data;

    if video.macroblock_intra != 0 {
        plane: s32;
        if block > 3 plane = block - 3;

        size: s32;
        if plane == 0 size = buffer_read_vlc(video.buffer, DCT_SIZE_LUMINANCE);
        else size = buffer_read_vlc(video.buffer, DCT_SIZE_CHROMINANCE);

        differential: s32;

        if size != 0 {
            differential = read_bits(video.buffer, size);
            if (differential & (1 << (size - 1))) == 0 differential += 1 - (1 << size);
        }

        predictor := cast(s64) video.dc_predictor[plane] + differential;

        B.assert(predictor >= -8388608 && predictor <= 8388607, "MPEG DC predictor overflow");
        video.dc_predictor[plane] = cast(s32) predictor;
        video.block_data[0] = cast(s32) (predictor * 256);
        matrix = video.intra_quant_matrix.data;
        n = 1;
    }

    while true {
        coefficient := buffer_read_vlc_uint(video.buffer, DCT_COEFF);
        if coefficient == 1 && n > 0 && read_bits(video.buffer, 1) == 0 break;

        run: s32;
        level: s32;

        if coefficient == 0xffff {
            run = read_bits(video.buffer, 6);
            level = read_bits(video.buffer, 8);
            if level == 0 level = read_bits(video.buffer, 8);
            else if level == 128 level = read_bits(video.buffer, 8) - 256;
            else if level > 128 level -= 256;
        } else {
            run = cast(s32) (coefficient >> 8);
            level = cast(s32) (coefficient & 255);
            if read_bits(video.buffer, 1) != 0 level = -level;
        }

        n += run;
        B.assert(n >= 0 && n < 64, "MPEG coefficient run out of bounds");
        index := ZIG_ZAG[n];

        n += 1;
        level *= 2;

        if video.macroblock_intra == 0 {
            if level < 0 level -= 1;
            else level += 1;
        }

        scaled := (cast(s64) level * video.quantizer_scale * matrix[index]) >> 4;
        if (scaled & 1) == 0 {
            if scaled > 0 scaled -= 1;
            else scaled += 1;
        }

        if scaled > 2047 scaled = 2047;
        else if scaled < -2048 scaled = -2048;

        video.block_data[index] = cast(s32) (scaled * PREMULTIPLIER_MATRIX[index]);
    }

    dest: *u8;
    width: s64;
    offset: s64;

    if block < 4 {
        dest = video.frame_current.y.data;
        width = video.luma_width;
        offset = (cast(s64) video.mb_row * width + video.mb_col) * 16;
        if (block & 1) != 0 offset += 8;
        if (block & 2) != 0 offset += width * 8;
    } else {
        if block == 4 dest = video.frame_current.cb.data;
        else dest = video.frame_current.cr.data;
        width = video.chroma_width;
        offset = (cast(s64) video.mb_row * width + video.mb_col) * 8;
    }

    B.assert(dest != null);
    video_idct(video.block_data.data);

    for row: 0..7 for col : 0..7 {
        index := offset + row * width + col;
        value := video.block_data[row * 8 + col];
        if video.macroblock_intra == 0 value += dest[index];
        dest[index] = clamp(value);
    }

    memset(video.block_data.data, 0, size_of(video.block_data));
}

video_decode_macroblock :: (video: *Video) {
    B.assert(video != null && video.has_sequence_header);
    increment: s32;
    t := buffer_read_vlc(video.buffer, MACROBLOCK_ADDRESS_INCREMENT);

    while t == 34 t = buffer_read_vlc(video.buffer, MACROBLOCK_ADDRESS_INCREMENT);

    while t == 35 {
        B.assert(increment <= video.mb_size, "MPEG address increment overflow");
        increment += 33;
        t = buffer_read_vlc(video.buffer, MACROBLOCK_ADDRESS_INCREMENT);
    }

    B.assert(t > 0 && t <= 33);
    increment += t;
    B.assert(video.macroblock_address + increment < video.mb_size, "MPEG macroblock address out of bounds");

    if video.slice_begin {
        video.slice_begin = false;
        video.macroblock_address += increment;
    } else {
        if increment > 1 {
            reset_dc(video);

            if video.picture_type == .PREDICTIVE {
                video.motion_forward.h = 0;
                video.motion_forward.v = 0;
            }

            B.assert(video.picture_type != .INTRA, "skipped intra MPEG macroblock");
        }

        while increment > 1 {
            video.macroblock_address += 1;
            video.mb_row = video.macroblock_address / video.mb_width;
            video.mb_col = video.macroblock_address % video.mb_width;
            video_predict_macroblock(video);
            increment -= 1;
        }

        video.macroblock_address += 1;
    }

    B.assert(video.macroblock_address >= 0);
    video.mb_row = video.macroblock_address / video.mb_width;
    video.mb_col = video.macroblock_address % video.mb_width;
    if video.picture_type == .INTRA video.macroblock_type = buffer_read_vlc(video.buffer, MACROBLOCK_TYPE_INTRA);
    else if video.picture_type == .PREDICTIVE video.macroblock_type = buffer_read_vlc(video.buffer, MACROBLOCK_TYPE_PREDICTIVE);
    else {
        B.assert(video.picture_type == .B);
        video.macroblock_type = buffer_read_vlc(video.buffer, MACROBLOCK_TYPE_B);
    }

    video.macroblock_intra = video.macroblock_type & 1;
    video.motion_forward.is_set = video.macroblock_type & 8;
    video.motion_backward.is_set = video.macroblock_type & 4;
    if (video.macroblock_type & 16) != 0 video.quantizer_scale = read_bits(video.buffer, 5);

    B.assert(video.quantizer_scale > 0);

    if video.macroblock_intra != 0 {
        video.motion_forward.h = 0;
        video.motion_forward.v = 0;
        video.motion_backward.h = 0;
        video.motion_backward.v = 0;
    } else {
        reset_dc(video);
        video_decode_motion_vectors(video);
        video_predict_macroblock(video);
    }

    cbp: s32;
    if (video.macroblock_type & 2) != 0 cbp = buffer_read_vlc(video.buffer, CODE_BLOCK_PATTERN);
    else if video.macroblock_intra != 0 cbp = 63;
    for block: 0..5 if (cbp & (32 >> block)) != 0 video_decode_block(video, cast(s32) block);
}

video_decode_slice :: (video: *Video, slice: Start_Code) {
    decode_slice_inner(video, slice);
}

#scope_file
decode_slice_inner :: (video: *Video, slice: Start_Code) -> (first: s32, last: s32) {
    B.assert(video != null && IS_SLICE(slice));
    row := cast(s32) slice - 1;

    B.assert(row >= 0 && row < video.mb_height, "MPEG slice row out of bounds");
    video.slice_begin = true;
    video.macroblock_address = row * video.mb_width - 1;
    video.motion_forward.h = 0;
    video.motion_forward.v = 0;
    video.motion_backward.h = 0;
    video.motion_backward.v = 0;
    reset_dc(video);
    video.quantizer_scale = read_bits(video.buffer, 5);
    while read_bits(video.buffer, 1) != 0 read_bits(video.buffer, 8);

    video_decode_macroblock(video);
    first = video.macroblock_address;

    while video.macroblock_address < video.mb_size - 1 {
        count: s64 = 23;
        if !buffer_has(video.buffer, count) {
            count = video.buffer.length * 8 - video.buffer.bit_index;
            if count == 0 break;
        }

        saved := video.buffer.bit_index;
        value := buffer_read(video.buffer, count);

        video.buffer.bit_index = saved;
        if value == 0 break;

        video_decode_macroblock(video);
    }

    return first, video.macroblock_address;
}

#scope_export

video_decode_picture :: (video: *Video) {
    B.assert(video != null && video.has_sequence_header);
    read_bits(video.buffer, 10);
    type := read_bits(video.buffer, 3);

    B.assert(type >= 1 && type <= 3, "unsupported MPEG picture type");
    video.picture_type = cast(Picture_Type) type;
    B.assert(!video.assume_no_b_frames || video.picture_type != .B, "MPEG B picture violates no-delay assumption");
    if video.picture_type == .PREDICTIVE B.assert(video.has_reference_frame || video.frames_decoded > 0, "MPEG P picture has no reference");
    if video.picture_type == .B B.assert(video.frames_decoded > 0, "MPEG B picture has insufficient references");

    read_bits(video.buffer, 16);

    if video.picture_type != .INTRA {
        video.motion_forward.full_px = read_bits(video.buffer, 1);
        f_code := read_bits(video.buffer, 3);
        B.assert(f_code != 0, "invalid MPEG forward f_code");
        video.motion_forward.r_size = f_code - 1;
    }

    if video.picture_type == .B {
        video.motion_backward.full_px = read_bits(video.buffer, 1);
        f_code := read_bits(video.buffer, 3);
        B.assert(f_code != 0, "invalid MPEG backward f_code");
        video.motion_backward.r_size = f_code - 1;
    }

    temp := video.frame_forward;

    if video.picture_type != .B video.frame_forward = video.frame_backward;

    video.start_code = buffer_next_start_code(video.buffer);
    while video.start_code == .EXTENSION || video.start_code == .USER_DATA video.start_code = buffer_next_start_code(video.buffer);

    B.assert(IS_SLICE(video.start_code), "MPEG picture has no slice");
    coverage := cast(*u8) B.alloc(video.mb_size);

    B.assert(coverage != null, "MPEG picture coverage allocation failed");
    defer B.free(coverage);
    memset(coverage, 0, video.mb_size);

    while IS_SLICE(video.start_code) {
        first, last := decode_slice_inner(video, video.start_code);
        for address: first..last {
            B.assert(coverage[address] == 0, "overlapping MPEG slices");
            coverage[address] = 1;
        }

        if video.macroblock_address >= video.mb_size - 1 break;

        video.start_code = buffer_next_start_code(video.buffer);
    }

    B.assert(video.macroblock_address == video.mb_size - 1, "incomplete MPEG picture");
    for 0..video.mb_size - 1 B.assert(coverage[it] != 0, "incomplete MPEG picture coverage");

    if video.picture_type != .B {
        video.frame_backward = video.frame_current;
        video.frame_current = temp;
    }
}

video_decode :: (video: *Video) -> *Frame {
    if !video_has_header(video) return null;

    frame: *Frame;
    while !frame {
        while video.start_code != .PICTURE {
            if video.start_code == .SEQUENCE && !video_decode_sequence_header(video) return null;

            video.start_code = buffer_next_start_code(video.buffer);
            if video.start_code == .NOT_FOUND {
                if video.has_reference_frame && !video.assume_no_b_frames && video.buffer.has_ended {
                    video.has_reference_frame = false;
                    frame = *video.frame_backward;
                    break;
                }

                return null;
            }

        }

        if frame break;

        if !buffer_has_start_code(video.buffer, .PICTURE) && !video.buffer.has_ended return null;
        if video.buffer.discard_read_bytes buffer_discard_read_bytes(video.buffer);

        video_decode_picture(video);
        if video.assume_no_b_frames frame = *video.frame_backward;
        else if video.picture_type == .B frame = *video.frame_current;
        else if video.has_reference_frame frame = *video.frame_forward;
        else video.has_reference_frame = true;
    }

    frame.time = video.time;
    B.assert(video.frames_decoded < 2147483647);
    video.frames_decoded += 1;
    video.time = cast(float64) video.frames_decoded / video.framerate;

    return frame;
}

#scope_file
B :: #import "Basic";

read_bits :: (buffer: *Buffer, count: s64) -> s32 {
    B.assert(buffer_has(buffer, count), "truncated MPEG picture data");
    return buffer_read(buffer, count);
}

reset_dc :: (video: *Video) {
    for *video.dc_predictor it.* = 128;
}
