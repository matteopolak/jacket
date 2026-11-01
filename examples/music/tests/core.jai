#import "Basic";
#import "File";
#import "Math";
#import "Extensions/Testing";
#import,dir "../player";
Audio :: #import "Extensions/Audio_Stream";

lrc_timestamps_offsets_and_duplicates :: () {
    lyrics := parse_lyrics("[ar:Someone]\r\n[00:02.50][00:01.2]hello\r\n[00:61]bad\n[00:01.200]second\n[offset:-200]\n[00:00.001]start\n");

    defer free_lyrics(*lyrics);
    expect_equal(lyrics.count, 4);
    expect_equal(lyrics[0].milliseconds, 0);
    expect_equal(lyrics[1].milliseconds, 1000);
    expect_equal(lyrics[1].text, "hello");
    expect_equal(lyrics[2].text, "second");
    expect_equal(lyrics[3].milliseconds, 2300);
    expect_equal(lyric_at(lyrics, -1), -1);
    expect_equal(lyric_at(lyrics, 999), 0);
    expect_equal(lyric_at(lyrics, 1000), 2);
    expect_equal(lyric_at(lyrics, 99999), 3);
} @test

queue_repeat_shuffle_and_removal :: () {
    q: Queue;

    defer array_free(q.items);
    expect(!advance(*q));
    enqueue(*q, 10);
    enqueue(*q, 20);
    enqueue(*q, 30);
    expect(advance(*q));
    expect_equal(current_track(*q), 10);
    q.repeat = .ONE;
    expect(advance(*q, true));
    expect_equal(current_track(*q), 10);
    expect(advance(*q));
    expect_equal(current_track(*q), 20);
    expect(!remove_queued(*q, 1));
    expect(remove_queued(*q, 0));
    expect_equal(current_track(*q), 20);
    expect(advance(*q));
    expect_equal(current_track(*q), 30);
    q.repeat = .OFF;
    expect(!advance(*q, true));
    q.repeat = .ALL;
    expect(advance(*q));
    expect_equal(current_track(*q), 20);
    q.shuffle = true;
    previous := q.cursor;

    for 0..20 {
        expect(advance(*q));
        expect(q.cursor != previous);
        previous = q.cursor;
    }
} @test

mix_math_saturates_and_interpolates :: () {
    expect_equal(mixed_sample(0, 16000, 0.5, 0.5), 4000);
    expect_equal(mixed_sample(32000, 32000, 0.25, 2), 32767);
    expect_equal(mixed_sample(-32000, -32000, 0, 2), -32768);
    expect_equal(mixed_sample(-100, 100, 0.25, 1), -50);
} @test

generated_decoder_resampling_chunking_and_seek :: (t: *Test) {
    path := tprint("%/sweep.wav", temp_dir(t));

    require(write_sweep(path));
    r: Audio.Reader;

    require(Audio.open(*r, path));
    defer Audio.close(*r);
    expect_equal(r.rate, 8000);
    expect_equal(r.channels, 1);
    expect_equal(r.frames, 8000);
    original := NewArray(8000, s16);

    defer array_free(original);
    expect_equal(Audio.read(*r, original), 8000);
    m: Mixer;

    init_mixer(*m, *r, 16000);
    defer free_mixer(*m);
    whole := NewArray(32000, s16);

    defer array_free(whole);
    expect_equal(mix(*m, whole, 0.5), 16000);
    expect(!m.failed);

    for i: 0..7999 {
        expected := mixed_sample(original[i], original[i], 0, 0.5);

        expect_equal(whole[i * 4], expected);
        expect_equal(whole[i * 4 + 1], expected);
        next := min(i + 1, 7999);

        expect_equal(whole[i * 4 + 2], mixed_sample(original[i], original[next], 0.5, 0.5));
    }

    seek_mixer(*m, 0);
    chunked := NewArray(32000, s16);

    defer array_free(chunked);
    // Deliberately odd frame counts cross both the source cache and half-sample boundaries.
    offset: s64;
    while offset < 32000 {
        count := min(cast(s64) 226, 32000 - offset);
        expect_equal(mix(*m, .{ count, chunked.data + offset }, 0.5), count / 2);
        offset += count;
    }

    expect_equal(checksum(chunked), checksum(whole));
    seek_mixer(*m, 0.375);
    sample: [6] s16;

    expect_equal(mix(*m, sample, 0.5), 3);
    for sample expect_equal(it, whole[12000 + it_index]);

    seek_mixer(*m, 999);
    expect_equal(mix(*m, sample), 0);
} @test

library_recurses_sorts_and_reports_bad_audio :: (t: *Test) {
    root := temp_dir(t);

    require(make_directory_if_it_does_not_exist(tprint("%/sub", root)));
    require(write_sweep(tprint("%/z.wav", root), frames = 16));
    require(write_sweep(tprint("%/sub/a.wav", root), frames = 16));
    require(write_entire_file(tprint("%/bad.wav", root), "bad"));
    require(write_entire_file(tprint("%/ignored.mp3", root), "bad"));
    library := scan_library(root);

    defer free_library(*library);
    expect_equal(library.tracks.count, 2);
    expect_equal(library.errors.count, 1);
    expect_equal(library.tracks[0].title, "a.wav");
    expect_equal(library.tracks[1].title, "z.wav");
} @test

waveform_runs_on_a_separate_decoder :: (t: *Test) {
    path := tprint("%/sweep.wav", temp_dir(t));

    require(write_sweep(path));
    wave := New(Waveform);

    defer {
        finish_waveform(wave);
        free(wave);
    }

    require(start_waveform(wave, path));
    while !poll_waveform(wave) sleep_milliseconds(1);

    expect(wave.ok);

    for wave.peaks {
        expect(it > 0.3);
        expect(it < 0.4);
    }
} @test

stereo_mix_keeps_the_channels_separate :: () {
    r := Audio.Reader.{ channels = 2, rate = 8000, frames = 2 };
    samples := s16.[1000, -2000, 3000, -4000];
    m := Mixer.{ reader = *r, samples = samples, start = 0, count = 2, output_rate = 8000 };
    output: [4] s16;

    expect_equal(mix(*m, output, 0.5), 2);
    expect_equal(output[0], 500);
    expect_equal(output[1], -1000);
    expect_equal(output[2], 1500);
    expect_equal(output[3], -2000);
} @test

shuffle_visits_each_entry_once_then_stops :: () {
    q: Queue;

    defer array_free(q.items);
    for 0..9 enqueue(*q, it);

    q.shuffle = true;
    seen: [10] bool;

    for 0..9 {
        require(advance(*q));
        current := current_track(*q);
        expect(!seen[current]);
        seen[current] = true;
    }

    expect(!advance(*q, true));
    last := current_track(*q);

    q.repeat = .ALL;
    require(advance(*q, true));
    expect(current_track(*q) != last);
} @test
