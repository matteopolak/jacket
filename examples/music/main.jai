#import "Basic";
#import "File";
#import "Thread";
Audio :: #import "Extensions/Audio_Stream";
#import,dir "player";
Args :: #import "Extensions/Args";
#load "playback.jai";
#load "interface.jai";
#load "smoke.jai";

Cli :: struct {
    folder: string = "."; @positional
    smoke: bool;
    device_smoke: bool;
    generate: bool;
    list: bool;
    screenshot: string;
    frames: s64; @range(0,100000)
}

HELP :: Args.Help(Cli).{
    about = "A streaming WAV / Ogg Vorbis music player with synced lyrics.",
    folder = "Library folder (recursive)",
    smoke = "Run offline decode/mix and headless UI checks; no device is needed",
    device_smoke = "Try brief muted device playback; skip when no device is available",
    generate = "Write a generated sine-sweep WAV and lyrics into the folder, then exit",
    list = "Print the sorted library and decoder metadata, then exit",
    screenshot = "Write one offscreen UI frame as a PNG (requires a GPU)",
    frames = "Close the window after this many frames (default: unlimited)",
};

main :: () {
    cli, _ := Args.parse(Cli, HELP);
    if cli.generate {
        if !make_directory_if_it_does_not_exist(cli.folder, recursive = true) exit(1);
        if !write_sweep(tprint("%/sweep.wav", cli.folder)) exit(1);
        if !write_entire_file(tprint("%/sweep.lrc", cli.folder), "[00:00.00]A generated sine sweep\n[00:00.50]Rising pitch\n") exit(1);
        return;
    }

    if cli.smoke {
        run_smoke(cli.folder);
        return;
    }

    if cli.device_smoke {
        run_device_smoke(cli.folder);
        return;
    }

    app := New(App);

    defer {
        close_app(app);
        free(app);
    }

    app.library = scan_library(cli.folder);

    if cli.list {
        for app.library.tracks print("% | % | % | % Hz | % frames | %\n", it.artist, it.album, it.title, it.rate, it.frames, it.path);
        for app.library.errors print("%\n", it, to_standard_error = true);
        if app.library.errors.count exit(1);
        return;
    }

    if cli.screenshot.count {
        app.offline = true;
        if app.library.tracks.count {
            enqueue(*app.queue, 0);
            app.queue.cursor = 0;
            select_track(app, 0);
            while !poll_waveform(*app.wave) sleep_milliseconds(1);

            app.wave_ready = true;
        }

        ui := headless(.{ width = 1100, height = 760, system_fonts = true });

        defer free_ui(*ui);
        frame(*ui, app, build);
        if !write_screenshot(*ui, cli.screenshot) exit(1);

        return;
    }

    init_playback(*app.playback);
    if !run_app(.{ title = "Music", width = 1100, height = 760, frame_limit = cli.frames }, app, build) exit(1);
}
