// Plane storage and the integer BT.601 transform adapted from the licensed
// public C pl_mpeg implementation; see third_party/pl_mpeg/README.md.
Plane :: struct {
    width: s32;
    height: s32;
    data: *u8;
}

Frame :: struct {
    time: float64;
    width: s32;
    height: s32;
    y, cr, cb: Plane;
}

Quantizer_Spec :: struct {
    levels: u16;
    group: u8;
    bits: u8;
}

Audio_Mode :: enum u8 {
    STEREO :: 0x0;
    JOINT_STEREO :: 0x1;
    DUAL_CHANNEL :: 0x2;
    MONO :: 0x3;
}

Picture_Type :: enum {
    INTRA :: 1;
    PREDICTIVE :: 2;
    B :: 3;
}

IS_SLICE :: inline (c: Start_Code) -> bool {
    return c >= .SLICE_FIRST && c <= .SLICE_LAST;
}

clamp :: inline (n: s32) -> u8 {
    if n < 0 return 0;
    if n > 255 return 255;
    return cast(u8) n;
}

convert_function :: (frame: *Frame, dest: *u8, stride: s32, BYTES_PER_PIXEL: s32, RI: s32, GI: s32, BI: s32) {
    B.assert(frame != null && dest != null);
    B.assert(frame.width >= 0 && frame.height >= 0);
    B.assert(frame.width % 2 == 0 && frame.height % 2 == 0, "MPEG pixel conversion requires even dimensions");
    B.assert(BYTES_PER_PIXEL == 3 || BYTES_PER_PIXEL == 4);
    B.assert(RI >= 0 && RI < BYTES_PER_PIXEL && GI >= 0 && GI < BYTES_PER_PIXEL && BI >= 0 && BI < BYTES_PER_PIXEL);
    B.assert(RI != GI && RI != BI && GI != BI);
    B.assert(cast(s64) stride >= cast(s64) frame.width * BYTES_PER_PIXEL);
    B.assert(frame.y.width >= frame.width && frame.y.height >= frame.height);
    B.assert(frame.cb.width >= frame.width / 2 && frame.cb.height >= frame.height / 2);
    B.assert(frame.cr.width >= frame.width / 2 && frame.cr.height >= frame.height / 2);
    if frame.width == 0 || frame.height == 0 return;

    B.assert(frame.y.data != null && frame.cb.data != null && frame.cr.data != null);

    for row: 0..frame.height - 1 {
        for col: 0..frame.width - 1 {
            cr := cast(s32) frame.cr.data[cast(s64) (row / 2) * frame.cr.width + col / 2] - 128;
            cb := cast(s32) frame.cb.data[cast(s64) (row / 2) * frame.cb.width + col / 2] - 128;
            y := ((cast(s32) frame.y.data[cast(s64) row * frame.y.width + col] - 16) * 76309) >> 16;
            offset := cast(s64) row * stride + cast(s64) col * BYTES_PER_PIXEL;
            dest[offset + RI] = clamp(y + ((cr * 104597) >> 16));
            dest[offset + GI] = clamp(y - ((cb * 25674 + cr * 53278) >> 16));
            dest[offset + BI] = clamp(y + ((cb * 132201) >> 16));
        }
    }
}

PUT_PIXEL :: (RI: s32, GI: s32, BI: s32, y_offset: s32, dest_offset: s32) #expand {
    sample := cast(s32) `frame.y.data[`y_index + y_offset];
    position := `d_index + dest_offset;
    `dest[position + RI] = byte_with_offset(sample, `r, false);
    `dest[position + GI] = byte_with_offset(sample, `g, true);
    `dest[position + BI] = byte_with_offset(sample, `b, false);
}

frame_to_rgb :: #bake_arguments convert_function(BYTES_PER_PIXEL = 3, RI = 0, GI = 1, BI = 2);
frame_to_bgr :: #bake_arguments convert_function(BYTES_PER_PIXEL = 3, RI = 2, GI = 1, BI = 0);
frame_to_rgba :: #bake_arguments convert_function(BYTES_PER_PIXEL = 4, RI = 0, GI = 1, BI = 2);
frame_to_bgra :: #bake_arguments convert_function(BYTES_PER_PIXEL = 4, RI = 2, GI = 1, BI = 0);
frame_to_argb :: #bake_arguments convert_function(BYTES_PER_PIXEL = 4, RI = 1, GI = 2, BI = 3);
frame_to_abgr :: #bake_arguments convert_function(BYTES_PER_PIXEL = 4, RI = 3, GI = 2, BI = 1);

#scope_file
B :: #import "Basic";

// Compare before narrowing/adding so all integer contributions clamp safely.
byte_with_offset :: inline (sample: s32, offset: $T, subtract: bool) -> u8 {
    if subtract {
        if offset > sample return 0;

        #if type_info(T).signed {
            if offset < sample - 255 return 255;
        }

        return cast(u8) (sample - cast(s32) offset);
    }

    if offset > 255 - sample return 255;

    #if type_info(T).signed {
        if offset < -sample return 0;
    }

    return cast(u8) (sample + cast(s32) offset);
}
