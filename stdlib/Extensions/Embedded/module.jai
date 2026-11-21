// Freestanding programs: no operating system, no libc, no heap unless a board provides one
// (`OS == .NONE`). The runtime is yours to start (`init_runtime`), and the board's console and
// failure behaviour are hooks you set.
//
// ```jai
// #import "Basic";
// #import "Extensions/Embedded";
//
// reset :: () #c_call #no_context {
//     init_runtime(); // .data, .bss, the static Context
//     set_console_hook(uart_write);
//     push_context static_context().* {
//         main();
//     }
// }
// ```
//
// Only what runs on the target is declared when `OS == .NONE`; the memory map, linker script and
// output-format procedures run in the metaprogram, on the host.

#if OS == .NONE #load "runtime.jai";
