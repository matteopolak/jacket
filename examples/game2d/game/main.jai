// game2d: a small arena shooter built on the standard library only (Simp, Window_Creation, Input,
// Gamepad, Sound_Player, File_Watcher, Zip_File_Directory, lz4, xxHash, PCG, Bucket_Array, Pool),
// with the Http client for its online leaderboard.
//
//     jaic examples/game2d/game/main.jai -exe game2d
//     ./game2d                                   # loose assets next to the source, hot reloaded
//     ./game2d --pack assets.zip                 # assets from a zip pack (tools/assetgen --pack)
//     ./game2d --server http://127.0.0.1:8080    # scores go to the backend (examples/game2d/server)
//     ./game2d --smoke                           # scripted session, see smoke.jai
//
// Keys: WASD or arrows move, Space or the left mouse button fires (the mouse aims), Enter starts and
// retries, L shows the leaderboard, P pauses, M mutes, Escape goes back. A gamepad works too: left
// stick, right stick to aim, A to fire.

#import "Basic";
#import "Math";
#import "String";
#import "File";
#import "File_Utilities";
#import "Thread";
#import "System";
#import "PCG";
#import "Bucket_Array";
#import "Pool";
#import "Input";
#import "Window_Creation";
#import "Simp";
#import "Gamepad";
#import "File_Watcher";
#import "Zip_File_Directory";
#import "lz4";
#import "xxHash";
#import "Extensions/Deflate";
#import "Extensions/Environment";
Args :: #import "Extensions/Args";
SP :: #import "Sound_Player";
#import,dir "../shared";
#import,dir "../sheet";

#load "world.jai";
#load "render.jai";
#load "assets.jai";
#load "input.jai";
#load "sound.jai";
#load "save.jai";
#load "online.jai";
#load "screens.jai";
#load "smoke.jai";

Cli :: struct {
    server: string; @short=s @value=URL
    roots: string; @value=FILE
    name: string = "pilot"; @value=NAME
    assets: string; @value=DIR
    pack: string; @value=FILE
    save: string; @value=FILE
    seed: int; @value=N
    scale: int = 4; @range(1,8)
    no_audio: bool;
    no_hot_reload: bool;
    autoplay: int; @value=FRAMES
    smoke: bool;
    smoke_frame: string; @value=PNG
    smoke_ticks: int = 1800;
}

HELP :: Args.Help(Cli).{
    about = "game2d: an arena shooter with an online leaderboard.",
    version = "0.1.0",
    server = "Backend URL for accounts and scores",
    roots = "PEM certificate to trust for an https server",
    name = "Player name for the leaderboard",
    assets = "Folder of loose assets (default: assets next to the source)",
    pack = "Zip pack to load assets from instead of a folder",
    save = "Save file (default: game2d.sav in the home folder)",
    seed = "Level seed (default: from the clock)",
    scale = "Window size as a multiple of 320x180",
    no_audio = "Do not open an audio device",
    no_hot_reload = "Do not watch the asset folder",
    autoplay = "Play by itself for this many frames, then quit (demo and test)",
    smoke = "Run the scripted smoke session and exit",
    smoke_frame = "With --smoke: write the offscreen frame to this PNG",
    smoke_ticks = "With --smoke: simulation ticks to run",
};

main :: () {
    cli, _ := Args.parse(Cli, HELP);

    if cli.smoke {
        exit(ifx run_smoke(cli) then cast(s32) 0 else cast(s32) 1);
    }

    game: Game;

    if !game_init(*game, cli) exit(1);

    game_run(*game);
    game_shutdown(*game);
}
