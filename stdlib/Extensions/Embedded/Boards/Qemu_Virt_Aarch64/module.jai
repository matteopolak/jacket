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
    // With the option `tests` it also exports the reset function, which runs `jacket test`'s
    // entry point (`+Extensions/Testing/Discovery` adds it), so the tests need no `jai_start`.
    get_plugin :: () -> *Metaprogram_Plugin {
        plugin := New(Board_Plugin);
        plugin.init = (plugin: *Metaprogram_Plugin, options: [] string) -> bool {
            for options {
                if it == "tests" {
                    (cast(*Board_Plugin) plugin).tests = true;
                } else {
                    print("Qemu_Virt_Aarch64: unknown option '%'\n", it);

                    return false;
                }
            }

            return true;
        };

        plugin.add_source = (plugin: *Metaprogram_Plugin) {
            options := get_build_options(plugin.workspace);

            configure(*options);
            set_build_options(options, plugin.workspace);

            if (cast(*Board_Plugin) plugin).tests {
                add_build_string(TEST_START, plugin.workspace);
            }
        };

        return plugin;
    }

    Board_Plugin :: struct {
        #as using base: Metaprogram_Plugin;
        tests: bool;
    }

    TEST_START :: #string START
Board :: #import "Extensions/Embedded/Boards/Qemu_Virt_Aarch64";

#program_export "jai_start"
reset :: () #c_call #no_context {
    Board.start(__jacket_test_main);
}
START;
} else {
    Program :: #type ();

    // Runs `program` as the machine starts it: sets up the runtime, sends output and panics to
    // the semihosting console, runs the program, and exits with status 0. Call it from the
    // exported reset function.
    start :: (program: Program) #c_call #no_context {
        Embedded.init_runtime();
        Embedded.set_console_hook(console_write);
        Embedded.set_panic_hook(panic);
        Embedded.set_exit_hook(exit);
        Embedded.set_arguments(command_line());

        heap := Embedded.make_heap();
        context_copy := Embedded.static_context().*;
        context_copy.allocator = Embedded.heap_allocator(*heap);

        push_context context_copy {
            program();
        }

        exit(0);
    }

    // What QEMU was told to start the program with (`-append`, or `-semihosting-config arg=`):
    // the semihosting command-line call, split at spaces. QEMU puts the kernel's file name first, as
    // a hosted program's name is. A line longer than the buffer is cut.
    command_line :: () -> [] string #no_context {
        words_used = 0;

        block: [2] u64;
        block[0] = cast,no_check(u64) line.data;
        block[1] = line.count;

        if semihost(0x15, *block) != 0 return .{ words_used, words_storage.data }; // SYS_GET_CMDLINE

        length := cast(s64) block[1];
        start := 0;

        for 0..length {
            at_end := it == length || line[it] == 0 || line[it] == #char " ";

            if !at_end continue;

            if it > start && words_used < words_storage.count {
                words_storage[words_used] = .{ it - start, line.data + start };
                words_used += 1;
            }

            start = it + 1;
            if it == length || line[it] == 0 break;
        }

        return .{ words_used, words_storage.data };
    }

    line: [1024] u8;
    words_storage: [64] string;
    words_used: s64;

    // Ends the run with `status`.
    exit :: (status: s32) #no_context {
        // SYS_EXIT_EXTENDED: reason ADP_Stopped_ApplicationExit and the status.
        block: [2] u64;
        block[0] = 0x20026;
        block[1] = cast,no_check(u64) status;
        semihost(0x20, *block);
    }

    // Standard output and standard error are two files of the host, opened as ARM semihosting
    // defines the console (`:tt`: opened for writing it is standard output, for appending
    // standard error), so what a program prints keeps its stream.
    console_write :: (text: string, to_standard_error: bool) #no_context {
        if !console_opened open_console();

        handle := ifx to_standard_error then error_handle else output_handle;
        block: [3] u64;
        block[0] = cast,no_check(u64) handle;
        block[1] = cast,no_check(u64) text.data;
        block[2] = cast,no_check(u64) text.count;

        semihost(0x05, *block); // SYS_WRITE
    }

    open_console :: () #no_context {
        console_opened = true;
        output_handle = open_console_file(4);
        error_handle = open_console_file(8);
    }

    open_console_file :: (mode: u64) -> s64 #no_context {
        name := ":tt";
        block: [3] u64;
        block[0] = cast,no_check(u64) name.data;
        block[1] = mode;
        block[2] = cast,no_check(u64) name.count;

        return cast,no_check(s64) semihost(0x01, *block); // SYS_OPEN
    }

    console_opened: bool;
    output_handle: s64;
    error_handle: s64;

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
