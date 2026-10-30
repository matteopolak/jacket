// Sound effects and a music loop, synthesised sample by sample. Everything is 16-bit mono at
// SAMPLE_RATE.

Noise :: struct {
    state: u32 = 0x12345678;
}

next_noise :: (noise: *Noise) -> float {
    // xorshift32 mapped to -1..1.
    noise.state ^= noise.state << 13;
    noise.state ^= noise.state >> 17;
    noise.state ^= noise.state << 5;

    return cast(float) noise.state / cast(float) 2147483648 - 1;
}

to_sample :: (value: float) -> s16 {
    return cast(s16) (clamp(value, -1, 1) * 30000);
}

seconds_to_samples :: (seconds: float) -> s64 {
    return cast(s64) (seconds * SAMPLE_RATE);
}

square_wave :: (phase: float) -> float {
    return ifx phase - floor(phase) < 0.5 then 1.0 else -1.0;
}

triangle_wave :: (phase: float) -> float {
    p := phase - floor(phase);

    return 4 * abs(p - 0.5) - 1;
}

// A falling chirp with a quick decay.
make_shoot :: () -> [] s16 {
    samples := NewArray(seconds_to_samples(0.14), s16);
    phase := 0.0;

    for *samples {
        t := cast(float) it_index / cast(float) samples.count;
        frequency := 1100 - 800 * t;

        phase += frequency / SAMPLE_RATE;
        it.* = to_sample(square_wave(phase) * 0.35 * (1 - t) * (1 - t));
    }

    return samples;
}

make_hit :: () -> [] s16 {
    samples := NewArray(seconds_to_samples(0.09), s16);
    noise: Noise;
    phase := 0.0;

    for *samples {
        t := cast(float) it_index / cast(float) samples.count;

        phase += 220 / SAMPLE_RATE;
        it.* = to_sample((next_noise(*noise) * 0.4 + triangle_wave(phase) * 0.5) * (1 - t));
    }

    return samples;
}

// Low-passed noise that rumbles down.
make_explosion :: () -> [] s16 {
    samples := NewArray(seconds_to_samples(0.45), s16);
    noise: Noise;
    low := 0.0;

    for *samples {
        t := cast(float) it_index / cast(float) samples.count;
        cutoff := 0.25 - 0.2 * t;

        low += (next_noise(*noise) - low) * cutoff;
        it.* = to_sample(low * 1.8 * (1 - t) * (1 - t));
    }

    return samples;
}

// Two rising notes.
make_pickup :: () -> [] s16 {
    samples := NewArray(seconds_to_samples(0.24), s16);
    phase := 0.0;

    for *samples {
        t := cast(float) it_index / cast(float) samples.count;
        frequency := ifx t < 0.5 then 660.0 else 990.0;
        local := ifx t < 0.5 then t * 2 else (t - 0.5) * 2;

        phase += frequency / SAMPLE_RATE;
        it.* = to_sample(triangle_wave(phase) * 0.4 * (1 - local * 0.8));
    }

    return samples;
}

// A looping arpeggio over four chords, a bass note under each, eight seconds long.
make_music :: () -> [] s16 {
    chords :: float.[
        220.00, 261.63, 329.63,
        174.61, 220.00, 261.63,
        196.00, 261.63, 329.63,
        196.00, 246.94, 293.66,
    ];
    seconds_per_chord := 2.0;
    step_seconds := 0.25;

    samples := NewArray(seconds_to_samples(seconds_per_chord * 4), s16);
    arp_phase := 0.0;
    bass_phase := 0.0;

    for *samples {
        time := cast(float) it_index / SAMPLE_RATE;
        chord := cast(s64) (time / seconds_per_chord);
        step := cast(s64) (time / step_seconds);
        into_step := time - cast(float) step * step_seconds;
        note := chords[chord * 3 + step % 3];

        arp_phase += note * 2 / SAMPLE_RATE;
        bass_phase += chords[chord * 3] / 2 / SAMPLE_RATE;

        envelope := max(0.0, 1 - into_step / step_seconds);
        value := triangle_wave(arp_phase) * 0.22 * envelope + triangle_wave(bass_phase) * 0.18;

        it.* = to_sample(value);
    }

    return samples;
}

wav_bytes :: (samples: [] s16) -> [] u8 {
    out: String_Builder;
    data_bytes := samples.count * 2;

    append(*out, "RIFF");
    put_u32(*out, cast(u32) (36 + data_bytes));
    append(*out, "WAVEfmt ");
    put_u32(*out, 16);
    put_u16(*out, 1);
    put_u16(*out, 1);
    put_u32(*out, SAMPLE_RATE);
    put_u32(*out, SAMPLE_RATE * 2);
    put_u16(*out, 2);
    put_u16(*out, 16);
    append(*out, "data");
    put_u32(*out, cast(u32) data_bytes);

    for samples put_u16(*out, cast(u16) ((cast(s32) it) & 0xffff));

    return cast([] u8) builder_to_string(*out);
}
