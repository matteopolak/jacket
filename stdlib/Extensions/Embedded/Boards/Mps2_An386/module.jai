// Arm's MPS2 FPGA board with the AN386 image (a Cortex-M4F), as QEMU models it
// (`qemu-system-arm -M mps2-an386`): 4 MiB of code memory at 0x0000_0000, which the core boots
// from, and 4 MiB of RAM at 0x2000_0000. ARM semihosting gives the console and the exit code, so
// it needs no device drivers and a run's exit status is the program's.
//
// ```jai,ignore
// // build.jai: the metaprogram, next to app.jai.
// #import "Compiler";
// Board :: #import "Extensions/Embedded/Boards/Mps2_An386";
// Embedded :: #import "Extensions/Embedded";
//
// #run {
//     w := compiler_create_workspace("app");
//     options := get_build_options(w);
//     Board.configure(*options);
//     Embedded.link_for(*options, Board.memory_map);
//     set_build_options(options, w);
//     add_build_file("app.jai", w);
// }
// ```
//
// ```jai,ignore
// // app.jai: builds only for the board (`OS == .NONE`).
// #import "Basic";
// Board :: #import "Extensions/Embedded/Boards/Mps2_An386";
//
// main :: () {
//     print("hello\n");
// }
//
// #program_export "jai_start"
// reset :: () #c_call #no_context {
//     Board.start(main);
// }
// ```
//
// Run the result with
//
// ```sh
// qemu-system-arm -M mps2-an386 -cpu cortex-m4 -nographic \
//     -semihosting-config enable=on,target=native -kernel app.elf
// ```
//
// The console goes to QEMU's standard output and the process exits with the program's status (a
// returning `main` is 0, a panic or a fault is 1).

// The target triple, and the command `jacket test --target` and `jacket run --target` use to
// run a program on this board. `{exe}` is the program and `{args}` the test arguments.
triple :: "thumbv7em-none-eabihf";
runner :: "qemu-system-arm -M mps2-an386 -cpu cortex-m4 -nographic -semihosting-config enable=on,target=native -kernel {exe} -append {args}";

Embedded :: #import "Extensions/Embedded";

// Code in the memory the core boots from, data in the RAM above it.
memory_map :: Embedded.Memory_Map.{
    flash = .{ 0x0000_0000, 0x40_0000 },
    ram = .{ 0x2000_0000, 0x40_0000 },
};

#if OS != .NONE {
    #import "Basic";
    #import "Compiler";

    // Makes the workspace build a freestanding Cortex-M4F program for this board.
    configure :: (options: *Build_Options) {
        options.os_target = .NONE;
        options.llvm_options.target_system_triple = triple;
        options.runtime_support_definitions = .ONLY_INIT;
    }

    // `jaic app.jai +Extensions/Embedded/Boards/Mps2_An386` builds the program for this board
    // without a metaprogram of its own: the plugin runs `configure` on the workspace.
    get_plugin :: () -> *Metaprogram_Plugin {
        return Embedded.board_plugin("Extensions/Embedded/Boards/Mps2_An386", memory_map, configure);
    }
} else {
    // Runs `program` as the machine starts it: sets up the runtime, sends output and panics to
    // the semihosting console, runs the program, and exits with status 0. Call it from the
    // exported reset function.
    start :: (program: Embedded.Program) #c_call #no_context {
        Embedded.start_semihosted(program);
    }
}
