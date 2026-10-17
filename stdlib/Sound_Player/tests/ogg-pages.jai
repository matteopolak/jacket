#import "Basic";
#import "File";
#import "Sound_Player";

main :: () {
    bytes, ok := read_entire_file("fixtures/tone.ogg");

    assert(ok);
    defer free(bytes);
    frames, channels, rate := get_compressed_rate_channels_and_size(bytes);

    assert(frames == 4416 && channels == 2 && rate == 44100, "metadata: % % %", frames, channels, rate);
    data: Sound_Data;
    data.loaded = true;
    data.type = .OGG_COMPRESSED;
    data.buffer = bytes;
    data.nchannels = 2;
    data.sampling_rate = 44100;
    data.nsamples_times_nchannels = frames * channels;
    data.filename_to_stream_from = "metadata-only-do-not-open.ogg";
    decoder := create_ogg_decoder(*data, 1024);

    assert(decoder != null);
    mark_relevant_pages(decoder, 0, frames);
    assert(decoder.pages.count == 5);
    energy: [2] float64;

    for page: decoder.pages {
        assert(page.num_samples_contained == min(cast(s64) 1024, frames - page.start_address));
        pcm := cast(*s16) page.data;

        for i: 0..page.num_samples_contained - 1 {
            for c: 0..1 {
                sample := cast(float64) pcm[i * 2 + c] / 32768.0;
                energy[c] += sample * sample;
            }
        }
    }

    assert(energy[0] / frames > 0.025 && energy[0] / frames < 0.04);
    assert(energy[1] / frames > 0.10 && energy[1] / frames < 0.15);
    unmark_all_pages(decoder);
    free_unmarked_pages(decoder);
    assert(decoder.pages.count == 0);
    mark_relevant_pages(decoder, 2048, 3072);
    assert(decoder.pages.count == 1 && decoder.pages[0].num_samples_contained == 1024);
    deinit(decoder);
    print("ok\n");
}
