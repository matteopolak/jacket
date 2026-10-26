// Extensions/Regex against rluba/uniform and Lunatoid/Rematch on log parsing, e-mail and URL
// extraction and searches over a megabyte of prose. The texts are generated here, with a fixed
// seed, ASCII only so that the three engines (Unicode-aware, Unicode-aware, ASCII `\w`) agree on
// every match; each benchmark checks its count against the one Jacket finds.
//
// The log pattern keeps `]` out of its classes: Rematch does not compile `[^\]]` and uniform not `\x5d`.
//
// `jacket` compiles the pattern at run time, as the others do; `jacket_static` compiles it while the
// program compiles (Regex.Static). Patterns are compiled before the timer starts, except in the
// `compile_*` workloads. Everything allocates from the default allocator and frees what it is given.
//
// uniform finds only the first match of a text, so its find-all loop searches the rest of the text
// after each match. The community libraries are imported from corpus/upstream through
// `-import_dir`; run this through tools/bench/library_compare.py, which keeps them in the sandbox.
#import "Basic";
#import "String";
#import "Extensions/Testing";
Regex :: #import "Extensions/Regex";

uniform :: #import "rluba--uniform";
rematch :: #import "Lunatoid--Rematch/modules/Rematch";

LOG_PATTERN :: "^(\\S+) \\S+ \\S+ \\[([\\w/:]+ \\+\\d{4})\\] \"(\\w+) (\\S+) [^\"]*\" (\\d{3}) (\\d+)";
ERROR_PATTERN :: "\" [45]\\d\\d ";
EMAIL_PATTERN :: "[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\\.[A-Za-z]{2,}";
URL_PATTERN :: "https?://[^\\s\"<>]+";
NAMES_PATTERN :: "Sherlock|Holmes|Watson|Irene|Adler";
ING_PATTERN :: "[a-zA-Z]+ing";

Log_Static :: Regex.Static(LOG_PATTERN);
Error_Static :: Regex.Static(ERROR_PATTERN);
Email_Static :: Regex.Static(EMAIL_PATTERN);
Url_Static :: Regex.Static(URL_PATTERN);
Names_Static :: Regex.Static(NAMES_PATTERN);
Ing_Static :: Regex.Static(ING_PATTERN);

PATTERNS :: string.[LOG_PATTERN, ERROR_PATTERN, EMAIL_PATTERN, URL_PATTERN, NAMES_PATTERN, ING_PATTERN];
LOG :: 0;
ERROR :: 1;
EMAIL :: 2;
URL :: 3;
NAMES :: 4;
ING :: 5;

Compiled :: struct {
    jacket: Regex.Regex;
    uniform: uniform.Regexp;
    rematch: rematch.Regex;
    expected: int;
}

compiled: [PATTERNS.count] Compiled;
lines: [..] string;
log_bytes: int;
prose: string;
prepared := false;
seed: u64 = 987654321;

next :: (range: int) -> int {
    seed = seed * 6364136223846793005 + 1442695040888963407;
    return cast(int) ((seed >> 33) % cast(u64) range);
}

WORDS :: string.["the", "of", "and", "to", "in", "was", "that", "he", "it", "his", "with", "as", "had", "for", "at", "be", "on", "which", "have", "but", "not", "you", "said", "from", "all", "were", "her", "she", "there", "would", "their", "we", "him", "been", "has", "when", "who", "will", "more", "no", "if", "out", "so", "up", "said", "what", "about", "than", "into", "them", "can", "only", "other", "new", "some", "could", "time", "these", "two", "may", "then", "do", "first", "any", "my", "now", "such", "like", "our", "over", "man", "me", "even", "most", "made", "after", "also", "did", "many", "before", "must", "through", "back", "years", "where", "much", "your", "way", "well", "down", "should", "because", "each", "just", "those", "people", "how", "too", "little", "state", "good", "very", "make", "world", "still", "own", "see", "men", "work", "long", "get", "here", "between", "both", "life", "being", "under", "never", "day", "same", "another", "know", "while", "last", "might", "us", "great", "old", "year", "off", "come", "since", "against", "go", "came", "right", "used", "take", "three", "running", "walking", "thinking", "morning", "nothing", "something", "everything", "looking", "coming", "going", "Sherlock", "Holmes", "Watson", "Irene", "Adler"];

