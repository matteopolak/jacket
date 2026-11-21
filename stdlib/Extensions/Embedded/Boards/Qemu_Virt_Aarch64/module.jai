// QEMU's `virt` machine with an AArch64 core (`qemu-system-aarch64 -M virt`): 64 MiB of RAM at
// 0x4000_0000, and ARM semihosting for the console and the exit code, so it needs no device
// drivers and a run's exit status is the program's.
//
// ```jai
// // build.jai
// Board :: #import "Extensions/Embedded/Boards/Qemu_Virt_Aarch64";
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
// Board :: #import "Extensions/Embedded/Boards/Qemu_Virt_Aarch64";
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
//     qemu-system-aarch64 -M virt -cpu cortex-a53 -nographic \
//         -semihosting-config enable=on,target=native -kernel app.elf
//
// The console goes to QEMU's standard output and the process exits with the program's status (a
// returning `main` is 0, a panic is 1).

Embedded :: #import "Extensions/Embedded";

// RAM is all there is: QEMU loads the ELF file's segments into it.
memory_map :: Embedded.Memory_Map.{
    ram = .{ 0x4000_0000, 0x400_0000 },
};

#if OS != .NONE {
    #import "Basic";
    #import "Compiler";

    // Makes the workspace build a freestanding AArch64 program for this board and writes its
    // linker script.
    configure :: (options: *Build_Options) {
        options.os_target = .NONE;
        options.cpu_target = .ARM64;
        options.llvm_options.target_system_triple = "aarch64-none-elf";
        options.runtime_support_definitions = .ONLY_INIT;
        Embedded.link_for(options, memory_map);
    }

    // `jaic app.jai +Extensions/Embedded/Boards/Qemu_Virt_Aarch64` builds the program for this
    // board without a metaprogram of its own: the plugin runs `configure` on the workspace.
    get_plugin :: () -> *Metaprogram_Plugin {
        plugin := New(Metaprogram_Plugin);
        plugin.add_source = (plugin: *Metaprogram_Plugin) {
            options := get_build_options(plugin.workspace);
            configure(*options);
            set_build_options(options, plugin.workspace);
        };

        return plugin;
    }
} else {
    Program :: #type ();

    // Runs `program` as the machine starts it: sets up the runtime, sends output and panics to
    // the semihosting console, runs the program, and exits with status 0. Call it from the
    // exported reset function.
    start :: (program: Program) #c_call #no_context {
        Embedded.init_runtime();
        Embedded.set_console_hook(console_write);
        Embedded.set_panic_hook(panic);

        heap := Embedded.make_heap();
        context_copy := Embedded.static_context().*;
        context_copy.allocator = Embedded.heap_allocator(*heap);

        push_context context_copy {
            program();
        }

        exit(0);
    }

    // Ends the run with `status`.
    exit :: (status: s32) #no_context {
        // SYS_EXIT_EXTENDED: reason ADP_Stopped_ApplicationExit and the status.
        block: [2] u64;
        block[0] = 0x20026;
        block[1] = cast,no_check(u64) status;
        semihost(0x20, *block);
    }

    console_write :: (text: string, to_standard_error: bool) #no_context {
        for 0..text.count - 1 {
            byte := text.data[it];
            semihost(3, *byte); // SYS_WRITEC
        }
    }

    panic :: (message: string) #no_context {
        console_write("panic: ", true);
        console_write(message, true);
        console_write("\n", true);
        exit(1);
    }

    semihost :: (operation: u64, block: *void) -> u64 #elsewhere #c_call;

    // The code the machine starts in: EL1 with no stack and floating point trapped. It sets
    // both up and calls the program's `jai_start`. `semihost` is the ARM semihosting call.
    reset_code :: #string ASM
.section .text.boot, "ax"
.global _start
_start:
    mrs x0, cpacr_el1
    orr x0, x0, #(3 << 20)
    msr cpacr_el1, x0
    isb
    ldr x0, =__stack_top
    mov sp, x0
    bl  jai_start
1:  b   1b

.text
.global semihost
semihost:
    hlt #0xf000
    ret
ASM; @Module_Asm
}
