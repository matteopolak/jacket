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
