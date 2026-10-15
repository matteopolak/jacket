// Independent MPEG-1 public decode regression. Arguments: enabled|disabled, clip.
// The disabled run must change the DEFAULT Basic parameter used by MPEG too.
B :: #import "Basic";
#import "Basic";
M :: #import "pl_mpeg";
F :: #import "File";

errors: s64;

capture :: (message: string, data: *void, info: Log_Info) {
    if (info.common_flags & .ERROR) != 0 errors += 1;
}

recover :: (loc: Source_Code_Location, message: string) -> bool {
    return false;
}

reject_pack :: (filename: string, mask: u8) {
    bytes, ok := F.read_entire_file(filename);
    if !ok || bytes.count < 12 exit(15);

    defer free(bytes);
    raw := cast(*u8) bytes.data;
    if raw[0] != 0 || raw[1] != 0 || raw[2] != 1 || raw[3] != 0xba exit(16);

    raw[4] &= mask;
    saved := context;
    scoped := context;
    scoped.logger = capture;
    scoped.assertion_failed = recover;
    context = scoped;
    defer context = saved;
    errors = 0;
    decoder := M.create_with_memory(.{ bytes.count, raw }, false);
    if !decoder exit(17);

    defer M.plm_destroy(decoder);
    if M.has_headers(decoder) || errors == 0 exit(18);
}

mark :: (calls: *s64) -> bool {
    calls.* += 1;
    return true;
}

main :: () {
    args := get_command_line_arguments();
    if args.count != 3 exit(2);

    calls: s64;

    B.assert(mark(*calls));

    if args[1] == "enabled" {
        if calls != 1 exit(10);
    } else if args[1] == "disabled" {
        if calls != 0 exit(11);
    } else exit(12);

    decoder := M.create_with_filename(args[2]);
    if !decoder exit(3);

    defer M.plm_destroy(decoder);
    if !M.has_headers(decoder) exit(4);

    width, height := M.get_size(decoder);
    if width != 32 || height != 32 exit(5);
    if M.get_framerate(decoder) != 25 || M.get_samplerate(decoder) != 44100 exit(6);

    frames: s64;
    pixels: [3072] u8;

    while true {
        frame := M.decode_video(decoder);
        if !frame break;

        M.frame_to_rgb(frame, pixels.data, 96);

        for pixel: 0..1023 {
            if pixels[pixel * 3] <= 235 || pixels[pixel * 3 + 1] >= 20 || pixels[pixel * 3 + 2] >= 20 exit(7);
        }

        frames += 1;
        if frames > 10 exit(8);
    }

    if frames != 10 || !decoder.has_ended exit(9);

    samples: s64;

    while M.decode_audio(decoder) {
        samples += 1;
        if samples > 16 exit(13);
    }

    if samples != 16 exit(14);

    reject_pack(args[2], 0x0f); // Unsupported pack version.
    reject_pack(args[2], 0xfe); // Missing marker bit, version retained.
    write_string("ok\n");
}
