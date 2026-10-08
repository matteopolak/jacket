// A game's build script, shaped like many small Simp projects': `jaic run first.jai` compiles
// src/main.jai in a second workspace into build/game(.exe), then runs it. CI runs it on Windows
// (windows-native.yml) without Clang on PATH, so the link goes through Visual Studio's link.exe
// (docs/native/windows.md); the game draws a few frames and exits with 0.
#run build();

#import "Compiler";
#import "File";
#import "Basic";
#import "Process";

build :: () {
    set_build_options_dc(.{ do_output = false, write_added_strings = false });

    w := compiler_create_workspace("Build");
    if !w {
        log_error("Workspace creation failed.\n");
        exit(1);
    }

    make_directory_if_it_does_not_exist("build");

    options := get_build_options(w);
    options.output_path = "build";
    options.output_executable_name = "game";
    set_build_options(options, w);

    compiler_begin_intercept(w, .SKIP_ALL);
    add_build_file("src/main.jai", w);

    failed := false;
    while true {
        message := compiler_wait_for_message();
        if message.kind == .ERROR failed = true;
        if message.kind == .COMPLETE {
            complete := cast(*Message_Complete) message;
            if complete.error_code != .NONE failed = true;
            break;
        }
    }
    compiler_end_intercept(w);
    if failed {
        log_error("The game did not build.\n");
        exit(1);
    }

    #if OS == .WINDOWS {
        result := run_command("build/game.exe");
    } else {
        result := run_command("./build/game");
    }
    if result.type != .EXITED || result.exit_code != 0 {
        log_error("The game stopped with %.\n", result);
        exit(1);
    }
    print("first.jai: the game built and ran\n");
}
