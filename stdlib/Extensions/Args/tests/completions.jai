// Args completion scripts for each shell against golden texts (data/golden.jai), plus checks
// that the scripts name what the declarations declare.
//
//     jaic run stdlib/Extensions/Args/tests/completions.jai
//
// After an intended change, regenerate the goldens with data/bless.jai.

#import "Basic";
#import "String";

#load "data/outputs.jai";
#load "data/golden.jai";

failures := 0;

fail :: (format: string, args: ..Any) {
    print("FAIL: ");
    print(format, ..args);
    print("\n");
    failures += 1;
}

expect :: (condition: bool, what: string) {
    if !condition fail("%", what);
}

golden :: (name: string) -> string {
    if name == {
        case "CLI_BASH"; return CLI_BASH;
        case "CLI_ZSH"; return CLI_ZSH;
        case "CLI_FISH"; return CLI_FISH;
        case "CLI_POWERSHELL"; return CLI_POWERSHELL;
        case "TOOL_BASH"; return TOOL_BASH;
        case "TOOL_ZSH"; return TOOL_ZSH;
        case "TOOL_FISH"; return TOOL_FISH;
        case "TOOL_POWERSHELL"; return TOOL_POWERSHELL;
    }
    return "";
}

first_difference :: (a: string, b: string) -> s64 {
    for 0..min(a.count, b.count) - 1 if a[it] != b[it] return it;
    return min(a.count, b.count);
}

main :: () {
    for completion_outputs() {
        expected := golden(it.name);
        if it.text != expected {
            fail("% differs from its golden text; first difference at byte %\n--- actual ---\n%\n--- expected ---\n%", it.name, first_difference(it.text, expected), it.text, expected);
        }
    }

    // Every option and subcommand the declarations make appears in every script.
    shells := Args.Shell.[.BASH, .ZSH, .FISH, .POWERSHELL];
    for shell: shells {
        script := Args.completions(Tool, TOOL_HELP, shell, "tool");
        for string.["verbose", "config", "quiet", "release", "target", "build", "run", "remote-add"] {
            expect(contains(script, it), tprint("% completions mention %", shell, it));
        }
        expect(!contains(script, "secret"), "no hidden options");
        script = Args.completions(Cli, HELP, shell, "jaifmt");
        for string.["check", "jobs", "color", "config", "verbose", "always", "never", "auto"] {
            expect(contains(script, it), tprint("% completions mention %", shell, it));
        }
    }

    // A hidden option is left out.
    script := Args.completions(Kinds, KINDS_HELP, .FISH, "kinds");
    expect(!contains(script, "secret") && contains(script, "-l lvl") && contains(script, "-l output-dir"), "hidden options are not completed; renamed ones use their new name");

    if failures {
        print("% failures\n", failures);
        exit(1);
    }
    print("ok\n");
}
