Args :: #import "Extensions/Args";
#load "app.jai";
#load "smoke.jai";

Cli :: struct {
    path: string = "."; @positional
    smoke: bool;
    cache_mb: s64 = 32; @range(1,512)
}

HELP :: Args.Help(Cli).{
    about = "Browse images, zoom, pan, rotate and export PNG/JPEG.",
    path = "File or folder to open; with --smoke, output folder for generated fixtures",
    smoke = "Generate fixtures and run a headless scripted session",
    cache_mb = "Thumbnail memory cap in MiB (default 32)",
};

main :: () {
    cli, _ := Args.parse(Cli, HELP);
    if cli.smoke {
        smoke(cli.path);
        return;
    }

    app: Viewer;

    defer close_viewer(*app);
    app.cache.cap = cli.cache_mb * 1024 * 1024;
    app.open_path.single_line = true;
    app.export_path.single_line = true;
    set_contents(*app.open_path, cli.path);
    set_contents(*app.export_path, "export.png");

    if !open_target(*app, cli.path) {
        print("%\n", app.status);
        exit(1);
    }

    if !run_app(.{ title = "Imageview", width = 1100, height = 760 }, *app, build) exit(1);
}
