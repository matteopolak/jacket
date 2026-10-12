// Regex benchmark for jaic's Extensions/Regex.
//
//     jaic build benchmarks/regex/jai/bench.jai -O2 -o bench
//     ./bench CORPUS RUNS [CASE...]
//
// Prints one tab-separated line per engine and case:
//     RESULT engine case status count check mbps compile_us note
// `jai-static` compiles every pattern while the program compiles; `jai-runtime` calls
// Regex.compile. `count` is the number of matches, `check` a checksum of the captured groups
// (0 when the case has none). Cases are named in benchmarks/regex/cases.tsv.

#import "Basic";
#import "File";
Regex :: #import "Extensions/Regex";

Date :: Regex.Static("(?<year>[0-9]{4})-(?<month>[0-9]{2})-(?<day>[0-9]{2})");
Date_Plain :: Regex.Static("[0-9]{4}-[0-9]{2}-[0-9]{2}");
Email :: Regex.Static("\\w+@\\w+\\.com");
Greek :: Regex.Static("\\p{Greek}+");
No_Match :: Regex.Static("zzz");
No_Case :: Regex.Static("(?i)HELLO");
Word_B :: Regex.Static("\\b\\w+\\b");
Alt :: Regex.Static("Sherlock|Holmes|Watson|Irene|Adler");
Literal :: Regex.Static("the");
Ing :: Regex.Static("[a-zA-Z]+ing");

corpus: string;
runs := 5;
selected: [..] string;

wanted :: (id: string) -> bool {
    if selected.count == 0 return true;
    for selected if it == id return true;
    return false;
}

now :: () -> float64 {
    return to_float64_seconds(current_time_monotonic());
}

report :: (engine: string, id: string, count: int, check: u64, best: float64, compile_us: float64) {
    mbps := cast(float64) corpus.count / 1000000.0 / best;
    print("RESULT\t%\t%\tok\t%\t%\t%\t%\t\n", engine, id, count, check, mbps, compile_us);
}

digits :: (s: string) -> u64 {
    n: u64;
    for c: s n = n * 10 + cast(u64) (c - #char "0");
    return n;
}

bench_static :: ($R: Type, id: string) {
    if !wanted(id) return;
    count := 0;
    best: float64 = 1.0e30;
    for 1..runs {
        t := now();
        matches := Regex.find_all(R, corpus);
        dt := now() - t;
        count = matches.count;
        array_free(matches);
        if dt < best best = dt;
    }
    report("jai-static", id, count, 0, best, 0);
}

bench_static_captures :: ($R: Type, id: string) {
    if !wanted(id) return;
    count := 0;
    check: u64;
    best: float64 = 1.0e30;
    for 1..runs {
        t := now();
        all := Regex.captures_all(R, corpus);
        sum: u64;
        for all sum += cast(u64) it.year + cast(u64) it.month + cast(u64) it.day;
        dt := now() - t;
        count = all.count;
        check = sum;
        array_free(all);
        if dt < best best = dt;
    }
    report("jai-static", id, count, check, best, 0);
}

bench_runtime :: (pattern: string, id: string) {
    if !wanted(id) return;
    best_compile: float64 = 1.0e30;
    for 1..20 {
        t := now();
        re, ok, error := Regex.compile(pattern);
        dt := now() - t;
        if !ok {
            print("RESULT\tjai-runtime\t%\tna\t0\t0\t0\t0\tcompile error: %\n", id, error.message);
            return;
        }
        Regex.free_regex(*re);
        if dt < best_compile best_compile = dt;
    }
    re := Regex.compile(pattern);
    count := 0;
    best: float64 = 1.0e30;
    for 1..runs {
        t := now();
        matches := Regex.find_all(*re, corpus);
        dt := now() - t;
        count = matches.count;
        array_free(matches);
        if dt < best best = dt;
    }
    Regex.free_regex(*re);
    report("jai-runtime", id, count, 0, best, best_compile * 1000000.0);
}

bench_runtime_captures :: (pattern: string, id: string) {
    if !wanted(id) return;
    best_compile: float64 = 1.0e30;
    for 1..20 {
        t := now();
        re := Regex.compile(pattern);
        dt := now() - t;
        Regex.free_regex(*re);
        if dt < best_compile best_compile = dt;
    }
    re := Regex.compile(pattern);
    count := 0;
    check: u64;
    best: float64 = 1.0e30;
    for 1..runs {
        t := now();
        all := Regex.captures_all(*re, corpus);
        sum: u64;
        for all {
            year, _ := Regex.group(it, "year");
            month, _ := Regex.group(it, "month");
            day, _ := Regex.group(it, "day");
            sum += digits(year) + digits(month) + digits(day);
        }
        dt := now() - t;
        count = all.count;
        check = sum;
        array_free(all);
        if dt < best best = dt;
    }
    Regex.free_regex(*re);
    report("jai-runtime", id, count, check, best, best_compile * 1000000.0);
}

main :: () {
    args := get_command_line_arguments();
    if args.count < 3 {
        print("usage: bench CORPUS RUNS [CASE...]\n");
        exit(2);
    }
    data, ok := read_entire_file(args[1]);
    if !ok exit(1);
    corpus = data;
    runs = cast(int) digits(args[2]);
    for 3..args.count - 1 array_add(*selected, args[it]);

    bench_static(Date_Plain, "date");
    bench_static(Email, "email");
    bench_static(Greek, "greek");
    bench_static(No_Match, "nomatch");
    bench_static(No_Case, "nocase");
    bench_static(Word_B, "wordb");
    bench_static(Alt, "alt");
    bench_static(Literal, "literal");
    bench_static(Ing, "ing");
    bench_static_captures(Date, "caps");

    bench_runtime("[0-9]{4}-[0-9]{2}-[0-9]{2}", "date");
    bench_runtime("\\w+@\\w+\\.com", "email");
    bench_runtime("\\p{Greek}+", "greek");
    bench_runtime("zzz", "nomatch");
    bench_runtime("(?i)HELLO", "nocase");
    bench_runtime("\\b\\w+\\b", "wordb");
    bench_runtime("Sherlock|Holmes|Watson|Irene|Adler", "alt");
    bench_runtime("the", "literal");
    bench_runtime("[a-zA-Z]+ing", "ing");
    bench_runtime_captures("(?<year>[0-9]{4})-(?<month>[0-9]{2})-(?<day>[0-9]{2})", "caps");
}
