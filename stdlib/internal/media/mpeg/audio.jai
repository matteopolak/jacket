// Canonical reviewed shapes retain ABI field order.
Samples :: struct {
    time: float64;
    count: u32;

    interleaved: [SAMPLES_PER_FRAME * 2] float;

}

Audio :: struct {
    time: float64;

    samples_decoded: s32;
    samplerate_index: s32;
    bitrate_index: s32;
    version: s32;
    layer: s32;

    mode: Audio_Mode;
    bound: s32;
    v_pos: s32;
    next_frame_data_size: s32;

    has_header := false;

    buffer: *Buffer;
    destroy_buffer_when_done: bool;

    allocation: [2][32] *Quantizer_Spec;

    scale_factor_info: [2][32] u8;

    scale_factor: [2][32][3] s32;
    sample: [2][32][3] s32;

    samples: Samples;
    D: [1024] float;
    V: [2][1024] float;
    U: [32] float;
}

audio_create_with_buffer :: (buffer: *Buffer, destroy_when_done: bool) -> *Audio {
    if !buffer return null;

    audio := B.New(Audio);
    audio.buffer = buffer;
    audio.destroy_buffer_when_done = destroy_when_done;
    audio.samples.count = SAMPLES_PER_FRAME;
    audio.samplerate_index = 3;
    for 0..1023 audio.D[it] = SYNTHESIS_WINDOW[it % 512];

    audio.next_frame_data_size = audio_decode_header(audio);

    return audio;
}

audio_destroy :: (audio: *Audio) {
    if !audio return;
    remove_audio_private(audio);
    if audio.destroy_buffer_when_done buffer_destroy(audio.buffer);
    B.free(audio);
}

audio_has_header :: (audio: *Audio) -> bool {
    B.assert(audio != null);
    if !audio.has_header audio.next_frame_data_size = audio_decode_header(audio);
    return audio.has_header;
}

audio_has_ended :: (audio: *Audio) -> bool {
    B.assert(audio != null && audio.buffer != null);
    return audio.buffer.has_ended;
}

audio_set_time :: (audio: *Audio, time: float64) {
    B.assert(audio != null && audio.samplerate_index >= 0 && audio.samplerate_index < SAMPLE_RATE.count);
    count := time * SAMPLE_RATE[audio.samplerate_index];

    B.assert(count >= -2147483648 && count <= 2147483647, "MPEG audio time out of range");
    audio.samples_decoded = cast(s32) count;
    audio.time = time;
}

audio_rewind :: (audio: *Audio) {
    B.assert(audio != null && audio.buffer != null);
    buffer_rewind(audio.buffer);
    audio.time = 0;
    audio.samples_decoded = 0;
    audio.next_frame_data_size = 0;
    audio.v_pos = 0;
    memset(audio.V.data, 0, size_of(audio.V));
    memset(audio.U.data, 0, size_of(audio.U));
}

audio_find_frame_sync :: (audio: *Audio) -> bool {
    B.assert(audio != null && audio.buffer != null);
    buffer := audio.buffer;

    buffer_align(buffer);

    while buffer_has(buffer, 16) {
        index := buffer.bit_index >> 3;
        if buffer.bytes[index] == 0xff && (buffer.bytes[index + 1] & 0xfe) == 0xfc {
            buffer.bit_index += 11;
            return true;
        }

        buffer.bit_index += 8;
    }

    return false;
}

