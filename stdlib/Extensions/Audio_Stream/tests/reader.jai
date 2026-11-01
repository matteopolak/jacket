#import "Basic";
#import "File";
#import "Math";
#import "Extensions/Testing";
Audio :: #import "Extensions/Audio_Stream";

pcm_seek_crosses_cache_boundary :: (t: *Test) {
    path := tprint("%/tone.wav", temp_dir(t));
    bytes := NewArray(44 + 10000, u8);

    defer array_free(bytes);
    prefix := "RIFF\x34\x27\x00\x00WAVEfmt \x10\x00\x00\x00\x01\x00\x01\x00\x40\x1f\x00\x00\x80\x3e\x00\x00\x02\x00\x10\x00data\x10\x27\x00\x00";

    memcpy(bytes.data, prefix.data, 44);

    for i: 0..4999 {
        bytes[44 + i * 2] = cast,trunc(u8) i;
        bytes[45 + i * 2] = cast,trunc(u8) (i >> 8);
    }

    require(write_entire_file(path, .{ bytes.count, bytes.data }));
    r: Audio.Reader;

    require(Audio.open(*r, path));
    defer Audio.close(*r);
    expect_equal(r.frames, 5000);
    expect_equal(r.rate, 8000);
    output: [12] s16;

    require(Audio.seek(*r, 4090));
    expect_equal(Audio.read(*r, output), 12);
    for output expect_equal(it, cast(s16) (4090 + it_index));

    require(Audio.seek(*r, 4998));
    expect_equal(Audio.read(*r, output), 2);
    expect_equal(output[0], 4998);
    expect_equal(Audio.read(*r, output), 0);
    require(Audio.seek(*r, -10));
    expect_equal(r.position, 0);
    expect_equal(Audio.read(*r, output), 12);
    expect_equal(output[0], 0);
} @test

ima_block_and_exact_seek :: (t: *Test) {
    path := tprint("%/ima.wav", temp_dir(t));
    // One IMA block: predictor 1000, index 0, eight zero nibbles -> nine samples of 1000.
    bytes := "RIFF\x30\x00\x00\x00WAVEfmt \x14\x00\x00\x00\x11\x00\x01\x00\x40\x1f\x00\x00\x00\x00\x00\x00\x08\x00\x04\x00\x02\x00\x09\x00data\x08\x00\x00\x00\xe8\x03\x00\x00\x00\x00\x00\x00";

    require(write_entire_file(path, bytes));
    r: Audio.Reader;

    require(Audio.open(*r, path));
    defer Audio.close(*r);
    expect_equal(r.frames, 9);
    output: [4] s16;

    require(Audio.seek(*r, 5));
    expect_equal(Audio.read(*r, output), 4);
    for output expect_equal(it, 1000);
} @test

bad_and_truncated_files_are_rejected :: (t: *Test) {
    path := tprint("%/bad.wav", temp_dir(t));

    require(write_entire_file(path, "RIFF\xff\xff\xff\x7fWAVE"));
    r: Audio.Reader;

    expect(!Audio.open(*r, path));
    Audio.close(*r);
    Audio.close(*r);
    expect(!Audio.open(*r, tprint("%/missing.ogg", temp_dir(t))));
} @test

vorbis_file_seek_preserves_stereo :: () {
    r: Audio.Reader;
    fixture := tprint("%../../../Sound_Player/tests/fixtures/tone.ogg", #filepath);

    require(Audio.open(*r, fixture));
    defer Audio.close(*r);
    expect_equal(r.channels, 2);
    expect_equal(r.rate, 44100);
    expect_equal(r.frames, 4416);
    first: [512] s16;
    again: [512] s16;

    require(Audio.seek(*r, 1000));
    expect_equal(Audio.read(*r, first), 256);
    require(Audio.seek(*r, 1000));
    expect_equal(Audio.read(*r, again), 256);
    energy: s64;

    for first {
        expect_equal(it, again[it_index]);
        energy += abs(cast(s64) it);
    }

    expect(energy > 0);
} @test

integer_pcm_widths_preserve_stereo :: (t: *Test) {
    for width: s64.[1, 2, 3, 4] {
        path := tprint("%/pcm%.wav", temp_dir(t), width);
        bytes := NewArray(44 + 4 * width, u8);

        defer array_free(bytes);
        prefix := "RIFF\x00\x00\x00\x00WAVEfmt \x10\x00\x00\x00\x01\x00\x02\x00\x40\x1f\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00data\x00\x00\x00\x00";

        memcpy(bytes.data, prefix.data, 44);

        for i: 0..3 {
            bytes[4 + i] = cast,trunc(u8) ((bytes.count - 8) >> (8 * i));
            bytes[40 + i] = cast,trunc(u8) ((4 * width) >> (8 * i));
        }

        bytes[32] = cast(u8) (width * 2);
        bytes[34] = cast(u8) (width * 8);

        if width == 1 {
            bytes[44] = 128;
            bytes[45] = 255;
            bytes[46] = 0;
            bytes[47] = 64;
        } else {
            values := s16.[0, 32767, -32768, -16384];
            for value, index: values {
                bits := cast,no_check(u16) value;
                bytes[44 + index * width + width - 2] = cast,trunc(u8) bits;
                bytes[44 + index * width + width - 1] = cast,trunc(u8) (bits >> 8);
            }
        }

        require(write_entire_file(path, .{ bytes.count, bytes.data }));
        r: Audio.Reader;

        require(Audio.open(*r, path));
        defer Audio.close(*r);
        output: [4] s16;

        expect_equal(Audio.read(*r, output), 2);
        expect_equal(output[0], 0);
        expect_equal(output[1], ifx width == 1 then 32512 else 32767);
        expect_equal(output[2], -32768);
        expect_equal(output[3], -16384);
    }
} @test

ima_rejects_an_invalid_step_index :: (t: *Test) {
    path := tprint("%/bad-ima.wav", temp_dir(t));
    bytes := "RIFF\x30\x00\x00\x00WAVEfmt \x14\x00\x00\x00\x11\x00\x01\x00\x40\x1f\x00\x00\x00\x00\x00\x00\x08\x00\x04\x00\x02\x00\x09\x00data\x08\x00\x00\x00\xe8\x03\x59\x00\x00\x00\x00\x00";

    require(write_entire_file(path, bytes));
    r: Audio.Reader;

    require(Audio.open(*r, path));
    defer Audio.close(*r);
    output: [4] s16;

    expect_equal(Audio.read(*r, output), -1);
} @test
