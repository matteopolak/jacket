// game3d: a small 3D scene built to use many standard-library and extension modules together.
//
//   Extensions/WebGPU    drawing (Metal, Vulkan or D3D12 underneath)
//   meshoptimizer        mesh clean-up, level of detail and compression
//   Soa, Math, SIMD      the entity data and its transforms
//   Thread, Atomics      a job system that updates entities in parallel
//   Iprof                a profile of the frame phases
//   pl_mpeg              the intro video
//   Extensions/Assets, Extensions/Image, Extensions/Args, Input, Window_Creation, PCG, Basic
//
//     jaic examples/game3d/game/main.jai -exe game3d
//     ./game3d                  # intro, then fly around the scene
//     ./game3d --no-intro       # straight to the scene
//     ./game3d --smoke          # scripted headless session, see smoke.jai
//
// Keys: W A S D move, Q and E go down and up, Shift is faster, the arrow keys or the right mouse
// button turn, P pauses the simulation, R prints the profile, Space skips the intro, Escape quits.

#import "Basic";
#import "Math";
#import "String";
#import "File";
#import "PCG";
#import "Soa";
#import "Atomics";
#import "Input";
#import "Window_Type";
#import "Window_Creation";
#import "Extensions/Assets";
#import "Extensions/WebGPU";
#import "Extensions/Image";
#import "meshoptimizer";
Args :: #import "Extensions/Args";
T :: #import "Thread";
P :: #import "pl_mpeg";
SIMD :: #import "Extensions/SIMD";
I :: #import "Iprof"(IMPORT_MODE = .CLIENT, MANUAL_MODE = true, USE_GRAPHICS = false);

#load "jobs.jai";
#load "meshes.jai";
#load "world.jai";
#load "camera.jai";
#load "profile.jai";
#load "render.jai";
#load "intro.jai";
#load "game.jai";
#load "smoke.jai";

INTRO_PATH :: "assets/intro.mpg";

Cli :: struct {
    entities: int = 768; @value=N @range(4,1024)
    threads: int = 3; @value=N @range(0,32)
    seed: int = 7; @value=N
    no_intro: bool;
    frames: int; @value=N
    smoke: bool;
    smoke_ticks: int = 240; @value=N
    smoke_frame: string; @value=PNG
    require_gpu: bool;
    profile_seconds: int = 5; @value=N @range(1,3600)
}

HELP :: Args.Help(Cli).{
    about = "game3d: a 3D scene with a job system, meshoptimizer meshes and an intro video.",
    version = "0.1.0",
    entities = "How many entities to simulate (a multiple of 4)",
    threads = "Worker threads for the job system (0 runs everything on the main thread)",
    seed = "Seed of the scene",
    no_intro = "Skip the intro video",
    frames = "Quit after this many frames of the scene (0 plays until the window is closed)",
    smoke = "Run the scripted headless smoke session and exit",
    smoke_ticks = "With --smoke: simulation ticks to run",
    smoke_frame = "With --smoke: write the last offscreen frame to this PNG",
    require_gpu = "With --smoke: fail instead of skipping when there is no GPU",
    profile_seconds = "Seconds between profile printouts while playing",
};

main :: () {
    cli, _ := Args.parse(Cli, HELP);

    if cli.smoke {
        exit(ifx run_smoke(cli) then cast(s32) 0 else cast(s32) 1);
    }

    exit(ifx run_game(cli) then cast(s32) 0 else cast(s32) 1);
}