HOSTS :: string.["example.com", "mail.example.org", "corp.example.co.uk", "users.example.net", "dev.example.io"];
FIRST :: string.["alice", "bob", "carol", "dave", "erin", "frank", "grace", "heidi"];
PATHS :: string.["/index.html", "/docs/guide?page=2", "/api/v1/items/42", "/search?q=regex&lang=en", "/a/b/c.png"];
METHODS :: string.["GET", "GET", "GET", "POST", "PUT", "DELETE"];
STATUSES :: string.["200", "200", "200", "200", "301", "304", "404", "500", "503"];

make_prose :: () -> string {
    builder: String_Builder;

    while builder_string_length(*builder) < 1_000_000 {
        roll := next(200);

        if roll == 0 {
            print(*builder, "% % % %.% ", FIRST[next(FIRST.count)], "wrote to", FIRST[next(FIRST.count)], next(1000), next(100));
            print(*builder, "<%@%> ", FIRST[next(FIRST.count)], HOSTS[next(HOSTS.count)]);
        } else if roll < 3 {
            print(*builder, "%.%@% ", FIRST[next(FIRST.count)], next(100), HOSTS[next(HOSTS.count)]);
        } else if roll < 5 {
            print(*builder, "see \"https://%%\" or ", HOSTS[next(HOSTS.count)], PATHS[next(PATHS.count)]);
        } else if roll < 6 {
            print(*builder, "http://%/ ", HOSTS[next(HOSTS.count)]);
        } else {
            append(*builder, WORDS[next(WORDS.count)]);
            append(*builder, ifx next(12) == 0 then ". " else " ");
        }
    }

    return builder_to_string(*builder);
}

make_lines :: () {
    for 0..19999 {
        builder: String_Builder;

        print(*builder, "%.%.%.% - - [%/Oct/2026:%:%:% +0000] \"% % HTTP/1.1\" % % \"-\" \"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36\"", 1 + next(254), next(256), next(256), 1 + next(254), 1 + next(28), 10 + next(14), 10 + next(50), 10 + next(50), METHODS[next(METHODS.count)], PATHS[next(PATHS.count)], STATUSES[next(STATUSES.count)], next(100000));
        line := builder_to_string(*builder);

        array_add(*lines, line);
        log_bytes += line.count;
    }
}

prepare :: () {
    if prepared return;

    make_lines();
    prose = make_prose();

    for pattern, index: PATTERNS {
        re, ok, error := Regex.compile(pattern);

        assert(ok, "Jacket could not compile %", pattern);
        compiled[index].jacket = re;

        u, uniform_ok := uniform.compile(pattern);

        assert(uniform_ok, "uniform could not compile %", pattern);
        compiled[index].uniform = u;

        r, rematch_ok := rematch.compile(pattern, .GLOBAL);

        assert(rematch_ok, "Rematch could not compile %", pattern);
        compiled[index].rematch = r;
    }

    // The counts Jacket finds are what the others must find.
    for line: lines {
        caps, captured := Regex.captures(*compiled[LOG].jacket, line);
        if captured compiled[LOG].expected += 1;
        if Regex.is_match(*compiled[ERROR].jacket, line) compiled[ERROR].expected += 1;
    }

    for EMAIL..ING {
        compiled[it].expected = jacket_find_all(*compiled[it].jacket, prose);
    }

    prepared = true;
}

require_count :: (got: int, want: int) {
    if got != want {
        print("a library found % matches where Jacket found %\n", got, want);
        exit(1);
    }
}

over :: (b: *Bench, bytes: int, work: ()) {
    prepare();
    set_bytes(b, bytes);
    reset_timer(b);

    for 0..b.iterations - 1 {
        work();
    }
}

// ----------------------------------------------------------------------------- counting helpers

jacket_find_all :: (re: *Regex.Regex, text: string) -> int {
    matches := Regex.find_all(re, text);
    count := matches.count;
    array_free(matches);
    return count;
}

static_find_all :: ($R: Type, text: string) -> int {
    matches := Regex.find_all(R, text);
    count := matches.count;
    array_free(matches);
    return count;
}

