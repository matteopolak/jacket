// What an AVR core needs to start a program: reset code that sets the stack, copies `.data` (and
// the `.rodata` the linker script moves into RAM, see `Memory_Map.rodata_in_ram`) out of flash,
// clears `.bss` and calls the program's `jai_start`, plus the serial console of the ATmega328P
// (USART0).
//
// Flash is a separate address space: an ordinary load never reads it, so only this reset code
// (with `lpm`) touches the load image. Execution begins at address 0, where `.text.boot` is
// placed; this code does not set up interrupt vectors, so a program that enables interrupts
// provides its own table with `@Section(".vector_table")`.

avr_code :: #string ASM
.section .text.boot, "ax"
.global _start
_start:
    clr r1                       // the compiler keeps r1 zero
    out 0x3f, r1                 // SREG: interrupts off
    ldi r28, lo8(__stack_top-1)  // the stack grows down from the last byte of RAM
    ldi r29, hi8(__stack_top-1)
    out 0x3e, r29
    out 0x3d, r28
    ldi r30, lo8(__data_load)    // Z: the flash image of .data
    ldi r31, hi8(__data_load)
    ldi r26, lo8(__data_start)   // X: where it goes
    ldi r27, hi8(__data_start)
    ldi r24, lo8(__data_end)
    ldi r25, hi8(__data_end)
1:  cp  r26, r24
    cpc r27, r25
    breq 2f
    lpm r0, Z+
    st  X+, r0
    rjmp 1b
2:  ldi r26, lo8(__bss_start)
    ldi r27, hi8(__bss_start)
    ldi r24, lo8(__bss_end)
    ldi r25, hi8(__bss_end)
3:  cp  r26, r24
    cpc r27, r25
    breq 4f
    st  X+, r1
    rjmp 3b
4:  call jai_start
5:  rjmp 5b
ASM; @Module_Asm

// The ATmega328P's USART0 registers, in data space.
UCSR0A :: 0xC0;
UCSR0B :: 0xC1;
UCSR0C :: 0xC2;
UBRR0L :: 0xC4;
UBRR0H :: 0xC5;
UDR0 :: 0xC6;

// Turns the transmitter on at `ubrr` (the baud-rate divisor: `F_CPU / (16 * baud) - 1`, 103 for
// 9600 baud at 16 MHz), 8 data bits, no parity, 1 stop bit.
serial_init :: (ubrr: u16) #no_context {
    volatile_store(cast(*u8) UBRR0H, cast(u8) (ubrr >> 8));
    volatile_store(cast(*u8) UBRR0L, cast(u8) ubrr);
    volatile_store(cast(*u8) UCSR0C, 0x06);
    volatile_store(cast(*u8) UCSR0B, 0x08); // TXEN0
}

// Sends `text` through USART0, waiting for the data register to empty before each byte.
serial_write :: (text: string, to_standard_error: bool) #no_context {
    for index: 0..text.count - 1 {
        while (volatile_load(cast(*u8) UCSR0A) & 0x20) == 0 {}

        volatile_store(cast(*u8) UDR0, text.data[index]);
    }
}

// Runs `program` as the machine starts it: sets up the runtime, sends output and panics to
// USART0, runs the program and halts. Call it from the exported reset function.
start_serial :: (program: Program, ubrr: u16) #c_call #no_context {
    init_runtime();
    serial_init(ubrr);
    set_console_hook(serial_write);
    set_panic_hook(serial_panic);
    set_exit_hook(serial_exit);

    heap := make_heap();
    context_copy := static_context().*;
    context_copy.allocator = heap_allocator(*heap);

    push_context context_copy {
        program();
    }

    serial_exit(0);
}

// Ends the run: the emulator has no exit call, so the core stops in a loop. A test harness stops
// the emulator when the program has printed what it checks.
serial_exit :: (status: s32) #no_context {
    while true {}
}

// The C `abort` the compiled runtime's failure paths call: the core stops in a loop.
#program_export "abort"
avr_abort :: () #c_call #no_context {
    while true {}
}

// 32-bit multiply. LLVM calls `__mulsi3` for it, and the compiler-rt builtins for AVR leave it
// out (libgcc supplies it). Shift-and-add, so the loop never multiplies.
#program_export "__mulsi3"
multiply_32 :: (a: u32, b: u32) -> u32 #c_call #no_context {
    result: u32;
    x := a;
    y := b;

    while y != 0 {
        if y & 1 result += x;
        x <<= 1;
        y >>= 1;
    }

    return result;
}

#scope_file

serial_panic :: (message: string) #no_context {
    serial_write("panic: ", true);
    serial_write(message, true);
    serial_write("\n", true);
    serial_exit(1);
}
