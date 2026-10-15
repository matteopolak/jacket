#import "Basic";
M :: #import "MacOS_Bundler";
F :: #import "File";
S :: #import "String";
P :: #import "Process";

capture_error :: (message: string, data: *void, info: Log_Info) {
    assert(message.count > 0 && (info.common_flags & .ERROR) != 0);
    (cast(*s64) data).* += 1;
}

command :: (args: ..string) {
    status := P.run_command(..args);
    assert(status.type == .EXITED && status.exit_code == 0);
}

main :: () {
    root := sprint("%/../../../target/portable-tests/bundle app fixtures", S.path_strip_filename(#file));

    defer free(root);
    command("/bin/mkdir", "-p", root);
    source := sprint("%/source files", root);

    defer free(source);
    command("/bin/mkdir", "-p", source);
    hidden := sprint("%/.hidden", source);

    defer free(hidden);
    assert(F.write_entire_file(hidden, "hidden resource"));
    executable := sprint("%/source program", root);

    defer free(executable);
    assert(F.write_entire_file(executable, "#!/bin/sh\nprintf 'bundle-child\\n'\n"));
    command("/bin/chmod", "+x", executable);
    image := sprint("%/../../../target/portable-tests/bundle-image.png", S.path_strip_filename(#file));

    defer free(image);
    icon := sprint("%/Converted.icns", root);

    defer free(icon);
    assert(M.convert_image_to_icns(image, icon));
    roundtrip := sprint("%/roundtrip.iconset", root);

    defer free(roundtrip);
    command("/usr/bin/iconutil", "-c", "iconset", icon, "-o", roundtrip);
    errors: s64;
    scoped := context;
    scoped.logger = capture_error;
    scoped.logger_data = *errors;
    push_context scoped {
        assert(!M.convert_image_to_icns(hidden, icon));
        failed, failed_resources := M.create_app_bundle("", executable, "", source, true, false);

        assert(failed.count == 0 && failed_resources.count == 0);
        missing := sprint("%/missing executable", root);
        failed_name := sprint("%/Failed", root);

        defer free(missing);
        defer free(failed_name);
        failed, failed_resources = M.create_app_bundle(failed_name, missing, "", "", true, true);
        assert(failed.count == 0 && failed_resources.count == 0);
    }

    assert(errors > 0);

    for contents_only: bool.[true, false] {
        name := sprint("%/Example %", root, contents_only);

        defer free(name);
        bundle, resources := M.create_app_bundle(name, executable, icon, source, contents_only, true);

        defer free(bundle);
        defer free(resources);
        assert(bundle.count > 0 && resources.count > 0);
        plist_path := sprint("%/Contents/Info.plist", bundle);

        defer free(plist_path);
        command("/usr/bin/plutil", "-lint", "-s", plist_path);
        copied := sprint("%/Contents/MacOS/source program", bundle);

        defer free(copied);
        status, out, err := P.run_command(copied, capture_and_return_output = true);

        defer free(out);
        defer free(err);
        assert(status.type == .EXITED && status.exit_code == 0 && out == "bundle-child\n" && err == "");
        hidden_copy := sprint("%/%", resources, ".hidden");

        if !contents_only {
            free(hidden_copy);
            hidden_copy = sprint("%/source files/.hidden", resources);
        }

        defer free(hidden_copy);
        text, ok := F.read_entire_file(hidden_copy);

        defer free(text);
        assert(ok && text == "hidden resource");
    }

    bare_name := sprint("%/Bare.app", root);

    defer free(bare_name);
    bare, bare_resources := M.create_app_bundle(bare_name, executable, "", "", true, false);

    defer free(bare);
    defer free(bare_resources);
    assert(bare == bare_name);
    absent := sprint("%/Contents/Info.plist", bare);

    defer free(absent);
    file, exists := F.file_open(absent, log_errors = false);

    assert(!exists);
    F.file_close(*file);
    print("ok\n");
}
