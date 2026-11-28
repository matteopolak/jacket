// Plane storage and the integer BT.601 transform adapted from the licensed
// public C pl_mpeg implementation; see third_party/pl_mpeg/README.md.

// One 8-bit image plane: `data` holds `height` rows of `width` bytes.
Plane :: struct {
    width: s32;
    height: s32;
    data: *u8;
}

// A decoded picture in YCbCr 4:2:0.
//
// `time` is the presentation time in seconds. `width` and `height` are the display size; each plane is padded to
// whole 16x16 macroblocks, so `y.width` can exceed `width`. The planes belong to the decoder and are overwritten
// by later decodes.
Frame :: struct {
    time: float64;
    width: s32;
    height: s32;
    y, cr, cb: Plane;
}

// How one audio subband is quantized: `levels` possible values, `bits` per code, and `group` nonzero when three
// samples share one code.
Quantizer_Spec :: struct {
    levels: u16;
    group: u8;
    bits: u8;
}

// The channel layout of an audio stream.
Audio_Mode :: enum u8 {
    STEREO :: 0x0;
    JOINT_STEREO :: 0x1;
    DUAL_CHANNEL :: 0x2;
    MONO :: 0x3;
}

// The coding type of a video picture: intra, predicted from the previous reference, or bidirectional.
Picture_Type :: enum {
    INTRA :: 1;
    PREDICTIVE :: 2;
    B :: 3;
}

// Returns whether `c` is a slice start code.
IS_SLICE :: inline (c: Start_Code) -> bool {
    return c >= .SLICE_FIRST && c <= .SLICE_LAST;
}

// Returns `n` limited to the range 0 to 255.
clamp :: inline (n: s32) -> u8 {
    if n < 0 return 0;
    if n > 255 return 255;
    return cast(u8) n;
}

// Converts `frame` to packed pixels in `dest`, `stride` bytes per row, with `BYTES_PER_PIXEL` bytes per pixel and
// the red, green and blue bytes at offsets `RI`, `GI` and `BI`.
//
// Baked into the `frame_to_*` procedures. Asserts on odd frame dimensions or a `stride` shorter than a row. The
// byte not covered by an offset in 4-byte formats is left untouched.
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

// Writes one pixel into `dest` at `d_index + dest_offset`, taking luma from `y_index + y_offset` and adding the
// surrounding `r`, `g` and `b` offsets, clamped to 0 to 255.
PUT_PIXEL :: (RI: s32, GI: s32, BI: s32, y_offset: s32, dest_offset: s32) #expand {
    sample := cast(s32) `frame.y.data[`y_index + y_offset];
    position := `d_index + dest_offset;
    `dest[position + RI] = byte_with_offset(sample, `r, false);
    `dest[position + GI] = byte_with_offset(sample, `g, true);
    `dest[position + BI] = byte_with_offset(sample, `b, false);
}

// Converts a [Frame] to packed RGB bytes; see [convert_function].
frame_to_rgb :: #bake_arguments convert_function(BYTES_PER_PIXEL = 3, RI = 0, GI = 1, BI = 2);
// Converts a [Frame] to packed BGR bytes; see [convert_function].
frame_to_bgr :: #bake_arguments convert_function(BYTES_PER_PIXEL = 3, RI = 2, GI = 1, BI = 0);
// Converts a [Frame] to packed RGBA bytes; see [convert_function]. The alpha byte is left unchanged.
frame_to_rgba :: #bake_arguments convert_function(BYTES_PER_PIXEL = 4, RI = 0, GI = 1, BI = 2);
// Converts a [Frame] to packed BGRA bytes; see [convert_function]. The alpha byte is left unchanged.
frame_to_bgra :: #bake_arguments convert_function(BYTES_PER_PIXEL = 4, RI = 2, GI = 1, BI = 0);
// Converts a [Frame] to packed ARGB bytes; see [convert_function]. The alpha byte is left unchanged.
frame_to_argb :: #bake_arguments convert_function(BYTES_PER_PIXEL = 4, RI = 1, GI = 2, BI = 3);
// Converts a [Frame] to packed ABGR bytes; see [convert_function]. The alpha byte is left unchanged.
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
