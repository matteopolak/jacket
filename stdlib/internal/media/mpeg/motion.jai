// Motion reconstruction and integer half-pixel filtering adapted from public C
// pl_mpeg (MIT); source attribution is in third_party/pl_mpeg/README.md.
video_decode_motion_vector :: (video: *Video, r_size: s32, motion: s32) -> s32 {
    B.assert(video != null && video.buffer != null && r_size >= 0 && r_size <= 6);
    scale: s32 = 1 << r_size;

    B.assert(motion >= -16 * scale && motion < 16 * scale);
    code := cast(s32) buffer_read_vlc(video.buffer, MOTION);
    delta := code;

    if code != 0 && scale != 1 {
        B.assert(buffer_has(video.buffer, r_size), "truncated MPEG motion residual");
        magnitude := code;
        if magnitude < 0 magnitude = -magnitude;

        delta = ((magnitude - 1) << r_size) + buffer_read(video.buffer, r_size) + 1;
        if code < 0 delta = -delta;
    }

    result := motion + delta;
    if result >= 16 * scale result -= 32 * scale;
    else if result < -16 * scale result += 32 * scale;

    return result;
}

video_decode_motion_vectors :: (video: *Video) {
    B.assert(video != null);

    if video.motion_forward.is_set != 0 {
        video.motion_forward.h = video_decode_motion_vector(video, video.motion_forward.r_size, video.motion_forward.h);
        video.motion_forward.v = video_decode_motion_vector(video, video.motion_forward.r_size, video.motion_forward.v);
    } else if video.picture_type == .PREDICTIVE {
        video.motion_forward.h = 0;
        video.motion_forward.v = 0;
    }

    if video.motion_backward.is_set != 0 {
        video.motion_backward.h = video_decode_motion_vector(video, video.motion_backward.r_size, video.motion_backward.h);
        video.motion_backward.v = video_decode_motion_vector(video, video.motion_backward.r_size, video.motion_backward.v);
    }
}

BLOCK_SET :: (dest: *u8, dest_width: s32, source_width: s32, block_size: s32, code: Code) #expand {
    target := dest;
    target_stride := dest_width;
    source_stride := source_width;
    side := block_size;
    row: s32 = 0;

    while row < side {
        column: s32 = 0;
        while column < side {
            target[`di] = cast,truncate(u8) (#insert code);
            `di += 1;
            `si += 1;
            column += 1;
        }

        `di += target_stride - side;
        `si += source_stride - side;
        row += 1;
    }
}

video_process_macroblock :: (video: *Video, d: *u8, s: *u8, motion_h: s32, motion_v: s32, block_size: s32, interpolate: bool) {
    B.assert(video != null && d != null && s != null);
    B.assert(block_size == 8 || block_size == 16);
    B.assert(video.mb_width > 0 && video.mb_height > 0);
    B.assert(video.mb_row >= 0 && video.mb_row < video.mb_height && video.mb_col >= 0 && video.mb_col < video.mb_width);
    width := cast(s64) video.mb_width * block_size;
    height := cast(s64) video.mb_height * block_size;
    x := cast(s64) video.mb_col * block_size;
    y := cast(s64) video.mb_row * block_size;
    sx := x + (motion_h >> 1);
    sy := y + (motion_v >> 1);
    odd_h := motion_h & 1;
    odd_v := motion_v & 1;

    B.assert(sx >= 0 && sy >= 0 && sx + block_size + odd_h <= width && sy + block_size + odd_v <= height,
        "MPEG motion region out of bounds");

    for row: 0..block_size - 1 {
        for col: 0..block_size - 1 {
            si := (sy + row) * width + sx + col;
            di := (y + row) * width + x + col;
            value := cast(s32) s[si];

            if odd_h != 0 && odd_v != 0 {
                value = (value + s[si + 1] + s[si + width] + s[si + width + 1] + 2) >> 2;
            } else if odd_h != 0 value = (value + s[si + 1] + 1) >> 1;
            else if odd_v != 0 value = (value + s[si + width] + 1) >> 1;

            if interpolate value = (cast(s32) d[di] + value + 1) >> 1;

            d[di] = cast(u8) value;
        }
    }
}

// The reviewed parameter is named d; it denotes the reference frame here.
video_copy_macroblock :: (video: *Video, motion_h: s32, motion_v: s32, d: *Frame) {
    B.assert(video != null && d != null);
    video_process_macroblock(video, video.frame_current.y.data, d.y.data, motion_h, motion_v, 16, false);
    video_process_macroblock(video, video.frame_current.cr.data, d.cr.data, motion_h / 2, motion_v / 2, 8, false);
    video_process_macroblock(video, video.frame_current.cb.data, d.cb.data, motion_h / 2, motion_v / 2, 8, false);
}

video_interpolate_macroblock :: (video: *Video, motion_h: s32, motion_v: s32, d: *Frame) {
    B.assert(video != null && d != null);
    video_process_macroblock(video, video.frame_current.y.data, d.y.data, motion_h, motion_v, 16, true);
    video_process_macroblock(video, video.frame_current.cr.data, d.cr.data, motion_h / 2, motion_v / 2, 8, true);
    video_process_macroblock(video, video.frame_current.cb.data, d.cb.data, motion_h / 2, motion_v / 2, 8, true);
}

video_predict_macroblock :: (using video: *Video) {
    B.assert(video != null);
    fw_h := motion_forward.h;
    fw_v := motion_forward.v;

    if motion_forward.full_px != 0 {
        fw_h *= 2;
        fw_v *= 2;
    }

    if picture_type == .B {
        bw_h := motion_backward.h;
        bw_v := motion_backward.v;

        if motion_backward.full_px != 0 {
            bw_h *= 2;
            bw_v *= 2;
        }

        if motion_forward.is_set != 0 {
            video_copy_macroblock(video, fw_h, fw_v, *frame_forward);
            if motion_backward.is_set != 0 video_interpolate_macroblock(video, bw_h, bw_v, *frame_backward);
        } else video_copy_macroblock(video, bw_h, bw_v, *frame_backward);
    } else {
        B.assert(picture_type == .PREDICTIVE);
        video_copy_macroblock(video, fw_h, fw_v, *frame_forward);
    }
}

#scope_file
B :: #import "Basic";
