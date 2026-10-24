// Checks the psql input scanner (scan.jai) against what psql does with the same text.
//
//   jaic -interp scan_test.jai                 prints `ok`
//   jaic -interp scan_test.jai -- file.sql [name=value ...]
//                                              prints the statements and commands psql would
//                                              extract from the file (see `dump_script`), for
//                                              comparison with `psql -e`; name=value presets a
//                                              variable like `psql -v`
#import "Basic";
#import "String";
#import "File";
#load "scan.jai";

main :: () {
    args := get_command_line_arguments();
    if args.count > 1 {
        dump_script(args[1], args);
        return;
    }

    test_statement_boundaries();
    test_text_sent();
    test_quotes();
    test_dollar_quotes();
    test_parentheses_and_bodies();
    test_variables();
    test_backslash_commands();
    test_conditionals();
    test_copy_from_stdin();
    test_lines_and_state();
    print("ok\n");
}

// ---- A miniature MainLoop: just enough of psql's to exercise the scanner ----

// The state of one `\if` level, as in psql's conditional stack.
If_State :: enum {
    TRUE;
    FALSE;
    IGNORED; // an earlier branch ran, or the whole block is inside a dead branch
    ELSE_TRUE;
    ELSE_FALSE;
}

If_Level :: struct {
    state: If_State;
    saved_length: s64; // query buffer length when the current branch started
    saved_lex: Lex_State;
}

Runner :: struct {
    scanner: Scanner;
    vars: Variables;
    query: [..] u8;
    events: [..] string; // "Q" + SQL for a statement, "C" + `\name|arg|...` for a command
    messages: [..] string;
    levels: [..] If_Level;
    quit: bool;
    copy_pending: s64; // COPY FROM STDIN commands whose data lines are still to be skipped
}

new_runner :: () -> *Runner {
    r := New(Runner);
    r.scanner.lookup = runner_lookup;
    r.scanner.report = record_message;
    r.scanner.hook_data = r;

    return r;
}

runner_lookup :: (name: string, data: *void) -> string, bool {
    return variables_get(*(cast(*Runner) data).vars, name);
}

record_message :: (kind: Message_Kind, text: string, data: *void) {
    r := cast(*Runner) data;
    array_add(*r.messages, copy_string(text));
}

branch_active :: (r: *Runner) -> bool {
    if r.levels.count == 0 return true;
    state := r.levels[r.levels.count - 1].state;
    return state == .TRUE || state == .ELSE_TRUE;
}

// psql's boolean spelling: unique prefixes of true, false, yes, no, on, off, and 1/0.
parse_bool :: (text: string) -> bool, bool {
    t := to_lower_copy(text,, temp);
    if t == "1" || t == "t" || t == "tr" || t == "tru" || t == "true" || t == "y" || t == "ye" || t == "yes" || t == "on" return true, true;
    if t == "0" || t == "f" || t == "fa" || t == "fal" || t == "fals" || t == "false" || t == "n" || t == "no" || t == "of" || t == "off" return false, true;
    return false, false;
}

send_query :: (r: *Runner) {
    if r.query.count > 0 {
        array_add(*r.events, join("Q", copy_string(buffer_string(r.query))));
        r.query.count = 0;
    }

    scan_reset(*r.scanner);
}

