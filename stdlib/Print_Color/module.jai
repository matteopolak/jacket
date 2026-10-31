// Colored text on the console. `print_color` prints with a color, `with_console_color` colors a block of code, and
// `set_console_color` with `reset_console_color` do it by hand. Colors are ANSI codes, which Windows terminals understand
// too; its older console is driven through the console API instead.
//
// Module parameters: `USE_ANSI_CODES_ON_WINDOWS` (default `false`) writes ANSI codes on Windows as well, and
// `SAVE_RESTORE_ON_WINDOWS` (default `false`) makes the outermost pop restore the console as it was just before its push, not as it was before the first color ever pushed.
//
// ```jai
// print_color("all good\n", color = .GREEN);
// with_console_color(.RED, #code print("%\n", "something failed"));
// ```
#module_parameters(SAVE_RESTORE_ON_WINDOWS := false)(USE_ANSI_CODES_ON_WINDOWS := false);

#load "../internal/output/print-color.jai";
