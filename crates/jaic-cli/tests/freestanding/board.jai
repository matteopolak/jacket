// The board of the test machine, QEMU's `virt`: semihosting for output and the exit code,
// `start.S` for the stack. Programs define `main`.
#import "Basic";
#import "Extensions/Embedded";

start :: #library "start";
semihost :: (operation: u64, block: *void) -> u64 #foreign start;

// SYS_EXIT_EXTENDED with ADP_Stopped_ApplicationExit and the exit code.
exit :: (code: u64) #no_context {
    block: [2] u64;
    block[0] = 0x20026;
    block[1] = code;
    semihost(0x20, *block);
}

// SYS_WRITEC, a byte at a time.
console :: (text: string, to_standard_error: bool) #no_context {
    for index: 0..text.count - 1 {
        byte := text.data[index];
        semihost(3, *byte);
    }
}

on_panic :: (message: string) #no_context {
    console("panic: ", true);
    console(message, true);
    console("\n", true);
    exit(70);
}

#program_export "jai_start"
reset :: () #c_call #no_context {
    init_runtime();
    set_console_hook(console);
    set_panic_hook(on_panic);

    push_context static_context().* {
        main();
    }

    exit(0);
}
