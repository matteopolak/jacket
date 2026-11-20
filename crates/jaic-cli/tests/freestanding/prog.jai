// A freestanding program with no runtime: `start.S` sets the stack and calls `jai_start`, which
// exits through the Arm semihosting call with the code the test substitutes for EXIT_CODE.
start :: #library "start";
semihost :: (operation: u64, block: *void) -> u64 #foreign start;

#program_export "jai_start"
jai_start :: () #c_call #no_context {
    // SYS_EXIT_EXTENDED with ADP_Stopped_ApplicationExit and the exit code.
    block: [2] u64;
    block[0] = 0x20026;
    block[1] = EXIT_CODE;
    semihost(0x20, *block);
}