run_into :: (r: *Runner, text: string) {
    s := *r.scanner;
    lines := split(text, "\n");

    for line, line_index: lines {
        // A final newline leaves an empty last element, which is not a line.
        if line_index == lines.count - 1 && line.count == 0 break;
        if r.quit break;

        // The data of COPY FROM STDIN is read by psql itself, up to a line holding `\.`. (This
        // assumes the COPY succeeds; after a failed one the data lines would be scanned as SQL.)
        if r.copy_pending > 0 {
            if line == "\\." r.copy_pending -= 1;
            continue;
        }

        if line.count == 0 && !scan_in_quote(s) continue;

        added_newline := -1;

        if r.query.count > 0 {
            array_add(*r.query, #char "\n");
            added_newline = r.query.count;
        }

        scan_setup(s, line);

        while true {
            s.active = branch_active(r);
            result, _ := scan(s, *r.query);

            if result == .SEMICOLON {
                // In a dead branch the statement is not sent, and the buffer keeps growing until
                // the branch ends and throws it away.
                if branch_active(r) {
                    copies := scan_count_copy_from_stdin(s);
                    sql := join("Q", copy_string(buffer_string(r.query)));

                    if copies > 0 sql = join(sql, tprint("\n[copy from stdin x%]", copies));

                    array_add(*r.events, sql);
                    track_settings(r, sql);
                    r.copy_pending += copies;
                    r.query.count = 0;
                    scan_reset(s);
                    added_newline = -1;
                }
            } else if result == .BACKSLASH {
                if r.query.count == added_newline r.query.count -= 1;
                added_newline = -1;
                run_command(r);
            }

            if result == .INCOMPLETE || result == .EOL break;
            if r.quit break;
        }

        scan_finish(s);
    }

    if r.query.count > 0 && branch_active(r) && !r.quit {
        array_add(*r.events, join("Q", copy_string(buffer_string(r.query))));
    }
}

// psql learns standard_conforming_strings from the server after each statement. A script that
// changes it with a plain SET or RESET is followed here, which is all the regression suite does.
track_settings :: (r: *Runner, sql: string) {
    text := to_lower_copy(sql,, temp);
    if !contains(text, "standard_conforming_strings") return;

    if begins_with(text, "reset") r.scanner.std_strings = true;
    else if contains(text, "off") r.scanner.std_strings = false;
    else if contains(text, "on") r.scanner.std_strings = true;
}

read_options :: (s: *Scanner, kind: Option_Kind, semicolon: bool) -> [..] string {
    list: [..] string;

    while true {
        text, found, _ := scan_slash_option(s, kind, semicolon);
        if !found break;
        array_add(*list, text);
    }

    return list;
}

save_branch :: (r: *Runner, level: *If_Level) {
    level.saved_length = r.query.count;
    level.saved_lex = scan_save_lex_state(*r.scanner);
}

// Throws away what a dead branch added to the query buffer.
discard_branch :: (r: *Runner, level: *If_Level) {
    r.query.count = level.saved_length;
    scan_restore_lex_state(*r.scanner, level.saved_lex);
}

run_command :: (r: *Runner) {
    s := *r.scanner;
    name := scan_slash_command(s);
    active := branch_active(r);

    s.active = active;

    kind := Option_Kind.NORMAL;
    semicolon := false;

    if name == {
        case "!"; #through;
        case "copy"; kind = .WHOLE_LINE;
        case "o"; #through;
        case "g"; #through;
        case "gx"; #through;
        case "w"; kind = .FILE_OR_PIPE;
        case "c"; #through;
        case "connect"; kind = .SQL_ID_KEEP_CASE;
        case "if"; #through;
        case "elif"; #through;
        case "else"; #through;
        case "endif"; #through;
        case "set"; #through;
        case "unset"; #through;
        case "echo"; #through;
        case "warn"; #through;
        case "qecho"; semicolon = false;
        case; semicolon = true;
    }

    // File-or-pipe and identifier commands take one argument of their kind.
    args: [..] string;

    if kind == .FILE_OR_PIPE || kind == .SQL_ID_KEEP_CASE {
        first, found, _ := scan_slash_option(s, kind, true);
        if found array_add(*args, first);
    } else {
        args = read_options(s, kind, semicolon && kind == .NORMAL);
    }

    sends := false;
    record := active;

    if name == "if" {
        record = true;
        level := If_Level.{ state = .IGNORED };

        if active {
            value, ok := parse_bool(ifx args.count then join_args(args) else "");
            level.state = ifx value && ok then If_State.TRUE else .FALSE;
        }

        save_branch(r, *level);
        array_add(*r.levels, level);
    } else if name == "elif" && r.levels.count > 0 {
        record = true;
        level := *r.levels[r.levels.count - 1];
        if level.state == {
            case .TRUE;
                save_branch(r, level);
                level.state = .IGNORED;
            case .FALSE;
                discard_branch(r, level);
                value, ok := parse_bool(ifx args.count then join_args(args) else "");

                level.state = ifx value && ok then If_State.TRUE else .FALSE;
            case .IGNORED;
                discard_branch(r, level);
        }
    } else if name == "else" && r.levels.count > 0 {
        record = true;
        level := *r.levels[r.levels.count - 1];
        if level.state == {
            case .TRUE;
                save_branch(r, level);
                level.state = .ELSE_FALSE;
            case .FALSE;
                discard_branch(r, level);
                level.state = .ELSE_TRUE;
            case .IGNORED;
                discard_branch(r, level);
                level.state = .ELSE_FALSE;
        }
    } else if name == "endif" && r.levels.count > 0 {
        record = true;
        level := *r.levels[r.levels.count - 1];
        if level.state == .FALSE || level.state == .IGNORED || level.state == .ELSE_FALSE discard_branch(r, level);
        r.levels.count -= 1;
    } else if active {
        if name == "set" {
            if args.count >= 1 {
                value: string;
                for 1..args.count - 1 value = join(value, args[it]);
                variables_set(*r.vars, args[0], value);
            }
        } else if name == "unset" {
            if args.count >= 1 variables_unset(*r.vars, args[0]);
        } else if name == "q" {
            r.quit = true;
        } else if name == "r" {
            r.query.count = 0;
            scan_reset(s);
        } else if name == "g" || name == "gx" || name == "gset" || name == "gexec" || name == "gdesc" || name == "crosstabview" || name == "watch" {
            sends = true;
        }
    }

    // Whatever the command left unread is discarded without expanding anything.
    s.active = false;

    while true {
        _, found, _ := scan_slash_option(s, .NORMAL, false);
        if !found break;
    }

    s.active = branch_active(r);

    scan_slash_command_end(s);

    if sends send_query(r);

    if record {
        text := join("C\\", name);
        for args text = join(text, "|", it);
        array_add(*r.events, text);
    }
}

join_args :: (args: [] string) -> string {
    text: string;

    for args {
        if it_index > 0 text = join(text, " ");
        text = join(text, it);
    }

    return text;
}

// The statements and commands of a script as "Q..." and "C..." strings.
events_of :: (text: string, vars: [] string = .[]) -> [] string {
    r := new_runner();
    for 0..vars.count / 2 - 1 variables_set(*r.vars, vars[it * 2], vars[it * 2 + 1]);
    run_into(r, text);
    return r.events;
}

split_name_value :: (text: string) -> string, string {
    for *text {
        if it.* == #char "=" return slice(text, 0, it_index), slice(text, it_index + 1, text.count - it_index - 1);
    }

    return text, "";
}

// Prints what psql would send for a file: `@@Q` or `@@C` on a line of its own, then the
// statement or backslash command, then `@@END`. Variables set by `\set` are tracked; ones that
// psql would get from the server (`\gset`) are not.
dump_script :: (path: string, args: [] string) {
    text, ok := read_entire_file(path);
    if !ok {
        print("cannot read %\n", path);
        return;
    }

    presets: [..] string;

    for 2..args.count - 1 {
        name, value := split_name_value(args[it]);
        array_add(*presets, name);
        array_add(*presets, value);
    }

    for events_of(text, presets) {
        print("@@%\n%\n@@END\n", slice(it, 0, 1), slice(it, 1, it.count - 1));
    }
}

// ---- Tests ----

expect :: (text: string, expected: ..string, vars: [] string = .[]) {
    got := events_of(text, vars);

    ok := got.count == expected.count;

    if ok for 0..got.count - 1 if got[it] != expected[it] ok = false;

    if !ok {
        print("FAIL for input:\n%\n--- expected (%):\n", text, expected.count);
        for expected print("  [%]\n", it);

        print("--- got (%):\n", got.count);
        for got print("  [%]\n", it);

        assert(false);
    }
}

test_statement_boundaries :: () {
    expect("select 1;", "Qselect 1;");
    expect("select 1;\nselect 2;\n", "Qselect 1;", "Qselect 2;");
    expect("select 1; select 2;", "Qselect 1;", "Qselect 2;");
    expect("select\n  1\n;", "Qselect\n  1\n;");
    expect("select 1", "Qselect 1"); // no semicolon: sent at the end of input
    expect("select 1;;", "Qselect 1;", "Q;");
    expect("\n\nselect 1;\n\n", "Qselect 1;");
    expect("select 1\n\n\n;", "Qselect 1\n;"); // empty lines outside quotes are skipped
}

test_text_sent :: () {
    expect("-- c\nselect 1;", "Qselect 1;");
    expect("  \t select 1;", "Qselect 1;");
    expect("/* c */ select 1;", "Q/* c */ select 1;");
    expect("select 1 -- x\n, 2;", "Qselect 1 -- x\n, 2;");
    expect("-- a\n-- b\n/* c */\nselect 1;", "Q/* c */\nselect 1;");
    expect("select 1\n-- c\n;", "Qselect 1\n-- c\n;");
    expect("select 1; -- trailing\nselect 2;", "Qselect 1;", "Qselect 2;");
    expect("select 1;  /* x */", "Qselect 1;", "Q/* x */");
    expect("select 1--c\n;", "Qselect 1--c\n;");
    expect("select 1 +--c\n 2;", "Qselect 1 +--c\n 2;");
    expect("select 1 \n  \n  from t", "Qselect 1 \n  \n  from t"); // a line of blanks is kept
    expect("select 1 \n\n  from t", "Qselect 1 \n  from t");
}

test_quotes :: () {
    expect("select ';';", "Qselect ';';");
    expect("select 'it''s;';", "Qselect 'it''s;';");
    expect("select E'a\\';b';", "Qselect E'a\\';b';");
    expect("select e'\\\\';select 2;", "Qselect e'\\\\';", "Qselect 2;");
    expect("select '\\';", "Qselect '\\';"); // standard strings: the backslash is an ordinary character
    expect("select \"a;b\";", "Qselect \"a;b\";");
    expect("select \"a\"\"b;\";", "Qselect \"a\"\"b;\";");
    expect("select U&'\\0041;';", "Qselect U&'\\0041;';");
    expect("select U&\"d\\0061;\";", "Qselect U&\"d\\0061;\";");
    expect("select B'101;', X'1f;', N'a;';", "Qselect B'101;', X'1f;', N'a;';");
    expect("select 'a\n;\nb';", "Qselect 'a\n;\nb';");

    // The line breaks inside a literal are data, so even empty lines stay.
    expect("select 'a\n\nb';", "Qselect 'a\n\nb';");

    // standard_conforming_strings off: backslash escapes in plain strings.
    r := new_runner();
    r.scanner.std_strings = false;
    run_into(r, "select 'a\\';b';");
    assert(r.events.count == 1 && r.events[0] == "Qselect 'a\\';b';");

    // A comment looks like one only outside quotes.
    expect("select '-- no;' -- yes;\n;", "Qselect '-- no;' -- yes;\n;");
    expect("select 1 /* ; */ ;", "Qselect 1 /* ; */ ;");
    expect("select 1 /* /* nested ; */ ; */ ;", "Qselect 1 /* /* nested ; */ ; */ ;");
    expect("select 1 /** ; **/ ;", "Qselect 1 /** ; **/ ;");
    expect("select 1 /* a\n; */ ;", "Qselect 1 /* a\n; */ ;");
}

test_dollar_quotes :: () {
    expect("select $$;$$;", "Qselect $$;$$;");
    expect("select $a$ $b$ ; $a$;", "Qselect $a$ $b$ ; $a$;");
    expect("select $a$ $a ; $b$ ; $a$;", "Qselect $a$ $a ; $b$ ; $a$;");
    expect("select $a$ $b$a$ ; $a$;", "Qselect $a$ $b$a$ ;", "Q$a$;"); // `$b$a$` ends the string at its last three characters
    expect("select $abc$x;$ab$;$abc$;", "Qselect $abc$x;$ab$;$abc$;");
    expect("select $1; select 2;", "Qselect $1;", "Qselect 2;"); // a parameter, not a tag
    expect("select $1$a; select 2;", "Qselect $1$a;", "Qselect 2;");
    expect("select a$b$c;", "Qselect a$b$c;"); // a dollar inside an identifier
    expect("select $a$ \n ; \n $a$;", "Qselect $a$ \n ; \n $a$;");
    expect("select $é$;$é$;", "Qselect $é$;$é$;");
    expect("select $tag; select 2;", "Qselect $tag;", "Qselect 2;"); // `$tag` without a closing dollar
    expect("select $a$ $$ $a$; select 2;", "Qselect $a$ $$ $a$;", "Qselect 2;");
}

test_parentheses_and_bodies :: () {
    expect("select (1;\n2);", "Qselect (1;\n2);");
    expect("select ')';", "Qselect ')';");
    expect("select 1);", "Qselect 1);"); // a stray close parenthesis does not go negative
    expect("select (1;", "Qselect (1;");

    // SQL-standard function bodies.
    body := "create function f() returns int\nbegin atomic\n select 1;\n select 2;\nend;\nselect 3;";

    expect(body, "Qcreate function f() returns int\nbegin atomic\n select 1;\n select 2;\nend;", "Qselect 3;");
    expect("CREATE OR REPLACE PROCEDURE p() BEGIN ATOMIC SELECT 1; END; select 2;", "QCREATE OR REPLACE PROCEDURE p() BEGIN ATOMIC SELECT 1; END;", "Qselect 2;");
    expect("create function f() returns int begin atomic select case when true then 1 else 2 end; end;", "Qcreate function f() returns int begin atomic select case when true then 1 else 2 end; end;");
    expect("create function f() returns int begin atomic select case when true then (case when false then 1 end) end; end; select 2;", "Qcreate function f() returns int begin atomic select case when true then (case when false then 1 end) end; end;", "Qselect 2;");

    // BEGIN outside a CREATE FUNCTION is a transaction command.
    expect("begin; select 1; end;", "Qbegin;", "Qselect 1;", "Qend;");
    expect("create table t(begin int); select 1;", "Qcreate table t(begin int);", "Qselect 1;");
    expect("create function f() returns int as $$ begin return 1; end; $$ language plpgsql; select 2;", "Qcreate function f() returns int as $$ begin return 1; end; $$ language plpgsql;", "Qselect 2;");
    expect("create function f(a int default 1) returns int language sql return a; select 2;", "Qcreate function f(a int default 1) returns int language sql return a;", "Qselect 2;");
}

test_variables :: () {
    expect("select :x;", "Qselect 5;", vars = .["x", "5"]);
    expect("select :'x', :\"x\";", "Qselect 'a''b', \"a'b\";", vars = .["x", "a'b"]);
    expect("select :'x';", "Qselect  E'a\\\\b';", vars = .["x", "a\\b"]);
    expect("select :\"x\";", "Qselect \"a\"\"b\";", vars = .["x", "a\"b"]);
    expect("select 1::int, :y;", "Qselect 1::int, :y;", vars = .["x", "1"]); // unknown stays
    expect("select ':x' \":x\" $$:x$$ /* :x */ -- :x\n;", "Qselect ':x' \":x\" $$:x$$ /* :x */ -- :x\n;", vars = .["x", "1"]);
    expect("select a::b, a:c, a:=b;", "Qselect a::b, a:c, a:=b;", vars = .["b", "1"]);
    expect("select :x:y;", "Qselect 1:y;", vars = .["x", "1"]);
    expect("select :x'a';", "Qselect 1'a';", vars = .["x", "1"]);
    expect("select :'x;", "Qselect :'x;", vars = .["x", "1"]);
    expect("select :'x\" ;", "Qselect :'x\" ;", vars = .["x", "1"]);
    expect("select :{?x}, :{?y};", "Qselect TRUE, FALSE;", vars = .["x", "1"]);
    expect("select :{?x;", "Qselect :{?x;", vars = .["x", "1"]);
    expect("select :'x'::text;", "Qselect '1'::text;", vars = .["x", "1"]);
    expect("select :'', :\"\";", "Qselect :'', :\"\";", vars = .["x", "1"]);
    expect("select a[1:2], 1 :x;", "Qselect a[1:2], 1 3;", vars = .["x", "3"]);

    // A value is scanned like the rest of the input: it may hold SQL, even a semicolon.
    expect("select :x", "Qselect 1;", "Qselect 2", vars = .["x", "1; select 2"]);
    expect(":x\nselect 3;", "Qselect 1;", "Qselect 2\nselect 3;", vars = .["x", "select 1; select 2"]);
    expect("select 1 :x 2;", "Qselect 1 a 'b ; c' d 2;", vars = .["x", "a 'b ; c' d"]);

    // A quote opened inside a value continues into the text after it.
    expect("select :x b';", "Qselect 'a b';", vars = .["x", "'a"]);

    // A value that names itself is left alone.
    r := new_runner();

    variables_set(*r.vars, "a", "x :a y");
    run_into(r, "select :a;");
    assert(r.events.count == 1 && r.events[0] == "Qselect x :a y;");
    assert(r.messages.count == 1 && r.messages[0] == "skipping recursive expansion of variable \"a\"");

    // Mutual recursion stops at the second visit.
    r2 := new_runner();

    variables_set(*r2.vars, "a", "[:b]");
    variables_set(*r2.vars, "b", "(:a)");
    run_into(r2, "select :a;");
    assert(r2.events.count == 1 && r2.events[0] == "Qselect [(:a)];");

    // Whitespace around a leading variable is dropped like any other leading whitespace.
    expect("  :x  ;", "Qa  ;", vars = .["x", "a"]);

    assert(escape_literal("it's") == "'it''s'");
    assert(escape_literal("a\\b'c") == " E'a\\\\b''c'");
    assert(escape_literal("") == "''");
    assert(escape_identifier("a\"b") == "\"a\"\"b\"");
    s, ok := shell_quote("it's");

    assert(ok && s == "'it'\\''s'");
    _, ok = shell_quote("a\nb");
    assert(!ok);
}

test_backslash_commands :: () {
    expect("\\echo hello world", "C\\echo|hello|world");
    expect("\\echo 'a  b' \"c  d\" e", "C\\echo|a  b|\"c  d\"|e");
    expect("\\echo 'it''s' 'x\\ny' 'tab\\there' '\\101' '\\x41' '\\q' ''", "C\\echo|it's|x\ny|tab\there|A|A|q|");
    expect("\\echo :x :'x' :\"x\" :y", "C\\echo|7|'7'|\"7\"|:y", vars = .["x", "7"]);
    expect("\\echo a:b ::x", "C\\echo|a:b|:7", vars = .["x", "7"]); // `::x` is a colon, then :x

    // A command ends at the end of the line, and at a backslash.
    expect("\\echo a \\echo b", "C\\echo|a", "C\\echo|b");
    expect("\\echo a\\echo b", "C\\echo|a", "C\\echo|b");

    // `\\` ends the command and lets SQL follow on the same line.
    expect("\\echo a \\\\ select 1;", "C\\echo|a", "Qselect 1;");
    expect("\\set x 1 \\\\ select :x;", "C\\set|x|1", "Qselect 1;");

    // A semicolon ends the statement but a trailing one in an argument is dropped by `\d`.
    expect("\\d+ foo;", "C\\d+|foo");
    expect("\\d 'foo;'", "C\\d|foo;");
    expect("\\echo a;", "C\\echo|a;");

    // Sending commands.
    expect("select 1 \\g", "Qselect 1 ", "C\\g");
    expect("select 1\n\\g", "Qselect 1", "C\\g"); // the newline of the next line is not part of it
    expect("select 1 \\g out.txt", "Qselect 1 ", "C\\g|out.txt");
    expect("select 1 \\g |cat -n", "Qselect 1 ", "C\\g||cat -n");
    expect("select 1 \\gx \\\\ select 2;", "Qselect 1 ", "C\\gx", "Qselect 2;");

    // A backslash command between two lines of one statement.
    expect("select 1\n\\echo hi\n, 2;", "C\\echo|hi", "Qselect 1\n, 2;");
    expect("select\n  \\echo hi\n1;", "C\\echo|hi", "Qselect\n  \n1;");

    // \; and \: put a literal character into the statement.
    expect("select 1 \\; select 2;", "Qselect 1 ; select 2;");
    expect("select 1\\:x, :x;", "Qselect 1:x, 9;", vars = .["x", "9"]);

    // Whole-line commands.
    expect("\\! echo  \"a  b\"   c ; ", "C\\!|echo  \"a  b\"   c ; "); // trailing space stays
    expect("\\copy t from stdin  ", "C\\copy|t from stdin  ");
    expect("\\!   ls -l", "C\\!|ls -l");
    expect("\\!", "C\\!");

    // File-or-pipe: a leading bar takes the rest of the line.
    expect("\\o |less -R", "C\\o||less -R");
    expect("\\o 'a b' c", "C\\o|a b");
    expect("\\o a|b", "C\\o|a|b");

    // Identifier arguments.
    s: Scanner;

    scan_setup(*s, "Foo.\"Bar\" \"A\"\"B\" \"NoClose");
    text, found, quote := scan_slash_option(*s, .SQL_ID, false);

    assert(found && text == "foo.Bar" && quote == #char "\"");
    text, found, quote = scan_slash_option(*s, .SQL_ID_KEEP_CASE, false);
    assert(found && text == "A\"B");
    s.report = null;
    text, found, quote = scan_slash_option(*s, .SQL_ID, false);
    assert(!found); // unterminated
    scan_finish(*s);

    // Backticks need a shell hook; without one the failure is flagged.
    scan_setup(*s, "`echo hi` after");
    text, found, quote = scan_slash_option(*s, .NORMAL, false);
    assert(s.backtick_failed && found && text == "" && quote == #char "`");
    scan_finish(*s);

    scan_setup(*s, "pre`echo :x`post :'x'");
    s.backtick_failed = false;
    s.run_shell = fake_shell;
    s.lookup = variables_lookup;
    vars: Variables;

    variables_set(*vars, "x", "a b");
    s.hook_data = *vars;
    text, found, quote = scan_slash_option(*s, .NORMAL, false);
    assert(found && text == "pre<echo a b>post" && quote == #char "`");
    text, found, quote = scan_slash_option(*s, .NORMAL, false);
    assert(found && text == "'a b'" && quote == #char ":");
    scan_finish(*s);

    // In a dead branch the command is not run.
    scan_setup(*s, "`echo hi`");
    s.active = false;
    text, found, quote = scan_slash_option(*s, .NORMAL, false);
    assert(found && text == "echo hi");
    s.active = true;
    scan_finish(*s);

    // An empty quoted argument is an argument; no argument is not.
    scan_setup(*s, "'' ");
    text, found, quote = scan_slash_option(*s, .NORMAL, false);
    assert(found && text == "" && quote == #char "'");
    text, found, quote = scan_slash_option(*s, .NORMAL, false);
    assert(!found);
    scan_finish(*s);

    // Trailing semicolons come off unquoted text only.
    scan_setup(*s, "a;; 'b;' c;'';");
    text, found, quote = scan_slash_option(*s, .NORMAL, true);
    assert(text == "a");
    text, found, quote = scan_slash_option(*s, .NORMAL, true);
    assert(text == "b;");
    text, found, quote = scan_slash_option(*s, .NORMAL, true);
    assert(text == "c;");
    text, found, quote = scan_slash_option(*s, .NORMAL, true);
    assert(!found);
    scan_finish(*s);
}

fake_shell :: (command: string, data: *void) -> string, bool {
    return tprint("<%>\n", command), true;
}

test_conditionals :: () {
    expect("\\if true\nselect 1;\n\\endif", "C\\if|true", "Qselect 1;", "C\\endif");
    expect("\\if false\nselect 1;\n\\else\nselect 2;\n\\endif", "C\\if|false", "C\\else", "Qselect 2;", "C\\endif");

    // A dead branch is still scanned: its statements end at semicolons and its commands are
    // consumed, but nothing is sent and variables stay unexpanded.
    expect("\\if false\nselect (1;\n2); select 3;\n\\echo :x\n\\endif\nselect 4;", "C\\if|false", "C\\endif", "Qselect 4;", vars = .["x", "1"]);
    expect("\\if false\nselect 1\n\\else\nselect 2;\n\\endif", "C\\if|false", "C\\else", "Qselect 2;", "C\\endif");
    expect("\\if false\n\\elif true\nselect 2;\n\\elif true\nselect 3;\n\\endif", "C\\if|false", "C\\elif|true", "Qselect 2;", "C\\elif|true", "C\\endif");
    expect("\\if true\n\\if false\nselect 1;\n\\else\nselect 2;\n\\endif\n\\endif", "C\\if|true", "C\\if|false", "C\\else", "Qselect 2;", "C\\endif", "C\\endif");
    expect("\\if false\n\\if true\nselect 1;\n\\endif\n\\endif\nselect 2;", "C\\if|false", "C\\if|true", "C\\endif", "C\\endif", "Qselect 2;");

    // A partial statement before \if stays in the buffer.
    expect("select 1\n\\if false\n, 2\n\\else\n, 3\n\\endif\n;", "C\\if|false", "C\\else", "C\\endif", "Qselect 1\n, 3\n;");
}

test_copy_from_stdin :: () {
    counts :: (sql: string) -> s64 {
        s: Scanner;
        q: [..] u8;

        scan_setup(*s, sql);
        scan(*s, *q);
        total := scan_count_copy_from_stdin(*s);

        scan_finish(*s);

        return total;
    }

    assert(counts("copy t from stdin;") == 1);
    assert(counts("COPY t FROM STDIN;") == 1);
    assert(counts("copy t (a, b) from stdin;") == 1);
    assert(counts("copy binary public.t from stdin;") == 1);
    assert(counts("copy t to stdout;") == 0);
    assert(counts("copy (select 1) to stdout;") == 0);
    assert(counts("select 'copy t from stdin';") == 0);
    assert(counts("copy a from stdin") == 1); // no semicolon yet

    // Several statements in one buffer joined by \; count each.
    assert(counts("copy a from stdin\\; select 1\\; copy b from stdin;") == 2);
}

test_lines_and_state :: () {
    s: Scanner;
    q: [..] u8;

    // Results and prompts line by line.
    scan_setup(*s, "select 1");
    result, prompt := scan(*s, *q);

    assert(result == .EOL && prompt == .CONTINUE);
    scan_finish(*s);

    array_add(*q, #char "\n");
    scan_setup(*s, "  from (");
    result, prompt = scan(*s, *q);
    assert(result == .INCOMPLETE && prompt == .PAREN);
    scan_finish(*s);

    array_add(*q, #char "\n");
    scan_setup(*s, "select 'abc");
    result, prompt = scan(*s, *q);
    assert(result == .INCOMPLETE && prompt == .SINGLE_QUOTE && scan_in_quote(*s));
    scan_finish(*s);

    array_add(*q, #char "\n");
    scan_setup(*s, "def') ;");
    result, prompt = scan(*s, *q);
    assert(result == .SEMICOLON && prompt == .READY && !scan_in_quote(*s));
    assert(buffer_string(q) == "select 1\n  from (\nselect 'abc\ndef') ;");
    scan_finish(*s);

    scan_reset(*s);
    q.count = 0;

    // Nothing worth sending: only whitespace and a line comment.
    scan_setup(*s, "   -- nothing");
    result, prompt = scan(*s, *q);
    assert(result == .INCOMPLETE && prompt == .READY && q.count == 0);
    scan_finish(*s);

    scan_setup(*s, "/* open");
    result, prompt = scan(*s, *q);
    assert(result == .INCOMPLETE && prompt == .COMMENT && scan_in_quote(*s));
    scan_finish(*s);
    scan_setup(*s, "still */ select 1");
    result, prompt = scan(*s, *q);
    assert(result == .EOL && buffer_string(q) == "/* openstill */ select 1");
    scan_finish(*s);
    scan_reset(*s);
    q.count = 0;

    scan_setup(*s, "select $q$");
    result, prompt = scan(*s, *q);
    assert(result == .INCOMPLETE && prompt == .DOLLAR_QUOTE);
    scan_finish(*s);
    scan_reset(*s);
    q.count = 0;

    scan_setup(*s, "select \"abc");
    result, prompt = scan(*s, *q);
    assert(result == .INCOMPLETE && prompt == .DOUBLE_QUOTE);
    scan_finish(*s);
    scan_reset(*s);
    q.count = 0;

    scan_setup(*s, "create function f() returns int begin atomic");
    result, prompt = scan(*s, *q);
    assert(result == .INCOMPLETE && prompt == .CONTINUE);
    scan_finish(*s);

    // Resuming after a semicolon continues on the same line.
    scan_reset(*s);
    q.count = 0;
    scan_setup(*s, "a; b; \\c");
    result, _ = scan(*s, *q);
    assert(result == .SEMICOLON && buffer_string(q) == "a;");
    q.count = 0;
    scan_reset(*s);
    result, _ = scan(*s, *q);
    assert(result == .SEMICOLON && buffer_string(q) == "b;");
    q.count = 0;
    scan_reset(*s);
    result, _ = scan(*s, *q);
    assert(result == .BACKSLASH && q.count == 0);
    name := scan_slash_command(*s);

    assert(name == "c");
    scan_finish(*s);
}
