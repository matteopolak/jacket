// pl_mpeg against the pinned C library (tools/build_native_libs.py): decode pl-mpeg-clip.mpg, a
// 0.4 second MPEG-1 program stream of a solid red 32x32 picture at 25 frames per second with a
// 440 Hz mono MPEG-1 Layer II tone. The clip was made for this test from synthetic sources
// (ffmpeg's `color` and `sine` generators, `-c:v mpeg1video -c:a mp2`), so it has no third-party
// content or license. Checks the headers, the video frames (count, size, colour), the audio
// samples, rewinding, and that input without a stream is refused.
#import "Basic";
#import "File";
#import "Math";
#import "pl_mpeg";

count_frames :: (plm: *Plm) -> s64 {
    count := 0;
    while true {
        frame := decode_video(plm);
        if !frame break;
        count += 1;
    }
    return count;
}

main :: () {
    plm := create_with_filename("pl-mpeg-clip.mpg");
    assert(plm != null, "the clip opens");
    assert(has_headers(plm));
    width, height := get_size(plm);
    assert(width == 32 && height == 32, "size % x %", width, height);
    assert(get_framerate(plm) == 25.0, "framerate %", get_framerate(plm));
    assert(get_num_video_streams(plm) == 1);

    // The first frame is red: BT.601 limited-range YCbCr of (255, 0, 0) is about (82, 90, 240).
    frame := decode_video(plm);
    assert(frame != null);
    assert(frame.width == 32 && frame.height == 32);
    rgb: [32 * 32 * 3] u8;
    frame_to_rgb(frame, rgb.data, 32 * 3);
    for pixel: 0..32 * 32 - 1 {
        red, green, blue := cast(s32) rgb[pixel * 3], cast(s32) rgb[pixel * 3 + 1], cast(s32) rgb[pixel * 3 + 2];
        assert(red > 235 && green < 20 && blue < 20, "pixel % is (%, %, %)", pixel, red, green, blue);
    }
    rgba: [32 * 32 * 4] u8;
    frame_to_rgba(frame, rgba.data, 32 * 4);
    assert(rgba[0] == rgb[0] && rgba[1] == rgb[1] && rgba[2] == rgb[2], "rgba carries the same colour");
    bgra: [32 * 32 * 4] u8;
    frame_to_bgra(frame, bgra.data, 32 * 4);
    assert(bgra[0] == rgb[2] && bgra[2] == rgb[0], "bgra swaps red and blue");

    // The other nine frames follow, then the stream ends.
    remaining := count_frames(plm);
    assert(remaining == 9, "% more frames after the first", remaining);

    // Audio: Layer II frames of 1152 samples at 44.1 kHz, carrying a tone (not silence).
    samples := decode_audio(plm);
    assert(samples != null, "audio decodes");
    assert(samples.count == SAMPLES_PER_FRAME, "% samples in the first block", samples.count);
    loudest: float;
    for 0..SAMPLES_PER_FRAME * 2 - 1 loudest = max(loudest, abs(samples.interleaved[it]));
    assert(loudest > 0.1 && loudest <= 1.0, "peak %", loudest);

    // Rewinding starts over: the same number of frames again.
    rewind(plm);
    assert(count_frames(plm) == 10, "ten frames after a rewind");
    plm_destroy(plm);

    // Video only: with audio switched off the demuxer still yields every frame.
    plm = create_with_filename("pl-mpeg-clip.mpg");
    set_audio_enabled(plm, false);
    assert(count_frames(plm) == 10, "ten frames with audio disabled");
    plm_destroy(plm);

    // A memory buffer decodes the same clip.
    clip, ok := read_entire_file("pl-mpeg-clip.mpg");
    assert(ok);
    bytes: [] u8;
    bytes.data = xx clip.data;
    bytes.count = clip.count;
    from_memory := create_with_memory(bytes, false);
    assert(from_memory != null && has_headers(from_memory));
    assert(count_frames(from_memory) == 10);
    plm_destroy(from_memory);

    // A file that is not there, and bytes that are not MPEG, give no decoder.
    assert(create_with_filename("no-such-clip.mpg") == null);
    junk: [64] u8;
    for 0..63 junk[it] = cast,trunc(u8) (it * 7 + 3);
    not_mpeg := create_with_memory(junk, false);
    assert(not_mpeg == null || !has_headers(not_mpeg), "junk has no headers");

    print("ok\n");
}
