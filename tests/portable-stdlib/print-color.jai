#import "Basic";
P :: #import "Print_Color"(USE_ANSI_CODES_ON_WINDOWS = true);

early_return :: () {
    P.with_console_color(.GREEN, #code {
        write_string("scope");
        `return;
    }, style = .UNDERLINE);
    assert(false, "Code return must leave the enclosing procedure");
}

main :: () {
    P.print_color("color %", 42, color = .RED, style = .BOLD);
    early_return();
    P.print_color("error", color = .BG_BLUE, to_standard_error = true);
    print("ok\n");
}