jacket_lines_captured :: (re: *Regex.Regex) -> int {
    count := 0;

    for line: lines {
        _, ok := Regex.captures(re, line);
        if ok count += 1;
    }

    return count;
}

jacket_lines_matching :: (re: *Regex.Regex) -> int {
    count := 0;
    for line: lines if Regex.is_match(re, line) count += 1;
    return count;
}

static_lines_captured :: ($R: Type) -> int {
    count := 0;

    for line: lines {
        groups, ok := Regex.captures(R, line);
        if ok count += 1;
    }

    return count;
}

static_lines_matching :: ($R: Type) -> int {
    count := 0;
    for line: lines if Regex.is_match(R, line) count += 1;
    return count;
}

uniform_lines_captured :: (re: *uniform.Regexp) -> int {
    count := 0;

    for line: lines {
        matched, captures := uniform.match(line, re.*);
        if matched count += 1;
        array_free(captures);
    }

    return count;
}

uniform_lines_matching :: (re: *uniform.Regexp) -> int {
    return uniform_lines_captured(re);
}

uniform_find_all :: (re: *uniform.Regexp, text: string) -> int {
    count := 0;
    rest := text;

    while rest.count {
        matched, captures := uniform.match(rest, re.*);
        if !matched break;

        count += 1;
        end := captures[0].data + captures[0].count;

        array_free(captures);
        advance := end - rest.data;
        if advance <= 0 advance = 1;

        rest.data += advance;
        rest.count -= advance;
    }

    return count;
}

rematch_lines_captured :: (re: *rematch.Regex) -> int {
    count := 0;

    for line: lines {
        matches, result := rematch.match(re, line);
        if matches.count count += 1;
        rematch.free_matches(matches);
    }

    return count;
}

rematch_find_all :: (re: *rematch.Regex, text: string) -> int {
    matches, result := rematch.match(re, text);
    count := matches.count;
    rematch.free_matches(matches);
    return count;
}

// ----------------------------------------------------------------------------- log parsing

log_parse__jacket_static :: (b: *Bench) {
    prepare();
    over(b, log_bytes, () {
        require_count(static_lines_captured(Log_Static), compiled[LOG].expected);
    });
} @bench

log_parse__jacket :: (b: *Bench) {
    prepare();
    over(b, log_bytes, () {
        require_count(jacket_lines_captured(*compiled[LOG].jacket), compiled[LOG].expected);
    });
} @bench

log_parse__uniform :: (b: *Bench) {
    prepare();
    over(b, log_bytes, () {
        require_count(uniform_lines_captured(*compiled[LOG].uniform), compiled[LOG].expected);
    });
} @bench

log_parse__rematch :: (b: *Bench) {
    prepare();
    over(b, log_bytes, () {
        require_count(rematch_lines_captured(*compiled[LOG].rematch), compiled[LOG].expected);
    });
} @bench

log_filter__jacket_static :: (b: *Bench) {
    prepare();
    over(b, log_bytes, () {
        require_count(static_lines_matching(Error_Static), compiled[ERROR].expected);
    });
} @bench

log_filter__jacket :: (b: *Bench) {
    prepare();
    over(b, log_bytes, () {
        require_count(jacket_lines_matching(*compiled[ERROR].jacket), compiled[ERROR].expected);
    });
} @bench

log_filter__uniform :: (b: *Bench) {
    prepare();
    over(b, log_bytes, () {
        require_count(uniform_lines_matching(*compiled[ERROR].uniform), compiled[ERROR].expected);
    });
} @bench

log_filter__rematch :: (b: *Bench) {
    prepare();
    over(b, log_bytes, () {
        require_count(rematch_lines_captured(*compiled[ERROR].rematch), compiled[ERROR].expected);
    });
} @bench

// ----------------------------------------------------------------------------- extraction and search

extract_emails__jacket_static :: (b: *Bench) {
    prepare();
    over(b, prose.count, () {
        require_count(static_find_all(Email_Static, prose), compiled[EMAIL].expected);
    });
} @bench

extract_emails__jacket :: (b: *Bench) {
    prepare();
    over(b, prose.count, () {
        require_count(jacket_find_all(*compiled[EMAIL].jacket, prose), compiled[EMAIL].expected);
    });
} @bench

