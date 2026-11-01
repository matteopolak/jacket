// Bounded, seekable decoding of WAV PCM/IMA ADPCM and Ogg Vorbis files.
// A reader owns its file and scratch block; it never loads the whole recording.

#import "Basic";
#import "String";
#import "File";
W :: #import "Wav_File";
A :: #import "Adpcm";
V :: #import "stb_vorbis";

// An open decoder. Keep it on one thread, close it before reuse, and do not copy it.
// `position` and `frames` count frames, not individual interleaved channel samples.
Reader :: struct {
    file: File;
    opened: bool;
    vorbis: *V.stb_vorbis;
    channels: s32;
    rate: s32;
    frames: s64;
    position: s64;
    artist: string;
    album: string;
    title: string;
    format: W.Waveformatex;
    extra: W.Wave_Extra_Info;
    offset: s64;
    block: [] u8;
    decoded: [] s16;
    cached_block: s64 = -1;
}

// Opens a file and copies Vorbis ARTIST/ALBUM/TITLE comments. Returns false on unsupported
// or malformed headers. PCM 8/16/24/32-bit and IMA ADPCM, one or two channels, are accepted.
// ```jai,no_run
// r: Reader;
// if open(*r, "song.ogg") { defer close(*r); assert(r.rate > 0); }
// ```
open :: (r: *Reader, path: string) -> bool {
    r.* = .{};
    lower := to_lower_copy(path);

    defer free(lower);

    if ends_with(lower, ".ogg") {
        error: s32;
        c_path := to_c_string(path);

        defer free(c_path);
        r.vorbis = V.stb_vorbis_open_filename(c_path, *error, null);
        if !r.vorbis return false;

        info := V.stb_vorbis_get_info(r.vorbis);

        if info.sample_rate > 0x7fffffff {
            close(r);
            return false;
        }

        r.channels = info.channels;
        r.rate = cast(s32) info.sample_rate;
        r.frames = V.stb_vorbis_stream_length_in_samples(r.vorbis);
        if !valid(r) {
            close(r);
            return false;
        }

        comments := V.stb_vorbis_get_comment(r.vorbis);
        for i: 0..comments.comment_list_length - 1 {
            comment := to_string(comments.comment_list[i]);
            equal := find_index_from_left(comment, #char "=");

            if equal < 0 continue;

            name := to_upper_copy(slice(comment, 0, equal));

            defer free(name);
            value := slice(comment, equal + 1, comment.count - equal - 1);

            if name == "ARTIST" {
                free(r.artist);
                r.artist = copy_string(value);
            }

            if name == "ALBUM" {
                free(r.album);
                r.album = copy_string(value);
            }

            if name == "TITLE" {
                free(r.title);
                r.title = copy_string(value);
            }
        }

        return true;
    }

    r.file, r.opened = file_open(path);
    if !r.opened return false;

    if !wav_header(r) {
        close(r);
        return false;
    }

    return true;
}

// Releases file handles, comments and block buffers. Safe after a failed open or repeated close.
close :: (r: *Reader) {
    if r.vorbis V.stb_vorbis_close(r.vorbis);
    if r.opened file_close(*r.file);

    free(r.artist);
    free(r.album);
    free(r.title);
    array_free(r.block);
    array_free(r.decoded);
    r.* = .{};
}

// Moves to an exact source frame; out-of-range requests clamp to the recording's ends.
seek :: (r: *Reader, frame: s64) -> bool {
    target := clamp(frame, cast(s64) 0, r.frames);
    if r.vorbis && target < r.frames && V.stb_vorbis_seek(r.vorbis, cast(u32) target) == 0 return false;
    r.position = target;
    return true;
}

// Reads interleaved signed 16-bit samples into caller storage. Returns frames decoded, zero at
// EOF, or -1 on a file/codec failure. Storage must fit a whole number of frames.
read :: (r: *Reader, output: [] s16) -> s64 {
    if !valid(r) || output.count % r.channels != 0 return -1;

    count := min(output.count / r.channels, r.frames - r.position);
    if count <= 0 return 0;

    if r.vorbis {
        count = min(count, cast(s64) 0x7fffffff / r.channels);
        got := V.stb_vorbis_get_samples_short_interleaved(r.vorbis, r.channels, output.data, cast(s32) count * r.channels);
        if got <= 0 return -1;

        r.position += got;

        return got;
    }

    for f: 0..count - 1 {
        if r.format.wFormatTag == W.WAVE_FORMAT_DVI_ADPCM {
            block_index := r.position / r.extra.wSamplesPerBlock;
            if block_index != r.cached_block {
                if !file_set_position(r.file, r.offset + block_index * r.block.count) return -1;

                ok, got := file_read(r.file, r.block.data, r.block.count);
                if !ok || got != r.block.count return -1;
                for c: 0..r.channels - 1 if r.block[c * 4 + 2] > 88 return -1;

                bytes: [] u8 = .{ r.decoded.count * 2, cast(*u8) r.decoded.data };

                A.IMA_ADPCM_decode(.{ r.channels, cast(s32) r.block.count, r.extra.wSamplesPerBlock }, r.block, bytes);
                r.cached_block = block_index;
            }

            index := r.position % r.extra.wSamplesPerBlock;
            for c: 0..r.channels - 1 output[f * r.channels + c] = r.decoded[index * r.channels + c];
        } else {
            // Refill a bounded PCM block. Its index is in frames, like the ADPCM cache.
            frames_per_block := r.block.count / r.format.nBlockAlign;
            block_index := r.position / frames_per_block;

            if block_index != r.cached_block {
                start := block_index * frames_per_block;
                bytes := min(frames_per_block, r.frames - start) * r.format.nBlockAlign;

                if !file_set_position(r.file, r.offset + start * r.format.nBlockAlign) return -1;

                ok, got := file_read(r.file, r.block.data, bytes);
                if !ok || got != bytes return -1;

                r.cached_block = block_index;
            }

            index := r.position % frames_per_block;
            width := r.format.wBitsPerSample / 8;

            for c: 0..r.channels - 1 {
                p := r.block.data + index * r.format.nBlockAlign + c * width;
                value: s16;

                if width == 1 value = cast(s16) ((cast(s32) p[0] - 128) * 256);
                else value = cast,no_check(s16) (cast(u16) p[width - 2] | ((cast(u16) p[width - 1]) << 8));

                output[f * r.channels + c] = value;
            }
        }

        r.position += 1;
    }

    return count;
}

#scope_file
valid :: (r: *Reader) -> bool {
    return r.channels > 0 && r.channels <= 2 && r.rate > 0 && r.frames > 0;
}

u16le :: (p: *u8) -> u16 {
    return cast(u16) p[0] | ((cast(u16) p[1]) << 8);
}

u32le :: (p: *u8) -> u32 {
    return cast(u32) u16le(p) | ((cast(u32) u16le(p + 2)) << 16);
}

wav_header :: (r: *Reader) -> bool {
    length, sized := file_length(r.file);
    if !sized || length < 12 return false;

    riff: [12] u8;
    ok, got := file_read(r.file, riff.data, 12);

    if !ok || got != 12 || string.{ 4, riff.data } != "RIFF" || string.{ 4, riff.data + 8 } != "WAVE" return false;

    limit := cast(s64) u32le(riff.data + 4) + 8;
    if limit > length return false;

    // Build a tiny RIFF header for Wav_File, keeping the sample payload on disk.
    header: [96] u8;

    memcpy(header.data, riff.data, 12);
    used: s64 = 12;
    cursor: s64 = 12;
    payload: s64 = -1;

    while cursor <= limit - 8 {
        chunk: [8] u8;

        if !file_set_position(r.file, cursor) return false;

        ok, got = file_read(r.file, chunk.data, 8);
        if !ok || got != 8 return false;

        size := cast(s64) u32le(chunk.data + 4);
        if size > limit - cursor - 8 return false;

        tag := string.{ 4, chunk.data };
        if tag == "fmt " || tag == "fact" {
            if size > 40 || used + 8 + size + (size & 1) > 88 return false;

            memcpy(header.data + used, chunk.data, 8);
            ok, got = file_read(r.file, header.data + used + 8, size);
            if !ok || got != size return false;

            used += 8 + size + (size & 1);
        } else if tag == "data" && payload < 0 {
            r.offset = cursor + 8;
            payload = size;
        }

        cursor += 8 + size + (size & 1);
    }

    if payload < 0 return false;

    memcpy(header.data + used, "data".data, 4);
    used += 8;
    // The bounded header's own RIFF length must include its empty data chunk.
    value := used - 8;
    for i: 0..3 header[4 + i] = cast,trunc(u8) (value >> (i * 8));

    success: bool;
    r.format, _, success, r.extra = W.get_wav_header(.{ used, header.data });
    if !success return false;

    r.channels = r.format.nChannels;
    r.rate = r.format.nSamplesPerSec;
    if r.channels < 1 || r.channels > 2 || r.rate <= 0 return false;

    align := cast(s64) r.format.nBlockAlign;

    if r.format.wFormatTag == W.WAVE_FORMAT_PCM {
        bits := r.format.wBitsPerSample;
        if (bits != 8 && bits != 16 && bits != 24 && bits != 32) || align != r.channels * bits / 8 return false;
        if payload % align != 0 return false;

        r.frames = payload / align;
        r.block = NewArray(4096 * align, u8);
    } else {
        if r.format.wBitsPerSample != 4 || align < r.channels * 4 || payload % align != 0 return false;

        rest := align - r.channels * 4;
        if rest % (r.channels * 4) != 0 || r.extra.wSamplesPerBlock != 1 + rest * 2 / r.channels return false;

        r.frames = payload / align * r.extra.wSamplesPerBlock;
        if r.extra.wSamplesAccordingToFactChunk > 0 r.frames = min(r.frames, cast(s64) r.extra.wSamplesAccordingToFactChunk);

        r.block = NewArray(align, u8);
        r.decoded = NewArray(r.extra.wSamplesPerBlock * r.channels, s16);
    }

    return valid(r);
}
