// The Arduino Uno: an ATmega328P (8-bit AVR, 2-byte pointers) as QEMU models it
// (`qemu-system-avr -M uno`): 32 KiB of flash that holds code and the initial values of data, and
// 2 KiB of RAM at data address 0x100. The console is USART0 (QEMU's first serial port), so a run
// ends when the program stops printing: the emulator has no exit call.
//
// AVR keeps code and data in separate address spaces. Constants cannot be read from flash by an
// ordinary load, so the linker script puts `.rodata` in RAM beside `.data` and the reset code
// copies both from flash, as avr-libc does.
//
// ```jai,ignore
// // app.jai: builds only for the board (`OS == .NONE`).
// #import "Basic";
// Board :: #import "Extensions/Embedded/Boards/Arduino_Uno";
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
// ```sh
// jaic app.jai +Extensions/Embedded/Boards/Arduino_Uno
// qemu-system-avr -M uno -bios app.elf -nographic -serial stdio -monitor none
// ```

Embedded :: #import "Extensions/Embedded";

// Data memory starts at 0x100 (the registers and I/O come first); the linker places it at
// 0x800100, the address avr-ld and LLVM use for RAM so it never overlaps flash.
memory_map :: Embedded.Memory_Map.{
    flash = .{ 0x0000_0000, 0x8000 },
    ram = .{ 0x0080_0100, 0x800 },
    stack_size = 256,
    rodata_in_ram = true,
};

#if OS != .NONE {
    #import "Basic";
    #import "Compiler";

    // Makes the workspace build a freestanding ATmega328P program for this board and writes its
    // linker script.
    configure :: (options: *Build_Options) {
        options.os_target = .NONE;
        options.llvm_options.target_system_triple = "avr-none-elf";
        options.llvm_options.target_system_cpu = "atmega328p";
        options.runtime_support_definitions = .ONLY_INIT;
        // Everything below is RAM: 2 KiB for `.data`, `.bss`, the heap and the stack. The trace
        // text of a stack trace and a large temporary storage would take most of it.
        // An unoptimized program keeps every compile-time scratch buffer and does not fit the
        // part, so the board builds small whatever the command line says.
        set_optimization(options, .OPTIMIZED_SMALL);
        options.stack_trace = false;
        // RAM starts at data address 0x100, so no low address is unmapped: the null check, which
        // takes everything below 4096 for a null pointer, would stop every global access.
        options.null_pointer_check = .OFF;
        options.temporary_storage_size = 64;
        Embedded.link_for(options, memory_map);
    }

    get_plugin :: () -> *Metaprogram_Plugin {
        return Embedded.board_plugin("Extensions/Embedded/Boards/Arduino_Uno", configure);
    }
} else {
    Program :: Embedded.Program;

    // Runs `program` as the machine starts it: sets up the runtime, sends output and panics to
    // USART0 (9600 baud for the Uno's 16 MHz clock), runs the program, and halts. Call it from
    // the exported reset function.
    start :: (program: Program) #c_call #no_context {
        Embedded.start_serial(program, 103);
    }
}
