// CHIP-8/SUPER-CHIP window runner, with deterministic headless mode.
#import "Basic";
#import "File";
Args :: #import "Extensions/Args";
#import,dir "core";
#load "window.jai";

Cli :: struct {
    rom: string; @positional @required
    preset: string = "modern";
    seed: u32 = 1;
    cycles_per_frame: int = 12; @range(1,10000)
    cycles: int = 1200; @range(1,100000000)
    scale: int = 6; @range(1,16)
    foreground: string = "A8FF60";
    background: string = "101820";
    no_audio: bool;
    smoke: bool;
    window_frames: int; @range(0,100000)
}

HELP :: Args.Help(Cli).{
    about = "CHIP-8 and SUPER-CHIP emulator. P pause, N step, R reset, Escape quit.",
    rom = "ROM file (assemble programs with tools/asm)",
    preset = "chip8, schip, or modern (default)",
    seed = "PCG random seed", cycles_per_frame = "CPU cycles per 60 Hz frame",
    cycles = "Headless instruction budget", scale = "Integer scale at 128x64",
    foreground = "Pixel RGB hex", background = "Background RGB hex",
    no_audio = "Do not open an audio device", smoke = "Headless: print Meow framebuffer digest",
    window_frames = "Close after N rendered frames (window smoke)",
};

main :: () {
    cli, _ := Args.parse(Cli, HELP);
    quirks := MODERN;

    if cli.preset == "chip8" quirks = CHIP8;
    else if cli.preset == "schip" quirks = SUPER_CHIP;
    else if cli.preset != "modern" {
        print("unknown preset\n", to_standard_error = true);
        exit(2);
    }

    rom, ok := read_entire_file(cli.rom);
    if !ok {
        print("cannot read ROM\n", to_standard_error = true);
        exit(1);
    }

    defer free(rom);
    machine: Machine;
    if !reset(*machine, rom, quirks, cli.seed) {
        print("%\n", machine.error, to_standard_error = true);
        exit(1);
    }

    if cli.smoke {

        remaining := cli.cycles;
        while remaining > 0 && !machine.halted {
            budget := min(remaining, cli.cycles_per_frame);
            step(*machine, budget);
            remaining -= budget;
            advance_time(*machine, 16666667);
        }

        if machine.error.count > 0 {
            print("%\n", machine.error, to_standard_error = true);
            exit(1);
        }

        low, high := frame_hash(*machine);

        print("% %\n", low, high);

        return;
    }

    exit(run_window(*machine, rom, cli));
}