extract_emails__uniform :: (b: *Bench) {
    prepare();
    over(b, prose.count, () {
        require_count(uniform_find_all(*compiled[EMAIL].uniform, prose), compiled[EMAIL].expected);
    });
} @bench

extract_emails__rematch :: (b: *Bench) {
    prepare();
    over(b, prose.count, () {
        require_count(rematch_find_all(*compiled[EMAIL].rematch, prose), compiled[EMAIL].expected);
    });
} @bench

extract_urls__jacket_static :: (b: *Bench) {
    prepare();
    over(b, prose.count, () {
        require_count(static_find_all(Url_Static, prose), compiled[URL].expected);
    });
} @bench

extract_urls__jacket :: (b: *Bench) {
    prepare();
    over(b, prose.count, () {
        require_count(jacket_find_all(*compiled[URL].jacket, prose), compiled[URL].expected);
    });
} @bench

extract_urls__uniform :: (b: *Bench) {
    prepare();
    over(b, prose.count, () {
        require_count(uniform_find_all(*compiled[URL].uniform, prose), compiled[URL].expected);
    });
} @bench

extract_urls__rematch :: (b: *Bench) {
    prepare();
    over(b, prose.count, () {
        require_count(rematch_find_all(*compiled[URL].rematch, prose), compiled[URL].expected);
    });
} @bench

search_names__jacket_static :: (b: *Bench) {
    prepare();
    over(b, prose.count, () {
        require_count(static_find_all(Names_Static, prose), compiled[NAMES].expected);
    });
} @bench

search_names__jacket :: (b: *Bench) {
    prepare();
    over(b, prose.count, () {
        require_count(jacket_find_all(*compiled[NAMES].jacket, prose), compiled[NAMES].expected);
    });
} @bench

search_names__uniform :: (b: *Bench) {
    prepare();
    over(b, prose.count, () {
        require_count(uniform_find_all(*compiled[NAMES].uniform, prose), compiled[NAMES].expected);
    });
} @bench

search_names__rematch :: (b: *Bench) {
    prepare();
    over(b, prose.count, () {
        require_count(rematch_find_all(*compiled[NAMES].rematch, prose), compiled[NAMES].expected);
    });
} @bench

search_ing__jacket_static :: (b: *Bench) {
    prepare();
    over(b, prose.count, () {
        require_count(static_find_all(Ing_Static, prose), compiled[ING].expected);
    });
} @bench

search_ing__jacket :: (b: *Bench) {
    prepare();
    over(b, prose.count, () {
        require_count(jacket_find_all(*compiled[ING].jacket, prose), compiled[ING].expected);
    });
} @bench

search_ing__uniform :: (b: *Bench) {
    prepare();
    over(b, prose.count, () {
        require_count(uniform_find_all(*compiled[ING].uniform, prose), compiled[ING].expected);
    });
} @bench

search_ing__rematch :: (b: *Bench) {
    prepare();
    over(b, prose.count, () {
        require_count(rematch_find_all(*compiled[ING].rematch, prose), compiled[ING].expected);
    });
} @bench

// ----------------------------------------------------------------------------- compiling

compile_email__jacket :: (b: *Bench) {
    prepare();
    over(b, EMAIL_PATTERN.count, () {
        re, ok := Regex.compile(black_box(EMAIL_PATTERN));
        black_box(re);
        Regex.free_regex(*re);
        require_count(cast(int) ok, 1);
    });
} @bench

compile_email__uniform :: (b: *Bench) {
    prepare();
    over(b, EMAIL_PATTERN.count, () {
        re, ok := uniform.compile(black_box(EMAIL_PATTERN));
        black_box(re);
        uniform.uninit(*re);
        require_count(cast(int) ok, 1);
    });
} @bench

compile_email__rematch :: (b: *Bench) {
    prepare();
    over(b, EMAIL_PATTERN.count, () {
        re, ok := rematch.compile(black_box(EMAIL_PATTERN), .GLOBAL);
        black_box(re);
        rematch.deinit(*re);
        require_count(cast(int) ok, 1);
    });
} @bench
