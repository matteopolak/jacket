Mixer :: struct {
    reader: *Audio.Reader;
    samples: [] s16;
    start: s64 = -1;
    count: s64;
    cursor: float64;
    output_rate: s32;
    failed: bool;
}

init_mixer :: (m: *Mixer, reader: *Audio.Reader, output_rate: s32) {
    m.* = .{ reader = reader, samples = NewArray(4096 * reader.channels, s16), output_rate = output_rate };
}

free_mixer :: (m: *Mixer) {
    array_free(m.samples);
    m.* = .{};
}

seek_mixer :: (m: *Mixer, seconds: float64) {
    m.cursor = clamp(seconds * m.reader.rate, cast(float64) 0, cast(float64) m.reader.frames);
    m.start = -1;
    m.count = 0;
    m.failed = false;
}

// Linear sample-rate conversion, mono duplication, stereo preservation and saturating gain.
// Chunk boundaries do not reset the fractional source cursor.
mix :: (m: *Mixer, stereo: [] s16, gain: float = 1) -> s64 {
    assert(stereo.count % 2 == 0 && m.output_rate > 0);
    written: s64;
    step := cast(float64) m.reader.rate / m.output_rate;

    while written < stereo.count / 2 && m.cursor < m.reader.frames && !m.failed {
        index := cast(s64) m.cursor;
        remainder := m.cursor - index;

        for c: 0..1 {
            a := source_sample(m, index, min(c, m.reader.channels - 1));
            b := source_sample(m, min(index + 1, m.reader.frames - 1), min(c, m.reader.channels - 1));
            stereo[written * 2 + c] = mixed_sample(a, b, remainder, gain);
        }

        if m.failed break;

        written += 1;
        m.cursor += step;
    }

    return written;
}

mixed_sample :: (a: s16, b: s16, fraction: float64, gain: float) -> s16 {
    value := (a + (cast(float64) b - a) * fraction) * gain;
    return cast(s16) clamp(value, cast(float64) -32768, cast(float64) 32767);
}

checksum :: (samples: [] s16) -> u32 {
    sum: u32 = 2166136261;

    for samples {
        bits := cast,no_check(u16) it;
        sum = (sum ^ cast(u32) (bits & 255)) * 16777619;
        sum = (sum ^ cast(u32) (bits >> 8)) * 16777619;
    }

    return sum;
}

#scope_file
source_sample :: (m: *Mixer, frame: s64, channel: s64) -> s16 {
    if frame < m.start || frame >= m.start + m.count {
        if !Audio.seek(m.reader, frame) {
            m.failed = true;
            return 0;
        }

        m.start = frame;
        m.count = Audio.read(m.reader, m.samples);
        if m.count <= 0 {
            m.failed = true;
            return 0;
        }
    }

    return m.samples[(frame - m.start) * m.reader.channels + channel];
}
