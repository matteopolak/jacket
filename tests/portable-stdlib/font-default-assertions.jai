// Reviewed Simp CPU font API and shared-face pixel-size contract.
// Arguments: enabled|disabled, metrics|features|glyphsize|sdk-error, private font.
#import "Basic";
S :: #import "Simp"(render_api = .NONE);
F :: #import "File";

errors: s64;

capture :: (message: string, data: *void, info: Log_Info) {
    if (info.common_flags & .ERROR) != 0 errors += 1;
}

recover :: (loc: Source_Code_Location, message: string) -> bool {
    return false;
}

mark :: (calls: *s64) -> bool {
    calls.* += 1;
    return true;
}

main :: () {
    args := get_command_line_arguments();
    if args.count != 4 exit(2);

    calls: s64;

    assert(mark(*calls));

    if args[1] == "enabled" {
        if calls != 1 exit(10);
    } else if args[1] == "disabled" {
        if calls != 0 exit(11);
    } else exit(12);

    bytes, read := F.read_entire_file(args[3]);
    if !read exit(3);

    // Loaded faces borrow these bytes for the module registry lifetime.
    S.init_fonts(128, 128);
    view: [] u8 = .{ bytes.count, bytes.data };

    if args[2] == "sdk-error" {
        saved := context;
        scoped := context;
        scoped.logger = capture;
        scoped.assertion_failed = recover;
        context = scoped;
        failed := S.get_font_at_size(view, 24);

        context = saved;
        if failed != null || errors == 0 exit(18);

        retry := S.get_font_at_size(view, 8);
        if !retry || retry.private1.size.metrics.y_ppem != 8 exit(19);

        write_string("ok\n");

        return;
    }

    font := S.get_font_at_size(view, 24);

    if !font || !font.private1 exit(4);
    if font.private1.size.metrics.y_ppem != 24 || font.em_width != 24 exit(5);
    if S.get_baseline_height(font) <= 0 exit(6);

    if args[2] == "features" {
        if !font.feature_smallcaps || !font.feature_lining_numbers exit(7);
        width := S.prepare_text(font, "a1", .SMALLCAPS | .LINING_FIGURES);
        if width <= 0 || font.temporary_glyphs.count != 2 exit(8);
        if !font.feature_smallcaps.expanded || !font.feature_lining_numbers.expanded exit(9);
    } else if args[2] == "glyphsize" {
        larger := S.get_font_at_size(view, 32);
        if !larger || larger.private1 != font.private1 exit(13);
        if larger.private1.size.metrics.y_ppem != 32 exit(14);

        width := S.prepare_text(font, "A");
        if width <= 0 || font.temporary_glyphs.count != 1 exit(15);
        if font.private1.size.metrics.y_ppem != 24 exit(16);
    } else if args[2] != "metrics" exit(17);

    write_string("ok\n");
}