audio_decode_header :: (audio: *Audio) -> s32 {
    B.assert(audio != null && audio.buffer != null);
    buffer := audio.buffer;
    if !buffer_has(buffer, 48) return 0;
    if !audio_find_frame_sync(audio) return 0;

    // Header fields after the 11-bit sync. Hold the cursor on partial input.
    discard := buffer.discard_read_bytes;
    saved := buffer.bit_index - 11;

    buffer.discard_read_bytes = false;
    committed := false;

    defer {
        buffer.discard_read_bytes = discard;
        if !committed buffer.bit_index = saved;
    }

    if !buffer_has(buffer, 21) return 0;

    header_index := saved >> 3;
    header_high := cast(u32) buffer.bytes[header_index + 2];
    header_low := cast(u32) buffer.bytes[header_index + 3];
    header_tail := (header_high << 8) | header_low;

    version := buffer_read(buffer, 2);
    layer := buffer_read(buffer, 2);
    has_crc := buffer_read(buffer, 1) == 0;
    bitrate := buffer_read(buffer, 4) - 1;
    samplerate := buffer_read(buffer, 2);
    padding := buffer_read(buffer, 1);

    buffer_skip(buffer, 1);
    mode := cast(Audio_Mode) buffer_read(buffer, 2);
    extension := buffer_read(buffer, 2);

    buffer_skip(buffer, 4);
    B.assert(version == MPEG_1 && layer == LAYER_II, "unsupported MPEG audio version/layer");
    B.assert(bitrate >= 0 && bitrate < 14 && samplerate < 3, "invalid MPEG audio rate");
    checksum: u16;

    if has_crc {
        if !buffer_has(buffer, 16) return 0;
        checksum = cast(u16) buffer_read(buffer, 16);
    }

    if audio.has_header {
        B.assert(audio.bitrate_index == bitrate && audio.samplerate_index == samplerate && audio.mode == mode,
            "MPEG audio format changes are unsupported");
    }

    audio.version = version;
    audio.layer = layer;
    audio.bitrate_index = bitrate;
    audio.samplerate_index = samplerate;
    audio.mode = mode;
    audio.bound = 32;
    if mode == .JOINT_STEREO audio.bound = (extension + 1) * 4;
    else if mode == .MONO audio.bound = 0;

    audio.has_header = true;
    private := get_audio_private(audio, true);
    private.has_crc = has_crc;
    private.expected_crc = checksum;
    private.seed_crc = crc_bits(0xffff, header_tail, 16);
    committed = true;

    overhead: s32 = 4;
    if has_crc overhead = 6;

    return cast(s32) (144000 * BIT_RATE[bitrate] / SAMPLE_RATE[samplerate] + padding - overhead);
}

audio_decode :: (audio: *Audio) -> *Samples {
    B.assert(audio != null && audio.buffer != null);
    if audio.next_frame_data_size == 0 audio.next_frame_data_size = audio_decode_header(audio);
    if audio.next_frame_data_size == 0 || !buffer_has(audio.buffer, cast(s64) audio.next_frame_data_size * 8) return null;

    payload_end := audio.buffer.bit_index + cast(s64) audio.next_frame_data_size * 8;

    audio_decode_frame(audio);
    B.assert(audio.buffer.bit_index <= payload_end, "MPEG audio frame exceeded packet length");
    buffer_skip(audio.buffer, payload_end - audio.buffer.bit_index); // Ancillary bits.
    audio.next_frame_data_size = 0;
    audio.samples.time = audio.time;
    B.assert(audio.samples_decoded <= 2147483647 - SAMPLES_PER_FRAME);
    audio.samples_decoded += SAMPLES_PER_FRAME;
    audio.time = cast(float64) audio.samples_decoded / SAMPLE_RATE[audio.samplerate_index];

    return *audio.samples;
}

audio_read_allocation :: (audio: *Audio, sb: s32, tab3: u8) -> *Quantizer_Spec {
    return read_allocation(audio, sb, tab3, get_audio_private(audio, false));
}

#scope_file
read_allocation :: (audio: *Audio, sb: s32, tab3: u8, private: *Audio_Private) -> *Quantizer_Spec {
    B.assert(audio != null && tab3 < QUANT_LUT_STEP_3.count && sb >= 0 && sb < 32);
    tab4 := QUANT_LUT_STEP_3[tab3][sb];
    value := control_bits(private, audio.buffer, tab4 >> 4);
    qtab := QUANT_LUT_STEP4[tab4 & 15][value];

    if qtab == 0 return null;

    // Take the canonical aggregate's static address before dynamic indexing.
    // The direct *QUANT_TAB[index] compiler path currently spills a temporary.
    table := *QUANT_TAB;

    return table.data + (qtab - 1);
}

#scope_export

audio_read_samples :: (audio: *Audio, ch: s32, sb: s32, part: s32) {
    B.assert(audio != null && ch >= 0 && ch < 2 && sb >= 0 && sb < 32 && part >= 0 && part < 3);
    q := audio.allocation[ch][sb];
    values: [3] s32;

    if q values = read_quantized(audio.buffer, q);

    scale_samples(audio, ch, sb, part, q, values);
}

