#import "Basic";
C :: #import "Command_Line";

Options :: struct {
    zeta: bool; @"last option"
    alpha: s64; @"?configuration comment"
}

main :: () {
    C.show_help(Options, C.Default_Argument_Flags | .DOUBLE_DASH_REQUIRED, .["help"]);
    print("ok\n");
}
