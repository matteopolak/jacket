Waveformatex :: struct #no_padding {
    wFormatTag: s16;
    nChannels: s16;
    nSamplesPerSec: s32;
    nAvgBytesPerSec: s32;
    nBlockAlign: s16;
    wBitsPerSample: s16;
    cbSize: s16;
}

Wave_Extra_Info :: struct {
    wSamplesPerBlock: s32;
    wSamplesAccordingToFactChunk: s32;
}

WAVE_FORMAT_PCM :: 0x0001;
WAVE_FORMAT_DVI_ADPCM :: 0x0011;

get_wav_header :: (file_data: string) -> (format: Waveformatex, samples: string, success: bool = false, extra: Wave_Extra_Info) {
    result: Waveformatex;
    details: Wave_Extra_Info;
    data: string;

    if file_data.count < 12 return result, data, false, details;
    if !tag(file_data.data, "RIFF") || !tag(file_data.data + 8, "WAVE") return result, data, false, details;

    limit := min(file_data.count, cast(s64) read_u32(file_data.data + 4) + 8);
    cursor: s64 = 12;
    have_format := false;
    have_data := false;

    while cursor <= limit - 8 {
        header := file_data.data + cursor;
        length := cast(s64) read_u32(header + 4);

        cursor += 8;
        available := min(length, limit - cursor);
        bytes := file_data.data + cursor;

        if tag(header, "fmt ") && !have_format {
            if available < 16 return result, data, false, details;

            result.wFormatTag = cast(s16) read_u16(bytes);
            result.nChannels = cast(s16) read_u16(bytes + 2);
            result.nSamplesPerSec = cast(s32) read_u32(bytes + 4);
            result.nAvgBytesPerSec = cast(s32) read_u32(bytes + 8);
            result.nBlockAlign = cast(s16) read_u16(bytes + 12);
            result.wBitsPerSample = cast(s16) read_u16(bytes + 14);
            if result.wFormatTag != WAVE_FORMAT_PCM && result.wFormatTag != WAVE_FORMAT_DVI_ADPCM return result, data, false, details;
            if available >= 18 result.cbSize = cast(s16) read_u16(bytes + 16);

            if result.wFormatTag == WAVE_FORMAT_DVI_ADPCM {
                if available < 20 || cast(u16) result.cbSize < 2 return result, data, false, details;
                details.wSamplesPerBlock = cast(s32) read_u16(bytes + 18);
            }

            have_format = true;
        } else if tag(header, "fact") {
            if available < 4 return result, data, false, details;
            details.wSamplesAccordingToFactChunk = cast(s32) read_u32(bytes);
        } else if tag(header, "data") && !have_data {
            data = .{ available, bytes };
            have_data = true;
        }

        if length > limit - cursor break;

        cursor += length + (length & 1);
    }

    return result, data, have_format && have_data, details;
}

#scope_file
#import "Basic";

tag :: (bytes: *u8, expected: string) -> bool {
    for i: 0..3 if bytes[i] != expected[i] return false;
    return true;
}

read_u16 :: (bytes: *u8) -> u16 {
    low := cast(u16) bytes[0];
    high := cast(u16) bytes[1];
    return low | (high << 8);
}

read_u32 :: (bytes: *u8) -> u32 {
    a := cast(u32) bytes[0];
    b := cast(u32) bytes[1];
    c := cast(u32) bytes[2];
    d := cast(u32) bytes[3];

    return a | (b << 8) | (c << 16) | (d << 24);
}
