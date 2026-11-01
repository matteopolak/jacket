#import "Basic";
#import "File";
Args :: #import "Extensions/Args";
#import,dir ".";

Cli :: struct {
    source: string; @positional @required
    output: string; @positional @required
}

HELP :: Args.Help(Cli).{ about = "Assemble CHIP-8 source into a ROM.", source = "Assembly input", output = "ROM output" };

main :: () {
    cli, _ := Args.parse(Cli, HELP);
    source, ok := read_entire_file(cli.source);

    if !ok {
        print("cannot read %\n", cli.source, to_standard_error = true);
        exit(1);
    }

    defer free(source);
    bytes, error := assemble(source);

    defer free(bytes);
    defer free(error);

    if error.count > 0 {
        print("asm: %\n", error, to_standard_error = true);
        exit(1);
    }

    if !write_entire_file(cli.output, bytes) exit(1);
}