audio_decode_frame :: (audio: *Audio) {
    B.assert(audio != null && audio.has_header);
    private := get_audio_private(audio, false);

    B.assert(private != null, "MPEG audio frame requires a parsed header");
    B.assert(!private.frame_active, "recursive MPEG audio frame decode");

    if private {
        private.frame_active = true;
        private.running_crc = private.seed_crc;
    }

    defer if private private.frame_active = false;
    tab1: s32 = 1;
    channels: s32 = 2;

    if audio.mode == .MONO {
        tab1 = 0;
        channels = 1;
    }

    tab2 := QUANT_LUT_STEP_1[tab1][audio.bitrate_index];
    tab3 := QUANT_LUT_STEP_2[tab2][audio.samplerate_index];
    limit := cast(s32) (tab3 & 63);

    tab3 >>= 6;
    if audio.bound > limit audio.bound = limit;

    for sb: 0..limit - 1 {
        audio.allocation[0][sb] = read_allocation(audio, cast(s32) sb, tab3, private);
        if sb < audio.bound audio.allocation[1][sb] = read_allocation(audio, cast(s32) sb, tab3, private);
        else audio.allocation[1][sb] = audio.allocation[0][sb];
    }

    for sb: 0..limit - 1 {
        for ch: 0..channels - 1 {
            if audio.allocation[ch][sb] audio.scale_factor_info[ch][sb] = cast(u8) control_bits(private, audio.buffer, 2);
        }

        if channels == 1 audio.scale_factor_info[1][sb] = audio.scale_factor_info[0][sb];
    }

    if private && private.has_crc B.assert(private.running_crc == private.expected_crc, "MPEG audio checksum mismatch");

    for sb: 0..limit - 1 {
        for ch: 0..channels - 1 {
            if !audio.allocation[ch][sb] continue;

            selection := audio.scale_factor_info[ch][sb];
            a := audio_bits(audio.buffer, 6);
            b := a;
            c := a;

            if selection == 0 {
                b = audio_bits(audio.buffer, 6);
                c = audio_bits(audio.buffer, 6);
            } else if selection == 1 c = audio_bits(audio.buffer, 6);
            else if selection == 3 {
                b = audio_bits(audio.buffer, 6);
                c = b;
            } else B.assert(selection == 2);

            audio.scale_factor[ch][sb] = .[a, b, c];
        }

        if channels == 1 audio.scale_factor[1][sb] = audio.scale_factor[0][sb];
    }

    out: s32;

    for part: 0..2 for granule : 0..3 {
        for sb: 0..limit - 1 {
            if sb < audio.bound {
                audio_read_samples(audio, 0, cast(s32) sb, cast(s32) part);
                audio_read_samples(audio, 1, cast(s32) sb, cast(s32) part);
            } else {
                // Shared sample codes retain independent channel scalefactors.
                q := audio.allocation[0][sb];

                B.assert(audio.allocation[1][sb] == q);
                values: [3] s32;
                if q values = read_quantized(audio.buffer, q);

                scale_samples(audio, 0, cast(s32) sb, cast(s32) part, q, values);
                scale_samples(audio, 1, cast(s32) sb, cast(s32) part, q, values);
            }
        }

        for sb: limit..31 {
            audio.sample[0][sb] = .[0, 0, 0];
            audio.sample[1][sb] = .[0, 0, 0];
        }

        for p: 0..2 {
            audio.v_pos = (audio.v_pos - 64) & 1023;

            for ch: 0..1 {
                audio_matrix_transform(audio.sample[ch], cast(s32) p, audio.V[ch].data, audio.v_pos);
                for *audio.U it.* = 0;

                di := 512 - (audio.v_pos >> 1);
                vi := (audio.v_pos % 128) >> 1;

                for sweep: 0..1 {
                    while vi < 1024 {
                        for i: 0..31 {
                            B.assert(di >= 0 && di < 1024 && vi >= 0 && vi < 1024);
                            audio.U[i] += audio.D[di] * audio.V[ch][vi];
                            di += 1;
                            vi += 1;
                        }

                        vi += 96;
                        di += 32;
                    }

                    if sweep == 0 {
                        di -= 480;
                        vi = 1120 - vi;
                    }
                }

                for j: 0..31 audio.samples.interleaved[(out + j) * 2 + ch] = audio.U[j] / -1090519040.0;
            }

            out += 32;
        }
    }

    B.assert(out == SAMPLES_PER_FRAME);
    buffer_align(audio.buffer);
}

#scope_file
B :: #import "Basic";

// Header/checksum state lives outside the canonical public Audio ABI. Registry
// membership is synchronized; callers still synchronize each decoder's lifetime.
Audio_Private :: struct {
    audio: *Audio;
    next: *Audio_Private;
    allocator: Allocator;
    has_crc: bool;
    frame_active: bool;
    expected_crc, seed_crc, running_crc: u16;
}

