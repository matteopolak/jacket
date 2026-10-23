// Plain `#run` code of a cross build sees the host: `OS`/`CPU` name the machine running the
// compiler, library code takes its host branches (process creation, file queries), and results
// flow into the target program as constants, strings and structs.
#import "Basic";
#import "Process";
#import "File";
#import "File_Utilities";
#import "String";

Pair :: struct {
    name: string;
    count: s64;
    ratio: float64;
}

Kind :: enum u8 {
    SMALL;
    LARGE;
}

Described :: struct {
    kind: Kind;
    words: [] string;
    pair: Pair;
}

describe :: () -> Described {
    result: Described;
    result.kind = .LARGE;
    result.pair = .{ "host", 42, 0.5 };
    words := NewArray(2, string);
    words[0] = "alpha";
    words[1] = "beta";
    result.words = words;

    return result;
}

host_is_native :: () -> bool {
    return OS == BUILD_OS && CPU == BUILD_CPU;
}

echo :: (word: string) -> string, Process_Result {
    #if OS == .WINDOWS {
        result, output := run_command("cmd", "/c", "echo", word, capture_and_return_output = true);
    } else {
        result, output := run_command("echo", word, capture_and_return_output = true);
    }

    assert(result.exit_code == 0, "echo failed");

    return trim(output), result;
}

exists :: (path: string) -> bool {
    return file_exists(path);
}

HOST_NATIVE :: #run host_is_native();
ECHOED, ECHO_RESULT :: #run echo("compile-time");
HERE :: #run exists(".");
DESCRIBED :: #run describe();

#assert HOST_NATIVE;
#assert HERE;
#assert DESCRIBED.kind == .LARGE;
#assert DESCRIBED.pair.count == 42;
#assert DESCRIBED.words.count == 2;
#assert ECHO_RESULT.exit_code == 0;

main :: () {
    assert(ECHOED == "compile-time");
    assert(DESCRIBED.words[1] == "beta");
    assert(DESCRIBED.pair.name == "host");
}
