// Arm's MPS2 FPGA board with the AN505 image (a Cortex-M33 with TrustZone), as QEMU models it
// (`qemu-system-arm -M mps2-an505`). The program runs in the secure state, which boots from the
// secure alias of the 4 MiB of code memory, 0x1000_0000, with 2 MiB of RAM at the secure alias
// 0x3800_0000 of the second SSRAM. ARM semihosting gives the console and the exit code, so it
// needs no device drivers and a run's exit status is the program's.
//
// ```jai
// // build.jai
// Board :: #import "Extensions/Embedded/Boards/Mps2_An505";
//
// #run {
//     w := compiler_create_workspace("app");
//     options := get_build_options(w);
//     Board.configure(*options);
//     set_build_options(options, w);
//     add_build_file("app.jai", w);
// }
//
// // app.jai
// Board :: #import "Extensions/Embedded/Boards/Mps2_An505";
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
//     qemu-system-arm -M mps2-an505 -cpu cortex-m33 -nographic \
//         -semihosting-config enable=on,target=native -kernel app.elf
//
// The console goes to QEMU's standard output and the process exits with the program's status (a
// returning `main` is 0, a panic or a fault is 1).

Embedded :: #import "Extensions/Embedded";

// Code in the memory the secure state boots from, data in RAM.
memory_map :: Embedded.Memory_Map.{
    flash = .{ 0x1000_0000, 0x40_0000 },
    ram = .{ 0x3800_0000, 0x20_0000 },
};

#if OS != .NONE {
    #import "Basic";
    #import "Compiler";

    // Makes the workspace build a freestanding Cortex-M33 program for this board and writes its
    // linker script.
    configure :: (options: *Build_Options) {
        options.os_target = .NONE;
        options.llvm_options.target_system_triple = "thumbv8m.main-none-eabihf";
        options.runtime_support_definitions = .ONLY_INIT;
        Embedded.link_for(options, memory_map);
    }

    // `jaic app.jai +Extensions/Embedded/Boards/Mps2_An505` builds the program for this board
    // without a metaprogram of its own: the plugin runs `configure` on the workspace.
    get_plugin :: () -> *Metaprogram_Plugin {
        return Embedded.board_plugin("Extensions/Embedded/Boards/Mps2_An505", configure);
    }
} else {
    // Runs `program` as the machine starts it: sets up the runtime, sends output and panics to
    // the semihosting console, runs the program, and exits with status 0. Call it from the
    // exported reset function.
    start :: (program: Embedded.Program) #c_call #no_context {
        Embedded.start_semihosted(program);
    }
}