audio_private_head: *Audio_Private;
audio_private_lock: s32;

lock_private :: () {
    while !compare_and_swap(*audio_private_lock, cast(s32) 0, cast(s32) 1) {}
}

unlock_private :: () {
    released := compare_and_swap(*audio_private_lock, cast(s32) 1, cast(s32) 0);
    B.assert(released);
}

find_private_locked :: (audio: *Audio) -> *Audio_Private {
    cursor := audio_private_head;
    while cursor {
        if cursor.audio == audio return cursor;
        cursor = cursor.next;
    }

    return null;
}

get_audio_private :: (audio: *Audio, create: bool) -> *Audio_Private {
    lock_private();
    found := find_private_locked(audio);

    unlock_private();
    if found || !create return found;

    // Do not invoke user allocation code while holding the registry lock.
    node := B.New(Audio_Private);
    node.audio = audio;
    node.allocator = context.allocator;
    lock_private();
    found = find_private_locked(audio);
    if !found {
        node.next = audio_private_head;
        audio_private_head = node;
        found = node;
    }

    unlock_private();
    if found != node free_private(node);

    return found;
}

remove_audio_private :: (audio: *Audio) {
    lock_private();
    link := *audio_private_head;
    while link.* {
        if link.*.audio == audio break;
        link = *link.*.next;
    }

    found := link.*;
    if found link.* = found.next;

    unlock_private();
    if found free_private(found);
}

free_private :: (node: *Audio_Private) {
    allocator := node.allocator;
    allocator.proc(.FREE, 0, 0, node, allocator.data);
}

crc_bits :: (current: u16, bits: u32, count: s64) -> u16 {
    B.assert(count >= 0 && count <= 32);
    crc := cast(u32) current;

    for i: 0..count - 1 {
        feedback := (crc >> 15) ^ ((bits >> (count - i - 1)) & 1);
        crc = (crc << 1) & 0xffff;
        if feedback != 0 crc ^= 0x8005;
    }

    return cast(u16) crc;
}

control_bits :: (private: *Audio_Private, buffer: *Buffer, count: s64) -> s32 {
    value := audio_bits(buffer, count);
    if private && private.frame_active && private.has_crc private.running_crc = crc_bits(private.running_crc, cast(u32) value, count);
    return value;
}

audio_bits :: (buffer: *Buffer, count: s64) -> s32 {
    B.assert(buffer_has(buffer, count), "truncated MPEG audio frame");
    return buffer_read(buffer, count);
}

read_quantized :: (buffer: *Buffer, q: *Quantizer_Spec) -> [3] s32 {
    B.assert(q != null && q.levels > 0 && q.bits > 0 && q.bits <= 16);
    levels := cast(s32) q.levels;
    values: [3] s32;

    if q.group != 0 {
        value := audio_bits(buffer, q.bits);

        values[0] = value % levels;
        value /= levels;
        values[1] = value % levels;
        values[2] = value / levels;
        B.assert(values[2] < levels, "invalid MPEG grouped quantizer value");
    } else {
        for 0..2 values[it] = audio_bits(buffer, q.bits);
    }

    return values;
}

scale_samples :: (audio: *Audio, ch: s32, sb: s32, part: s32, q: *Quantizer_Spec, values: [3] s32) {
    if !q {
        audio.sample[ch][sb] = .[0, 0, 0];
        return;
    }

    sf := audio.scale_factor[ch][sb][part];

    B.assert(sf >= 0 && sf <= 63 && q.levels > 0 && q.bits > 0 && q.bits <= 16);
    factor: s64;

    if sf != 63 {
        shift := sf / 3;
        factor = (cast(s64) SCALEFACTOR_BASE[sf % 3] + ((cast(s64) 1 << shift) >> 1)) >> shift;
    }

    levels := cast(s32) q.levels;
    scale := 65536 / (levels + 1);
    adjustment := ((levels + 1) >> 1) - 1;

    for 0..2 {
        B.assert(values[it] >= 0 && values[it] < levels, "invalid MPEG quantizer code");
        value := cast(s64) (adjustment - values[it]) * scale;
        result := (value * (factor >> 12) + ((value * (factor & 4095) + 2048) >> 12)) >> 12;

        B.assert(result >= -2147483648 && result <= 2147483647);
        audio.sample[ch][sb][it] = cast(s32) result;
    }
}
