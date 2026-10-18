// The case table a generated harness hands to `run_cases`, and its command line.

Case :: struct {
    name: string;
    run: (r: *Rng);
    pointers: bool;
    nullable: bool;
}

// Arguments (after `--`): --seed N, --count N (seeds per case), --only NAME, --max-len N, --list.
// Prints `CASE <name> <seed> <max_len>` before every call so the last line names the culprit when
// the process dies, and `DONE <calls>` at the end. Returns the process exit code.
run_cases :: (cases: [] Case) -> s32 {
    seed: s64 = 1;
    count: s64 = 100;
    max_len: s64 = 256;
    only := "";
    list := false;

    arguments := get_command_line_arguments();
    i := 1;
    while i < arguments.count {
        name := arguments[i];
        i += 1;
        if name == "--list" { list = true; continue; }
        if i >= arguments.count {
            print("missing value for %\n", name);
            return 2;
        }
        value := arguments[i];
        i += 1;
        number, ok := to_integer(value);
        if name == {
            case "--only"; only = value;
            case "--seed"; seed = number;
            case "--count"; count = number;
            case "--max-len"; max_len = number;
            case; print("unknown argument %\n", name); return 2;
        }
        if name != "--only" && !ok {
            print("bad number for %: %\n", name, value);
            return 2;
        }
    }

    if list {
        for cases print("%\n", it.name);
        return 0;
    }

    r: Rng;
    r.max_len = max_len;
    calls: s64 = 0;
    for step: 0..count - 1 {
        for cases {
            if only.count > 0 && it.name != only continue;
            case_seed := seed + step;
            rng_seed(*r, cast(u64) case_seed);
            r.pointers = it.pointers;
            r.nullable = it.nullable;
            print("CASE % % %\n", it.name, case_seed, max_len);
            it.run(*r);
            case_end(*r);
            calls += 1;
        }
    }
    print("DONE %\n", calls);
    return 0;
}

// Coverage-guided mode. The program is built with `-sanitize fuzzer` (SanitizerCoverage plus the
// libFuzzer engine without its `main`); `run_engine` hands the engine control, and it calls
// `engine_input` with each input. Command line: libFuzzer's own flags and corpus directories,
// plus `--only NAME` to fuzz one case and `--trace` to print each input's case name (to learn
// which case a stored crash belongs to). The first input byte picks the case.
//
//   harness corpus/ -max_len=1024 -max_total_time=60 -artifact_prefix=crashes/
run_engine :: (cases: [] Case) -> s32 {
    engine_cases = cases;
    engine_only = -1;

    arguments := get_command_line_arguments();
    forwarded: [..] *u8;
    i := 0;
    while i < arguments.count {
        if arguments[i] == "--only" && i + 1 < arguments.count {
            for cases if it.name == arguments[i + 1] engine_only = it_index;
            if engine_only < 0 {
                print("no case named %\n", arguments[i + 1]);
                return 2;
            }
            i += 2;
            continue;
        }
        if arguments[i] == "--trace" {
            engine_trace = true;
            i += 1;
            continue;
        }
        array_add(*forwarded, to_c_string(arguments[i]));
        i += 1;
    }
    array_add(*forwarded, null);

    count := cast(s32) (forwarded.count - 1);
    argv := forwarded.data;
    return LLVMFuzzerRunDriver(*count, *argv, engine_input);
}

#scope_file

engine_runtime :: #system_library "libc";
engine_abort :: () #foreign engine_runtime "abort";

LLVMFuzzerRunDriver :: (argc: *s32, argv: ***u8, callback: (data: *u8, size: u64) -> s32 #c_call) -> s32 #foreign engine_runtime;

engine_cases: [] Case;
engine_only: s64;
engine_trace: bool;
engine_rng: Rng;

engine_print_name :: (name: string) {
    print("CASE %\n", name);
}

engine_input :: (data: *u8, size: u64) -> s32 #c_call {
    push_context {
        r := *engine_rng;
        r.from_input = true;
        r.data = data;
        r.size = cast(s64) size;
        r.position = 0;

        index := engine_only;
        selector := take(r, 1);
        if index < 0 index = cast(s64) (selector % cast(u64) engine_cases.count);

        c := engine_cases[index];
        if engine_trace {
            engine_print_name(c.name);
        }
        r.pointers = c.pointers;
        r.nullable = c.nullable;
        c.run(r);
        case_end(r);
    }
    return 0;
}
