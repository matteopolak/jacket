// Checks print.jai against what psql 17 prints. Every expected string below was captured from a
// real psql run over the same data and settings. Run it with `jaic -interp print_test.jai`.
#import "Basic";
#import "String";
#load "print.jai";

failures: s64;

check_query :: (name: string, headers: [] string, type_oids: [] u32, cells: [] string, nulls: [] bool, rows: s64, options: Print_Options, expected: string) {
    out: String_Builder;
    defer free_buffers(*out);
    print_query_result(*out, headers, type_oids, cells, nulls, options, rows);
    compare(name, builder_to_string(*out,, temp), expected);
}

check_table :: (name: string, table: Table, options: Print_Options, expected: string) {
    out: String_Builder;
    defer free_buffers(*out);
    print_table(*out, table, options);
    compare(name, builder_to_string(*out,, temp), expected);
}

compare :: (name: string, actual: string, expected: string) {
    if actual == expected return;

    failures += 1;
    print("FAIL: %\n--- expected\n%--- actual\n%---\n", name, expected, actual);
}

main :: () {
    test_layouts();
    test_formats();
    test_widths();
    test_alignment();
    test_pieces();

    if failures != 0 {
        print("% failed\n", failures);
        assert(false);
    }

    print("ok\n");
}

// A result printed as its rows arrive (FETCH_COUNT 2): each piece is laid out on its own, and an
// empty last piece finishes the table.
test_pieces :: () {
    print_pieces :: (options: Print_Options) -> string {
        out: String_Builder;
        pieces := ([] string).[.["1", "x", "2", "xx"], .["3", "xxx"], .[]];
        prior := 0;

        for pieces {
            table: Table;
            table.headers = string.["g", "r"];
            table.align = u8.[#char "r", #char "l"];
            table.cells = it;
            table.rows = it.count / 2;
            table.default_footer = true;
            table.start_table = it_index == 0;
            table.stop_table = it_index == pieces.count - 1;
            table.prior_records = prior;
            print_table(*out, table, options);
            prior += table.rows;
        }

        return builder_to_string(*out,, temp);
    }

    options: Print_Options;
    options.border = 2;
    compare("pieces border 2", print_pieces(options), "+---+----+\n| g | r  |\n+---+----+\n| 1 | x  |\n| 2 | xx |\n| 3 | xxx |\n+---+---+\n(3 rows)\n\n");

    options.border = 1;
    options.expanded = .ON;
    compare("pieces expanded", print_pieces(options), "-[ RECORD 1 ]\ng | 1\nr | x\n-[ RECORD 2 ]\ng | 2\nr | xx\n-[ RECORD 3 ]\ng | 3\nr | xxx\n\n");
}

test_widths :: () {
    assert(display_width("") == 0);
    assert(display_width("abc") == 3);
    assert(display_width("日本語") == 6);
    assert(display_width("e\u0301") == 1);
    assert(display_width("a\tb") == 9);
    assert(display_width("\x01") == 4);
    assert(display_width("\r") == 2);
    assert(display_width("\u0085") == 6);
    assert(display_width("😀") == 2);
    assert(display_width("a\u200bb") == 2);
    assert(display_width("one\ntwo three") == 9);
    assert(display_width("a\xffb") == 2); // bytes that are not UTF-8 are dropped
    assert(display_width("a\xe6\x97b") == 2);
}

test_alignment :: () {
    for u32.[20, 21, 23, 26, 28, 29, 700, 701, 790, 1700, 5069] assert(column_alignment(it) == #char "r");
    for u32.[16, 17, 18, 19, 25, 114, 1042, 1043, 1082, 1114, 1184, 2950, 3802] assert(column_alignment(it) == #char "l");
}

test_layouts :: () {
    {
        options: Print_Options;
        options.border = 0;
        check_query("aligned people border 0", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "id  name   score  \n--- ----- -------\n  1 alice    10.5\n 22 bob          \n333            -7\n  4       1000.25\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.null_display = "<null>";
        check_query("aligned people null border 0", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "id   name   score  \n--- ------ -------\n  1 alice     10.5\n 22 bob     <null>\n333 <null>      -7\n  4        1000.25\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        check_query("aligned empty border 0", string.["id", "name", "score"], u32.[23, 25, 1700], string.[], bool.[], 0, options, "id name score \n-- ---- -----\n(0 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        check_query("aligned one row border 0", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5"], bool.[false, false, false], 1, options, "id name  score \n-- ----- -----\n 1 alice  10.5\n(1 row)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.tuples_only = true;
        check_query("aligned tuples only border 0", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "  1 alice    10.5\n 22 bob          \n333            -7\n  4       1000.25\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.footer = false;
        check_query("aligned footer off border 0", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "id  name   score  \n--- ----- -------\n  1 alice    10.5\n 22 bob          \n333            -7\n  4       1000.25\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.title = "People";
        check_query("aligned title border 0", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "      People\nid  name   score  \n--- ----- -------\n  1 alice    10.5\n 22 bob          \n333            -7\n  4       1000.25\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.title = "A title much longer than the table";
        check_query("aligned long title border 0", string.["a"], u32.[25], string.["x"], bool.[false], 1, options, "A title much longer than the table\na \n-\nx\n(1 row)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.title = "People";
        options.tuples_only = true;
        check_query("aligned title tuples only border 0", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "  1 alice    10.5\n 22 bob          \n333            -7\n  4       1000.25\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        check_query("aligned zero columns border 0", string.[], u32.[], string.[], bool.[], 3, options, "\n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        check_query("aligned zero columns one row border 0", string.[], u32.[], string.[], bool.[], 1, options, "\n(1 row)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        check_query("aligned zero columns none border 0", string.[], u32.[], string.[], bool.[], 0, options, "\n(0 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.tuples_only = true;
        check_query("aligned zero columns tuples only border 0", string.[], u32.[], string.[], bool.[], 2, options, "\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        check_query("aligned single column left border 0", string.["t"], u32.[25], string.["a", "bbb", "cc"], bool.[false, false, false], 3, options, " t  \n---\na\nbbb\ncc\n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        check_query("aligned single column right border 0", string.["n"], u32.[23], string.["1", "222", "33"], bool.[false, false, false], 3, options, " n  \n---\n  1\n222\n 33\n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        check_query("aligned wide header border 0", string.["a_very_long_column_name", "b"], u32.[23, 25], string.["1", "x", "22", "yy"], bool.[false, false, false, false], 2, options, "a_very_long_column_name b  \n----------------------- --\n                      1 x\n                     22 yy\n(2 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        check_query("aligned empty strings border 0", string.["a", "b"], u32.[25, 25], string.["", "", "", "x"], bool.[false, false, false, false], 2, options, "a b \n- -\n  \n  x\n(2 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        check_query("aligned numbers border 0", string.["i2", "i8", "f", "n", "b", "t"], u32.[21, 20, 701, 1700, 16, 25], string.["1", "-9223372036854775808", "1.5", "0.0000", "t", "x", "-32768", "7", "-0.25", "123456789.123456789", "f", "y"], bool.[false, false, false, false, false, false, false, false, false, false, false, false], 2, options, "  i2            i8            f            n          b t \n------ -------------------- ----- ------------------- - -\n     1 -9223372036854775808   1.5              0.0000 t x\n-32768                    7 -0.25 123456789.123456789 f y\n(2 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        check_query("aligned people border 1", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, " id  | name  |  score  \n-----+-------+---------\n   1 | alice |    10.5\n  22 | bob   |        \n 333 |       |      -7\n   4 |       | 1000.25\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.null_display = "<null>";
        check_query("aligned people null border 1", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, " id  |  name  |  score  \n-----+--------+---------\n   1 | alice  |    10.5\n  22 | bob    |  <null>\n 333 | <null> |      -7\n   4 |        | 1000.25\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        check_query("aligned empty border 1", string.["id", "name", "score"], u32.[23, 25, 1700], string.[], bool.[], 0, options, " id | name | score \n----+------+-------\n(0 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        check_query("aligned one row border 1", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5"], bool.[false, false, false], 1, options, " id | name  | score \n----+-------+-------\n  1 | alice |  10.5\n(1 row)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.tuples_only = true;
        check_query("aligned tuples only border 1", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "   1 | alice |    10.5\n  22 | bob   |        \n 333 |       |      -7\n   4 |       | 1000.25\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.footer = false;
        check_query("aligned footer off border 1", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, " id  | name  |  score  \n-----+-------+---------\n   1 | alice |    10.5\n  22 | bob   |        \n 333 |       |      -7\n   4 |       | 1000.25\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.title = "People";
        check_query("aligned title border 1", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "        People\n id  | name  |  score  \n-----+-------+---------\n   1 | alice |    10.5\n  22 | bob   |        \n 333 |       |      -7\n   4 |       | 1000.25\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.title = "A title much longer than the table";
        check_query("aligned long title border 1", string.["a"], u32.[25], string.["x"], bool.[false], 1, options, "A title much longer than the table\n a \n---\n x\n(1 row)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.title = "People";
        options.tuples_only = true;
        check_query("aligned title tuples only border 1", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "   1 | alice |    10.5\n  22 | bob   |        \n 333 |       |      -7\n   4 |       | 1000.25\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        check_query("aligned zero columns border 1", string.[], u32.[], string.[], bool.[], 3, options, "--\n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        check_query("aligned zero columns one row border 1", string.[], u32.[], string.[], bool.[], 1, options, "--\n(1 row)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        check_query("aligned zero columns none border 1", string.[], u32.[], string.[], bool.[], 0, options, "--\n(0 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.tuples_only = true;
        check_query("aligned zero columns tuples only border 1", string.[], u32.[], string.[], bool.[], 2, options, "\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        check_query("aligned single column left border 1", string.["t"], u32.[25], string.["a", "bbb", "cc"], bool.[false, false, false], 3, options, "  t  \n-----\n a\n bbb\n cc\n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        check_query("aligned single column right border 1", string.["n"], u32.[23], string.["1", "222", "33"], bool.[false, false, false], 3, options, "  n  \n-----\n   1\n 222\n  33\n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        check_query("aligned wide header border 1", string.["a_very_long_column_name", "b"], u32.[23, 25], string.["1", "x", "22", "yy"], bool.[false, false, false, false], 2, options, " a_very_long_column_name | b  \n-------------------------+----\n                       1 | x\n                      22 | yy\n(2 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        check_query("aligned empty strings border 1", string.["a", "b"], u32.[25, 25], string.["", "", "", "x"], bool.[false, false, false, false], 2, options, " a | b \n---+---\n   | \n   | x\n(2 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        check_query("aligned numbers border 1", string.["i2", "i8", "f", "n", "b", "t"], u32.[21, 20, 701, 1700, 16, 25], string.["1", "-9223372036854775808", "1.5", "0.0000", "t", "x", "-32768", "7", "-0.25", "123456789.123456789", "f", "y"], bool.[false, false, false, false, false, false, false, false, false, false, false, false], 2, options, "   i2   |          i8          |   f   |          n          | b | t \n--------+----------------------+-------+---------------------+---+---\n      1 | -9223372036854775808 |   1.5 |              0.0000 | t | x\n -32768 |                    7 | -0.25 | 123456789.123456789 | f | y\n(2 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        check_query("aligned people border 2", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "+-----+-------+---------+\n| id  | name  |  score  |\n+-----+-------+---------+\n|   1 | alice |    10.5 |\n|  22 | bob   |         |\n| 333 |       |      -7 |\n|   4 |       | 1000.25 |\n+-----+-------+---------+\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.null_display = "<null>";
        check_query("aligned people null border 2", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "+-----+--------+---------+\n| id  |  name  |  score  |\n+-----+--------+---------+\n|   1 | alice  |    10.5 |\n|  22 | bob    |  <null> |\n| 333 | <null> |      -7 |\n|   4 |        | 1000.25 |\n+-----+--------+---------+\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        check_query("aligned empty border 2", string.["id", "name", "score"], u32.[23, 25, 1700], string.[], bool.[], 0, options, "+----+------+-------+\n| id | name | score |\n+----+------+-------+\n+----+------+-------+\n(0 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        check_query("aligned one row border 2", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5"], bool.[false, false, false], 1, options, "+----+-------+-------+\n| id | name  | score |\n+----+-------+-------+\n|  1 | alice |  10.5 |\n+----+-------+-------+\n(1 row)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.tuples_only = true;
        check_query("aligned tuples only border 2", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "|   1 | alice |    10.5 |\n|  22 | bob   |         |\n| 333 |       |      -7 |\n|   4 |       | 1000.25 |\n+-----+-------+---------+\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.footer = false;
        check_query("aligned footer off border 2", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "+-----+-------+---------+\n| id  | name  |  score  |\n+-----+-------+---------+\n|   1 | alice |    10.5 |\n|  22 | bob   |         |\n| 333 |       |      -7 |\n|   4 |       | 1000.25 |\n+-----+-------+---------+\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.title = "People";
        check_query("aligned title border 2", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "         People\n+-----+-------+---------+\n| id  | name  |  score  |\n+-----+-------+---------+\n|   1 | alice |    10.5 |\n|  22 | bob   |         |\n| 333 |       |      -7 |\n|   4 |       | 1000.25 |\n+-----+-------+---------+\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.title = "A title much longer than the table";
        check_query("aligned long title border 2", string.["a"], u32.[25], string.["x"], bool.[false], 1, options, "A title much longer than the table\n+---+\n| a |\n+---+\n| x |\n+---+\n(1 row)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.title = "People";
        options.tuples_only = true;
        check_query("aligned title tuples only border 2", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "|   1 | alice |    10.5 |\n|  22 | bob   |         |\n| 333 |       |      -7 |\n|   4 |       | 1000.25 |\n+-----+-------+---------+\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        check_query("aligned zero columns border 2", string.[], u32.[], string.[], bool.[], 3, options, "+--+\n+--+\n+--+\n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        check_query("aligned zero columns one row border 2", string.[], u32.[], string.[], bool.[], 1, options, "+--+\n+--+\n+--+\n(1 row)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        check_query("aligned zero columns none border 2", string.[], u32.[], string.[], bool.[], 0, options, "+--+\n+--+\n+--+\n(0 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.tuples_only = true;
        check_query("aligned zero columns tuples only border 2", string.[], u32.[], string.[], bool.[], 2, options, "+--+\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        check_query("aligned single column left border 2", string.["t"], u32.[25], string.["a", "bbb", "cc"], bool.[false, false, false], 3, options, "+-----+\n|  t  |\n+-----+\n| a   |\n| bbb |\n| cc  |\n+-----+\n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        check_query("aligned single column right border 2", string.["n"], u32.[23], string.["1", "222", "33"], bool.[false, false, false], 3, options, "+-----+\n|  n  |\n+-----+\n|   1 |\n| 222 |\n|  33 |\n+-----+\n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        check_query("aligned wide header border 2", string.["a_very_long_column_name", "b"], u32.[23, 25], string.["1", "x", "22", "yy"], bool.[false, false, false, false], 2, options, "+-------------------------+----+\n| a_very_long_column_name | b  |\n+-------------------------+----+\n|                       1 | x  |\n|                      22 | yy |\n+-------------------------+----+\n(2 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        check_query("aligned empty strings border 2", string.["a", "b"], u32.[25, 25], string.["", "", "", "x"], bool.[false, false, false, false], 2, options, "+---+---+\n| a | b |\n+---+---+\n|   |   |\n|   | x |\n+---+---+\n(2 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        check_query("aligned numbers border 2", string.["i2", "i8", "f", "n", "b", "t"], u32.[21, 20, 701, 1700, 16, 25], string.["1", "-9223372036854775808", "1.5", "0.0000", "t", "x", "-32768", "7", "-0.25", "123456789.123456789", "f", "y"], bool.[false, false, false, false, false, false, false, false, false, false, false, false], 2, options, "+--------+----------------------+-------+---------------------+---+---+\n|   i2   |          i8          |   f   |          n          | b | t |\n+--------+----------------------+-------+---------------------+---+---+\n|      1 | -9223372036854775808 |   1.5 |              0.0000 | t | x |\n| -32768 |                    7 | -0.25 | 123456789.123456789 | f | y |\n+--------+----------------------+-------+---------------------+---+---+\n(2 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        check_query("aligned multiline border 0", string.["id", "body", "n"], u32.[23, 25, 1700], string.["1", "line one\nline two\nthree", "5", "2", "single", "6.5", "3", "ends\n", "7", "4", "\nstarts", "8", "5", "a\n\nb", "9"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 5, options, "id   body    n  \n-- -------- ---\n 1 line one+  5\n   line two+\n   three    \n 2 single   6.5\n 3 ends    +  7\n            \n 4         +  8\n   starts   \n 5 a       +  9\n           +\n   b        \n(5 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.linestyle = .UNICODE;
        check_query("aligned multiline unicode border 0", string.["id", "body", "n"], u32.[23, 25, 1700], string.["1", "line one\nline two\nthree", "5", "2", "single", "6.5", "3", "ends\n", "7", "4", "\nstarts", "8", "5", "a\n\nb", "9"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 5, options, "id   body    n  \n── ──────── ───\n 1 line one↵  5\n   line two↵\n   three    \n 2 single   6.5\n 3 ends    ↵  7\n            \n 4         ↵  8\n   starts   \n 5 a       ↵  9\n           ↵\n   b        \n(5 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.linestyle = .OLD_ASCII;
        check_query("aligned multiline old-ascii border 0", string.["id", "body", "n"], u32.[23, 25, 1700], string.["1", "line one\nline two\nthree", "5", "2", "single", "6.5", "3", "ends\n", "7", "4", "\nstarts", "8", "5", "a\n\nb", "9"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 5, options, "id   body    n \n-- -------- ---\n 1 line one   5\n   line two \n   three    \n 2 single   6.5\n 3 ends       7\n            \n 4            8\n   starts   \n 5 a          9\n            \n   b        \n(5 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        check_query("aligned multiline header border 0", string.["two\nline head", "x", "three\nline\nhead"], u32.[25, 23, 25], string.["a", "1", "bb", "c\nd", "2", "e"], bool.[false, false, false, false, false, false], 2, options, "   two   +x three+\nline head   line +\n            head  \n--------- - -----\na         1 bb\nc        +2 e\nd           \n(2 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.linestyle = .OLD_ASCII;
        check_query("aligned multiline header old-ascii border 0", string.["two\nline head", "x"], u32.[25, 23], string.["a", "1", "c\nd", "2"], bool.[false, false, false, false], 2, options, "   two    x\nline head+ \n--------- -\na         1\nc         2\nd         \n(2 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        check_query("aligned ctrl border 0", string.["t", "u"], u32.[25, 25], string.["a\tb", "x\ry", "\t", "\x01\x1f\x7f", "12345678\t9", "tab\t\ttab", "", ""], bool.[false, false, false, false, false, false, false, false], 4, options, "        t                  u          \n----------------- -------------------\na       b         x\\ry\n                  \\x01\\x1F\\x7F\n12345678        9 tab             tab\n\\u0085            \\u009F\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        check_query("aligned unicode text border 0", string.["name", "v"], u32.[25, 23], string.["日本語", "1", "café", "22", "é", "3", "😀😀", "4", "a​b", "5", "ｆｕｌｌ", "6", "한글", "7"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false], 7, options, "  name   v  \n-------- --\n日本語    1\ncafé     22\né         3\n😀😀      4\na​b        5\nｆｕｌｌ  6\n한글      7\n(7 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        check_query("aligned unicode header border 0", string.["名前", "é"], u32.[25, 23], string.["x", "1", "日本語日本語", "2"], bool.[false, false, false, false], 2, options, "    名前     é \n------------ -\nx            1\n日本語日本語 2\n(2 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        check_query("aligned multiline border 1", string.["id", "body", "n"], u32.[23, 25, 1700], string.["1", "line one\nline two\nthree", "5", "2", "single", "6.5", "3", "ends\n", "7", "4", "\nstarts", "8", "5", "a\n\nb", "9"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 5, options, " id |   body   |  n  \n----+----------+-----\n  1 | line one+|   5\n    | line two+| \n    | three    | \n  2 | single   | 6.5\n  3 | ends    +|   7\n    |          | \n  4 |         +|   8\n    | starts   | \n  5 | a       +|   9\n    |         +| \n    | b        | \n(5 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.linestyle = .UNICODE;
        check_query("aligned multiline unicode border 1", string.["id", "body", "n"], u32.[23, 25, 1700], string.["1", "line one\nline two\nthree", "5", "2", "single", "6.5", "3", "ends\n", "7", "4", "\nstarts", "8", "5", "a\n\nb", "9"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 5, options, " id │   body   │  n  \n────┼──────────┼─────\n  1 │ line one↵│   5\n    │ line two↵│ \n    │ three    │ \n  2 │ single   │ 6.5\n  3 │ ends    ↵│   7\n    │          │ \n  4 │         ↵│   8\n    │ starts   │ \n  5 │ a       ↵│   9\n    │         ↵│ \n    │ b        │ \n(5 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.linestyle = .OLD_ASCII;
        check_query("aligned multiline old-ascii border 1", string.["id", "body", "n"], u32.[23, 25, 1700], string.["1", "line one\nline two\nthree", "5", "2", "single", "6.5", "3", "ends\n", "7", "4", "\nstarts", "8", "5", "a\n\nb", "9"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 5, options, " id |   body   |  n  \n----+----------+-----\n  1 | line one |   5\n    : line two   \n    : three      \n  2 | single   | 6.5\n  3 | ends     |   7\n    :            \n  4 |          |   8\n    : starts     \n  5 | a        |   9\n    :            \n    : b          \n(5 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        check_query("aligned multiline header border 1", string.["two\nline head", "x", "three\nline\nhead"], u32.[25, 23, 25], string.["a", "1", "bb", "c\nd", "2", "e"], bool.[false, false, false, false, false, false], 2, options, "    two   +| x | three+\n line head |   | line +\n           |   | head  \n-----------+---+-------\n a         | 1 | bb\n c        +| 2 | e\n d         |   | \n(2 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.linestyle = .OLD_ASCII;
        check_query("aligned multiline header old-ascii border 1", string.["two\nline head", "x"], u32.[25, 23], string.["a", "1", "c\nd", "2"], bool.[false, false, false, false], 2, options, "    two    | x \n+line head |+  \n-----------+---\n a         | 1\n c         | 2\n d           \n(2 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        check_query("aligned ctrl border 1", string.["t", "u"], u32.[25, 25], string.["a\tb", "x\ry", "\t", "\x01\x1f\x7f", "12345678\t9", "tab\t\ttab", "", ""], bool.[false, false, false, false, false, false, false, false], 4, options, "         t         |          u          \n-------------------+---------------------\n a       b         | x\\ry\n                   | \\x01\\x1F\\x7F\n 12345678        9 | tab             tab\n \\u0085            | \\u009F\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        check_query("aligned unicode text border 1", string.["name", "v"], u32.[25, 23], string.["日本語", "1", "café", "22", "é", "3", "😀😀", "4", "a​b", "5", "ｆｕｌｌ", "6", "한글", "7"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false], 7, options, "   name   | v  \n----------+----\n 日本語   |  1\n café     | 22\n é        |  3\n 😀😀     |  4\n a​b       |  5\n ｆｕｌｌ |  6\n 한글     |  7\n(7 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        check_query("aligned unicode header border 1", string.["名前", "é"], u32.[25, 23], string.["x", "1", "日本語日本語", "2"], bool.[false, false, false, false], 2, options, "     名前     | é \n--------------+---\n x            | 1\n 日本語日本語 | 2\n(2 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        check_query("aligned multiline border 2", string.["id", "body", "n"], u32.[23, 25, 1700], string.["1", "line one\nline two\nthree", "5", "2", "single", "6.5", "3", "ends\n", "7", "4", "\nstarts", "8", "5", "a\n\nb", "9"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 5, options, "+----+----------+-----+\n| id |   body   |  n  |\n+----+----------+-----+\n|  1 | line one+|   5 |\n|    | line two+|     |\n|    | three    |     |\n|  2 | single   | 6.5 |\n|  3 | ends    +|   7 |\n|    |          |     |\n|  4 |         +|   8 |\n|    | starts   |     |\n|  5 | a       +|   9 |\n|    |         +|     |\n|    | b        |     |\n+----+----------+-----+\n(5 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.linestyle = .UNICODE;
        check_query("aligned multiline unicode border 2", string.["id", "body", "n"], u32.[23, 25, 1700], string.["1", "line one\nline two\nthree", "5", "2", "single", "6.5", "3", "ends\n", "7", "4", "\nstarts", "8", "5", "a\n\nb", "9"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 5, options, "┌────┬──────────┬─────┐\n│ id │   body   │  n  │\n├────┼──────────┼─────┤\n│  1 │ line one↵│   5 │\n│    │ line two↵│     │\n│    │ three    │     │\n│  2 │ single   │ 6.5 │\n│  3 │ ends    ↵│   7 │\n│    │          │     │\n│  4 │         ↵│   8 │\n│    │ starts   │     │\n│  5 │ a       ↵│   9 │\n│    │         ↵│     │\n│    │ b        │     │\n└────┴──────────┴─────┘\n(5 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.linestyle = .OLD_ASCII;
        check_query("aligned multiline old-ascii border 2", string.["id", "body", "n"], u32.[23, 25, 1700], string.["1", "line one\nline two\nthree", "5", "2", "single", "6.5", "3", "ends\n", "7", "4", "\nstarts", "8", "5", "a\n\nb", "9"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 5, options, "+----+----------+-----+\n| id |   body   |  n  |\n+----+----------+-----+\n|  1 | line one |   5 |\n|    : line two       |\n|    : three          |\n|  2 | single   | 6.5 |\n|  3 | ends     |   7 |\n|    :                |\n|  4 |          |   8 |\n|    : starts         |\n|  5 | a        |   9 |\n|    :                |\n|    : b              |\n+----+----------+-----+\n(5 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        check_query("aligned multiline header border 2", string.["two\nline head", "x", "three\nline\nhead"], u32.[25, 23, 25], string.["a", "1", "bb", "c\nd", "2", "e"], bool.[false, false, false, false, false, false], 2, options, "+-----------+---+-------+\n|    two   +| x | three+|\n| line head |   | line +|\n|           |   | head  |\n+-----------+---+-------+\n| a         | 1 | bb    |\n| c        +| 2 | e     |\n| d         |   |       |\n+-----------+---+-------+\n(2 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.linestyle = .OLD_ASCII;
        check_query("aligned multiline header old-ascii border 2", string.["two\nline head", "x"], u32.[25, 23], string.["a", "1", "c\nd", "2"], bool.[false, false, false, false], 2, options, "+-----------+---+\n|    two    | x |\n|+line head |+  |\n+-----------+---+\n| a         | 1 |\n| c         | 2 |\n| d             |\n+-----------+---+\n(2 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        check_query("aligned ctrl border 2", string.["t", "u"], u32.[25, 25], string.["a\tb", "x\ry", "\t", "\x01\x1f\x7f", "12345678\t9", "tab\t\ttab", "", ""], bool.[false, false, false, false, false, false, false, false], 4, options, "+-------------------+---------------------+\n|         t         |          u          |\n+-------------------+---------------------+\n| a       b         | x\\ry                |\n|                   | \\x01\\x1F\\x7F        |\n| 12345678        9 | tab             tab |\n| \\u0085            | \\u009F              |\n+-------------------+---------------------+\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        check_query("aligned unicode text border 2", string.["name", "v"], u32.[25, 23], string.["日本語", "1", "café", "22", "é", "3", "😀😀", "4", "a​b", "5", "ｆｕｌｌ", "6", "한글", "7"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false], 7, options, "+----------+----+\n|   name   | v  |\n+----------+----+\n| 日本語   |  1 |\n| café     | 22 |\n| é        |  3 |\n| 😀😀     |  4 |\n| a​b       |  5 |\n| ｆｕｌｌ |  6 |\n| 한글     |  7 |\n+----------+----+\n(7 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        check_query("aligned unicode header border 2", string.["名前", "é"], u32.[25, 23], string.["x", "1", "日本語日本語", "2"], bool.[false, false, false, false], 2, options, "+--------------+---+\n|     名前     | é |\n+--------------+---+\n| x            | 1 |\n| 日本語日本語 | 2 |\n+--------------+---+\n(2 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.linestyle = .UNICODE;
        check_query("aligned people unicode border 0", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "id  name   score  \n─── ───── ───────\n  1 alice    10.5\n 22 bob          \n333            -7\n  4       1000.25\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.linestyle = .UNICODE;
        check_query("aligned empty unicode border 0", string.["id", "name", "score"], u32.[23, 25, 1700], string.[], bool.[], 0, options, "id name score \n── ──── ─────\n(0 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.linestyle = .UNICODE;
        options.title = "T";
        check_query("aligned title unicode border 0", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "        T\nid  name   score  \n─── ───── ───────\n  1 alice    10.5\n 22 bob          \n333            -7\n  4       1000.25\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.linestyle = .UNICODE;
        options.expanded = .ON;
        check_query("expanded people unicode border 0", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "* Record 1  \nid    1\nname  alice\nscore 10.5\n* Record 2  \nid    22\nname  bob\nscore \n* Record 3  \nid    333\nname  \nscore -7\n* Record 4  \nid    4\nname  \nscore 1000.25\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.linestyle = .UNICODE;
        options.expanded = .ON;
        check_query("expanded multiline unicode border 0", string.["id", "body", "n"], u32.[23, 25, 1700], string.["1", "line one\nline two\nthree", "5", "2", "single", "6.5", "3", "ends\n", "7", "4", "\nstarts", "8", "5", "a\n\nb", "9"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 5, options, "* Record 1  \nid   1\nbody line one↵\n     line two↵\n     three\nn    5\n* Record 2  \nid   2\nbody single\nn    6.5\n* Record 3  \nid   3\nbody ends    ↵\n     \nn    7\n* Record 4  \nid   4\nbody         ↵\n     starts\nn    8\n* Record 5  \nid   5\nbody a       ↵\n             ↵\n     b\nn    9\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.linestyle = .UNICODE;
        check_query("aligned people unicode border 1", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, " id  │ name  │  score  \n─────┼───────┼─────────\n   1 │ alice │    10.5\n  22 │ bob   │        \n 333 │       │      -7\n   4 │       │ 1000.25\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.linestyle = .UNICODE;
        check_query("aligned empty unicode border 1", string.["id", "name", "score"], u32.[23, 25, 1700], string.[], bool.[], 0, options, " id │ name │ score \n────┼──────┼───────\n(0 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.linestyle = .UNICODE;
        options.title = "T";
        check_query("aligned title unicode border 1", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "           T\n id  │ name  │  score  \n─────┼───────┼─────────\n   1 │ alice │    10.5\n  22 │ bob   │        \n 333 │       │      -7\n   4 │       │ 1000.25\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.linestyle = .UNICODE;
        options.expanded = .ON;
        check_query("expanded people unicode border 1", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "─[ RECORD 1 ]──\nid    │ 1\nname  │ alice\nscore │ 10.5\n─[ RECORD 2 ]──\nid    │ 22\nname  │ bob\nscore │ \n─[ RECORD 3 ]──\nid    │ 333\nname  │ \nscore │ -7\n─[ RECORD 4 ]──\nid    │ 4\nname  │ \nscore │ 1000.25\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.linestyle = .UNICODE;
        options.expanded = .ON;
        check_query("expanded multiline unicode border 1", string.["id", "body", "n"], u32.[23, 25, 1700], string.["1", "line one\nline two\nthree", "5", "2", "single", "6.5", "3", "ends\n", "7", "4", "\nstarts", "8", "5", "a\n\nb", "9"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 5, options, "─[ RECORD 1 ]──\nid   │ 1\nbody │ line one↵\n     │ line two↵\n     │ three\nn    │ 5\n─[ RECORD 2 ]──\nid   │ 2\nbody │ single\nn    │ 6.5\n─[ RECORD 3 ]──\nid   │ 3\nbody │ ends    ↵\n     │ \nn    │ 7\n─[ RECORD 4 ]──\nid   │ 4\nbody │         ↵\n     │ starts\nn    │ 8\n─[ RECORD 5 ]──\nid   │ 5\nbody │ a       ↵\n     │         ↵\n     │ b\nn    │ 9\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.linestyle = .UNICODE;
        check_query("aligned people unicode border 2", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "┌─────┬───────┬─────────┐\n│ id  │ name  │  score  │\n├─────┼───────┼─────────┤\n│   1 │ alice │    10.5 │\n│  22 │ bob   │         │\n│ 333 │       │      -7 │\n│   4 │       │ 1000.25 │\n└─────┴───────┴─────────┘\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.linestyle = .UNICODE;
        check_query("aligned empty unicode border 2", string.["id", "name", "score"], u32.[23, 25, 1700], string.[], bool.[], 0, options, "┌────┬──────┬───────┐\n│ id │ name │ score │\n├────┼──────┼───────┤\n└────┴──────┴───────┘\n(0 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.linestyle = .UNICODE;
        options.title = "T";
        check_query("aligned title unicode border 2", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "            T\n┌─────┬───────┬─────────┐\n│ id  │ name  │  score  │\n├─────┼───────┼─────────┤\n│   1 │ alice │    10.5 │\n│  22 │ bob   │         │\n│ 333 │       │      -7 │\n│   4 │       │ 1000.25 │\n└─────┴───────┴─────────┘\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.linestyle = .UNICODE;
        options.expanded = .ON;
        check_query("expanded people unicode border 2", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "┌─[ RECORD 1 ]────┐\n│ id    │ 1       │\n│ name  │ alice   │\n│ score │ 10.5    │\n├─[ RECORD 2 ]────┤\n│ id    │ 22      │\n│ name  │ bob     │\n│ score │         │\n├─[ RECORD 3 ]────┤\n│ id    │ 333     │\n│ name  │         │\n│ score │ -7      │\n├─[ RECORD 4 ]────┤\n│ id    │ 4       │\n│ name  │         │\n│ score │ 1000.25 │\n└───────┴─────────┘\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.linestyle = .UNICODE;
        options.expanded = .ON;
        check_query("expanded multiline unicode border 2", string.["id", "body", "n"], u32.[23, 25, 1700], string.["1", "line one\nline two\nthree", "5", "2", "single", "6.5", "3", "ends\n", "7", "4", "\nstarts", "8", "5", "a\n\nb", "9"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 5, options, "┌─[ RECORD 1 ]────┐\n│ id   │ 1        │\n│ body │ line one↵│\n│      │ line two↵│\n│      │ three    │\n│ n    │ 5        │\n├─[ RECORD 2 ]────┤\n│ id   │ 2        │\n│ body │ single   │\n│ n    │ 6.5      │\n├─[ RECORD 3 ]────┤\n│ id   │ 3        │\n│ body │ ends    ↵│\n│      │          │\n│ n    │ 7        │\n├─[ RECORD 4 ]────┤\n│ id   │ 4        │\n│ body │         ↵│\n│      │ starts   │\n│ n    │ 8        │\n├─[ RECORD 5 ]────┤\n│ id   │ 5        │\n│ body │ a       ↵│\n│      │         ↵│\n│      │ b        │\n│ n    │ 9        │\n└──────┴──────────┘\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.linestyle = .OLD_ASCII;
        check_query("aligned people old-ascii border 0", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "id  name   score \n--- ----- -------\n  1 alice    10.5\n 22 bob          \n333            -7\n  4       1000.25\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.linestyle = .OLD_ASCII;
        check_query("aligned empty old-ascii border 0", string.["id", "name", "score"], u32.[23, 25, 1700], string.[], bool.[], 0, options, "id name score\n-- ---- -----\n(0 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.linestyle = .OLD_ASCII;
        options.title = "T";
        check_query("aligned title old-ascii border 0", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "        T\nid  name   score \n--- ----- -------\n  1 alice    10.5\n 22 bob          \n333            -7\n  4       1000.25\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.linestyle = .OLD_ASCII;
        options.expanded = .ON;
        check_query("expanded people old-ascii border 0", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "* Record 1  \nid    1\nname  alice\nscore 10.5\n* Record 2  \nid    22\nname  bob\nscore \n* Record 3  \nid    333\nname  \nscore -7\n* Record 4  \nid    4\nname  \nscore 1000.25\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.linestyle = .OLD_ASCII;
        options.expanded = .ON;
        check_query("expanded multiline old-ascii border 0", string.["id", "body", "n"], u32.[23, 25, 1700], string.["1", "line one\nline two\nthree", "5", "2", "single", "6.5", "3", "ends\n", "7", "4", "\nstarts", "8", "5", "a\n\nb", "9"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 5, options, "* Record 1  \nid   1\nbody line one\n     line two\n     three\nn    5\n* Record 2  \nid   2\nbody single\nn    6.5\n* Record 3  \nid   3\nbody ends\n     \nn    7\n* Record 4  \nid   4\nbody \n     starts\nn    8\n* Record 5  \nid   5\nbody a\n     \n     b\nn    9\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.linestyle = .OLD_ASCII;
        check_query("aligned people old-ascii border 1", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, " id  | name  |  score  \n-----+-------+---------\n   1 | alice |    10.5\n  22 | bob   |        \n 333 |       |      -7\n   4 |       | 1000.25\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.linestyle = .OLD_ASCII;
        check_query("aligned empty old-ascii border 1", string.["id", "name", "score"], u32.[23, 25, 1700], string.[], bool.[], 0, options, " id | name | score \n----+------+-------\n(0 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.linestyle = .OLD_ASCII;
        options.title = "T";
        check_query("aligned title old-ascii border 1", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "           T\n id  | name  |  score  \n-----+-------+---------\n   1 | alice |    10.5\n  22 | bob   |        \n 333 |       |      -7\n   4 |       | 1000.25\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.linestyle = .OLD_ASCII;
        options.expanded = .ON;
        check_query("expanded people old-ascii border 1", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "-[ RECORD 1 ]--\nid    | 1\nname  | alice\nscore | 10.5\n-[ RECORD 2 ]--\nid    | 22\nname  | bob\nscore | \n-[ RECORD 3 ]--\nid    | 333\nname  | \nscore | -7\n-[ RECORD 4 ]--\nid    | 4\nname  | \nscore | 1000.25\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.linestyle = .OLD_ASCII;
        options.expanded = .ON;
        check_query("expanded multiline old-ascii border 1", string.["id", "body", "n"], u32.[23, 25, 1700], string.["1", "line one\nline two\nthree", "5", "2", "single", "6.5", "3", "ends\n", "7", "4", "\nstarts", "8", "5", "a\n\nb", "9"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 5, options, "-[ RECORD 1 ]--\nid   | 1\nbody | line one\n     : line two\n     : three\nn    | 5\n-[ RECORD 2 ]--\nid   | 2\nbody | single\nn    | 6.5\n-[ RECORD 3 ]--\nid   | 3\nbody | ends\n     : \nn    | 7\n-[ RECORD 4 ]--\nid   | 4\nbody | \n     : starts\nn    | 8\n-[ RECORD 5 ]--\nid   | 5\nbody | a\n     : \n     : b\nn    | 9\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.linestyle = .OLD_ASCII;
        check_query("aligned people old-ascii border 2", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "+-----+-------+---------+\n| id  | name  |  score  |\n+-----+-------+---------+\n|   1 | alice |    10.5 |\n|  22 | bob   |         |\n| 333 |       |      -7 |\n|   4 |       | 1000.25 |\n+-----+-------+---------+\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.linestyle = .OLD_ASCII;
        check_query("aligned empty old-ascii border 2", string.["id", "name", "score"], u32.[23, 25, 1700], string.[], bool.[], 0, options, "+----+------+-------+\n| id | name | score |\n+----+------+-------+\n+----+------+-------+\n(0 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.linestyle = .OLD_ASCII;
        options.title = "T";
        check_query("aligned title old-ascii border 2", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "            T\n+-----+-------+---------+\n| id  | name  |  score  |\n+-----+-------+---------+\n|   1 | alice |    10.5 |\n|  22 | bob   |         |\n| 333 |       |      -7 |\n|   4 |       | 1000.25 |\n+-----+-------+---------+\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.linestyle = .OLD_ASCII;
        options.expanded = .ON;
        check_query("expanded people old-ascii border 2", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "+-[ RECORD 1 ]----+\n| id    | 1       |\n| name  | alice   |\n| score | 10.5    |\n+-[ RECORD 2 ]----+\n| id    | 22      |\n| name  | bob     |\n| score |         |\n+-[ RECORD 3 ]----+\n| id    | 333     |\n| name  |         |\n| score | -7      |\n+-[ RECORD 4 ]----+\n| id    | 4       |\n| name  |         |\n| score | 1000.25 |\n+-------+---------+\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.linestyle = .OLD_ASCII;
        options.expanded = .ON;
        check_query("expanded multiline old-ascii border 2", string.["id", "body", "n"], u32.[23, 25, 1700], string.["1", "line one\nline two\nthree", "5", "2", "single", "6.5", "3", "ends\n", "7", "4", "\nstarts", "8", "5", "a\n\nb", "9"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 5, options, "+-[ RECORD 1 ]----+\n| id   | 1        |\n| body | line one |\n|      : line two |\n|      : three    |\n| n    | 5        |\n+-[ RECORD 2 ]----+\n| id   | 2        |\n| body | single   |\n| n    | 6.5      |\n+-[ RECORD 3 ]----+\n| id   | 3        |\n| body | ends     |\n|      :          |\n| n    | 7        |\n+-[ RECORD 4 ]----+\n| id   | 4        |\n| body |          |\n|      : starts   |\n| n    | 8        |\n+-[ RECORD 5 ]----+\n| id   | 5        |\n| body | a        |\n|      :          |\n|      : b        |\n| n    | 9        |\n+------+----------+\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.linestyle = .ASCII;
        check_query("aligned people ascii border 0", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "id  name   score  \n--- ----- -------\n  1 alice    10.5\n 22 bob          \n333            -7\n  4       1000.25\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.linestyle = .ASCII;
        check_query("aligned empty ascii border 0", string.["id", "name", "score"], u32.[23, 25, 1700], string.[], bool.[], 0, options, "id name score \n-- ---- -----\n(0 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.linestyle = .ASCII;
        options.title = "T";
        check_query("aligned title ascii border 0", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "        T\nid  name   score  \n--- ----- -------\n  1 alice    10.5\n 22 bob          \n333            -7\n  4       1000.25\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.linestyle = .ASCII;
        options.expanded = .ON;
        check_query("expanded people ascii border 0", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "* Record 1  \nid    1\nname  alice\nscore 10.5\n* Record 2  \nid    22\nname  bob\nscore \n* Record 3  \nid    333\nname  \nscore -7\n* Record 4  \nid    4\nname  \nscore 1000.25\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.linestyle = .ASCII;
        options.expanded = .ON;
        check_query("expanded multiline ascii border 0", string.["id", "body", "n"], u32.[23, 25, 1700], string.["1", "line one\nline two\nthree", "5", "2", "single", "6.5", "3", "ends\n", "7", "4", "\nstarts", "8", "5", "a\n\nb", "9"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 5, options, "* Record 1  \nid   1\nbody line one+\n     line two+\n     three\nn    5\n* Record 2  \nid   2\nbody single\nn    6.5\n* Record 3  \nid   3\nbody ends    +\n     \nn    7\n* Record 4  \nid   4\nbody         +\n     starts\nn    8\n* Record 5  \nid   5\nbody a       +\n             +\n     b\nn    9\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.linestyle = .ASCII;
        check_query("aligned people ascii border 1", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, " id  | name  |  score  \n-----+-------+---------\n   1 | alice |    10.5\n  22 | bob   |        \n 333 |       |      -7\n   4 |       | 1000.25\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.linestyle = .ASCII;
        check_query("aligned empty ascii border 1", string.["id", "name", "score"], u32.[23, 25, 1700], string.[], bool.[], 0, options, " id | name | score \n----+------+-------\n(0 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.linestyle = .ASCII;
        options.title = "T";
        check_query("aligned title ascii border 1", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "           T\n id  | name  |  score  \n-----+-------+---------\n   1 | alice |    10.5\n  22 | bob   |        \n 333 |       |      -7\n   4 |       | 1000.25\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.linestyle = .ASCII;
        options.expanded = .ON;
        check_query("expanded people ascii border 1", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "-[ RECORD 1 ]--\nid    | 1\nname  | alice\nscore | 10.5\n-[ RECORD 2 ]--\nid    | 22\nname  | bob\nscore | \n-[ RECORD 3 ]--\nid    | 333\nname  | \nscore | -7\n-[ RECORD 4 ]--\nid    | 4\nname  | \nscore | 1000.25\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.linestyle = .ASCII;
        options.expanded = .ON;
        check_query("expanded multiline ascii border 1", string.["id", "body", "n"], u32.[23, 25, 1700], string.["1", "line one\nline two\nthree", "5", "2", "single", "6.5", "3", "ends\n", "7", "4", "\nstarts", "8", "5", "a\n\nb", "9"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 5, options, "-[ RECORD 1 ]--\nid   | 1\nbody | line one+\n     | line two+\n     | three\nn    | 5\n-[ RECORD 2 ]--\nid   | 2\nbody | single\nn    | 6.5\n-[ RECORD 3 ]--\nid   | 3\nbody | ends    +\n     | \nn    | 7\n-[ RECORD 4 ]--\nid   | 4\nbody |         +\n     | starts\nn    | 8\n-[ RECORD 5 ]--\nid   | 5\nbody | a       +\n     |         +\n     | b\nn    | 9\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.linestyle = .ASCII;
        check_query("aligned people ascii border 2", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "+-----+-------+---------+\n| id  | name  |  score  |\n+-----+-------+---------+\n|   1 | alice |    10.5 |\n|  22 | bob   |         |\n| 333 |       |      -7 |\n|   4 |       | 1000.25 |\n+-----+-------+---------+\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.linestyle = .ASCII;
        check_query("aligned empty ascii border 2", string.["id", "name", "score"], u32.[23, 25, 1700], string.[], bool.[], 0, options, "+----+------+-------+\n| id | name | score |\n+----+------+-------+\n+----+------+-------+\n(0 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.linestyle = .ASCII;
        options.title = "T";
        check_query("aligned title ascii border 2", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "            T\n+-----+-------+---------+\n| id  | name  |  score  |\n+-----+-------+---------+\n|   1 | alice |    10.5 |\n|  22 | bob   |         |\n| 333 |       |      -7 |\n|   4 |       | 1000.25 |\n+-----+-------+---------+\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.linestyle = .ASCII;
        options.expanded = .ON;
        check_query("expanded people ascii border 2", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "+-[ RECORD 1 ]----+\n| id    | 1       |\n| name  | alice   |\n| score | 10.5    |\n+-[ RECORD 2 ]----+\n| id    | 22      |\n| name  | bob     |\n| score |         |\n+-[ RECORD 3 ]----+\n| id    | 333     |\n| name  |         |\n| score | -7      |\n+-[ RECORD 4 ]----+\n| id    | 4       |\n| name  |         |\n| score | 1000.25 |\n+-------+---------+\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.linestyle = .ASCII;
        options.expanded = .ON;
        check_query("expanded multiline ascii border 2", string.["id", "body", "n"], u32.[23, 25, 1700], string.["1", "line one\nline two\nthree", "5", "2", "single", "6.5", "3", "ends\n", "7", "4", "\nstarts", "8", "5", "a\n\nb", "9"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 5, options, "+-[ RECORD 1 ]----+\n| id   | 1        |\n| body | line one+|\n|      | line two+|\n|      | three    |\n| n    | 5        |\n+-[ RECORD 2 ]----+\n| id   | 2        |\n| body | single   |\n| n    | 6.5      |\n+-[ RECORD 3 ]----+\n| id   | 3        |\n| body | ends    +|\n|      |          |\n| n    | 7        |\n+-[ RECORD 4 ]----+\n| id   | 4        |\n| body |         +|\n|      | starts   |\n| n    | 8        |\n+-[ RECORD 5 ]----+\n| id   | 5        |\n| body | a       +|\n|      |         +|\n|      | b        |\n| n    | 9        |\n+------+----------+\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        check_query("aligned many border 0", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "row 1", "1.5", "2", "row 2", "3.0", "3", "row 3", "4.5", "4", "row 4", "6.0", "5", "row 5", "7.5", "6", "row 6", "9.0", "7", "row 7", "10.5", "8", "row 8", "12.0", "9", "row 9", "13.5", "10", "row 10", "15.0", "11", "row 11", "16.5"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 11, options, "id  name  score \n-- ------ -----\n 1 row 1    1.5\n 2 row 2    3.0\n 3 row 3    4.5\n 4 row 4    6.0\n 5 row 5    7.5\n 6 row 6    9.0\n 7 row 7   10.5\n 8 row 8   12.0\n 9 row 9   13.5\n10 row 10  15.0\n11 row 11  16.5\n(11 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.expanded = .ON;
        check_query("expanded many border 0", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "row 1", "1.5", "2", "row 2", "3.0", "3", "row 3", "4.5", "4", "row 4", "6.0", "5", "row 5", "7.5", "6", "row 6", "9.0", "7", "row 7", "10.5", "8", "row 8", "12.0", "9", "row 9", "13.5", "10", "row 10", "15.0", "11", "row 11", "16.5"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 11, options, "* Record 1 \nid    1\nname  row 1\nscore 1.5\n* Record 2 \nid    2\nname  row 2\nscore 3.0\n* Record 3 \nid    3\nname  row 3\nscore 4.5\n* Record 4 \nid    4\nname  row 4\nscore 6.0\n* Record 5 \nid    5\nname  row 5\nscore 7.5\n* Record 6 \nid    6\nname  row 6\nscore 9.0\n* Record 7 \nid    7\nname  row 7\nscore 10.5\n* Record 8 \nid    8\nname  row 8\nscore 12.0\n* Record 9 \nid    9\nname  row 9\nscore 13.5\n* Record 10\nid    10\nname  row 10\nscore 15.0\n* Record 11\nid    11\nname  row 11\nscore 16.5\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.expanded = .ON;
        check_query("expanded 100 rows border 0", string.["i", "t"], u32.[23, 25], string.["1", "r", "2", "r", "3", "r", "4", "r", "5", "r", "6", "r", "7", "r", "8", "r", "9", "r", "10", "r", "11", "r", "12", "r", "13", "r", "14", "r", "15", "r", "16", "r", "17", "r", "18", "r", "19", "r", "20", "r", "21", "r", "22", "r", "23", "r", "24", "r", "25", "r", "26", "r", "27", "r", "28", "r", "29", "r", "30", "r", "31", "r", "32", "r", "33", "r", "34", "r", "35", "r", "36", "r", "37", "r", "38", "r", "39", "r", "40", "r", "41", "r", "42", "r", "43", "r", "44", "r", "45", "r", "46", "r", "47", "r", "48", "r", "49", "r", "50", "r", "51", "r", "52", "r", "53", "r", "54", "r", "55", "r", "56", "r", "57", "r", "58", "r", "59", "r", "60", "r", "61", "r", "62", "r", "63", "r", "64", "r", "65", "r", "66", "r", "67", "r", "68", "r", "69", "r", "70", "r", "71", "r", "72", "r", "73", "r", "74", "r", "75", "r", "76", "r", "77", "r", "78", "r", "79", "r", "80", "r", "81", "r", "82", "r", "83", "r", "84", "r", "85", "r", "86", "r", "87", "r", "88", "r", "89", "r", "90", "r", "91", "r", "92", "r", "93", "r", "94", "r", "95", "r", "96", "r", "97", "r", "98", "r", "99", "r", "100", "r"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 100, options, "* Record 1 \ni 1\nt r\n* Record 2 \ni 2\nt r\n* Record 3 \ni 3\nt r\n* Record 4 \ni 4\nt r\n* Record 5 \ni 5\nt r\n* Record 6 \ni 6\nt r\n* Record 7 \ni 7\nt r\n* Record 8 \ni 8\nt r\n* Record 9 \ni 9\nt r\n* Record 10\ni 10\nt r\n* Record 11\ni 11\nt r\n* Record 12\ni 12\nt r\n* Record 13\ni 13\nt r\n* Record 14\ni 14\nt r\n* Record 15\ni 15\nt r\n* Record 16\ni 16\nt r\n* Record 17\ni 17\nt r\n* Record 18\ni 18\nt r\n* Record 19\ni 19\nt r\n* Record 20\ni 20\nt r\n* Record 21\ni 21\nt r\n* Record 22\ni 22\nt r\n* Record 23\ni 23\nt r\n* Record 24\ni 24\nt r\n* Record 25\ni 25\nt r\n* Record 26\ni 26\nt r\n* Record 27\ni 27\nt r\n* Record 28\ni 28\nt r\n* Record 29\ni 29\nt r\n* Record 30\ni 30\nt r\n* Record 31\ni 31\nt r\n* Record 32\ni 32\nt r\n* Record 33\ni 33\nt r\n* Record 34\ni 34\nt r\n* Record 35\ni 35\nt r\n* Record 36\ni 36\nt r\n* Record 37\ni 37\nt r\n* Record 38\ni 38\nt r\n* Record 39\ni 39\nt r\n* Record 40\ni 40\nt r\n* Record 41\ni 41\nt r\n* Record 42\ni 42\nt r\n* Record 43\ni 43\nt r\n* Record 44\ni 44\nt r\n* Record 45\ni 45\nt r\n* Record 46\ni 46\nt r\n* Record 47\ni 47\nt r\n* Record 48\ni 48\nt r\n* Record 49\ni 49\nt r\n* Record 50\ni 50\nt r\n* Record 51\ni 51\nt r\n* Record 52\ni 52\nt r\n* Record 53\ni 53\nt r\n* Record 54\ni 54\nt r\n* Record 55\ni 55\nt r\n* Record 56\ni 56\nt r\n* Record 57\ni 57\nt r\n* Record 58\ni 58\nt r\n* Record 59\ni 59\nt r\n* Record 60\ni 60\nt r\n* Record 61\ni 61\nt r\n* Record 62\ni 62\nt r\n* Record 63\ni 63\nt r\n* Record 64\ni 64\nt r\n* Record 65\ni 65\nt r\n* Record 66\ni 66\nt r\n* Record 67\ni 67\nt r\n* Record 68\ni 68\nt r\n* Record 69\ni 69\nt r\n* Record 70\ni 70\nt r\n* Record 71\ni 71\nt r\n* Record 72\ni 72\nt r\n* Record 73\ni 73\nt r\n* Record 74\ni 74\nt r\n* Record 75\ni 75\nt r\n* Record 76\ni 76\nt r\n* Record 77\ni 77\nt r\n* Record 78\ni 78\nt r\n* Record 79\ni 79\nt r\n* Record 80\ni 80\nt r\n* Record 81\ni 81\nt r\n* Record 82\ni 82\nt r\n* Record 83\ni 83\nt r\n* Record 84\ni 84\nt r\n* Record 85\ni 85\nt r\n* Record 86\ni 86\nt r\n* Record 87\ni 87\nt r\n* Record 88\ni 88\nt r\n* Record 89\ni 89\nt r\n* Record 90\ni 90\nt r\n* Record 91\ni 91\nt r\n* Record 92\ni 92\nt r\n* Record 93\ni 93\nt r\n* Record 94\ni 94\nt r\n* Record 95\ni 95\nt r\n* Record 96\ni 96\nt r\n* Record 97\ni 97\nt r\n* Record 98\ni 98\nt r\n* Record 99\ni 99\nt r\n* Record 100\ni 100\nt r\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.expanded = .ON;
        options.tuples_only = true;
        check_query("expanded 100 rows tuples only border 0", string.["i", "t"], u32.[23, 25], string.["1", "r", "2", "r", "3", "r"], bool.[false, false, false, false, false, false], 3, options, "i 1\nt r\n  \ni 2\nt r\n  \ni 3\nt r\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        check_query("aligned many border 1", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "row 1", "1.5", "2", "row 2", "3.0", "3", "row 3", "4.5", "4", "row 4", "6.0", "5", "row 5", "7.5", "6", "row 6", "9.0", "7", "row 7", "10.5", "8", "row 8", "12.0", "9", "row 9", "13.5", "10", "row 10", "15.0", "11", "row 11", "16.5"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 11, options, " id |  name  | score \n----+--------+-------\n  1 | row 1  |   1.5\n  2 | row 2  |   3.0\n  3 | row 3  |   4.5\n  4 | row 4  |   6.0\n  5 | row 5  |   7.5\n  6 | row 6  |   9.0\n  7 | row 7  |  10.5\n  8 | row 8  |  12.0\n  9 | row 9  |  13.5\n 10 | row 10 |  15.0\n 11 | row 11 |  16.5\n(11 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.expanded = .ON;
        check_query("expanded many border 1", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "row 1", "1.5", "2", "row 2", "3.0", "3", "row 3", "4.5", "4", "row 4", "6.0", "5", "row 5", "7.5", "6", "row 6", "9.0", "7", "row 7", "10.5", "8", "row 8", "12.0", "9", "row 9", "13.5", "10", "row 10", "15.0", "11", "row 11", "16.5"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 11, options, "-[ RECORD 1 ]-\nid    | 1\nname  | row 1\nscore | 1.5\n-[ RECORD 2 ]-\nid    | 2\nname  | row 2\nscore | 3.0\n-[ RECORD 3 ]-\nid    | 3\nname  | row 3\nscore | 4.5\n-[ RECORD 4 ]-\nid    | 4\nname  | row 4\nscore | 6.0\n-[ RECORD 5 ]-\nid    | 5\nname  | row 5\nscore | 7.5\n-[ RECORD 6 ]-\nid    | 6\nname  | row 6\nscore | 9.0\n-[ RECORD 7 ]-\nid    | 7\nname  | row 7\nscore | 10.5\n-[ RECORD 8 ]-\nid    | 8\nname  | row 8\nscore | 12.0\n-[ RECORD 9 ]-\nid    | 9\nname  | row 9\nscore | 13.5\n-[ RECORD 10 ]\nid    | 10\nname  | row 10\nscore | 15.0\n-[ RECORD 11 ]\nid    | 11\nname  | row 11\nscore | 16.5\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.expanded = .ON;
        check_query("expanded 100 rows border 1", string.["i", "t"], u32.[23, 25], string.["1", "r", "2", "r", "3", "r", "4", "r", "5", "r", "6", "r", "7", "r", "8", "r", "9", "r", "10", "r", "11", "r", "12", "r", "13", "r", "14", "r", "15", "r", "16", "r", "17", "r", "18", "r", "19", "r", "20", "r", "21", "r", "22", "r", "23", "r", "24", "r", "25", "r", "26", "r", "27", "r", "28", "r", "29", "r", "30", "r", "31", "r", "32", "r", "33", "r", "34", "r", "35", "r", "36", "r", "37", "r", "38", "r", "39", "r", "40", "r", "41", "r", "42", "r", "43", "r", "44", "r", "45", "r", "46", "r", "47", "r", "48", "r", "49", "r", "50", "r", "51", "r", "52", "r", "53", "r", "54", "r", "55", "r", "56", "r", "57", "r", "58", "r", "59", "r", "60", "r", "61", "r", "62", "r", "63", "r", "64", "r", "65", "r", "66", "r", "67", "r", "68", "r", "69", "r", "70", "r", "71", "r", "72", "r", "73", "r", "74", "r", "75", "r", "76", "r", "77", "r", "78", "r", "79", "r", "80", "r", "81", "r", "82", "r", "83", "r", "84", "r", "85", "r", "86", "r", "87", "r", "88", "r", "89", "r", "90", "r", "91", "r", "92", "r", "93", "r", "94", "r", "95", "r", "96", "r", "97", "r", "98", "r", "99", "r", "100", "r"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 100, options, "-[ RECORD 1 ]--\ni | 1\nt | r\n-[ RECORD 2 ]--\ni | 2\nt | r\n-[ RECORD 3 ]--\ni | 3\nt | r\n-[ RECORD 4 ]--\ni | 4\nt | r\n-[ RECORD 5 ]--\ni | 5\nt | r\n-[ RECORD 6 ]--\ni | 6\nt | r\n-[ RECORD 7 ]--\ni | 7\nt | r\n-[ RECORD 8 ]--\ni | 8\nt | r\n-[ RECORD 9 ]--\ni | 9\nt | r\n-[ RECORD 10 ]-\ni | 10\nt | r\n-[ RECORD 11 ]-\ni | 11\nt | r\n-[ RECORD 12 ]-\ni | 12\nt | r\n-[ RECORD 13 ]-\ni | 13\nt | r\n-[ RECORD 14 ]-\ni | 14\nt | r\n-[ RECORD 15 ]-\ni | 15\nt | r\n-[ RECORD 16 ]-\ni | 16\nt | r\n-[ RECORD 17 ]-\ni | 17\nt | r\n-[ RECORD 18 ]-\ni | 18\nt | r\n-[ RECORD 19 ]-\ni | 19\nt | r\n-[ RECORD 20 ]-\ni | 20\nt | r\n-[ RECORD 21 ]-\ni | 21\nt | r\n-[ RECORD 22 ]-\ni | 22\nt | r\n-[ RECORD 23 ]-\ni | 23\nt | r\n-[ RECORD 24 ]-\ni | 24\nt | r\n-[ RECORD 25 ]-\ni | 25\nt | r\n-[ RECORD 26 ]-\ni | 26\nt | r\n-[ RECORD 27 ]-\ni | 27\nt | r\n-[ RECORD 28 ]-\ni | 28\nt | r\n-[ RECORD 29 ]-\ni | 29\nt | r\n-[ RECORD 30 ]-\ni | 30\nt | r\n-[ RECORD 31 ]-\ni | 31\nt | r\n-[ RECORD 32 ]-\ni | 32\nt | r\n-[ RECORD 33 ]-\ni | 33\nt | r\n-[ RECORD 34 ]-\ni | 34\nt | r\n-[ RECORD 35 ]-\ni | 35\nt | r\n-[ RECORD 36 ]-\ni | 36\nt | r\n-[ RECORD 37 ]-\ni | 37\nt | r\n-[ RECORD 38 ]-\ni | 38\nt | r\n-[ RECORD 39 ]-\ni | 39\nt | r\n-[ RECORD 40 ]-\ni | 40\nt | r\n-[ RECORD 41 ]-\ni | 41\nt | r\n-[ RECORD 42 ]-\ni | 42\nt | r\n-[ RECORD 43 ]-\ni | 43\nt | r\n-[ RECORD 44 ]-\ni | 44\nt | r\n-[ RECORD 45 ]-\ni | 45\nt | r\n-[ RECORD 46 ]-\ni | 46\nt | r\n-[ RECORD 47 ]-\ni | 47\nt | r\n-[ RECORD 48 ]-\ni | 48\nt | r\n-[ RECORD 49 ]-\ni | 49\nt | r\n-[ RECORD 50 ]-\ni | 50\nt | r\n-[ RECORD 51 ]-\ni | 51\nt | r\n-[ RECORD 52 ]-\ni | 52\nt | r\n-[ RECORD 53 ]-\ni | 53\nt | r\n-[ RECORD 54 ]-\ni | 54\nt | r\n-[ RECORD 55 ]-\ni | 55\nt | r\n-[ RECORD 56 ]-\ni | 56\nt | r\n-[ RECORD 57 ]-\ni | 57\nt | r\n-[ RECORD 58 ]-\ni | 58\nt | r\n-[ RECORD 59 ]-\ni | 59\nt | r\n-[ RECORD 60 ]-\ni | 60\nt | r\n-[ RECORD 61 ]-\ni | 61\nt | r\n-[ RECORD 62 ]-\ni | 62\nt | r\n-[ RECORD 63 ]-\ni | 63\nt | r\n-[ RECORD 64 ]-\ni | 64\nt | r\n-[ RECORD 65 ]-\ni | 65\nt | r\n-[ RECORD 66 ]-\ni | 66\nt | r\n-[ RECORD 67 ]-\ni | 67\nt | r\n-[ RECORD 68 ]-\ni | 68\nt | r\n-[ RECORD 69 ]-\ni | 69\nt | r\n-[ RECORD 70 ]-\ni | 70\nt | r\n-[ RECORD 71 ]-\ni | 71\nt | r\n-[ RECORD 72 ]-\ni | 72\nt | r\n-[ RECORD 73 ]-\ni | 73\nt | r\n-[ RECORD 74 ]-\ni | 74\nt | r\n-[ RECORD 75 ]-\ni | 75\nt | r\n-[ RECORD 76 ]-\ni | 76\nt | r\n-[ RECORD 77 ]-\ni | 77\nt | r\n-[ RECORD 78 ]-\ni | 78\nt | r\n-[ RECORD 79 ]-\ni | 79\nt | r\n-[ RECORD 80 ]-\ni | 80\nt | r\n-[ RECORD 81 ]-\ni | 81\nt | r\n-[ RECORD 82 ]-\ni | 82\nt | r\n-[ RECORD 83 ]-\ni | 83\nt | r\n-[ RECORD 84 ]-\ni | 84\nt | r\n-[ RECORD 85 ]-\ni | 85\nt | r\n-[ RECORD 86 ]-\ni | 86\nt | r\n-[ RECORD 87 ]-\ni | 87\nt | r\n-[ RECORD 88 ]-\ni | 88\nt | r\n-[ RECORD 89 ]-\ni | 89\nt | r\n-[ RECORD 90 ]-\ni | 90\nt | r\n-[ RECORD 91 ]-\ni | 91\nt | r\n-[ RECORD 92 ]-\ni | 92\nt | r\n-[ RECORD 93 ]-\ni | 93\nt | r\n-[ RECORD 94 ]-\ni | 94\nt | r\n-[ RECORD 95 ]-\ni | 95\nt | r\n-[ RECORD 96 ]-\ni | 96\nt | r\n-[ RECORD 97 ]-\ni | 97\nt | r\n-[ RECORD 98 ]-\ni | 98\nt | r\n-[ RECORD 99 ]-\ni | 99\nt | r\n-[ RECORD 100 ]\ni | 100\nt | r\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.expanded = .ON;
        options.tuples_only = true;
        check_query("expanded 100 rows tuples only border 1", string.["i", "t"], u32.[23, 25], string.["1", "r", "2", "r", "3", "r"], bool.[false, false, false, false, false, false], 3, options, "i | 1\nt | r\n--+--\ni | 2\nt | r\n--+--\ni | 3\nt | r\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        check_query("aligned many border 2", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "row 1", "1.5", "2", "row 2", "3.0", "3", "row 3", "4.5", "4", "row 4", "6.0", "5", "row 5", "7.5", "6", "row 6", "9.0", "7", "row 7", "10.5", "8", "row 8", "12.0", "9", "row 9", "13.5", "10", "row 10", "15.0", "11", "row 11", "16.5"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 11, options, "+----+--------+-------+\n| id |  name  | score |\n+----+--------+-------+\n|  1 | row 1  |   1.5 |\n|  2 | row 2  |   3.0 |\n|  3 | row 3  |   4.5 |\n|  4 | row 4  |   6.0 |\n|  5 | row 5  |   7.5 |\n|  6 | row 6  |   9.0 |\n|  7 | row 7  |  10.5 |\n|  8 | row 8  |  12.0 |\n|  9 | row 9  |  13.5 |\n| 10 | row 10 |  15.0 |\n| 11 | row 11 |  16.5 |\n+----+--------+-------+\n(11 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.expanded = .ON;
        check_query("expanded many border 2", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "row 1", "1.5", "2", "row 2", "3.0", "3", "row 3", "4.5", "4", "row 4", "6.0", "5", "row 5", "7.5", "6", "row 6", "9.0", "7", "row 7", "10.5", "8", "row 8", "12.0", "9", "row 9", "13.5", "10", "row 10", "15.0", "11", "row 11", "16.5"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 11, options, "+-[ RECORD 1 ]---+\n| id    | 1      |\n| name  | row 1  |\n| score | 1.5    |\n+-[ RECORD 2 ]---+\n| id    | 2      |\n| name  | row 2  |\n| score | 3.0    |\n+-[ RECORD 3 ]---+\n| id    | 3      |\n| name  | row 3  |\n| score | 4.5    |\n+-[ RECORD 4 ]---+\n| id    | 4      |\n| name  | row 4  |\n| score | 6.0    |\n+-[ RECORD 5 ]---+\n| id    | 5      |\n| name  | row 5  |\n| score | 7.5    |\n+-[ RECORD 6 ]---+\n| id    | 6      |\n| name  | row 6  |\n| score | 9.0    |\n+-[ RECORD 7 ]---+\n| id    | 7      |\n| name  | row 7  |\n| score | 10.5   |\n+-[ RECORD 8 ]---+\n| id    | 8      |\n| name  | row 8  |\n| score | 12.0   |\n+-[ RECORD 9 ]---+\n| id    | 9      |\n| name  | row 9  |\n| score | 13.5   |\n+-[ RECORD 10 ]--+\n| id    | 10     |\n| name  | row 10 |\n| score | 15.0   |\n+-[ RECORD 11 ]--+\n| id    | 11     |\n| name  | row 11 |\n| score | 16.5   |\n+-------+--------+\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.expanded = .ON;
        check_query("expanded 100 rows border 2", string.["i", "t"], u32.[23, 25], string.["1", "r", "2", "r", "3", "r", "4", "r", "5", "r", "6", "r", "7", "r", "8", "r", "9", "r", "10", "r", "11", "r", "12", "r", "13", "r", "14", "r", "15", "r", "16", "r", "17", "r", "18", "r", "19", "r", "20", "r", "21", "r", "22", "r", "23", "r", "24", "r", "25", "r", "26", "r", "27", "r", "28", "r", "29", "r", "30", "r", "31", "r", "32", "r", "33", "r", "34", "r", "35", "r", "36", "r", "37", "r", "38", "r", "39", "r", "40", "r", "41", "r", "42", "r", "43", "r", "44", "r", "45", "r", "46", "r", "47", "r", "48", "r", "49", "r", "50", "r", "51", "r", "52", "r", "53", "r", "54", "r", "55", "r", "56", "r", "57", "r", "58", "r", "59", "r", "60", "r", "61", "r", "62", "r", "63", "r", "64", "r", "65", "r", "66", "r", "67", "r", "68", "r", "69", "r", "70", "r", "71", "r", "72", "r", "73", "r", "74", "r", "75", "r", "76", "r", "77", "r", "78", "r", "79", "r", "80", "r", "81", "r", "82", "r", "83", "r", "84", "r", "85", "r", "86", "r", "87", "r", "88", "r", "89", "r", "90", "r", "91", "r", "92", "r", "93", "r", "94", "r", "95", "r", "96", "r", "97", "r", "98", "r", "99", "r", "100", "r"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 100, options, "+-[ RECORD 1 ]---+\n| i | 1          |\n| t | r          |\n+-[ RECORD 2 ]---+\n| i | 2          |\n| t | r          |\n+-[ RECORD 3 ]---+\n| i | 3          |\n| t | r          |\n+-[ RECORD 4 ]---+\n| i | 4          |\n| t | r          |\n+-[ RECORD 5 ]---+\n| i | 5          |\n| t | r          |\n+-[ RECORD 6 ]---+\n| i | 6          |\n| t | r          |\n+-[ RECORD 7 ]---+\n| i | 7          |\n| t | r          |\n+-[ RECORD 8 ]---+\n| i | 8          |\n| t | r          |\n+-[ RECORD 9 ]---+\n| i | 9          |\n| t | r          |\n+-[ RECORD 10 ]--+\n| i | 10         |\n| t | r          |\n+-[ RECORD 11 ]--+\n| i | 11         |\n| t | r          |\n+-[ RECORD 12 ]--+\n| i | 12         |\n| t | r          |\n+-[ RECORD 13 ]--+\n| i | 13         |\n| t | r          |\n+-[ RECORD 14 ]--+\n| i | 14         |\n| t | r          |\n+-[ RECORD 15 ]--+\n| i | 15         |\n| t | r          |\n+-[ RECORD 16 ]--+\n| i | 16         |\n| t | r          |\n+-[ RECORD 17 ]--+\n| i | 17         |\n| t | r          |\n+-[ RECORD 18 ]--+\n| i | 18         |\n| t | r          |\n+-[ RECORD 19 ]--+\n| i | 19         |\n| t | r          |\n+-[ RECORD 20 ]--+\n| i | 20         |\n| t | r          |\n+-[ RECORD 21 ]--+\n| i | 21         |\n| t | r          |\n+-[ RECORD 22 ]--+\n| i | 22         |\n| t | r          |\n+-[ RECORD 23 ]--+\n| i | 23         |\n| t | r          |\n+-[ RECORD 24 ]--+\n| i | 24         |\n| t | r          |\n+-[ RECORD 25 ]--+\n| i | 25         |\n| t | r          |\n+-[ RECORD 26 ]--+\n| i | 26         |\n| t | r          |\n+-[ RECORD 27 ]--+\n| i | 27         |\n| t | r          |\n+-[ RECORD 28 ]--+\n| i | 28         |\n| t | r          |\n+-[ RECORD 29 ]--+\n| i | 29         |\n| t | r          |\n+-[ RECORD 30 ]--+\n| i | 30         |\n| t | r          |\n+-[ RECORD 31 ]--+\n| i | 31         |\n| t | r          |\n+-[ RECORD 32 ]--+\n| i | 32         |\n| t | r          |\n+-[ RECORD 33 ]--+\n| i | 33         |\n| t | r          |\n+-[ RECORD 34 ]--+\n| i | 34         |\n| t | r          |\n+-[ RECORD 35 ]--+\n| i | 35         |\n| t | r          |\n+-[ RECORD 36 ]--+\n| i | 36         |\n| t | r          |\n+-[ RECORD 37 ]--+\n| i | 37         |\n| t | r          |\n+-[ RECORD 38 ]--+\n| i | 38         |\n| t | r          |\n+-[ RECORD 39 ]--+\n| i | 39         |\n| t | r          |\n+-[ RECORD 40 ]--+\n| i | 40         |\n| t | r          |\n+-[ RECORD 41 ]--+\n| i | 41         |\n| t | r          |\n+-[ RECORD 42 ]--+\n| i | 42         |\n| t | r          |\n+-[ RECORD 43 ]--+\n| i | 43         |\n| t | r          |\n+-[ RECORD 44 ]--+\n| i | 44         |\n| t | r          |\n+-[ RECORD 45 ]--+\n| i | 45         |\n| t | r          |\n+-[ RECORD 46 ]--+\n| i | 46         |\n| t | r          |\n+-[ RECORD 47 ]--+\n| i | 47         |\n| t | r          |\n+-[ RECORD 48 ]--+\n| i | 48         |\n| t | r          |\n+-[ RECORD 49 ]--+\n| i | 49         |\n| t | r          |\n+-[ RECORD 50 ]--+\n| i | 50         |\n| t | r          |\n+-[ RECORD 51 ]--+\n| i | 51         |\n| t | r          |\n+-[ RECORD 52 ]--+\n| i | 52         |\n| t | r          |\n+-[ RECORD 53 ]--+\n| i | 53         |\n| t | r          |\n+-[ RECORD 54 ]--+\n| i | 54         |\n| t | r          |\n+-[ RECORD 55 ]--+\n| i | 55         |\n| t | r          |\n+-[ RECORD 56 ]--+\n| i | 56         |\n| t | r          |\n+-[ RECORD 57 ]--+\n| i | 57         |\n| t | r          |\n+-[ RECORD 58 ]--+\n| i | 58         |\n| t | r          |\n+-[ RECORD 59 ]--+\n| i | 59         |\n| t | r          |\n+-[ RECORD 60 ]--+\n| i | 60         |\n| t | r          |\n+-[ RECORD 61 ]--+\n| i | 61         |\n| t | r          |\n+-[ RECORD 62 ]--+\n| i | 62         |\n| t | r          |\n+-[ RECORD 63 ]--+\n| i | 63         |\n| t | r          |\n+-[ RECORD 64 ]--+\n| i | 64         |\n| t | r          |\n+-[ RECORD 65 ]--+\n| i | 65         |\n| t | r          |\n+-[ RECORD 66 ]--+\n| i | 66         |\n| t | r          |\n+-[ RECORD 67 ]--+\n| i | 67         |\n| t | r          |\n+-[ RECORD 68 ]--+\n| i | 68         |\n| t | r          |\n+-[ RECORD 69 ]--+\n| i | 69         |\n| t | r          |\n+-[ RECORD 70 ]--+\n| i | 70         |\n| t | r          |\n+-[ RECORD 71 ]--+\n| i | 71         |\n| t | r          |\n+-[ RECORD 72 ]--+\n| i | 72         |\n| t | r          |\n+-[ RECORD 73 ]--+\n| i | 73         |\n| t | r          |\n+-[ RECORD 74 ]--+\n| i | 74         |\n| t | r          |\n+-[ RECORD 75 ]--+\n| i | 75         |\n| t | r          |\n+-[ RECORD 76 ]--+\n| i | 76         |\n| t | r          |\n+-[ RECORD 77 ]--+\n| i | 77         |\n| t | r          |\n+-[ RECORD 78 ]--+\n| i | 78         |\n| t | r          |\n+-[ RECORD 79 ]--+\n| i | 79         |\n| t | r          |\n+-[ RECORD 80 ]--+\n| i | 80         |\n| t | r          |\n+-[ RECORD 81 ]--+\n| i | 81         |\n| t | r          |\n+-[ RECORD 82 ]--+\n| i | 82         |\n| t | r          |\n+-[ RECORD 83 ]--+\n| i | 83         |\n| t | r          |\n+-[ RECORD 84 ]--+\n| i | 84         |\n| t | r          |\n+-[ RECORD 85 ]--+\n| i | 85         |\n| t | r          |\n+-[ RECORD 86 ]--+\n| i | 86         |\n| t | r          |\n+-[ RECORD 87 ]--+\n| i | 87         |\n| t | r          |\n+-[ RECORD 88 ]--+\n| i | 88         |\n| t | r          |\n+-[ RECORD 89 ]--+\n| i | 89         |\n| t | r          |\n+-[ RECORD 90 ]--+\n| i | 90         |\n| t | r          |\n+-[ RECORD 91 ]--+\n| i | 91         |\n| t | r          |\n+-[ RECORD 92 ]--+\n| i | 92         |\n| t | r          |\n+-[ RECORD 93 ]--+\n| i | 93         |\n| t | r          |\n+-[ RECORD 94 ]--+\n| i | 94         |\n| t | r          |\n+-[ RECORD 95 ]--+\n| i | 95         |\n| t | r          |\n+-[ RECORD 96 ]--+\n| i | 96         |\n| t | r          |\n+-[ RECORD 97 ]--+\n| i | 97         |\n| t | r          |\n+-[ RECORD 98 ]--+\n| i | 98         |\n| t | r          |\n+-[ RECORD 99 ]--+\n| i | 99         |\n| t | r          |\n+-[ RECORD 100 ]-+\n| i | 100        |\n| t | r          |\n+---+------------+\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.expanded = .ON;
        options.tuples_only = true;
        check_query("expanded 100 rows tuples only border 2", string.["i", "t"], u32.[23, 25], string.["1", "r", "2", "r", "3", "r"], bool.[false, false, false, false, false, false], 3, options, "+---+---+\n| i | 1 |\n| t | r |\n+---+---+\n| i | 2 |\n| t | r |\n+---+---+\n| i | 3 |\n| t | r |\n+---+---+\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.expanded = .ON;
        check_query("expanded people border 0", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "* Record 1  \nid    1\nname  alice\nscore 10.5\n* Record 2  \nid    22\nname  bob\nscore \n* Record 3  \nid    333\nname  \nscore -7\n* Record 4  \nid    4\nname  \nscore 1000.25\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.expanded = .ON;
        check_query("expanded empty border 0", string.["id", "name", "score"], u32.[23, 25, 1700], string.[], bool.[], 0, options, "(0 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.expanded = .ON;
        check_query("expanded one border 0", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5"], bool.[false, false, false], 1, options, "* Record 1\nid    1\nname  alice\nscore 10.5\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.expanded = .ON;
        options.null_display = "~";
        check_query("expanded null border 0", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "* Record 1  \nid    1\nname  alice\nscore 10.5\n* Record 2  \nid    22\nname  bob\nscore ~\n* Record 3  \nid    333\nname  ~\nscore -7\n* Record 4  \nid    4\nname  \nscore 1000.25\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.expanded = .ON;
        options.tuples_only = true;
        check_query("expanded tuples only border 0", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "id    1\nname  alice\nscore 10.5\n            \nid    22\nname  bob\nscore \n            \nid    333\nname  \nscore -7\n            \nid    4\nname  \nscore 1000.25\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.expanded = .ON;
        options.title = "People";
        check_query("expanded title border 0", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "People\n* Record 1  \nid    1\nname  alice\nscore 10.5\n* Record 2  \nid    22\nname  bob\nscore \n* Record 3  \nid    333\nname  \nscore -7\n* Record 4  \nid    4\nname  \nscore 1000.25\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.expanded = .ON;
        options.footer = false;
        check_query("expanded footer off border 0", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "* Record 1  \nid    1\nname  alice\nscore 10.5\n* Record 2  \nid    22\nname  bob\nscore \n* Record 3  \nid    333\nname  \nscore -7\n* Record 4  \nid    4\nname  \nscore 1000.25\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.expanded = .ON;
        check_query("expanded zero columns border 0", string.[], u32.[], string.[], bool.[], 2, options, "(2 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.expanded = .ON;
        options.tuples_only = true;
        check_query("expanded empty tuples only border 0", string.["id", "name", "score"], u32.[23, 25, 1700], string.[], bool.[], 0, options, "\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.expanded = .ON;
        check_query("expanded multiline border 0", string.["id", "body", "n"], u32.[23, 25, 1700], string.["1", "line one\nline two\nthree", "5", "2", "single", "6.5", "3", "ends\n", "7", "4", "\nstarts", "8", "5", "a\n\nb", "9"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 5, options, "* Record 1  \nid   1\nbody line one+\n     line two+\n     three\nn    5\n* Record 2  \nid   2\nbody single\nn    6.5\n* Record 3  \nid   3\nbody ends    +\n     \nn    7\n* Record 4  \nid   4\nbody         +\n     starts\nn    8\n* Record 5  \nid   5\nbody a       +\n             +\n     b\nn    9\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.expanded = .ON;
        check_query("expanded multiline header border 0", string.["two\nline head", "x"], u32.[25, 23], string.["a", "1", "c\nd", "2"], bool.[false, false, false, false], 2, options, "* Record 1\ntwo      + a\nline head \nx          1\n* Record 2\ntwo      + c+\nline head  d\nx          2\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.expanded = .ON;
        options.linestyle = .OLD_ASCII;
        check_query("expanded multiline header old-ascii border 0", string.["two\nline head", "x"], u32.[25, 23], string.["a", "1", "c\nd", "2"], bool.[false, false, false, false], 2, options, "* Record 1 \n two       a\n+line head\n x         1\n* Record 2 \n two       c\n+line head d\n x         2\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.expanded = .ON;
        check_query("expanded ctrl border 0", string.["t", "u"], u32.[25, 25], string.["a\tb", "x\ry", "\x01", ""], bool.[false, false, false, false], 2, options, "* Record 1\nt a       b\nu x\\ry\n* Record 2\nt \\x01\nu \\u0085\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.expanded = .ON;
        check_query("expanded unicode text border 0", string.["名前", "v"], u32.[25, 23], string.["日本語", "1", "café", "22"], bool.[false, false, false, false], 2, options, "* Record 1\n名前 日本語\nv    1\n* Record 2\n名前 café\nv    22\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.expanded = .ON;
        check_query("expanded short values border 0", string.["a_long_column_name_here"], u32.[25], string.["x", "y"], bool.[false, false], 2, options, "* Record 1              \na_long_column_name_here x\n* Record 2              \na_long_column_name_here y\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.expanded = .ON;
        check_query("expanded wide label border 0", string.["a"], u32.[25], string.["x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x"], bool.[false, false, false, false, false, false, false, false, false, false, false, false], 12, options, "* Record 1\na x\n* Record 2\na x\n* Record 3\na x\n* Record 4\na x\n* Record 5\na x\n* Record 6\na x\n* Record 7\na x\n* Record 8\na x\n* Record 9\na x\n* Record 10\na x\n* Record 11\na x\n* Record 12\na x\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.expanded = .AUTO;
        check_query("expanded auto no columns border 0", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "id  name   score  \n--- ----- -------\n  1 alice    10.5\n 22 bob          \n333            -7\n  4       1000.25\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.expanded = .ON;
        check_query("expanded people border 1", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "-[ RECORD 1 ]--\nid    | 1\nname  | alice\nscore | 10.5\n-[ RECORD 2 ]--\nid    | 22\nname  | bob\nscore | \n-[ RECORD 3 ]--\nid    | 333\nname  | \nscore | -7\n-[ RECORD 4 ]--\nid    | 4\nname  | \nscore | 1000.25\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.expanded = .ON;
        check_query("expanded empty border 1", string.["id", "name", "score"], u32.[23, 25, 1700], string.[], bool.[], 0, options, "(0 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.expanded = .ON;
        check_query("expanded one border 1", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5"], bool.[false, false, false], 1, options, "-[ RECORD 1 ]\nid    | 1\nname  | alice\nscore | 10.5\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.expanded = .ON;
        options.null_display = "~";
        check_query("expanded null border 1", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "-[ RECORD 1 ]--\nid    | 1\nname  | alice\nscore | 10.5\n-[ RECORD 2 ]--\nid    | 22\nname  | bob\nscore | ~\n-[ RECORD 3 ]--\nid    | 333\nname  | ~\nscore | -7\n-[ RECORD 4 ]--\nid    | 4\nname  | \nscore | 1000.25\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.expanded = .ON;
        options.tuples_only = true;
        check_query("expanded tuples only border 1", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "id    | 1\nname  | alice\nscore | 10.5\n------+--------\nid    | 22\nname  | bob\nscore | \n------+--------\nid    | 333\nname  | \nscore | -7\n------+--------\nid    | 4\nname  | \nscore | 1000.25\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.expanded = .ON;
        options.title = "People";
        check_query("expanded title border 1", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "People\n-[ RECORD 1 ]--\nid    | 1\nname  | alice\nscore | 10.5\n-[ RECORD 2 ]--\nid    | 22\nname  | bob\nscore | \n-[ RECORD 3 ]--\nid    | 333\nname  | \nscore | -7\n-[ RECORD 4 ]--\nid    | 4\nname  | \nscore | 1000.25\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.expanded = .ON;
        options.footer = false;
        check_query("expanded footer off border 1", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "-[ RECORD 1 ]--\nid    | 1\nname  | alice\nscore | 10.5\n-[ RECORD 2 ]--\nid    | 22\nname  | bob\nscore | \n-[ RECORD 3 ]--\nid    | 333\nname  | \nscore | -7\n-[ RECORD 4 ]--\nid    | 4\nname  | \nscore | 1000.25\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.expanded = .ON;
        check_query("expanded zero columns border 1", string.[], u32.[], string.[], bool.[], 2, options, "(2 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.expanded = .ON;
        options.tuples_only = true;
        check_query("expanded empty tuples only border 1", string.["id", "name", "score"], u32.[23, 25, 1700], string.[], bool.[], 0, options, "\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.expanded = .ON;
        check_query("expanded multiline border 1", string.["id", "body", "n"], u32.[23, 25, 1700], string.["1", "line one\nline two\nthree", "5", "2", "single", "6.5", "3", "ends\n", "7", "4", "\nstarts", "8", "5", "a\n\nb", "9"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 5, options, "-[ RECORD 1 ]--\nid   | 1\nbody | line one+\n     | line two+\n     | three\nn    | 5\n-[ RECORD 2 ]--\nid   | 2\nbody | single\nn    | 6.5\n-[ RECORD 3 ]--\nid   | 3\nbody | ends    +\n     | \nn    | 7\n-[ RECORD 4 ]--\nid   | 4\nbody |         +\n     | starts\nn    | 8\n-[ RECORD 5 ]--\nid   | 5\nbody | a       +\n     |         +\n     | b\nn    | 9\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.expanded = .ON;
        check_query("expanded multiline header border 1", string.["two\nline head", "x"], u32.[25, 23], string.["a", "1", "c\nd", "2"], bool.[false, false, false, false], 2, options, "-[ RECORD 1 ]\ntwo      +| a\nline head |\nx         | 1\n-[ RECORD 2 ]\ntwo      +| c+\nline head | d\nx         | 2\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.expanded = .ON;
        options.linestyle = .OLD_ASCII;
        check_query("expanded multiline header old-ascii border 1", string.["two\nline head", "x"], u32.[25, 23], string.["a", "1", "c\nd", "2"], bool.[false, false, false, false], 2, options, "-[ RECORD 1 ]-\n two       | a\n+line head ;\n x         | 1\n-[ RECORD 2 ]-\n two       | c\n+line head : d\n x         | 2\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.expanded = .ON;
        check_query("expanded ctrl border 1", string.["t", "u"], u32.[25, 25], string.["a\tb", "x\ry", "\x01", ""], bool.[false, false, false, false], 2, options, "-[ RECORD 1 ]\nt | a       b\nu | x\\ry\n-[ RECORD 2 ]\nt | \\x01\nu | \\u0085\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.expanded = .ON;
        check_query("expanded unicode text border 1", string.["名前", "v"], u32.[25, 23], string.["日本語", "1", "café", "22"], bool.[false, false, false, false], 2, options, "-[ RECORD 1 ]\n名前 | 日本語\nv    | 1\n-[ RECORD 2 ]\n名前 | café\nv    | 22\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.expanded = .ON;
        check_query("expanded short values border 1", string.["a_long_column_name_here"], u32.[25], string.["x", "y"], bool.[false, false], 2, options, "-[ RECORD 1 ]-----------+--\na_long_column_name_here | x\n-[ RECORD 2 ]-----------+--\na_long_column_name_here | y\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.expanded = .ON;
        check_query("expanded wide label border 1", string.["a"], u32.[25], string.["x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x"], bool.[false, false, false, false, false, false, false, false, false, false, false, false], 12, options, "-[ RECORD 1 ]-\na | x\n-[ RECORD 2 ]-\na | x\n-[ RECORD 3 ]-\na | x\n-[ RECORD 4 ]-\na | x\n-[ RECORD 5 ]-\na | x\n-[ RECORD 6 ]-\na | x\n-[ RECORD 7 ]-\na | x\n-[ RECORD 8 ]-\na | x\n-[ RECORD 9 ]-\na | x\n-[ RECORD 10 ]\na | x\n-[ RECORD 11 ]\na | x\n-[ RECORD 12 ]\na | x\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.expanded = .AUTO;
        check_query("expanded auto no columns border 1", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, " id  | name  |  score  \n-----+-------+---------\n   1 | alice |    10.5\n  22 | bob   |        \n 333 |       |      -7\n   4 |       | 1000.25\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.expanded = .ON;
        check_query("expanded people border 2", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "+-[ RECORD 1 ]----+\n| id    | 1       |\n| name  | alice   |\n| score | 10.5    |\n+-[ RECORD 2 ]----+\n| id    | 22      |\n| name  | bob     |\n| score |         |\n+-[ RECORD 3 ]----+\n| id    | 333     |\n| name  |         |\n| score | -7      |\n+-[ RECORD 4 ]----+\n| id    | 4       |\n| name  |         |\n| score | 1000.25 |\n+-------+---------+\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.expanded = .ON;
        check_query("expanded empty border 2", string.["id", "name", "score"], u32.[23, 25, 1700], string.[], bool.[], 0, options, "(0 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.expanded = .ON;
        check_query("expanded one border 2", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5"], bool.[false, false, false], 1, options, "+-[ RECORD 1 ]--+\n| id    | 1     |\n| name  | alice |\n| score | 10.5  |\n+-------+-------+\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.expanded = .ON;
        options.null_display = "~";
        check_query("expanded null border 2", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "+-[ RECORD 1 ]----+\n| id    | 1       |\n| name  | alice   |\n| score | 10.5    |\n+-[ RECORD 2 ]----+\n| id    | 22      |\n| name  | bob     |\n| score | ~       |\n+-[ RECORD 3 ]----+\n| id    | 333     |\n| name  | ~       |\n| score | -7      |\n+-[ RECORD 4 ]----+\n| id    | 4       |\n| name  |         |\n| score | 1000.25 |\n+-------+---------+\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.expanded = .ON;
        options.tuples_only = true;
        check_query("expanded tuples only border 2", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "+-------+---------+\n| id    | 1       |\n| name  | alice   |\n| score | 10.5    |\n+-------+---------+\n| id    | 22      |\n| name  | bob     |\n| score |         |\n+-------+---------+\n| id    | 333     |\n| name  |         |\n| score | -7      |\n+-------+---------+\n| id    | 4       |\n| name  |         |\n| score | 1000.25 |\n+-------+---------+\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.expanded = .ON;
        options.title = "People";
        check_query("expanded title border 2", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "People\n+-[ RECORD 1 ]----+\n| id    | 1       |\n| name  | alice   |\n| score | 10.5    |\n+-[ RECORD 2 ]----+\n| id    | 22      |\n| name  | bob     |\n| score |         |\n+-[ RECORD 3 ]----+\n| id    | 333     |\n| name  |         |\n| score | -7      |\n+-[ RECORD 4 ]----+\n| id    | 4       |\n| name  |         |\n| score | 1000.25 |\n+-------+---------+\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.expanded = .ON;
        options.footer = false;
        check_query("expanded footer off border 2", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "+-[ RECORD 1 ]----+\n| id    | 1       |\n| name  | alice   |\n| score | 10.5    |\n+-[ RECORD 2 ]----+\n| id    | 22      |\n| name  | bob     |\n| score |         |\n+-[ RECORD 3 ]----+\n| id    | 333     |\n| name  |         |\n| score | -7      |\n+-[ RECORD 4 ]----+\n| id    | 4       |\n| name  |         |\n| score | 1000.25 |\n+-------+---------+\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.expanded = .ON;
        check_query("expanded zero columns border 2", string.[], u32.[], string.[], bool.[], 2, options, "(2 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.expanded = .ON;
        options.tuples_only = true;
        check_query("expanded empty tuples only border 2", string.["id", "name", "score"], u32.[23, 25, 1700], string.[], bool.[], 0, options, "\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.expanded = .ON;
        check_query("expanded multiline border 2", string.["id", "body", "n"], u32.[23, 25, 1700], string.["1", "line one\nline two\nthree", "5", "2", "single", "6.5", "3", "ends\n", "7", "4", "\nstarts", "8", "5", "a\n\nb", "9"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 5, options, "+-[ RECORD 1 ]----+\n| id   | 1        |\n| body | line one+|\n|      | line two+|\n|      | three    |\n| n    | 5        |\n+-[ RECORD 2 ]----+\n| id   | 2        |\n| body | single   |\n| n    | 6.5      |\n+-[ RECORD 3 ]----+\n| id   | 3        |\n| body | ends    +|\n|      |          |\n| n    | 7        |\n+-[ RECORD 4 ]----+\n| id   | 4        |\n| body |         +|\n|      | starts   |\n| n    | 8        |\n+-[ RECORD 5 ]----+\n| id   | 5        |\n| body | a       +|\n|      |         +|\n|      | b        |\n| n    | 9        |\n+------+----------+\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.expanded = .ON;
        check_query("expanded multiline header border 2", string.["two\nline head", "x"], u32.[25, 23], string.["a", "1", "c\nd", "2"], bool.[false, false, false, false], 2, options, "+-[ RECORD 1 ]--+\n| two      +| a |\n| line head |   |\n| x         | 1 |\n+-[ RECORD 2 ]--+\n| two      +| c+|\n| line head | d |\n| x         | 2 |\n+-----------+---+\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.expanded = .ON;
        options.linestyle = .OLD_ASCII;
        check_query("expanded multiline header old-ascii border 2", string.["two\nline head", "x"], u32.[25, 23], string.["a", "1", "c\nd", "2"], bool.[false, false, false, false], 2, options, "+-[ RECORD 1 ]--+\n| two       | a |\n|+line head ;   |\n| x         | 1 |\n+-[ RECORD 2 ]--+\n| two       | c |\n|+line head : d |\n| x         | 2 |\n+-----------+---+\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.expanded = .ON;
        check_query("expanded ctrl border 2", string.["t", "u"], u32.[25, 25], string.["a\tb", "x\ry", "\x01", ""], bool.[false, false, false, false], 2, options, "+-[ RECORD 1 ]--+\n| t | a       b |\n| u | x\\ry      |\n+-[ RECORD 2 ]--+\n| t | \\x01      |\n| u | \\u0085    |\n+---+-----------+\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.expanded = .ON;
        check_query("expanded unicode text border 2", string.["名前", "v"], u32.[25, 23], string.["日本語", "1", "café", "22"], bool.[false, false, false, false], 2, options, "+-[ RECORD 1 ]--+\n| 名前 | 日本語 |\n| v    | 1      |\n+-[ RECORD 2 ]--+\n| 名前 | café   |\n| v    | 22     |\n+------+--------+\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.expanded = .ON;
        check_query("expanded short values border 2", string.["a_long_column_name_here"], u32.[25], string.["x", "y"], bool.[false, false], 2, options, "+-[ RECORD 1 ]------------+---+\n| a_long_column_name_here | x |\n+-[ RECORD 2 ]------------+---+\n| a_long_column_name_here | y |\n+-------------------------+---+\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.expanded = .ON;
        check_query("expanded wide label border 2", string.["a"], u32.[25], string.["x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x"], bool.[false, false, false, false, false, false, false, false, false, false, false, false], 12, options, "+-[ RECORD 1 ]--+\n| a | x         |\n+-[ RECORD 2 ]--+\n| a | x         |\n+-[ RECORD 3 ]--+\n| a | x         |\n+-[ RECORD 4 ]--+\n| a | x         |\n+-[ RECORD 5 ]--+\n| a | x         |\n+-[ RECORD 6 ]--+\n| a | x         |\n+-[ RECORD 7 ]--+\n| a | x         |\n+-[ RECORD 8 ]--+\n| a | x         |\n+-[ RECORD 9 ]--+\n| a | x         |\n+-[ RECORD 10 ]-+\n| a | x         |\n+-[ RECORD 11 ]-+\n| a | x         |\n+-[ RECORD 12 ]-+\n| a | x         |\n+---+-----------+\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.expanded = .AUTO;
        check_query("expanded auto no columns border 2", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "+-----+-------+---------+\n| id  | name  |  score  |\n+-----+-------+---------+\n|   1 | alice |    10.5 |\n|  22 | bob   |         |\n| 333 |       |      -7 |\n|   4 |       | 1000.25 |\n+-----+-------+---------+\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        check_query("unaligned people", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "id|name|score\n1|alice|10.5\n22|bob|\n333||-7\n4||1000.25\n(4 rows)\n");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        check_query("unaligned empty", string.["id", "name", "score"], u32.[23, 25, 1700], string.[], bool.[], 0, options, "id|name|score\n(0 rows)\n");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        check_query("unaligned one row", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5"], bool.[false, false, false], 1, options, "id|name|score\n1|alice|10.5\n(1 row)\n");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        options.tuples_only = true;
        check_query("unaligned tuples only", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "1|alice|10.5\n22|bob|\n333||-7\n4||1000.25\n");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        options.tuples_only = true;
        check_query("unaligned tuples only empty", string.["id", "name", "score"], u32.[23, 25, 1700], string.[], bool.[], 0, options, "");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        options.footer = false;
        check_query("unaligned footer off", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "id|name|score\n1|alice|10.5\n22|bob|\n333||-7\n4||1000.25\n");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        options.field_separator = ";";
        check_query("unaligned fieldsep", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "id;name;score\n1;alice;10.5\n22;bob;\n333;;-7\n4;;1000.25\n(4 rows)\n");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        options.field_separator = "<>";
        check_query("unaligned fieldsep long", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "id<>name<>score\n1<>alice<>10.5\n22<>bob<>\n333<><>-7\n4<><>1000.25\n(4 rows)\n");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        options.record_separator = "|";
        check_query("unaligned recordsep", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "id|name|score|1|alice|10.5|22|bob||333||-7|4||1000.25|(4 rows)\n");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        options.record_separator = "|";
        options.tuples_only = true;
        check_query("unaligned recordsep tuples only", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "1|alice|10.5|22|bob||333||-7|4||1000.25\n");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        options.null_display = "NULL!";
        check_query("unaligned null", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "id|name|score\n1|alice|10.5\n22|bob|NULL!\n333|NULL!|-7\n4||1000.25\n(4 rows)\n");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        options.title = "People";
        check_query("unaligned title", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "People\nid|name|score\n1|alice|10.5\n22|bob|\n333||-7\n4||1000.25\n(4 rows)\n");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        options.title = "People";
        options.tuples_only = true;
        check_query("unaligned title tuples only", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "1|alice|10.5\n22|bob|\n333||-7\n4||1000.25\n");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        check_query("unaligned zero columns", string.[], u32.[], string.[], bool.[], 2, options, "\n(2 rows)\n");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        check_query("unaligned zero columns none", string.[], u32.[], string.[], bool.[], 0, options, "\n(0 rows)\n");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        check_query("unaligned multiline", string.["id", "body", "n"], u32.[23, 25, 1700], string.["1", "line one\nline two\nthree", "5", "2", "single", "6.5", "3", "ends\n", "7", "4", "\nstarts", "8", "5", "a\n\nb", "9"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 5, options, "id|body|n\n1|line one\nline two\nthree|5\n2|single|6.5\n3|ends\n|7\n4|\nstarts|8\n5|a\n\nb|9\n(5 rows)\n");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        check_query("unaligned ctrl", string.["t"], u32.[25], string.["a\tb", "x\ry"], bool.[false, false], 2, options, "t\na\tb\nx\ry\n(2 rows)\n");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        check_query("unaligned unicode", string.["名前"], u32.[25], string.["日本語"], bool.[false], 1, options, "名前\n日本語\n(1 row)\n");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        options.expanded = .ON;
        check_query("unaligned expanded", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "id|1\nname|alice\nscore|10.5\n\nid|22\nname|bob\nscore|\n\nid|333\nname|\nscore|-7\n\nid|4\nname|\nscore|1000.25\n");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        options.expanded = .ON;
        check_query("unaligned expanded empty", string.["id", "name", "score"], u32.[23, 25, 1700], string.[], bool.[], 0, options, "");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        options.expanded = .ON;
        check_query("unaligned expanded one row", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5"], bool.[false, false, false], 1, options, "id|1\nname|alice\nscore|10.5\n");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        options.expanded = .ON;
        options.tuples_only = true;
        check_query("unaligned expanded tuples only", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "id|1\nname|alice\nscore|10.5\n\nid|22\nname|bob\nscore|\n\nid|333\nname|\nscore|-7\n\nid|4\nname|\nscore|1000.25\n");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        options.expanded = .ON;
        options.field_separator = ": ";
        check_query("unaligned expanded fieldsep", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "id: 1\nname: alice\nscore: 10.5\n\nid: 22\nname: bob\nscore: \n\nid: 333\nname: \nscore: -7\n\nid: 4\nname: \nscore: 1000.25\n");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        options.expanded = .ON;
        options.record_separator = "|";
        check_query("unaligned expanded recordsep", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "id|1|name|alice|score|10.5||id|22|name|bob|score|||id|333|name||score|-7||id|4|name||score|1000.25\n");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        options.expanded = .ON;
        options.title = "People";
        check_query("unaligned expanded title", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "People\n\nid|1\nname|alice\nscore|10.5\n\nid|22\nname|bob\nscore|\n\nid|333\nname|\nscore|-7\n\nid|4\nname|\nscore|1000.25\n");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        options.expanded = .ON;
        options.footer = false;
        check_query("unaligned expanded footer off", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "id|1\nname|alice\nscore|10.5\n\nid|22\nname|bob\nscore|\n\nid|333\nname|\nscore|-7\n\nid|4\nname|\nscore|1000.25\n");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        options.expanded = .ON;
        check_query("unaligned expanded zero columns", string.[], u32.[], string.[], bool.[], 2, options, "");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        options.expanded = .AUTO;
        check_query("unaligned expanded auto", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "id|name|score\n1|alice|10.5\n22|bob|\n333||-7\n4||1000.25\n(4 rows)\n");
    }

    {
        options: Print_Options;
        options.format = .CSV;
        check_query("csv people", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "id,name,score\n1,alice,10.5\n22,bob,\n333,,-7\n4,,1000.25\n");
    }

    {
        options: Print_Options;
        options.format = .CSV;
        check_query("csv empty", string.["id", "name", "score"], u32.[23, 25, 1700], string.[], bool.[], 0, options, "id,name,score\n");
    }

    {
        options: Print_Options;
        options.format = .CSV;
        check_query("csv tricky", string.["text col", "n"], u32.[25, 23], string.["plain", "1", "with,comma", "2", "with \"quote\"", "3", "multi\nline", "4", "\\.", "5", "cr\rhere", "6", "", "7", "", "8", " lead", "9", "日本語", "10", "tab\there", "11", ";semi", "12"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, true, false, false, false, false, false, false, false, false, false], 12, options, "text col,n\nplain,1\n\"with,comma\",2\n\"with \"\"quote\"\"\",3\n\"multi\nline\",4\n\"\\.\",5\n\"cr\rhere\",6\n,7\n,8\n lead,9\n日本語,10\ntab\there,11\n;semi,12\n");
    }

    {
        options: Print_Options;
        options.format = .CSV;
        options.tuples_only = true;
        check_query("csv tricky tuples only", string.["text col", "n"], u32.[25, 23], string.["plain", "1", "with,comma", "2", "with \"quote\"", "3", "multi\nline", "4", "\\.", "5", "cr\rhere", "6", "", "7", "", "8", " lead", "9", "日本語", "10", "tab\there", "11", ";semi", "12"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, true, false, false, false, false, false, false, false, false, false], 12, options, "plain,1\n\"with,comma\",2\n\"with \"\"quote\"\"\",3\n\"multi\nline\",4\n\"\\.\",5\n\"cr\rhere\",6\n,7\n,8\n lead,9\n日本語,10\ntab\there,11\n;semi,12\n");
    }

    {
        options: Print_Options;
        options.format = .CSV;
        options.csv_field_separator = ";";
        check_query("csv tricky semicolon", string.["text col", "n"], u32.[25, 23], string.["plain", "1", "with,comma", "2", "with \"quote\"", "3", "multi\nline", "4", "\\.", "5", "cr\rhere", "6", "", "7", "", "8", " lead", "9", "日本語", "10", "tab\there", "11", ";semi", "12"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, true, false, false, false, false, false, false, false, false, false], 12, options, "text col;n\nplain;1\nwith,comma;2\n\"with \"\"quote\"\"\";3\n\"multi\nline\";4\n\"\\.\";5\n\"cr\rhere\";6\n;7\n;8\n lead;9\n日本語;10\ntab\there;11\n\";semi\";12\n");
    }

    {
        options: Print_Options;
        options.format = .CSV;
        options.csv_field_separator = "|";
        check_query("csv tricky pipe", string.["text col", "n"], u32.[25, 23], string.["plain", "1", "with,comma", "2", "with \"quote\"", "3", "multi\nline", "4", "\\.", "5", "cr\rhere", "6", "", "7", "", "8", " lead", "9", "日本語", "10", "tab\there", "11", ";semi", "12"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, true, false, false, false, false, false, false, false, false, false], 12, options, "text col|n\nplain|1\nwith,comma|2\n\"with \"\"quote\"\"\"|3\n\"multi\nline\"|4\n\"\\.\"|5\n\"cr\rhere\"|6\n|7\n|8\n lead|9\n日本語|10\ntab\there|11\n;semi|12\n");
    }

    {
        options: Print_Options;
        options.format = .CSV;
        options.csv_field_separator = ".";
        check_query("csv sep dot", string.["text col", "n"], u32.[25, 23], string.["plain", "1", "with,comma", "2", "with \"quote\"", "3"], bool.[false, false, false, false, false, false], 3, options, "\"text col\".\"n\"\n\"plain\".\"1\"\n\"with,comma\".\"2\"\n\"with \"\"quote\"\"\".\"3\"\n");
    }

    {
        options: Print_Options;
        options.format = .CSV;
        options.csv_field_separator = "\\";
        check_query("csv sep backslash", string.["text col", "n"], u32.[25, 23], string.["plain", "1", "with,comma", "2", "with \"quote\"", "3"], bool.[false, false, false, false, false, false], 3, options, "\"text col\"\\\"n\"\n\"plain\"\\\"1\"\n\"with,comma\"\\\"2\"\n\"with \"\"quote\"\"\"\\\"3\"\n");
    }

    {
        options: Print_Options;
        options.format = .CSV;
        options.null_display = "NULL";
        check_query("csv null", string.["text col", "n"], u32.[25, 23], string.["plain", "1", "with,comma", "2", "with \"quote\"", "3", "multi\nline", "4", "\\.", "5", "cr\rhere", "6", "", "7", "", "8", " lead", "9", "日本語", "10", "tab\there", "11", ";semi", "12"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, true, false, false, false, false, false, false, false, false, false], 12, options, "text col,n\nplain,1\n\"with,comma\",2\n\"with \"\"quote\"\"\",3\n\"multi\nline\",4\n\"\\.\",5\n\"cr\rhere\",6\n,7\nNULL,8\n lead,9\n日本語,10\ntab\there,11\n;semi,12\n");
    }

    {
        options: Print_Options;
        options.format = .CSV;
        options.title = "T";
        check_query("csv title footer", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "id,name,score\n1,alice,10.5\n22,bob,\n333,,-7\n4,,1000.25\n");
    }

    {
        options: Print_Options;
        options.format = .CSV;
        check_query("csv header quoting", string.["a,b", "c\"d", "e\nf"], u32.[25, 25, 23], string.["1", "2", "3"], bool.[false, false, false], 1, options, "\"a,b\",\"c\"\"d\",\"e\nf\"\n1,2,3\n");
    }

    {
        options: Print_Options;
        options.format = .CSV;
        check_query("csv zero columns", string.[], u32.[], string.[], bool.[], 2, options, "\n");
    }

    {
        options: Print_Options;
        options.format = .CSV;
        options.expanded = .ON;
        check_query("csv expanded", string.["text col", "n"], u32.[25, 23], string.["plain", "1", "with,comma", "2", "with \"quote\"", "3", "multi\nline", "4", "\\.", "5", "cr\rhere", "6", "", "7", "", "8", " lead", "9", "日本語", "10", "tab\there", "11", ";semi", "12"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, true, false, false, false, false, false, false, false, false, false], 12, options, "text col,plain\nn,1\ntext col,\"with,comma\"\nn,2\ntext col,\"with \"\"quote\"\"\"\nn,3\ntext col,\"multi\nline\"\nn,4\ntext col,\"\\.\"\nn,5\ntext col,\"cr\rhere\"\nn,6\ntext col,\nn,7\ntext col,\nn,8\ntext col, lead\nn,9\ntext col,日本語\nn,10\ntext col,tab\there\nn,11\ntext col,;semi\nn,12\n");
    }

    {
        options: Print_Options;
        options.format = .CSV;
        options.expanded = .ON;
        check_query("csv expanded empty", string.["text col", "n"], u32.[25, 23], string.[], bool.[], 0, options, "");
    }

    {
        options: Print_Options;
        options.format = .CSV;
        options.expanded = .ON;
        options.tuples_only = true;
        check_query("csv expanded tuples only", string.["text col", "n"], u32.[25, 23], string.["plain", "1", "with,comma", "2", "with \"quote\"", "3"], bool.[false, false, false, false, false, false], 3, options, "text col,plain\nn,1\ntext col,\"with,comma\"\nn,2\ntext col,\"with \"\"quote\"\"\"\nn,3\n");
    }

    {
        options: Print_Options;
        options.format = .CSV;
        options.expanded = .ON;
        options.csv_field_separator = ";";
        check_query("csv expanded semicolon", string.["text col", "n"], u32.[25, 23], string.["plain", "1", "with,comma", "2", "with \"quote\"", "3"], bool.[false, false, false, false, false, false], 3, options, "text col;plain\nn;1\ntext col;with,comma\nn;2\ntext col;\"with \"\"quote\"\"\"\nn;3\n");
    }

    {
        options: Print_Options;
        options.format = .CSV;
        options.expanded = .AUTO;
        check_query("csv expanded auto", string.["text col", "n"], u32.[25, 23], string.["plain", "1", "with,comma", "2", "with \"quote\"", "3"], bool.[false, false, false, false, false, false], 3, options, "text col,n\nplain,1\n\"with,comma\",2\n\"with \"\"quote\"\"\",3\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.format = .WRAPPED;
        options.columns = 10;
        check_query("wrapped long border 0 columns 10", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "id                                  description                                   n   \n--- ---------------------------------------------------------------------------- ----\n  1 The quick brown fox jumps over the lazy dog and keeps running far away        1.5\n 22 short                                                                        2.25\n333 another reasonably long piece of text that will need to be wrapped somewhere    3\n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.format = .WRAPPED;
        options.columns = 20;
        check_query("wrapped long border 0 columns 20", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "id  description  n  \n-- ------------- --\n 1 The quick bro.1..\n   wn fox jumps . 5\n   over the lazy.\n    dog and keep.\n   s running far.\n    away         \n22 short         2..\n                 25\n33.another reaso. 3\n 3 nably long pi.\n   ece of text t.\n   hat will need.\n    to be wrappe.\n   d somewhere   \n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.format = .WRAPPED;
        options.columns = 30;
        check_query("wrapped long border 0 columns 30", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "id       description       n  \n-- ----------------------- --\n 1 The quick brown fox jum.1..\n   ps over the lazy dog an. 5\n   d keeps running far awa.\n   y                       \n22 short                   2..\n                           25\n33.another reasonably long. 3\n 3  piece of text that wil.\n   l need to be wrapped so.\n   mewhere                 \n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.format = .WRAPPED;
        options.columns = 40;
        check_query("wrapped long border 0 columns 40", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "id            description            n  \n-- --------------------------------- --\n 1 The quick brown fox jumps over th.1..\n   e lazy dog and keeps running far . 5\n   away                              \n22 short                             2..\n                                     25\n33.another reasonably long piece of . 3\n 3 text that will need to be wrapped.\n    somewhere                        \n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.format = .WRAPPED;
        options.columns = 60;
        check_query("wrapped long border 0 columns 60", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "id                      description                      n  \n--- --------------------------------------------------- ---\n  1 The quick brown fox jumps over the lazy dog and kee.1.5\n    ps running far away                                 \n 22 short                                               2.2.\n                                                          5\n333 another reasonably long piece of text that will nee.  3\n    d to be wrapped somewhere                           \n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.format = .WRAPPED;
        options.columns = 200;
        check_query("wrapped long border 0 columns 200", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "id                                  description                                   n   \n--- ---------------------------------------------------------------------------- ----\n  1 The quick brown fox jumps over the lazy dog and keeps running far away        1.5\n 22 short                                                                        2.25\n333 another reasonably long piece of text that will need to be wrapped somewhere    3\n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.columns = 40;
        check_query("aligned columns set border 0", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "id                                  description                                   n   \n--- ---------------------------------------------------------------------------- ----\n  1 The quick brown fox jumps over the lazy dog and keeps running far away        1.5\n 22 short                                                                        2.25\n333 another reasonably long piece of text that will need to be wrapped somewhere    3\n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.expanded = .AUTO;
        options.columns = 40;
        check_query("auto wide border 0", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "* Record 1                                                                             \nid          1\ndescription The quick brown fox jumps over the lazy dog and keeps running far away\nn           1.5\n* Record 2                                                                             \nid          22\ndescription short\nn           2.25\n* Record 3                                                                             \nid          333\ndescription another reasonably long piece of text that will need to be wrapped somewhere\nn           3\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.expanded = .AUTO;
        options.columns = 80;
        check_query("auto fits border 0", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "id  name   score  \n--- ----- -------\n  1 alice    10.5\n 22 bob          \n333            -7\n  4       1000.25\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.expanded = .AUTO;
        options.columns = 22;
        check_query("auto exact border 0", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "id  name   score  \n--- ----- -------\n  1 alice    10.5\n 22 bob          \n333            -7\n  4       1000.25\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.expanded = .AUTO;
        options.columns = 19;
        check_query("auto one below border 0", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "id  name   score  \n--- ----- -------\n  1 alice    10.5\n 22 bob          \n333            -7\n  4       1000.25\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.expanded = .AUTO;
        options.format = .WRAPPED;
        options.columns = 60;
        check_query("auto wrapped border 0", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "id                      description                      n  \n--- --------------------------------------------------- ---\n  1 The quick brown fox jumps over the lazy dog and kee.1.5\n    ps running far away                                 \n 22 short                                               2.2.\n                                                          5\n333 another reasonably long piece of text that will nee.  3\n    d to be wrapped somewhere                           \n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.expanded = .AUTO;
        options.format = .WRAPPED;
        options.columns = 20;
        check_query("auto wrapped narrow border 0", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "id  description  n  \n-- ------------- --\n 1 The quick bro.1..\n   wn fox jumps . 5\n   over the lazy.\n    dog and keep.\n   s running far.\n    away         \n22 short         2..\n                 25\n33.another reaso. 3\n 3 nably long pi.\n   ece of text t.\n   hat will need.\n    to be wrappe.\n   d somewhere   \n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.expanded = .ON;
        options.format = .WRAPPED;
        options.columns = 30;
        check_query("expanded wrapped border 0 columns 30", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "* Record 1                  \nid          1\ndescription The quick brown f.\n           .ox jumps over the.\n           . lazy dog and kee.\n           .ps running far aw.\n           .ay\nn           1.5\n* Record 2                  \nid          22\ndescription short\nn           2.25\n* Record 3                  \nid          333\ndescription another reasonabl.\n           .y long piece of t.\n           .ext that will nee.\n           .d to be wrapped s.\n           .omewhere\nn           3\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.expanded = .ON;
        options.format = .WRAPPED;
        options.columns = 20;
        check_query("expanded wrapped border 0 columns 20", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "* Record 1        \nid          1\ndescription The qui.\n           .ck brow.\n           .n fox j.\n           .umps ov.\n           .er the .\n           .lazy do.\n           .g and k.\n           .eeps ru.\n           .nning f.\n           .ar away\nn           1.5\n* Record 2        \nid          22\ndescription short\nn           2.25\n* Record 3        \nid          333\ndescription another.\n           . reason.\n           .ably lo.\n           .ng piec.\n           .e of te.\n           .xt that.\n           . will n.\n           .eed to .\n           .be wrap.\n           .ped som.\n           .ewhere\nn           3\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.expanded = .ON;
        options.format = .WRAPPED;
        options.columns = 5;
        check_query("expanded wrapped border 0 columns 5", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "* Record 1    \nid          1\ndescription The.\n           . qu.\n           .ick.\n           . br.\n           .own.\n           . fo.\n           .x j.\n           .ump.\n           .s o.\n           .ver.\n           . th.\n           .e l.\n           .azy.\n           . do.\n           .g a.\n           .nd .\n           .kee.\n           .ps .\n           .run.\n           .nin.\n           .g f.\n           .ar .\n           .awa.\n           .y\nn           1.5\n* Record 2    \nid          22\ndescription sho.\n           .rt\nn           2.2.\n           .5\n* Record 3    \nid          333\ndescription ano.\n           .the.\n           .r r.\n           .eas.\n           .ona.\n           .bly.\n           . lo.\n           .ng .\n           .pie.\n           .ce .\n           .of .\n           .tex.\n           .t t.\n           .hat.\n           . wi.\n           .ll .\n           .nee.\n           .d t.\n           .o b.\n           .e w.\n           .rap.\n           .ped.\n           . so.\n           .mew.\n           .her.\n           .e\nn           3\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.expanded = .ON;
        options.format = .WRAPPED;
        options.columns = 200;
        check_query("expanded wrapped fits border 0", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "* Record 1                                                                             \nid          1\ndescription The quick brown fox jumps over the lazy dog and keeps running far away\nn           1.5\n* Record 2                                                                             \nid          22\ndescription short\nn           2.25\n* Record 3                                                                             \nid          333\ndescription another reasonably long piece of text that will need to be wrapped somewhere\nn           3\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.expanded = .ON;
        options.format = .WRAPPED;
        options.columns = 12;
        check_query("expanded wrapped multiline border 0", string.["id", "body", "n"], u32.[23, 25, 1700], string.["1", "line one\nline two\nthree", "5", "2", "single", "6.5", "3", "ends\n", "7", "4", "\nstarts", "8", "5", "a\n\nb", "9"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 5, options, "* Record 1\nid   1\nbody line o.\n    .ne    +\n     line t.\n    .wo    +\n     three\nn    5\n* Record 2\nid   2\nbody single\nn    6.5\n* Record 3\nid   3\nbody ends  +\n     \nn    7\n* Record 4\nid   4\nbody       +\n     starts\nn    8\n* Record 5\nid   5\nbody a     +\n           +\n     b\nn    9\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.format = .WRAPPED;
        options.columns = 12;
        check_query("wrapped multiline border 0", string.["id", "body", "n"], u32.[23, 25, 1700], string.["1", "line one\nline two\nthree", "5", "2", "single", "6.5", "3", "ends\n", "7", "4", "\nstarts", "8", "5", "a\n\nb", "9"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 5, options, "id  body  n \n-- ------ -\n 1 line o.5\n   ne    +\n   line t.\n   wo    +\n   three  \n 2 single 6.\n          ..\n          5\n 3 ends  +7\n          \n 4       +8\n   starts \n 5 a     +9\n         +\n   b      \n(5 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.format = .WRAPPED;
        options.columns = 14;
        check_query("wrapped unicode border 0", string.["t", "u"], u32.[25, 25], string.["日本語日本語日本語日本語", "ab", "x", "y"], bool.[false, false, false, false], 2, options, "     t      u \n----------- -\n日本語日本 .a.\n語日本語日 .b\n本語        \nx           y\n(2 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.format = .WRAPPED;
        options.columns = 5;
        check_query("wrapped narrow border 0", string.["a", "b"], u32.[25, 25], string.["abcdefghijklmnopqrstuvwxyz", "0123456789"], bool.[false, false], 1, options, "a b  \n- --\na.01.\nb.23.\nc.45.\nd.67.\ne.89\nf.\ng.\nh.\ni.\nj.\nk.\nl.\nm.\nn.\no.\np.\nq.\nr.\ns.\nt.\nu.\nv.\nw.\nx.\ny.\nz \n(1 row)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.format = .WRAPPED;
        options.columns = 40;
        options.tuples_only = true;
        check_query("wrapped tuples only border 0", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, " 1 The quick brown fox jumps over th.1..\n   e lazy dog and keeps running far . 5\n   away                              \n22 short                             2..\n                                     25\n33.another reasonably long piece of . 3\n 3 text that will need to be wrapped.\n    somewhere                        \n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.format = .WRAPPED;
        options.columns = 40;
        options.linestyle = .UNICODE;
        check_query("wrapped unicode style border 0", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "id            description            n  \n── ───────────────────────────────── ──\n 1 The quick brown fox jumps over th…1.…\n   e lazy dog and keeps running far … 5\n   away                              \n22 short                             2.…\n                                     25\n33…another reasonably long piece of … 3\n 3 text that will need to be wrapped…\n    somewhere                        \n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.format = .WRAPPED;
        options.columns = 40;
        options.linestyle = .OLD_ASCII;
        check_query("wrapped old-ascii border 0", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "id            description            n \n-- --------------------------------- --\n 1 The quick brown fox jumps over th 1. \n   e lazy dog and keeps running far   5\n   away                              \n22 short                             2. \n                                     25\n33 another reasonably long piece of   3\n 3 text that will need to be wrapped \n    somewhere                        \n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.format = .WRAPPED;
        check_query("wrapped no columns border 0", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "id                                  description                                   n   \n--- ---------------------------------------------------------------------------- ----\n  1 The quick brown fox jumps over the lazy dog and keeps running far away        1.5\n 22 short                                                                        2.25\n333 another reasonably long piece of text that will need to be wrapped somewhere    3\n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.format = .WRAPPED;
        options.columns = 40;
        options.title = "Wrapped table";
        check_query("wrapped title border 0", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "             Wrapped table\nid            description            n  \n-- --------------------------------- --\n 1 The quick brown fox jumps over th.1..\n   e lazy dog and keeps running far . 5\n   away                              \n22 short                             2..\n                                     25\n33.another reasonably long piece of . 3\n 3 text that will need to be wrapped.\n    somewhere                        \n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.format = .WRAPPED;
        options.columns = 10;
        check_query("wrapped long border 1 columns 10", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, " id  |                                 description                                  |  n   \n-----+------------------------------------------------------------------------------+------\n   1 | The quick brown fox jumps over the lazy dog and keeps running far away       |  1.5\n  22 | short                                                                        | 2.25\n 333 | another reasonably long piece of text that will need to be wrapped somewhere |    3\n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.format = .WRAPPED;
        options.columns = 20;
        check_query("wrapped long border 1 columns 20", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, " id  |                                 description                                  |  n   \n-----+------------------------------------------------------------------------------+------\n   1 | The quick brown fox jumps over the lazy dog and keeps running far away       |  1.5\n  22 | short                                                                        | 2.25\n 333 | another reasonably long piece of text that will need to be wrapped somewhere |    3\n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.format = .WRAPPED;
        options.columns = 30;
        check_query("wrapped long border 1 columns 30", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, " id |    description     | n  \n----+--------------------+----\n  1 | The quick brown fo.| 1..\n    |.x jumps over the l.|. 5\n    |.azy dog and keeps .| \n    |.running far away   | \n 22 | short              | 2..\n    |                    |.25\n 33.| another reasonably.|  3\n. 3 |. long piece of tex.| \n    |.t that will need t.| \n    |.o be wrapped somew.| \n    |.here               | \n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.format = .WRAPPED;
        options.columns = 40;
        check_query("wrapped long border 1 columns 40", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, " id |         description          | n  \n----+------------------------------+----\n  1 | The quick brown fox jumps ov.| 1..\n    |.er the lazy dog and keeps ru.|. 5\n    |.nning far away               | \n 22 | short                        | 2..\n    |                              |.25\n 33.| another reasonably long piec.|  3\n. 3 |.e of text that will need to .| \n    |.be wrapped somewhere         | \n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.format = .WRAPPED;
        options.columns = 60;
        check_query("wrapped long border 1 columns 60", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, " id  |                  description                   |  n  \n-----+------------------------------------------------+-----\n   1 | The quick brown fox jumps over the lazy dog an.| 1.5\n     |.d keeps running far away                       | \n  22 | short                                          | 2.2.\n     |                                                |.  5\n 333 | another reasonably long piece of text that wil.|   3\n     |.l need to be wrapped somewhere                 | \n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.format = .WRAPPED;
        options.columns = 200;
        check_query("wrapped long border 1 columns 200", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, " id  |                                 description                                  |  n   \n-----+------------------------------------------------------------------------------+------\n   1 | The quick brown fox jumps over the lazy dog and keeps running far away       |  1.5\n  22 | short                                                                        | 2.25\n 333 | another reasonably long piece of text that will need to be wrapped somewhere |    3\n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.columns = 40;
        check_query("aligned columns set border 1", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, " id  |                                 description                                  |  n   \n-----+------------------------------------------------------------------------------+------\n   1 | The quick brown fox jumps over the lazy dog and keeps running far away       |  1.5\n  22 | short                                                                        | 2.25\n 333 | another reasonably long piece of text that will need to be wrapped somewhere |    3\n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.expanded = .AUTO;
        options.columns = 40;
        check_query("auto wide border 1", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "-[ RECORD 1 ]-----------------------------------------------------------------------------\nid          | 1\ndescription | The quick brown fox jumps over the lazy dog and keeps running far away\nn           | 1.5\n-[ RECORD 2 ]-----------------------------------------------------------------------------\nid          | 22\ndescription | short\nn           | 2.25\n-[ RECORD 3 ]-----------------------------------------------------------------------------\nid          | 333\ndescription | another reasonably long piece of text that will need to be wrapped somewhere\nn           | 3\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.expanded = .AUTO;
        options.columns = 80;
        check_query("auto fits border 1", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, " id  | name  |  score  \n-----+-------+---------\n   1 | alice |    10.5\n  22 | bob   |        \n 333 |       |      -7\n   4 |       | 1000.25\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.expanded = .AUTO;
        options.columns = 22;
        check_query("auto exact border 1", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "-[ RECORD 1 ]--\nid    | 1\nname  | alice\nscore | 10.5\n-[ RECORD 2 ]--\nid    | 22\nname  | bob\nscore | \n-[ RECORD 3 ]--\nid    | 333\nname  | \nscore | -7\n-[ RECORD 4 ]--\nid    | 4\nname  | \nscore | 1000.25\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.expanded = .AUTO;
        options.columns = 19;
        check_query("auto one below border 1", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "-[ RECORD 1 ]--\nid    | 1\nname  | alice\nscore | 10.5\n-[ RECORD 2 ]--\nid    | 22\nname  | bob\nscore | \n-[ RECORD 3 ]--\nid    | 333\nname  | \nscore | -7\n-[ RECORD 4 ]--\nid    | 4\nname  | \nscore | 1000.25\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.expanded = .AUTO;
        options.format = .WRAPPED;
        options.columns = 60;
        check_query("auto wrapped border 1", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, " id  |                  description                   |  n  \n-----+------------------------------------------------+-----\n   1 | The quick brown fox jumps over the lazy dog an.| 1.5\n     |.d keeps running far away                       | \n  22 | short                                          | 2.2.\n     |                                                |.  5\n 333 | another reasonably long piece of text that wil.|   3\n     |.l need to be wrapped somewhere                 | \n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.expanded = .AUTO;
        options.format = .WRAPPED;
        options.columns = 20;
        check_query("auto wrapped narrow border 1", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "-[ RECORD 1 ]------\nid          | 1\ndescription | The q.\n            |.uick .\n            |.brown.\n            |. fox .\n            |.jumps.\n            |. over.\n            |. the .\n            |.lazy .\n            |.dog a.\n            |.nd ke.\n            |.eps r.\n            |.unnin.\n            |.g far.\n            |. away\nn           | 1.5\n-[ RECORD 2 ]------\nid          | 22\ndescription | short\nn           | 2.25\n-[ RECORD 3 ]------\nid          | 333\ndescription | anoth.\n            |.er re.\n            |.asona.\n            |.bly l.\n            |.ong p.\n            |.iece .\n            |.of te.\n            |.xt th.\n            |.at wi.\n            |.ll ne.\n            |.ed to.\n            |. be w.\n            |.rappe.\n            |.d som.\n            |.ewher.\n            |.e\nn           | 3\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.expanded = .ON;
        options.format = .WRAPPED;
        options.columns = 30;
        check_query("expanded wrapped border 1 columns 30", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "-[ RECORD 1 ]----------------\nid          | 1\ndescription | The quick brown.\n            |. fox jumps over.\n            |. the lazy dog a.\n            |.nd keeps runnin.\n            |.g far away\nn           | 1.5\n-[ RECORD 2 ]----------------\nid          | 22\ndescription | short\nn           | 2.25\n-[ RECORD 3 ]----------------\nid          | 333\ndescription | another reasona.\n            |.bly long piece .\n            |.of text that wi.\n            |.ll need to be w.\n            |.rapped somewher.\n            |.e\nn           | 3\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.expanded = .ON;
        options.format = .WRAPPED;
        options.columns = 20;
        check_query("expanded wrapped border 1 columns 20", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "-[ RECORD 1 ]------\nid          | 1\ndescription | The q.\n            |.uick .\n            |.brown.\n            |. fox .\n            |.jumps.\n            |. over.\n            |. the .\n            |.lazy .\n            |.dog a.\n            |.nd ke.\n            |.eps r.\n            |.unnin.\n            |.g far.\n            |. away\nn           | 1.5\n-[ RECORD 2 ]------\nid          | 22\ndescription | short\nn           | 2.25\n-[ RECORD 3 ]------\nid          | 333\ndescription | anoth.\n            |.er re.\n            |.asona.\n            |.bly l.\n            |.ong p.\n            |.iece .\n            |.of te.\n            |.xt th.\n            |.at wi.\n            |.ll ne.\n            |.ed to.\n            |. be w.\n            |.rappe.\n            |.d som.\n            |.ewher.\n            |.e\nn           | 3\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.expanded = .ON;
        options.format = .WRAPPED;
        options.columns = 5;
        check_query("expanded wrapped border 1 columns 5", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "-[ RECORD 1 ]----\nid          | 1\ndescription | The.\n            |. qu.\n            |.ick.\n            |. br.\n            |.own.\n            |. fo.\n            |.x j.\n            |.ump.\n            |.s o.\n            |.ver.\n            |. th.\n            |.e l.\n            |.azy.\n            |. do.\n            |.g a.\n            |.nd .\n            |.kee.\n            |.ps .\n            |.run.\n            |.nin.\n            |.g f.\n            |.ar .\n            |.awa.\n            |.y\nn           | 1.5\n-[ RECORD 2 ]----\nid          | 22\ndescription | sho.\n            |.rt\nn           | 2.2.\n            |.5\n-[ RECORD 3 ]----\nid          | 333\ndescription | ano.\n            |.the.\n            |.r r.\n            |.eas.\n            |.ona.\n            |.bly.\n            |. lo.\n            |.ng .\n            |.pie.\n            |.ce .\n            |.of .\n            |.tex.\n            |.t t.\n            |.hat.\n            |. wi.\n            |.ll .\n            |.nee.\n            |.d t.\n            |.o b.\n            |.e w.\n            |.rap.\n            |.ped.\n            |. so.\n            |.mew.\n            |.her.\n            |.e\nn           | 3\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.expanded = .ON;
        options.format = .WRAPPED;
        options.columns = 200;
        check_query("expanded wrapped fits border 1", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "-[ RECORD 1 ]-----------------------------------------------------------------------------\nid          | 1\ndescription | The quick brown fox jumps over the lazy dog and keeps running far away\nn           | 1.5\n-[ RECORD 2 ]-----------------------------------------------------------------------------\nid          | 22\ndescription | short\nn           | 2.25\n-[ RECORD 3 ]-----------------------------------------------------------------------------\nid          | 333\ndescription | another reasonably long piece of text that will need to be wrapped somewhere\nn           | 3\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.expanded = .ON;
        options.format = .WRAPPED;
        options.columns = 12;
        check_query("expanded wrapped multiline border 1", string.["id", "body", "n"], u32.[23, 25, 1700], string.["1", "line one\nline two\nthree", "5", "2", "single", "6.5", "3", "ends\n", "7", "4", "\nstarts", "8", "5", "a\n\nb", "9"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 5, options, "-[ RECORD 1 ]\nid   | 1\nbody | line .\n     |.one  +\n     | line .\n     |.two  +\n     | three\nn    | 5\n-[ RECORD 2 ]\nid   | 2\nbody | singl.\n     |.e\nn    | 6.5\n-[ RECORD 3 ]\nid   | 3\nbody | ends +\n     | \nn    | 7\n-[ RECORD 4 ]\nid   | 4\nbody |      +\n     | start.\n     |.s\nn    | 8\n-[ RECORD 5 ]\nid   | 5\nbody | a    +\n     |      +\n     | b\nn    | 9\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.format = .WRAPPED;
        options.columns = 12;
        check_query("wrapped multiline border 1", string.["id", "body", "n"], u32.[23, 25, 1700], string.["1", "line one\nline two\nthree", "5", "2", "single", "6.5", "3", "ends\n", "7", "4", "\nstarts", "8", "5", "a\n\nb", "9"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 5, options, " id |   body   |  n  \n----+----------+-----\n  1 | line one+|   5\n    | line two+| \n    | three    | \n  2 | single   | 6.5\n  3 | ends    +|   7\n    |          | \n  4 |         +|   8\n    | starts   | \n  5 | a       +|   9\n    |         +| \n    | b        | \n(5 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.format = .WRAPPED;
        options.columns = 14;
        check_query("wrapped unicode border 1", string.["t", "u"], u32.[25, 25], string.["日本語日本語日本語日本語", "ab", "x", "y"], bool.[false, false, false, false], 2, options, "    t     | u \n----------+---\n 日本語日.| a.\n.本語日本.|.b\n.語日本語 | \n x        | y\n(2 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.format = .WRAPPED;
        options.columns = 5;
        check_query("wrapped narrow border 1", string.["a", "b"], u32.[25, 25], string.["abcdefghijklmnopqrstuvwxyz", "0123456789"], bool.[false, false], 1, options, "             a              |     b      \n----------------------------+------------\n abcdefghijklmnopqrstuvwxyz | 0123456789\n(1 row)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.format = .WRAPPED;
        options.columns = 40;
        options.tuples_only = true;
        check_query("wrapped tuples only border 1", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "  1 | The quick brown fox jumps ov.| 1..\n    |.er the lazy dog and keeps ru.|. 5\n    |.nning far away               | \n 22 | short                        | 2..\n    |                              |.25\n 33.| another reasonably long piec.|  3\n. 3 |.e of text that will need to .| \n    |.be wrapped somewhere         | \n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.format = .WRAPPED;
        options.columns = 40;
        options.linestyle = .UNICODE;
        check_query("wrapped unicode style border 1", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, " id │         description          │ n  \n────┼──────────────────────────────┼────\n  1 │ The quick brown fox jumps ov…│ 1.…\n    │…er the lazy dog and keeps ru…│… 5\n    │…nning far away               │ \n 22 │ short                        │ 2.…\n    │                              │…25\n 33…│ another reasonably long piec…│  3\n… 3 │…e of text that will need to …│ \n    │…be wrapped somewhere         │ \n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.format = .WRAPPED;
        options.columns = 40;
        options.linestyle = .OLD_ASCII;
        check_query("wrapped old-ascii border 1", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, " id |         description          | n  \n----+------------------------------+----\n  1 | The quick brown fox jumps ov | 1. \n    ; er the lazy dog and keeps ru ;  5\n    ; nning far away                 \n 22 | short                        | 2. \n                                   ; 25\n 33 | another reasonably long piec |  3\n  3 ; e of text that will need to    \n    ; be wrapped somewhere           \n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.format = .WRAPPED;
        check_query("wrapped no columns border 1", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, " id  |                                 description                                  |  n   \n-----+------------------------------------------------------------------------------+------\n   1 | The quick brown fox jumps over the lazy dog and keeps running far away       |  1.5\n  22 | short                                                                        | 2.25\n 333 | another reasonably long piece of text that will need to be wrapped somewhere |    3\n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.format = .WRAPPED;
        options.columns = 40;
        options.title = "Wrapped table";
        check_query("wrapped title border 1", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "             Wrapped table\n id |         description          | n  \n----+------------------------------+----\n  1 | The quick brown fox jumps ov.| 1..\n    |.er the lazy dog and keeps ru.|. 5\n    |.nning far away               | \n 22 | short                        | 2..\n    |                              |.25\n 33.| another reasonably long piec.|  3\n. 3 |.e of text that will need to .| \n    |.be wrapped somewhere         | \n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.format = .WRAPPED;
        options.columns = 10;
        check_query("wrapped long border 2 columns 10", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "+-----+------------------------------------------------------------------------------+------+\n| id  |                                 description                                  |  n   |\n+-----+------------------------------------------------------------------------------+------+\n|   1 | The quick brown fox jumps over the lazy dog and keeps running far away       |  1.5 |\n|  22 | short                                                                        | 2.25 |\n| 333 | another reasonably long piece of text that will need to be wrapped somewhere |    3 |\n+-----+------------------------------------------------------------------------------+------+\n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.format = .WRAPPED;
        options.columns = 20;
        check_query("wrapped long border 2 columns 20", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "+-----+------------------------------------------------------------------------------+------+\n| id  |                                 description                                  |  n   |\n+-----+------------------------------------------------------------------------------+------+\n|   1 | The quick brown fox jumps over the lazy dog and keeps running far away       |  1.5 |\n|  22 | short                                                                        | 2.25 |\n| 333 | another reasonably long piece of text that will need to be wrapped somewhere |    3 |\n+-----+------------------------------------------------------------------------------+------+\n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.format = .WRAPPED;
        options.columns = 30;
        check_query("wrapped long border 2 columns 30", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "+----+------------------+----+\n| id |   description    | n  |\n+----+------------------+----+\n|  1 | The quick brown .| 1..|\n|    |.fox jumps over t.|. 5 |\n|    |.he lazy dog and .|    |\n|    |.keeps running fa.|    |\n|    |.r away           |    |\n| 22 | short            | 2..|\n|    |                  |.25 |\n| 33.| another reasonab.|  3 |\n|. 3 |.ly long piece of.|    |\n|    |. text that will .|    |\n|    |.need to be wrapp.|    |\n|    |.ed somewhere     |    |\n+----+------------------+----+\n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.format = .WRAPPED;
        options.columns = 40;
        check_query("wrapped long border 2 columns 40", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "+----+----------------------------+----+\n| id |        description         | n  |\n+----+----------------------------+----+\n|  1 | The quick brown fox jumps .| 1..|\n|    |.over the lazy dog and keep.|. 5 |\n|    |.s running far away         |    |\n| 22 | short                      | 2..|\n|    |                            |.25 |\n| 33.| another reasonably long pi.|  3 |\n|. 3 |.ece of text that will need.|    |\n|    |. to be wrapped somewhere   |    |\n+----+----------------------------+----+\n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.format = .WRAPPED;
        options.columns = 60;
        check_query("wrapped long border 2 columns 60", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "+-----+----------------------------------------------+-----+\n| id  |                 description                  |  n  |\n+-----+----------------------------------------------+-----+\n|   1 | The quick brown fox jumps over the lazy dog .| 1.5 |\n|     |.and keeps running far away                   |     |\n|  22 | short                                        | 2.2.|\n|     |                                              |.  5 |\n| 333 | another reasonably long piece of text that w.|   3 |\n|     |.ill need to be wrapped somewhere             |     |\n+-----+----------------------------------------------+-----+\n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.format = .WRAPPED;
        options.columns = 200;
        check_query("wrapped long border 2 columns 200", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "+-----+------------------------------------------------------------------------------+------+\n| id  |                                 description                                  |  n   |\n+-----+------------------------------------------------------------------------------+------+\n|   1 | The quick brown fox jumps over the lazy dog and keeps running far away       |  1.5 |\n|  22 | short                                                                        | 2.25 |\n| 333 | another reasonably long piece of text that will need to be wrapped somewhere |    3 |\n+-----+------------------------------------------------------------------------------+------+\n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.columns = 40;
        check_query("aligned columns set border 2", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "+-----+------------------------------------------------------------------------------+------+\n| id  |                                 description                                  |  n   |\n+-----+------------------------------------------------------------------------------+------+\n|   1 | The quick brown fox jumps over the lazy dog and keeps running far away       |  1.5 |\n|  22 | short                                                                        | 2.25 |\n| 333 | another reasonably long piece of text that will need to be wrapped somewhere |    3 |\n+-----+------------------------------------------------------------------------------+------+\n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.expanded = .AUTO;
        options.columns = 40;
        check_query("auto wide border 2", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "+-[ RECORD 1 ]+------------------------------------------------------------------------------+\n| id          | 1                                                                            |\n| description | The quick brown fox jumps over the lazy dog and keeps running far away       |\n| n           | 1.5                                                                          |\n+-[ RECORD 2 ]+------------------------------------------------------------------------------+\n| id          | 22                                                                           |\n| description | short                                                                        |\n| n           | 2.25                                                                         |\n+-[ RECORD 3 ]+------------------------------------------------------------------------------+\n| id          | 333                                                                          |\n| description | another reasonably long piece of text that will need to be wrapped somewhere |\n| n           | 3                                                                            |\n+-------------+------------------------------------------------------------------------------+\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.expanded = .AUTO;
        options.columns = 80;
        check_query("auto fits border 2", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "+-----+-------+---------+\n| id  | name  |  score  |\n+-----+-------+---------+\n|   1 | alice |    10.5 |\n|  22 | bob   |         |\n| 333 |       |      -7 |\n|   4 |       | 1000.25 |\n+-----+-------+---------+\n(4 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.expanded = .AUTO;
        options.columns = 22;
        check_query("auto exact border 2", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "+-[ RECORD 1 ]----+\n| id    | 1       |\n| name  | alice   |\n| score | 10.5    |\n+-[ RECORD 2 ]----+\n| id    | 22      |\n| name  | bob     |\n| score |         |\n+-[ RECORD 3 ]----+\n| id    | 333     |\n| name  |         |\n| score | -7      |\n+-[ RECORD 4 ]----+\n| id    | 4       |\n| name  |         |\n| score | 1000.25 |\n+-------+---------+\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.expanded = .AUTO;
        options.columns = 19;
        check_query("auto one below border 2", string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, "+-[ RECORD 1 ]----+\n| id    | 1       |\n| name  | alice   |\n| score | 10.5    |\n+-[ RECORD 2 ]----+\n| id    | 22      |\n| name  | bob     |\n| score |         |\n+-[ RECORD 3 ]----+\n| id    | 333     |\n| name  |         |\n| score | -7      |\n+-[ RECORD 4 ]----+\n| id    | 4       |\n| name  |         |\n| score | 1000.25 |\n+-------+---------+\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.expanded = .AUTO;
        options.format = .WRAPPED;
        options.columns = 60;
        check_query("auto wrapped border 2", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "+-----+----------------------------------------------+-----+\n| id  |                 description                  |  n  |\n+-----+----------------------------------------------+-----+\n|   1 | The quick brown fox jumps over the lazy dog .| 1.5 |\n|     |.and keeps running far away                   |     |\n|  22 | short                                        | 2.2.|\n|     |                                              |.  5 |\n| 333 | another reasonably long piece of text that w.|   3 |\n|     |.ill need to be wrapped somewhere             |     |\n+-----+----------------------------------------------+-----+\n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.expanded = .AUTO;
        options.format = .WRAPPED;
        options.columns = 20;
        check_query("auto wrapped narrow border 2", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "+-[ RECORD 1 ]+-----+\n| id          | 1   |\n| description | The.|\n|             |. qu.|\n|             |.ick.|\n|             |. br.|\n|             |.own.|\n|             |. fo.|\n|             |.x j.|\n|             |.ump.|\n|             |.s o.|\n|             |.ver.|\n|             |. th.|\n|             |.e l.|\n|             |.azy.|\n|             |. do.|\n|             |.g a.|\n|             |.nd .|\n|             |.kee.|\n|             |.ps .|\n|             |.run.|\n|             |.nin.|\n|             |.g f.|\n|             |.ar .|\n|             |.awa.|\n|             |.y   |\n| n           | 1.5 |\n+-[ RECORD 2 ]+-----+\n| id          | 22  |\n| description | sho.|\n|             |.rt  |\n| n           | 2.2.|\n|             |.5   |\n+-[ RECORD 3 ]+-----+\n| id          | 333 |\n| description | ano.|\n|             |.the.|\n|             |.r r.|\n|             |.eas.|\n|             |.ona.|\n|             |.bly.|\n|             |. lo.|\n|             |.ng .|\n|             |.pie.|\n|             |.ce .|\n|             |.of .|\n|             |.tex.|\n|             |.t t.|\n|             |.hat.|\n|             |. wi.|\n|             |.ll .|\n|             |.nee.|\n|             |.d t.|\n|             |.o b.|\n|             |.e w.|\n|             |.rap.|\n|             |.ped.|\n|             |. so.|\n|             |.mew.|\n|             |.her.|\n|             |.e   |\n| n           | 3   |\n+-------------+-----+\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.expanded = .ON;
        options.format = .WRAPPED;
        options.columns = 30;
        check_query("expanded wrapped border 2 columns 30", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "+-[ RECORD 1 ]+--------------+\n| id          | 1            |\n| description | The quick br.|\n|             |.own fox jump.|\n|             |.s over the l.|\n|             |.azy dog and .|\n|             |.keeps runnin.|\n|             |.g far away   |\n| n           | 1.5          |\n+-[ RECORD 2 ]+--------------+\n| id          | 22           |\n| description | short        |\n| n           | 2.25         |\n+-[ RECORD 3 ]+--------------+\n| id          | 333          |\n| description | another reas.|\n|             |.onably long .|\n|             |.piece of tex.|\n|             |.t that will .|\n|             |.need to be w.|\n|             |.rapped somew.|\n|             |.here         |\n| n           | 3            |\n+-------------+--------------+\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.expanded = .ON;
        options.format = .WRAPPED;
        options.columns = 20;
        check_query("expanded wrapped border 2 columns 20", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "+-[ RECORD 1 ]+-----+\n| id          | 1   |\n| description | The.|\n|             |. qu.|\n|             |.ick.|\n|             |. br.|\n|             |.own.|\n|             |. fo.|\n|             |.x j.|\n|             |.ump.|\n|             |.s o.|\n|             |.ver.|\n|             |. th.|\n|             |.e l.|\n|             |.azy.|\n|             |. do.|\n|             |.g a.|\n|             |.nd .|\n|             |.kee.|\n|             |.ps .|\n|             |.run.|\n|             |.nin.|\n|             |.g f.|\n|             |.ar .|\n|             |.awa.|\n|             |.y   |\n| n           | 1.5 |\n+-[ RECORD 2 ]+-----+\n| id          | 22  |\n| description | sho.|\n|             |.rt  |\n| n           | 2.2.|\n|             |.5   |\n+-[ RECORD 3 ]+-----+\n| id          | 333 |\n| description | ano.|\n|             |.the.|\n|             |.r r.|\n|             |.eas.|\n|             |.ona.|\n|             |.bly.|\n|             |. lo.|\n|             |.ng .|\n|             |.pie.|\n|             |.ce .|\n|             |.of .|\n|             |.tex.|\n|             |.t t.|\n|             |.hat.|\n|             |. wi.|\n|             |.ll .|\n|             |.nee.|\n|             |.d t.|\n|             |.o b.|\n|             |.e w.|\n|             |.rap.|\n|             |.ped.|\n|             |. so.|\n|             |.mew.|\n|             |.her.|\n|             |.e   |\n| n           | 3   |\n+-------------+-----+\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.expanded = .ON;
        options.format = .WRAPPED;
        options.columns = 5;
        check_query("expanded wrapped border 2 columns 5", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "+-[ RECORD 1 ]+-----+\n| id          | 1   |\n| description | The.|\n|             |. qu.|\n|             |.ick.|\n|             |. br.|\n|             |.own.|\n|             |. fo.|\n|             |.x j.|\n|             |.ump.|\n|             |.s o.|\n|             |.ver.|\n|             |. th.|\n|             |.e l.|\n|             |.azy.|\n|             |. do.|\n|             |.g a.|\n|             |.nd .|\n|             |.kee.|\n|             |.ps .|\n|             |.run.|\n|             |.nin.|\n|             |.g f.|\n|             |.ar .|\n|             |.awa.|\n|             |.y   |\n| n           | 1.5 |\n+-[ RECORD 2 ]+-----+\n| id          | 22  |\n| description | sho.|\n|             |.rt  |\n| n           | 2.2.|\n|             |.5   |\n+-[ RECORD 3 ]+-----+\n| id          | 333 |\n| description | ano.|\n|             |.the.|\n|             |.r r.|\n|             |.eas.|\n|             |.ona.|\n|             |.bly.|\n|             |. lo.|\n|             |.ng .|\n|             |.pie.|\n|             |.ce .|\n|             |.of .|\n|             |.tex.|\n|             |.t t.|\n|             |.hat.|\n|             |. wi.|\n|             |.ll .|\n|             |.nee.|\n|             |.d t.|\n|             |.o b.|\n|             |.e w.|\n|             |.rap.|\n|             |.ped.|\n|             |. so.|\n|             |.mew.|\n|             |.her.|\n|             |.e   |\n| n           | 3   |\n+-------------+-----+\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.expanded = .ON;
        options.format = .WRAPPED;
        options.columns = 200;
        check_query("expanded wrapped fits border 2", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "+-[ RECORD 1 ]+------------------------------------------------------------------------------+\n| id          | 1                                                                            |\n| description | The quick brown fox jumps over the lazy dog and keeps running far away       |\n| n           | 1.5                                                                          |\n+-[ RECORD 2 ]+------------------------------------------------------------------------------+\n| id          | 22                                                                           |\n| description | short                                                                        |\n| n           | 2.25                                                                         |\n+-[ RECORD 3 ]+------------------------------------------------------------------------------+\n| id          | 333                                                                          |\n| description | another reasonably long piece of text that will need to be wrapped somewhere |\n| n           | 3                                                                            |\n+-------------+------------------------------------------------------------------------------+\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.expanded = .ON;
        options.format = .WRAPPED;
        options.columns = 12;
        check_query("expanded wrapped multiline border 2", string.["id", "body", "n"], u32.[23, 25, 1700], string.["1", "line one\nline two\nthree", "5", "2", "single", "6.5", "3", "ends\n", "7", "4", "\nstarts", "8", "5", "a\n\nb", "9"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 5, options, "+-[ RECORD 1 ]-+\n| id   | 1     |\n| body | line .|\n|      |.one  +|\n|      | line .|\n|      |.two  +|\n|      | three |\n| n    | 5     |\n+-[ RECORD 2 ]-+\n| id   | 2     |\n| body | singl.|\n|      |.e     |\n| n    | 6.5   |\n+-[ RECORD 3 ]-+\n| id   | 3     |\n| body | ends +|\n|      |       |\n| n    | 7     |\n+-[ RECORD 4 ]-+\n| id   | 4     |\n| body |      +|\n|      | start.|\n|      |.s     |\n| n    | 8     |\n+-[ RECORD 5 ]-+\n| id   | 5     |\n| body | a    +|\n|      |      +|\n|      | b     |\n| n    | 9     |\n+------+-------+\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.format = .WRAPPED;
        options.columns = 12;
        check_query("wrapped multiline border 2", string.["id", "body", "n"], u32.[23, 25, 1700], string.["1", "line one\nline two\nthree", "5", "2", "single", "6.5", "3", "ends\n", "7", "4", "\nstarts", "8", "5", "a\n\nb", "9"], bool.[false, false, false, false, false, false, false, false, false, false, false, false, false, false, false], 5, options, "+----+----------+-----+\n| id |   body   |  n  |\n+----+----------+-----+\n|  1 | line one+|   5 |\n|    | line two+|     |\n|    | three    |     |\n|  2 | single   | 6.5 |\n|  3 | ends    +|   7 |\n|    |          |     |\n|  4 |         +|   8 |\n|    | starts   |     |\n|  5 | a       +|   9 |\n|    |         +|     |\n|    | b        |     |\n+----+----------+-----+\n(5 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.format = .WRAPPED;
        options.columns = 14;
        check_query("wrapped unicode border 2", string.["t", "u"], u32.[25, 25], string.["日本語日本語日本語日本語", "ab", "x", "y"], bool.[false, false, false, false], 2, options, "+--------+---+\n|   t    | u |\n+--------+---+\n| 日本語.| a.|\n|.日本語.|.b |\n|.日本語.|   |\n|.日本語 |   |\n| x      | y |\n+--------+---+\n(2 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.format = .WRAPPED;
        options.columns = 5;
        check_query("wrapped narrow border 2", string.["a", "b"], u32.[25, 25], string.["abcdefghijklmnopqrstuvwxyz", "0123456789"], bool.[false, false], 1, options, "+----------------------------+------------+\n|             a              |     b      |\n+----------------------------+------------+\n| abcdefghijklmnopqrstuvwxyz | 0123456789 |\n+----------------------------+------------+\n(1 row)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.format = .WRAPPED;
        options.columns = 40;
        options.tuples_only = true;
        check_query("wrapped tuples only border 2", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "|  1 | The quick brown fox jumps .| 1..|\n|    |.over the lazy dog and keep.|. 5 |\n|    |.s running far away         |    |\n| 22 | short                      | 2..|\n|    |                            |.25 |\n| 33.| another reasonably long pi.|  3 |\n|. 3 |.ece of text that will need.|    |\n|    |. to be wrapped somewhere   |    |\n+----+----------------------------+----+\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.format = .WRAPPED;
        options.columns = 40;
        options.linestyle = .UNICODE;
        check_query("wrapped unicode style border 2", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "┌────┬────────────────────────────┬────┐\n│ id │        description         │ n  │\n├────┼────────────────────────────┼────┤\n│  1 │ The quick brown fox jumps …│ 1.…│\n│    │…over the lazy dog and keep…│… 5 │\n│    │…s running far away         │    │\n│ 22 │ short                      │ 2.…│\n│    │                            │…25 │\n│ 33…│ another reasonably long pi…│  3 │\n│… 3 │…ece of text that will need…│    │\n│    │… to be wrapped somewhere   │    │\n└────┴────────────────────────────┴────┘\n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.format = .WRAPPED;
        options.columns = 40;
        options.linestyle = .OLD_ASCII;
        check_query("wrapped old-ascii border 2", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "+----+----------------------------+----+\n| id |        description         | n  |\n+----+----------------------------+----+\n|  1 | The quick brown fox jumps  | 1. |\n|    ; over the lazy dog and keep ;  5 |\n|    ; s running far away              |\n| 22 | short                      | 2. |\n|                                 ; 25 |\n| 33 | another reasonably long pi |  3 |\n|  3 ; ece of text that will need      |\n|    ;  to be wrapped somewhere        |\n+----+----------------------------+----+\n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.format = .WRAPPED;
        check_query("wrapped no columns border 2", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "+-----+------------------------------------------------------------------------------+------+\n| id  |                                 description                                  |  n   |\n+-----+------------------------------------------------------------------------------+------+\n|   1 | The quick brown fox jumps over the lazy dog and keeps running far away       |  1.5 |\n|  22 | short                                                                        | 2.25 |\n| 333 | another reasonably long piece of text that will need to be wrapped somewhere |    3 |\n+-----+------------------------------------------------------------------------------+------+\n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.format = .WRAPPED;
        options.columns = 40;
        options.title = "Wrapped table";
        check_query("wrapped title border 2", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "             Wrapped table\n+----+----------------------------+----+\n| id |        description         | n  |\n+----+----------------------------+----+\n|  1 | The quick brown fox jumps .| 1..|\n|    |.over the lazy dog and keep.|. 5 |\n|    |.s running far away         |    |\n| 22 | short                      | 2..|\n|    |                            |.25 |\n| 33.| another reasonably long pi.|  3 |\n|. 3 |.ece of text that will need.|    |\n|    |. to be wrapped somewhere   |    |\n+----+----------------------------+----+\n(3 rows)\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.expanded = .ON;
        options.format = .WRAPPED;
        options.columns = 30;
        options.linestyle = .UNICODE;
        check_query("expanded wrapped unicode border 0", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "* Record 1                  \nid          1\ndescription The quick brown f…\n           …ox jumps over the…\n           … lazy dog and kee…\n           …ps running far aw…\n           …ay\nn           1.5\n* Record 2                  \nid          22\ndescription short\nn           2.25\n* Record 3                  \nid          333\ndescription another reasonabl…\n           …y long piece of t…\n           …ext that will nee…\n           …d to be wrapped s…\n           …omewhere\nn           3\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.expanded = .ON;
        options.format = .WRAPPED;
        options.columns = 30;
        options.linestyle = .UNICODE;
        check_query("expanded wrapped unicode border 1", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "─[ RECORD 1 ]────────────────\nid          │ 1\ndescription │ The quick brown…\n            │… fox jumps over…\n            │… the lazy dog a…\n            │…nd keeps runnin…\n            │…g far away\nn           │ 1.5\n─[ RECORD 2 ]────────────────\nid          │ 22\ndescription │ short\nn           │ 2.25\n─[ RECORD 3 ]────────────────\nid          │ 333\ndescription │ another reasona…\n            │…bly long piece …\n            │…of text that wi…\n            │…ll need to be w…\n            │…rapped somewher…\n            │…e\nn           │ 3\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.expanded = .ON;
        options.format = .WRAPPED;
        options.columns = 30;
        options.linestyle = .UNICODE;
        check_query("expanded wrapped unicode border 2", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "┌─[ RECORD 1 ]┬──────────────┐\n│ id          │ 1            │\n│ description │ The quick br…│\n│             │…own fox jump…│\n│             │…s over the l…│\n│             │…azy dog and …│\n│             │…keeps runnin…│\n│             │…g far away   │\n│ n           │ 1.5          │\n├─[ RECORD 2 ]┼──────────────┤\n│ id          │ 22           │\n│ description │ short        │\n│ n           │ 2.25         │\n├─[ RECORD 3 ]┼──────────────┤\n│ id          │ 333          │\n│ description │ another reas…│\n│             │…onably long …│\n│             │…piece of tex…│\n│             │…t that will …│\n│             │…need to be w…│\n│             │…rapped somew…│\n│             │…here         │\n│ n           │ 3            │\n└─────────────┴──────────────┘\n\n");
    }

    {
        options: Print_Options;
        options.border = 0;
        options.expanded = .ON;
        options.format = .WRAPPED;
        options.columns = 30;
        options.linestyle = .OLD_ASCII;
        check_query("expanded wrapped old-ascii border 0", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "* Record 1                   \nid          1\ndescription The quick brown fo\n            x jumps over the l\n            azy dog and keeps \n            running far away\nn           1.5\n* Record 2                   \nid          22\ndescription short\nn           2.25\n* Record 3                   \nid          333\ndescription another reasonably\n             long piece of tex\n            t that will need t\n            o be wrapped somew\n            here\nn           3\n\n");
    }

    {
        options: Print_Options;
        options.border = 1;
        options.expanded = .ON;
        options.format = .WRAPPED;
        options.columns = 30;
        options.linestyle = .OLD_ASCII;
        check_query("expanded wrapped old-ascii border 1", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "-[ RECORD 1 ]-----------------\nid          | 1\ndescription | The quick brown \n            ; fox jumps over t\n            ; he lazy dog and \n            ; keeps running fa\n            ; r away\nn           | 1.5\n-[ RECORD 2 ]-----------------\nid          | 22\ndescription | short\nn           | 2.25\n-[ RECORD 3 ]-----------------\nid          | 333\ndescription | another reasonab\n            ; ly long piece of\n            ;  text that will \n            ; need to be wrapp\n            ; ed somewhere\nn           | 3\n\n");
    }

    {
        options: Print_Options;
        options.border = 2;
        options.expanded = .ON;
        options.format = .WRAPPED;
        options.columns = 30;
        options.linestyle = .OLD_ASCII;
        check_query("expanded wrapped old-ascii border 2", string.["id", "description", "n"], u32.[23, 25, 1700], string.["1", "The quick brown fox jumps over the lazy dog and keeps running far away", "1.5", "22", "short", "2.25", "333", "another reasonably long piece of text that will need to be wrapped somewhere", "3"], bool.[false, false, false, false, false, false, false, false, false], 3, options, "+-[ RECORD 1 ]+--------------+\n| id          | 1            |\n| description | The quick br |\n|             ; own fox jump |\n|             ; s over the l |\n|             ; azy dog and  |\n|             ; keeps runnin |\n|             ; g far away   |\n| n           | 1.5          |\n+-[ RECORD 2 ]+--------------+\n| id          | 22           |\n| description | short        |\n| n           | 2.25         |\n+-[ RECORD 3 ]+--------------+\n| id          | 333          |\n| description | another reas |\n|             ; onably long  |\n|             ; piece of tex |\n|             ; t that will  |\n|             ; need to be w |\n|             ; rapped somew |\n|             ; here         |\n| n           | 3            |\n+-------------+--------------+\n\n");
    }

    {
        options: Print_Options;
        table: Table;
        table.title = "Table \"pg_catalog.pg_namespace\"";
        table.headers = string.["Column", "Type", "Collation", "Nullable", "Default"];
        table.cells = string.[
            "oid", "oid", "", "not null", "",
            "nspname", "name", "", "not null", "",
            "nspowner", "oid", "", "not null", "",
            "nspacl", "aclitem[]", "", "", "",
        ];
        table.footers = string.[
            "Indexes:",
            "    \"pg_namespace_oid_index\" PRIMARY KEY, btree (oid)",
            "    \"pg_namespace_nspname_index\" UNIQUE CONSTRAINT, btree (nspname)",
        ];
        check_table("describe pg_namespace", table, options, "            Table \"pg_catalog.pg_namespace\"\n  Column  |   Type    | Collation | Nullable | Default \n----------+-----------+-----------+----------+---------\n oid      | oid       |           | not null | \n nspname  | name      |           | not null | \n nspowner | oid       |           | not null | \n nspacl   | aclitem[] |           |          | \nIndexes:\n    \"pg_namespace_oid_index\" PRIMARY KEY, btree (oid)\n    \"pg_namespace_nspname_index\" UNIQUE CONSTRAINT, btree (nspname)\n\n");
    }

    {
        options: Print_Options;
        table: Table;

        options.border = 0;
        table.title = "Table \"pg_catalog.pg_namespace\"";
        table.headers = string.["Column", "Type", "Collation", "Nullable", "Default"];
        table.cells = string.[
            "oid", "oid", "", "not null", "",
            "nspname", "name", "", "not null", "",
            "nspowner", "oid", "", "not null", "",
            "nspacl", "aclitem[]", "", "", "",
        ];
        table.footers = string.[
            "Indexes:",
            "    \"pg_namespace_oid_index\" PRIMARY KEY, btree (oid)",
            "    \"pg_namespace_nspname_index\" UNIQUE CONSTRAINT, btree (nspname)",
        ];
        check_table("describe pg_namespace border 0", table, options, "       Table \"pg_catalog.pg_namespace\"\n Column    Type    Collation Nullable Default \n-------- --------- --------- -------- -------\noid      oid                 not null \nnspname  name                not null \nnspowner oid                 not null \nnspacl   aclitem[]                    \nIndexes:\n    \"pg_namespace_oid_index\" PRIMARY KEY, btree (oid)\n    \"pg_namespace_nspname_index\" UNIQUE CONSTRAINT, btree (nspname)\n\n");
    }

    {
        options: Print_Options;
        table: Table;

        options.border = 1;
        table.title = "Table \"pg_catalog.pg_namespace\"";
        table.headers = string.["Column", "Type", "Collation", "Nullable", "Default"];
        table.cells = string.[
            "oid", "oid", "", "not null", "",
            "nspname", "name", "", "not null", "",
            "nspowner", "oid", "", "not null", "",
            "nspacl", "aclitem[]", "", "", "",
        ];
        table.footers = string.[
            "Indexes:",
            "    \"pg_namespace_oid_index\" PRIMARY KEY, btree (oid)",
            "    \"pg_namespace_nspname_index\" UNIQUE CONSTRAINT, btree (nspname)",
        ];
        check_table("describe pg_namespace border 1", table, options, "            Table \"pg_catalog.pg_namespace\"\n  Column  |   Type    | Collation | Nullable | Default \n----------+-----------+-----------+----------+---------\n oid      | oid       |           | not null | \n nspname  | name      |           | not null | \n nspowner | oid       |           | not null | \n nspacl   | aclitem[] |           |          | \nIndexes:\n    \"pg_namespace_oid_index\" PRIMARY KEY, btree (oid)\n    \"pg_namespace_nspname_index\" UNIQUE CONSTRAINT, btree (nspname)\n\n");
    }

    {
        options: Print_Options;
        table: Table;

        options.border = 2;
        table.title = "Table \"pg_catalog.pg_namespace\"";
        table.headers = string.["Column", "Type", "Collation", "Nullable", "Default"];
        table.cells = string.[
            "oid", "oid", "", "not null", "",
            "nspname", "name", "", "not null", "",
            "nspowner", "oid", "", "not null", "",
            "nspacl", "aclitem[]", "", "", "",
        ];
        table.footers = string.[
            "Indexes:",
            "    \"pg_namespace_oid_index\" PRIMARY KEY, btree (oid)",
            "    \"pg_namespace_nspname_index\" UNIQUE CONSTRAINT, btree (nspname)",
        ];
        check_table("describe pg_namespace border 2", table, options, "             Table \"pg_catalog.pg_namespace\"\n+----------+-----------+-----------+----------+---------+\n|  Column  |   Type    | Collation | Nullable | Default |\n+----------+-----------+-----------+----------+---------+\n| oid      | oid       |           | not null |         |\n| nspname  | name      |           | not null |         |\n| nspowner | oid       |           | not null |         |\n| nspacl   | aclitem[] |           |          |         |\n+----------+-----------+-----------+----------+---------+\nIndexes:\n    \"pg_namespace_oid_index\" PRIMARY KEY, btree (oid)\n    \"pg_namespace_nspname_index\" UNIQUE CONSTRAINT, btree (nspname)\n\n");
    }
}

people_query :: (name: string, options: Print_Options, expected: string) {
    check_query(name, string.["id", "name", "score"], u32.[23, 25, 1700], string.["1", "alice", "10.5", "22", "bob", "", "333", "", "-7", "4", "", "1000.25"], bool.[false, false, false, false, false, true, false, true, false, false, false, false], 4, options, expected);
}

special_query :: (name: string, options: Print_Options, expected: string) {
    cells := string.[
        "<b>&amp;\"q\"", "  lead  trail", "1",
        "line1\nline2", " ", "2",
        "", "", "",
        "p|q \\ # $ % _ { } ~ ^ >", "\t", "3",
        "two  spaces  inside", "back\\slash", "40",
    ];
    nulls := bool.[false, false, false, false, false, false, false, true, true, false, false, false, false, false, false];

    check_query(name, string.["a&b", "x<y", "q\"r"], u32.[25, 25, 23], cells, nulls, 5, options, expected);
}

empty_query :: (name: string, options: Print_Options, expected: string) {
    check_query(name, string.["id", "name"], u32.[23, 25], string.[], bool.[], 0, options, expected);
}

one_query :: (name: string, options: Print_Options, expected: string) {
    check_query(name, string.["only"], u32.[25], string.["x"], bool.[false], 1, options, expected);
}

namespace_table :: (name: string, options: Print_Options, expected: string) {
    table: Table;
    table.title = "Table \"pg_catalog.pg_namespace\"";
    table.headers = string.["Column", "Type", "Collation", "Nullable", "Default"];
    table.cells = string.[
        "oid", "oid", "", "not null", "",
        "nspname", "name", "", "not null", "",
        "nspowner", "oid", "", "not null", "",
        "nspacl", "aclitem[]", "", "", "",
    ];
    table.footers = string.[
        "Indexes:",
        "    \"pg_namespace_oid_index\" PRIMARY KEY, btree (oid)",
        "    \"pg_namespace_nspname_index\" UNIQUE CONSTRAINT, btree (nspname)",
    ];
    check_table(name, table, options, expected);
}

// Expected output of HTML, AsciiDoc, LaTeX, troff and the zero-byte separators.
test_formats :: () {
    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 0;
        options.title = "People";
        people_query("people html border 0 titled", options, "<table border=\"0\">\n  <caption>People</caption>\n  <tr>\n    <th align=\"center\">id</th>\n    <th align=\"center\">name</th>\n    <th align=\"center\">score</th>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">1</td>\n    <td align=\"left\">alice</td>\n    <td align=\"right\">10.5</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">22</td>\n    <td align=\"left\">bob</td>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">333</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">-7</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">4</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">1000.25</td>\n  </tr>\n</table>\n<p>(4 rows)<br />\n</p>\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 0;
        options.tuples_only = true;
        options.title = "People";
        people_query("people html border 0 tuples only titled", options, "<table border=\"0\">\n  <tr valign=\"top\">\n    <td align=\"right\">1</td>\n    <td align=\"left\">alice</td>\n    <td align=\"right\">10.5</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">22</td>\n    <td align=\"left\">bob</td>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">333</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">-7</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">4</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">1000.25</td>\n  </tr>\n</table>\n\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 1;
        people_query("people html border 1", options, "<table border=\"1\">\n  <tr>\n    <th align=\"center\">id</th>\n    <th align=\"center\">name</th>\n    <th align=\"center\">score</th>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">1</td>\n    <td align=\"left\">alice</td>\n    <td align=\"right\">10.5</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">22</td>\n    <td align=\"left\">bob</td>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">333</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">-7</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">4</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">1000.25</td>\n  </tr>\n</table>\n<p>(4 rows)<br />\n</p>\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 1;
        options.tuples_only = true;
        people_query("people html border 1 tuples only", options, "<table border=\"1\">\n  <tr valign=\"top\">\n    <td align=\"right\">1</td>\n    <td align=\"left\">alice</td>\n    <td align=\"right\">10.5</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">22</td>\n    <td align=\"left\">bob</td>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">333</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">-7</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">4</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">1000.25</td>\n  </tr>\n</table>\n\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 2;
        options.title = "People";
        people_query("people html border 2 titled", options, "<table border=\"2\">\n  <caption>People</caption>\n  <tr>\n    <th align=\"center\">id</th>\n    <th align=\"center\">name</th>\n    <th align=\"center\">score</th>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">1</td>\n    <td align=\"left\">alice</td>\n    <td align=\"right\">10.5</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">22</td>\n    <td align=\"left\">bob</td>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">333</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">-7</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">4</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">1000.25</td>\n  </tr>\n</table>\n<p>(4 rows)<br />\n</p>\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 2;
        options.tuples_only = true;
        options.title = "People";
        people_query("people html border 2 tuples only titled", options, "<table border=\"2\">\n  <tr valign=\"top\">\n    <td align=\"right\">1</td>\n    <td align=\"left\">alice</td>\n    <td align=\"right\">10.5</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">22</td>\n    <td align=\"left\">bob</td>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">333</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">-7</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">4</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">1000.25</td>\n  </tr>\n</table>\n\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 3;
        people_query("people html border 3", options, "<table border=\"3\">\n  <tr>\n    <th align=\"center\">id</th>\n    <th align=\"center\">name</th>\n    <th align=\"center\">score</th>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">1</td>\n    <td align=\"left\">alice</td>\n    <td align=\"right\">10.5</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">22</td>\n    <td align=\"left\">bob</td>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">333</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">-7</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">4</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">1000.25</td>\n  </tr>\n</table>\n<p>(4 rows)<br />\n</p>\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 3;
        options.tuples_only = true;
        people_query("people html border 3 tuples only", options, "<table border=\"3\">\n  <tr valign=\"top\">\n    <td align=\"right\">1</td>\n    <td align=\"left\">alice</td>\n    <td align=\"right\">10.5</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">22</td>\n    <td align=\"left\">bob</td>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">333</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">-7</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">4</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">1000.25</td>\n  </tr>\n</table>\n\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 0;
        options.expanded = .ON;
        options.title = "People";
        people_query("people html border 0 expanded titled", options, "<table border=\"0\">\n  <caption>People</caption>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 1</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">1</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">alice</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">10.5</td>\n  </tr>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 2</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">22</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">bob</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 3</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">333</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">-7</td>\n  </tr>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 4</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">4</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">1000.25</td>\n  </tr>\n</table>\n\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 0;
        options.expanded = .ON;
        options.tuples_only = true;
        options.title = "People";
        people_query("people html border 0 expanded tuples only titled", options, "<table border=\"0\">\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">1</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">alice</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">10.5</td>\n  </tr>\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">22</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">bob</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">333</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">-7</td>\n  </tr>\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">4</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">1000.25</td>\n  </tr>\n</table>\n\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 1;
        options.expanded = .ON;
        people_query("people html border 1 expanded", options, "<table border=\"1\">\n\n  <tr><td colspan=\"2\" align=\"center\">Record 1</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">1</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">alice</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">10.5</td>\n  </tr>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 2</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">22</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">bob</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 3</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">333</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">-7</td>\n  </tr>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 4</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">4</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">1000.25</td>\n  </tr>\n</table>\n\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 1;
        options.expanded = .ON;
        options.tuples_only = true;
        people_query("people html border 1 expanded tuples only", options, "<table border=\"1\">\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">1</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">alice</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">10.5</td>\n  </tr>\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">22</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">bob</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">333</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">-7</td>\n  </tr>\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">4</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">1000.25</td>\n  </tr>\n</table>\n\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 2;
        options.expanded = .ON;
        options.title = "People";
        people_query("people html border 2 expanded titled", options, "<table border=\"2\">\n  <caption>People</caption>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 1</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">1</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">alice</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">10.5</td>\n  </tr>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 2</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">22</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">bob</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 3</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">333</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">-7</td>\n  </tr>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 4</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">4</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">1000.25</td>\n  </tr>\n</table>\n\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 2;
        options.expanded = .ON;
        options.tuples_only = true;
        options.title = "People";
        people_query("people html border 2 expanded tuples only titled", options, "<table border=\"2\">\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">1</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">alice</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">10.5</td>\n  </tr>\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">22</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">bob</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">333</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">-7</td>\n  </tr>\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">4</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">1000.25</td>\n  </tr>\n</table>\n\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 3;
        options.expanded = .ON;
        people_query("people html border 3 expanded", options, "<table border=\"3\">\n\n  <tr><td colspan=\"2\" align=\"center\">Record 1</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">1</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">alice</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">10.5</td>\n  </tr>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 2</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">22</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">bob</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 3</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">333</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">-7</td>\n  </tr>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 4</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">4</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">1000.25</td>\n  </tr>\n</table>\n\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 3;
        options.expanded = .ON;
        options.tuples_only = true;
        people_query("people html border 3 expanded tuples only", options, "<table border=\"3\">\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">1</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">alice</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">10.5</td>\n  </tr>\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">22</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">bob</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">333</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">-7</td>\n  </tr>\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">4</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">1000.25</td>\n  </tr>\n</table>\n\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 0;
        options.title = "People";
        people_query("people asciidoc border 0 titled", options, "\n.People\n[options=\"header\",cols=\">l,<l,>l\",frame=\"none\",grid=\"none\"]\n|====\n^l|id ^l|name ^l|score\n|1 |alice |10.5\n|22 |bob |\n|333 |  |-7\n|4 |  |1000.25\n|====\n\n....\n(4 rows)\n....\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 0;
        options.tuples_only = true;
        options.title = "People";
        people_query("people asciidoc border 0 tuples only titled", options, "\n[cols=\">l,<l,>l\",frame=\"none\",grid=\"none\"]\n|====\n|1 |alice |10.5\n|22 |bob |\n|333 |  |-7\n|4 |  |1000.25\n|====\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 1;
        people_query("people asciidoc border 1", options, "\n[options=\"header\",cols=\">l,<l,>l\",frame=\"none\"]\n|====\n^l|id ^l|name ^l|score\n|1 |alice |10.5\n|22 |bob |\n|333 |  |-7\n|4 |  |1000.25\n|====\n\n....\n(4 rows)\n....\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 1;
        options.tuples_only = true;
        people_query("people asciidoc border 1 tuples only", options, "\n[cols=\">l,<l,>l\",frame=\"none\"]\n|====\n|1 |alice |10.5\n|22 |bob |\n|333 |  |-7\n|4 |  |1000.25\n|====\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 2;
        options.title = "People";
        people_query("people asciidoc border 2 titled", options, "\n.People\n[options=\"header\",cols=\">l,<l,>l\",frame=\"all\",grid=\"all\"]\n|====\n^l|id ^l|name ^l|score\n|1 |alice |10.5\n|22 |bob |\n|333 |  |-7\n|4 |  |1000.25\n|====\n\n....\n(4 rows)\n....\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 2;
        options.tuples_only = true;
        options.title = "People";
        people_query("people asciidoc border 2 tuples only titled", options, "\n[cols=\">l,<l,>l\",frame=\"all\",grid=\"all\"]\n|====\n|1 |alice |10.5\n|22 |bob |\n|333 |  |-7\n|4 |  |1000.25\n|====\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 3;
        people_query("people asciidoc border 3", options, "\n[options=\"header\",cols=\">l,<l,>l\"]\n|====\n^l|id ^l|name ^l|score\n|1 |alice |10.5\n|22 |bob |\n|333 |  |-7\n|4 |  |1000.25\n|====\n\n....\n(4 rows)\n....\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 3;
        options.tuples_only = true;
        people_query("people asciidoc border 3 tuples only", options, "\n[cols=\">l,<l,>l\"]\n|====\n|1 |alice |10.5\n|22 |bob |\n|333 |  |-7\n|4 |  |1000.25\n|====\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 0;
        options.expanded = .ON;
        options.title = "People";
        people_query("people asciidoc border 0 expanded titled", options, "\n.People\n[cols=\"h,l\",frame=\"none\",grid=\"none\"]\n|====\n2+^|Record 1\n<l|id >l|1\n<l|name <l|alice\n<l|score >l|10.5\n2+^|Record 2\n<l|id >l|22\n<l|name <l|bob\n<l|score >l| \n2+^|Record 3\n<l|id >l|333\n<l|name <l| \n<l|score >l|-7\n2+^|Record 4\n<l|id >l|4\n<l|name <l| \n<l|score >l|1000.25\n|====\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 0;
        options.expanded = .ON;
        options.tuples_only = true;
        options.title = "People";
        people_query("people asciidoc border 0 expanded tuples only titled", options, "\n[cols=\"h,l\",frame=\"none\",grid=\"none\"]\n|====\n2+|\n<l|id >l|1\n<l|name <l|alice\n<l|score >l|10.5\n2+|\n<l|id >l|22\n<l|name <l|bob\n<l|score >l| \n2+|\n<l|id >l|333\n<l|name <l| \n<l|score >l|-7\n2+|\n<l|id >l|4\n<l|name <l| \n<l|score >l|1000.25\n|====\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 1;
        options.expanded = .ON;
        people_query("people asciidoc border 1 expanded", options, "\n[cols=\"h,l\",frame=\"none\"]\n|====\n2+^|Record 1\n<l|id >l|1\n<l|name <l|alice\n<l|score >l|10.5\n2+^|Record 2\n<l|id >l|22\n<l|name <l|bob\n<l|score >l| \n2+^|Record 3\n<l|id >l|333\n<l|name <l| \n<l|score >l|-7\n2+^|Record 4\n<l|id >l|4\n<l|name <l| \n<l|score >l|1000.25\n|====\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 1;
        options.expanded = .ON;
        options.tuples_only = true;
        people_query("people asciidoc border 1 expanded tuples only", options, "\n[cols=\"h,l\",frame=\"none\"]\n|====\n2+|\n<l|id >l|1\n<l|name <l|alice\n<l|score >l|10.5\n2+|\n<l|id >l|22\n<l|name <l|bob\n<l|score >l| \n2+|\n<l|id >l|333\n<l|name <l| \n<l|score >l|-7\n2+|\n<l|id >l|4\n<l|name <l| \n<l|score >l|1000.25\n|====\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 2;
        options.expanded = .ON;
        options.title = "People";
        people_query("people asciidoc border 2 expanded titled", options, "\n.People\n[cols=\"h,l\",frame=\"all\",grid=\"all\"]\n|====\n2+^|Record 1\n<l|id >l|1\n<l|name <l|alice\n<l|score >l|10.5\n2+^|Record 2\n<l|id >l|22\n<l|name <l|bob\n<l|score >l| \n2+^|Record 3\n<l|id >l|333\n<l|name <l| \n<l|score >l|-7\n2+^|Record 4\n<l|id >l|4\n<l|name <l| \n<l|score >l|1000.25\n|====\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 2;
        options.expanded = .ON;
        options.tuples_only = true;
        options.title = "People";
        people_query("people asciidoc border 2 expanded tuples only titled", options, "\n[cols=\"h,l\",frame=\"all\",grid=\"all\"]\n|====\n2+|\n<l|id >l|1\n<l|name <l|alice\n<l|score >l|10.5\n2+|\n<l|id >l|22\n<l|name <l|bob\n<l|score >l| \n2+|\n<l|id >l|333\n<l|name <l| \n<l|score >l|-7\n2+|\n<l|id >l|4\n<l|name <l| \n<l|score >l|1000.25\n|====\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 3;
        options.expanded = .ON;
        people_query("people asciidoc border 3 expanded", options, "\n[cols=\"h,l\"]\n|====\n2+^|Record 1\n<l|id >l|1\n<l|name <l|alice\n<l|score >l|10.5\n2+^|Record 2\n<l|id >l|22\n<l|name <l|bob\n<l|score >l| \n2+^|Record 3\n<l|id >l|333\n<l|name <l| \n<l|score >l|-7\n2+^|Record 4\n<l|id >l|4\n<l|name <l| \n<l|score >l|1000.25\n|====\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 3;
        options.expanded = .ON;
        options.tuples_only = true;
        people_query("people asciidoc border 3 expanded tuples only", options, "\n[cols=\"h,l\"]\n|====\n2+|\n<l|id >l|1\n<l|name <l|alice\n<l|score >l|10.5\n2+|\n<l|id >l|22\n<l|name <l|bob\n<l|score >l| \n2+|\n<l|id >l|333\n<l|name <l| \n<l|score >l|-7\n2+|\n<l|id >l|4\n<l|name <l| \n<l|score >l|1000.25\n|====\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 0;
        options.title = "People";
        people_query("people latex border 0 titled", options, "\\begin{center}\nPeople\n\\end{center}\n\n\\begin{tabular}{rlr}\n\\textit{id} & \\textit{name} & \\textit{score} \\\\\n\\hline\n1 & alice & 10.5 \\\\\n22 & bob &  \\\\\n333 &  & -7 \\\\\n4 &  & 1000.25 \\\\\n\\end{tabular}\n\n\\noindent (4 rows) \\\\\n\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 0;
        options.tuples_only = true;
        options.title = "People";
        people_query("people latex border 0 tuples only titled", options, "\\begin{tabular}{rlr}\n1 & alice & 10.5 \\\\\n22 & bob &  \\\\\n333 &  & -7 \\\\\n4 &  & 1000.25 \\\\\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 1;
        people_query("people latex border 1", options, "\\begin{tabular}{r | l | r}\n\\textit{id} & \\textit{name} & \\textit{score} \\\\\n\\hline\n1 & alice & 10.5 \\\\\n22 & bob &  \\\\\n333 &  & -7 \\\\\n4 &  & 1000.25 \\\\\n\\end{tabular}\n\n\\noindent (4 rows) \\\\\n\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 1;
        options.tuples_only = true;
        people_query("people latex border 1 tuples only", options, "\\begin{tabular}{r | l | r}\n1 & alice & 10.5 \\\\\n22 & bob &  \\\\\n333 &  & -7 \\\\\n4 &  & 1000.25 \\\\\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 2;
        options.title = "People";
        people_query("people latex border 2 titled", options, "\\begin{center}\nPeople\n\\end{center}\n\n\\begin{tabular}{| r | l | r |}\n\\hline\n\\textit{id} & \\textit{name} & \\textit{score} \\\\\n\\hline\n1 & alice & 10.5 \\\\\n22 & bob &  \\\\\n333 &  & -7 \\\\\n4 &  & 1000.25 \\\\\n\\hline\n\\end{tabular}\n\n\\noindent (4 rows) \\\\\n\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 2;
        options.tuples_only = true;
        options.title = "People";
        people_query("people latex border 2 tuples only titled", options, "\\begin{tabular}{| r | l | r |}\n1 & alice & 10.5 \\\\\n22 & bob &  \\\\\n333 &  & -7 \\\\\n4 &  & 1000.25 \\\\\n\\hline\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 3;
        people_query("people latex border 3", options, "\\begin{tabular}{| r | l | r |}\n\\hline\n\\textit{id} & \\textit{name} & \\textit{score} \\\\\n\\hline\n1 & alice & 10.5 \\\\\n\\hline\n22 & bob &  \\\\\n\\hline\n333 &  & -7 \\\\\n\\hline\n4 &  & 1000.25 \\\\\n\\hline\n\\end{tabular}\n\n\\noindent (4 rows) \\\\\n\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 3;
        options.tuples_only = true;
        people_query("people latex border 3 tuples only", options, "\\begin{tabular}{| r | l | r |}\n1 & alice & 10.5 \\\\\n\\hline\n22 & bob &  \\\\\n\\hline\n333 &  & -7 \\\\\n\\hline\n4 &  & 1000.25 \\\\\n\\hline\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 0;
        options.expanded = .ON;
        options.title = "People";
        people_query("people latex border 0 expanded titled", options, "\\begin{center}\nPeople\n\\end{center}\n\n\\begin{tabular}{cl}\n\\multicolumn{2}{c}{\\textit{Record 1}} \\\\\nid & 1 \\\\\nname & alice \\\\\nscore & 10.5 \\\\\n\\multicolumn{2}{c}{\\textit{Record 2}} \\\\\nid & 22 \\\\\nname & bob \\\\\nscore &  \\\\\n\\multicolumn{2}{c}{\\textit{Record 3}} \\\\\nid & 333 \\\\\nname &  \\\\\nscore & -7 \\\\\n\\multicolumn{2}{c}{\\textit{Record 4}} \\\\\nid & 4 \\\\\nname &  \\\\\nscore & 1000.25 \\\\\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 0;
        options.expanded = .ON;
        options.tuples_only = true;
        options.title = "People";
        people_query("people latex border 0 expanded tuples only titled", options, "\\begin{tabular}{cl}\nid & 1 \\\\\nname & alice \\\\\nscore & 10.5 \\\\\nid & 22 \\\\\nname & bob \\\\\nscore &  \\\\\nid & 333 \\\\\nname &  \\\\\nscore & -7 \\\\\nid & 4 \\\\\nname &  \\\\\nscore & 1000.25 \\\\\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 1;
        options.expanded = .ON;
        people_query("people latex border 1 expanded", options, "\\begin{tabular}{c|l}\n\\multicolumn{2}{c}{\\textit{Record 1}} \\\\\n\\hline\nid & 1 \\\\\nname & alice \\\\\nscore & 10.5 \\\\\n\\multicolumn{2}{c}{\\textit{Record 2}} \\\\\n\\hline\nid & 22 \\\\\nname & bob \\\\\nscore &  \\\\\n\\multicolumn{2}{c}{\\textit{Record 3}} \\\\\n\\hline\nid & 333 \\\\\nname &  \\\\\nscore & -7 \\\\\n\\multicolumn{2}{c}{\\textit{Record 4}} \\\\\n\\hline\nid & 4 \\\\\nname &  \\\\\nscore & 1000.25 \\\\\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 1;
        options.expanded = .ON;
        options.tuples_only = true;
        people_query("people latex border 1 expanded tuples only", options, "\\begin{tabular}{c|l}\n\\hline\nid & 1 \\\\\nname & alice \\\\\nscore & 10.5 \\\\\n\\hline\nid & 22 \\\\\nname & bob \\\\\nscore &  \\\\\n\\hline\nid & 333 \\\\\nname &  \\\\\nscore & -7 \\\\\n\\hline\nid & 4 \\\\\nname &  \\\\\nscore & 1000.25 \\\\\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 2;
        options.expanded = .ON;
        options.title = "People";
        people_query("people latex border 2 expanded titled", options, "\\begin{center}\nPeople\n\\end{center}\n\n\\begin{tabular}{|c|l|}\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 1}} \\\\\n\\hline\nid & 1 \\\\\nname & alice \\\\\nscore & 10.5 \\\\\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 2}} \\\\\n\\hline\nid & 22 \\\\\nname & bob \\\\\nscore &  \\\\\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 3}} \\\\\n\\hline\nid & 333 \\\\\nname &  \\\\\nscore & -7 \\\\\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 4}} \\\\\n\\hline\nid & 4 \\\\\nname &  \\\\\nscore & 1000.25 \\\\\n\\hline\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 2;
        options.expanded = .ON;
        options.tuples_only = true;
        options.title = "People";
        people_query("people latex border 2 expanded tuples only titled", options, "\\begin{tabular}{|c|l|}\n\\hline\nid & 1 \\\\\nname & alice \\\\\nscore & 10.5 \\\\\n\\hline\nid & 22 \\\\\nname & bob \\\\\nscore &  \\\\\n\\hline\nid & 333 \\\\\nname &  \\\\\nscore & -7 \\\\\n\\hline\nid & 4 \\\\\nname &  \\\\\nscore & 1000.25 \\\\\n\\hline\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 3;
        options.expanded = .ON;
        people_query("people latex border 3 expanded", options, "\\begin{tabular}{|c|l|}\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 1}} \\\\\n\\hline\nid & 1 \\\\\nname & alice \\\\\nscore & 10.5 \\\\\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 2}} \\\\\n\\hline\nid & 22 \\\\\nname & bob \\\\\nscore &  \\\\\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 3}} \\\\\n\\hline\nid & 333 \\\\\nname &  \\\\\nscore & -7 \\\\\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 4}} \\\\\n\\hline\nid & 4 \\\\\nname &  \\\\\nscore & 1000.25 \\\\\n\\hline\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 3;
        options.expanded = .ON;
        options.tuples_only = true;
        people_query("people latex border 3 expanded tuples only", options, "\\begin{tabular}{|c|l|}\n\\hline\nid & 1 \\\\\nname & alice \\\\\nscore & 10.5 \\\\\n\\hline\nid & 22 \\\\\nname & bob \\\\\nscore &  \\\\\n\\hline\nid & 333 \\\\\nname &  \\\\\nscore & -7 \\\\\n\\hline\nid & 4 \\\\\nname &  \\\\\nscore & 1000.25 \\\\\n\\hline\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 0;
        options.title = "People";
        people_query("people latex-longtable border 0 titled", options, "\\begin{longtable}{rlr}\n\\small\\textbf{\\textit{id}} & \\small\\textbf{\\textit{name}} & \\small\\textbf{\\textit{score}} \\\\\n\\midrule\n\\endfirsthead\n\\small\\textbf{\\textit{id}} & \\small\\textbf{\\textit{name}} & \\small\\textbf{\\textit{score}} \\\\\n\\midrule\n\\endhead\n\\caption[People (Continued)]{People}\n\\endfoot\n\\caption[People]{People}\n\\endlastfoot\n\\raggedright{1}\n&\n\\raggedright{alice}\n&\n\\raggedright{10.5} \\tabularnewline\n\\raggedright{22}\n&\n\\raggedright{bob}\n&\n\\raggedright{} \\tabularnewline\n\\raggedright{333}\n&\n\\raggedright{}\n&\n\\raggedright{-7} \\tabularnewline\n\\raggedright{4}\n&\n\\raggedright{}\n&\n\\raggedright{1000.25} \\tabularnewline\n\\end{longtable}\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 0;
        options.tuples_only = true;
        options.title = "People";
        people_query("people latex-longtable border 0 tuples only titled", options, "\\begin{longtable}{rlr}\n\\raggedright{1}\n&\n\\raggedright{alice}\n&\n\\raggedright{10.5} \\tabularnewline\n\\raggedright{22}\n&\n\\raggedright{bob}\n&\n\\raggedright{} \\tabularnewline\n\\raggedright{333}\n&\n\\raggedright{}\n&\n\\raggedright{-7} \\tabularnewline\n\\raggedright{4}\n&\n\\raggedright{}\n&\n\\raggedright{1000.25} \\tabularnewline\n\\end{longtable}\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 1;
        people_query("people latex-longtable border 1", options, "\\begin{longtable}{r | l | r}\n\\small\\textbf{\\textit{id}} & \\small\\textbf{\\textit{name}} & \\small\\textbf{\\textit{score}} \\\\\n\\midrule\n\\endfirsthead\n\\small\\textbf{\\textit{id}} & \\small\\textbf{\\textit{name}} & \\small\\textbf{\\textit{score}} \\\\\n\\midrule\n\\endhead\n\\raggedright{1}\n&\n\\raggedright{alice}\n&\n\\raggedright{10.5} \\tabularnewline\n\\raggedright{22}\n&\n\\raggedright{bob}\n&\n\\raggedright{} \\tabularnewline\n\\raggedright{333}\n&\n\\raggedright{}\n&\n\\raggedright{-7} \\tabularnewline\n\\raggedright{4}\n&\n\\raggedright{}\n&\n\\raggedright{1000.25} \\tabularnewline\n\\end{longtable}\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 1;
        options.tuples_only = true;
        people_query("people latex-longtable border 1 tuples only", options, "\\begin{longtable}{r | l | r}\n\\raggedright{1}\n&\n\\raggedright{alice}\n&\n\\raggedright{10.5} \\tabularnewline\n\\raggedright{22}\n&\n\\raggedright{bob}\n&\n\\raggedright{} \\tabularnewline\n\\raggedright{333}\n&\n\\raggedright{}\n&\n\\raggedright{-7} \\tabularnewline\n\\raggedright{4}\n&\n\\raggedright{}\n&\n\\raggedright{1000.25} \\tabularnewline\n\\end{longtable}\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 2;
        options.title = "People";
        people_query("people latex-longtable border 2 titled", options, "\\begin{longtable}{| r | l | r |}\n\\toprule\n\\small\\textbf{\\textit{id}} & \\small\\textbf{\\textit{name}} & \\small\\textbf{\\textit{score}} \\\\\n\\midrule\n\\endfirsthead\n\\toprule\n\\small\\textbf{\\textit{id}} & \\small\\textbf{\\textit{name}} & \\small\\textbf{\\textit{score}} \\\\\n\\midrule\n\\endhead\n\\bottomrule\n\\caption[People (Continued)]{People}\n\\endfoot\n\\bottomrule\n\\caption[People]{People}\n\\endlastfoot\n\\raggedright{1}\n&\n\\raggedright{alice}\n&\n\\raggedright{10.5} \\tabularnewline\n\\raggedright{22}\n&\n\\raggedright{bob}\n&\n\\raggedright{} \\tabularnewline\n\\raggedright{333}\n&\n\\raggedright{}\n&\n\\raggedright{-7} \\tabularnewline\n\\raggedright{4}\n&\n\\raggedright{}\n&\n\\raggedright{1000.25} \\tabularnewline\n\\end{longtable}\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 2;
        options.tuples_only = true;
        options.title = "People";
        people_query("people latex-longtable border 2 tuples only titled", options, "\\begin{longtable}{| r | l | r |}\n\\raggedright{1}\n&\n\\raggedright{alice}\n&\n\\raggedright{10.5} \\tabularnewline\n\\raggedright{22}\n&\n\\raggedright{bob}\n&\n\\raggedright{} \\tabularnewline\n\\raggedright{333}\n&\n\\raggedright{}\n&\n\\raggedright{-7} \\tabularnewline\n\\raggedright{4}\n&\n\\raggedright{}\n&\n\\raggedright{1000.25} \\tabularnewline\n\\end{longtable}\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 3;
        people_query("people latex-longtable border 3", options, "\\begin{longtable}{| r | l | r |}\n\\toprule\n\\small\\textbf{\\textit{id}} & \\small\\textbf{\\textit{name}} & \\small\\textbf{\\textit{score}} \\\\\n\\midrule\n\\endfirsthead\n\\toprule\n\\small\\textbf{\\textit{id}} & \\small\\textbf{\\textit{name}} & \\small\\textbf{\\textit{score}} \\\\\n\\endhead\n\\bottomrule\n\\endfoot\n\\bottomrule\n\\endlastfoot\n\\raggedright{1}\n&\n\\raggedright{alice}\n&\n\\raggedright{10.5} \\tabularnewline\n \\hline\n\\raggedright{22}\n&\n\\raggedright{bob}\n&\n\\raggedright{} \\tabularnewline\n \\hline\n\\raggedright{333}\n&\n\\raggedright{}\n&\n\\raggedright{-7} \\tabularnewline\n \\hline\n\\raggedright{4}\n&\n\\raggedright{}\n&\n\\raggedright{1000.25} \\tabularnewline\n \\hline\n\\end{longtable}\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 3;
        options.tuples_only = true;
        people_query("people latex-longtable border 3 tuples only", options, "\\begin{longtable}{| r | l | r |}\n\\raggedright{1}\n&\n\\raggedright{alice}\n&\n\\raggedright{10.5} \\tabularnewline\n \\hline\n\\raggedright{22}\n&\n\\raggedright{bob}\n&\n\\raggedright{} \\tabularnewline\n \\hline\n\\raggedright{333}\n&\n\\raggedright{}\n&\n\\raggedright{-7} \\tabularnewline\n \\hline\n\\raggedright{4}\n&\n\\raggedright{}\n&\n\\raggedright{1000.25} \\tabularnewline\n \\hline\n\\end{longtable}\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 0;
        options.expanded = .ON;
        options.title = "People";
        people_query("people latex-longtable border 0 expanded titled", options, "\\begin{center}\nPeople\n\\end{center}\n\n\\begin{tabular}{cl}\n\\multicolumn{2}{c}{\\textit{Record 1}} \\\\\nid & 1 \\\\\nname & alice \\\\\nscore & 10.5 \\\\\n\\multicolumn{2}{c}{\\textit{Record 2}} \\\\\nid & 22 \\\\\nname & bob \\\\\nscore &  \\\\\n\\multicolumn{2}{c}{\\textit{Record 3}} \\\\\nid & 333 \\\\\nname &  \\\\\nscore & -7 \\\\\n\\multicolumn{2}{c}{\\textit{Record 4}} \\\\\nid & 4 \\\\\nname &  \\\\\nscore & 1000.25 \\\\\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 0;
        options.expanded = .ON;
        options.tuples_only = true;
        options.title = "People";
        people_query("people latex-longtable border 0 expanded tuples only titled", options, "\\begin{tabular}{cl}\nid & 1 \\\\\nname & alice \\\\\nscore & 10.5 \\\\\nid & 22 \\\\\nname & bob \\\\\nscore &  \\\\\nid & 333 \\\\\nname &  \\\\\nscore & -7 \\\\\nid & 4 \\\\\nname &  \\\\\nscore & 1000.25 \\\\\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 1;
        options.expanded = .ON;
        people_query("people latex-longtable border 1 expanded", options, "\\begin{tabular}{c|l}\n\\multicolumn{2}{c}{\\textit{Record 1}} \\\\\n\\hline\nid & 1 \\\\\nname & alice \\\\\nscore & 10.5 \\\\\n\\multicolumn{2}{c}{\\textit{Record 2}} \\\\\n\\hline\nid & 22 \\\\\nname & bob \\\\\nscore &  \\\\\n\\multicolumn{2}{c}{\\textit{Record 3}} \\\\\n\\hline\nid & 333 \\\\\nname &  \\\\\nscore & -7 \\\\\n\\multicolumn{2}{c}{\\textit{Record 4}} \\\\\n\\hline\nid & 4 \\\\\nname &  \\\\\nscore & 1000.25 \\\\\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 1;
        options.expanded = .ON;
        options.tuples_only = true;
        people_query("people latex-longtable border 1 expanded tuples only", options, "\\begin{tabular}{c|l}\n\\hline\nid & 1 \\\\\nname & alice \\\\\nscore & 10.5 \\\\\n\\hline\nid & 22 \\\\\nname & bob \\\\\nscore &  \\\\\n\\hline\nid & 333 \\\\\nname &  \\\\\nscore & -7 \\\\\n\\hline\nid & 4 \\\\\nname &  \\\\\nscore & 1000.25 \\\\\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 2;
        options.expanded = .ON;
        options.title = "People";
        people_query("people latex-longtable border 2 expanded titled", options, "\\begin{center}\nPeople\n\\end{center}\n\n\\begin{tabular}{|c|l|}\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 1}} \\\\\n\\hline\nid & 1 \\\\\nname & alice \\\\\nscore & 10.5 \\\\\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 2}} \\\\\n\\hline\nid & 22 \\\\\nname & bob \\\\\nscore &  \\\\\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 3}} \\\\\n\\hline\nid & 333 \\\\\nname &  \\\\\nscore & -7 \\\\\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 4}} \\\\\n\\hline\nid & 4 \\\\\nname &  \\\\\nscore & 1000.25 \\\\\n\\hline\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 2;
        options.expanded = .ON;
        options.tuples_only = true;
        options.title = "People";
        people_query("people latex-longtable border 2 expanded tuples only titled", options, "\\begin{tabular}{|c|l|}\n\\hline\nid & 1 \\\\\nname & alice \\\\\nscore & 10.5 \\\\\n\\hline\nid & 22 \\\\\nname & bob \\\\\nscore &  \\\\\n\\hline\nid & 333 \\\\\nname &  \\\\\nscore & -7 \\\\\n\\hline\nid & 4 \\\\\nname &  \\\\\nscore & 1000.25 \\\\\n\\hline\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 3;
        options.expanded = .ON;
        people_query("people latex-longtable border 3 expanded", options, "\\begin{tabular}{|c|l|}\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 1}} \\\\\n\\hline\nid & 1 \\\\\nname & alice \\\\\nscore & 10.5 \\\\\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 2}} \\\\\n\\hline\nid & 22 \\\\\nname & bob \\\\\nscore &  \\\\\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 3}} \\\\\n\\hline\nid & 333 \\\\\nname &  \\\\\nscore & -7 \\\\\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 4}} \\\\\n\\hline\nid & 4 \\\\\nname &  \\\\\nscore & 1000.25 \\\\\n\\hline\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 3;
        options.expanded = .ON;
        options.tuples_only = true;
        people_query("people latex-longtable border 3 expanded tuples only", options, "\\begin{tabular}{|c|l|}\n\\hline\nid & 1 \\\\\nname & alice \\\\\nscore & 10.5 \\\\\n\\hline\nid & 22 \\\\\nname & bob \\\\\nscore &  \\\\\n\\hline\nid & 333 \\\\\nname &  \\\\\nscore & -7 \\\\\n\\hline\nid & 4 \\\\\nname &  \\\\\nscore & 1000.25 \\\\\n\\hline\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 0;
        options.title = "People";
        people_query("people troff-ms border 0 titled", options, ".LP\n.DS C\nPeople\n.DE\n.LP\n.TS\ncenter;\nrlr.\n\\fIid\\fP\t\\fIname\\fP\t\\fIscore\\fP\n_\n1\talice\t10.5\n22\tbob\t\n333\t\t-7\n4\t\t1000.25\n.TE\n.DS L\n(4 rows)\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 0;
        options.tuples_only = true;
        options.title = "People";
        people_query("people troff-ms border 0 tuples only titled", options, ".LP\n.TS\ncenter;\nrlr.\n1\talice\t10.5\n22\tbob\t\n333\t\t-7\n4\t\t1000.25\n.TE\n.DS L\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 1;
        people_query("people troff-ms border 1", options, ".LP\n.TS\ncenter;\nr | l | r.\n\\fIid\\fP\t\\fIname\\fP\t\\fIscore\\fP\n_\n1\talice\t10.5\n22\tbob\t\n333\t\t-7\n4\t\t1000.25\n.TE\n.DS L\n(4 rows)\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 1;
        options.tuples_only = true;
        people_query("people troff-ms border 1 tuples only", options, ".LP\n.TS\ncenter;\nr | l | r.\n1\talice\t10.5\n22\tbob\t\n333\t\t-7\n4\t\t1000.25\n.TE\n.DS L\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 2;
        options.title = "People";
        people_query("people troff-ms border 2 titled", options, ".LP\n.DS C\nPeople\n.DE\n.LP\n.TS\ncenter box;\nr | l | r.\n\\fIid\\fP\t\\fIname\\fP\t\\fIscore\\fP\n_\n1\talice\t10.5\n22\tbob\t\n333\t\t-7\n4\t\t1000.25\n.TE\n.DS L\n(4 rows)\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 2;
        options.tuples_only = true;
        options.title = "People";
        people_query("people troff-ms border 2 tuples only titled", options, ".LP\n.TS\ncenter box;\nr | l | r.\n1\talice\t10.5\n22\tbob\t\n333\t\t-7\n4\t\t1000.25\n.TE\n.DS L\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 3;
        people_query("people troff-ms border 3", options, ".LP\n.TS\ncenter box;\nr | l | r.\n\\fIid\\fP\t\\fIname\\fP\t\\fIscore\\fP\n_\n1\talice\t10.5\n22\tbob\t\n333\t\t-7\n4\t\t1000.25\n.TE\n.DS L\n(4 rows)\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 3;
        options.tuples_only = true;
        people_query("people troff-ms border 3 tuples only", options, ".LP\n.TS\ncenter box;\nr | l | r.\n1\talice\t10.5\n22\tbob\t\n333\t\t-7\n4\t\t1000.25\n.TE\n.DS L\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 0;
        options.expanded = .ON;
        options.title = "People";
        people_query("people troff-ms border 0 expanded titled", options, ".LP\n.DS C\nPeople\n.DE\n.LP\n.TS\ncenter;\nc s.\n\\fIRecord 1\\fP\n.T&\nc l.\nid\t1\nname\talice\nscore\t10.5\n.T&\nc s.\n\\fIRecord 2\\fP\n.T&\nc l.\nid\t22\nname\tbob\nscore\t\n.T&\nc s.\n\\fIRecord 3\\fP\n.T&\nc l.\nid\t333\nname\t\nscore\t-7\n.T&\nc s.\n\\fIRecord 4\\fP\n.T&\nc l.\nid\t4\nname\t\nscore\t1000.25\n.TE\n.DS L\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 0;
        options.expanded = .ON;
        options.tuples_only = true;
        options.title = "People";
        people_query("people troff-ms border 0 expanded tuples only titled", options, ".LP\n.TS\ncenter;\nc l;\nid\t1\nname\talice\nscore\t10.5\nid\t22\nname\tbob\nscore\t\nid\t333\nname\t\nscore\t-7\nid\t4\nname\t\nscore\t1000.25\n.TE\n.DS L\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 1;
        options.expanded = .ON;
        people_query("people troff-ms border 1 expanded", options, ".LP\n.TS\ncenter;\nc s.\n\\fIRecord 1\\fP\n_\n.T&\nc | l.\nid\t1\nname\talice\nscore\t10.5\n.T&\nc s.\n\\fIRecord 2\\fP\n_\n.T&\nc | l.\nid\t22\nname\tbob\nscore\t\n.T&\nc s.\n\\fIRecord 3\\fP\n_\n.T&\nc | l.\nid\t333\nname\t\nscore\t-7\n.T&\nc s.\n\\fIRecord 4\\fP\n_\n.T&\nc | l.\nid\t4\nname\t\nscore\t1000.25\n.TE\n.DS L\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 1;
        options.expanded = .ON;
        options.tuples_only = true;
        people_query("people troff-ms border 1 expanded tuples only", options, ".LP\n.TS\ncenter;\nc l;\n_\nid\t1\nname\talice\nscore\t10.5\n_\nid\t22\nname\tbob\nscore\t\n_\nid\t333\nname\t\nscore\t-7\n_\nid\t4\nname\t\nscore\t1000.25\n.TE\n.DS L\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 2;
        options.expanded = .ON;
        options.title = "People";
        people_query("people troff-ms border 2 expanded titled", options, ".LP\n.DS C\nPeople\n.DE\n.LP\n.TS\ncenter box;\nc s.\n\\fIRecord 1\\fP\n_\n.T&\nc l.\nid\t1\nname\talice\nscore\t10.5\n_\n.T&\nc s.\n\\fIRecord 2\\fP\n_\n.T&\nc l.\nid\t22\nname\tbob\nscore\t\n_\n.T&\nc s.\n\\fIRecord 3\\fP\n_\n.T&\nc l.\nid\t333\nname\t\nscore\t-7\n_\n.T&\nc s.\n\\fIRecord 4\\fP\n_\n.T&\nc l.\nid\t4\nname\t\nscore\t1000.25\n.TE\n.DS L\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 2;
        options.expanded = .ON;
        options.tuples_only = true;
        options.title = "People";
        people_query("people troff-ms border 2 expanded tuples only titled", options, ".LP\n.TS\ncenter box;\nc l;\n_\nid\t1\nname\talice\nscore\t10.5\n_\nid\t22\nname\tbob\nscore\t\n_\nid\t333\nname\t\nscore\t-7\n_\nid\t4\nname\t\nscore\t1000.25\n.TE\n.DS L\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 3;
        options.expanded = .ON;
        people_query("people troff-ms border 3 expanded", options, ".LP\n.TS\ncenter box;\nc s.\n\\fIRecord 1\\fP\n_\n.T&\nc l.\nid\t1\nname\talice\nscore\t10.5\n_\n.T&\nc s.\n\\fIRecord 2\\fP\n_\n.T&\nc l.\nid\t22\nname\tbob\nscore\t\n_\n.T&\nc s.\n\\fIRecord 3\\fP\n_\n.T&\nc l.\nid\t333\nname\t\nscore\t-7\n_\n.T&\nc s.\n\\fIRecord 4\\fP\n_\n.T&\nc l.\nid\t4\nname\t\nscore\t1000.25\n.TE\n.DS L\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 3;
        options.expanded = .ON;
        options.tuples_only = true;
        people_query("people troff-ms border 3 expanded tuples only", options, ".LP\n.TS\ncenter box;\nc l;\n_\nid\t1\nname\talice\nscore\t10.5\n_\nid\t22\nname\tbob\nscore\t\n_\nid\t333\nname\t\nscore\t-7\n_\nid\t4\nname\t\nscore\t1000.25\n.TE\n.DS L\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 0;
        options.title = "People";
        special_query("special html border 0 titled", options, "<table border=\"0\">\n  <caption>People</caption>\n  <tr>\n    <th align=\"center\">a&amp;b</th>\n    <th align=\"center\">x&lt;y</th>\n    <th align=\"center\">q&quot;r</th>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">&lt;b&gt;&amp;amp;&quot;q&quot;</td>\n    <td align=\"left\">&nbsp;&nbsp;lead  trail</td>\n    <td align=\"right\">1</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">line1<br />\nline2</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">2</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">p|q \\ # $ % _ { } ~ ^ &gt;</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">3</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">two  spaces  inside</td>\n    <td align=\"left\">back\\slash</td>\n    <td align=\"right\">40</td>\n  </tr>\n</table>\n<p>(5 rows)<br />\n</p>\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 0;
        options.tuples_only = true;
        options.title = "People";
        special_query("special html border 0 tuples only titled", options, "<table border=\"0\">\n  <tr valign=\"top\">\n    <td align=\"left\">&lt;b&gt;&amp;amp;&quot;q&quot;</td>\n    <td align=\"left\">&nbsp;&nbsp;lead  trail</td>\n    <td align=\"right\">1</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">line1<br />\nline2</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">2</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">p|q \\ # $ % _ { } ~ ^ &gt;</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">3</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">two  spaces  inside</td>\n    <td align=\"left\">back\\slash</td>\n    <td align=\"right\">40</td>\n  </tr>\n</table>\n\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 1;
        options.title = "People";
        special_query("special html border 1 titled", options, "<table border=\"1\">\n  <caption>People</caption>\n  <tr>\n    <th align=\"center\">a&amp;b</th>\n    <th align=\"center\">x&lt;y</th>\n    <th align=\"center\">q&quot;r</th>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">&lt;b&gt;&amp;amp;&quot;q&quot;</td>\n    <td align=\"left\">&nbsp;&nbsp;lead  trail</td>\n    <td align=\"right\">1</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">line1<br />\nline2</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">2</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">p|q \\ # $ % _ { } ~ ^ &gt;</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">3</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">two  spaces  inside</td>\n    <td align=\"left\">back\\slash</td>\n    <td align=\"right\">40</td>\n  </tr>\n</table>\n<p>(5 rows)<br />\n</p>\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 1;
        options.tuples_only = true;
        special_query("special html border 1 tuples only", options, "<table border=\"1\">\n  <tr valign=\"top\">\n    <td align=\"left\">&lt;b&gt;&amp;amp;&quot;q&quot;</td>\n    <td align=\"left\">&nbsp;&nbsp;lead  trail</td>\n    <td align=\"right\">1</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">line1<br />\nline2</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">2</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">p|q \\ # $ % _ { } ~ ^ &gt;</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">3</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">two  spaces  inside</td>\n    <td align=\"left\">back\\slash</td>\n    <td align=\"right\">40</td>\n  </tr>\n</table>\n\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 2;
        options.title = "People";
        special_query("special html border 2 titled", options, "<table border=\"2\">\n  <caption>People</caption>\n  <tr>\n    <th align=\"center\">a&amp;b</th>\n    <th align=\"center\">x&lt;y</th>\n    <th align=\"center\">q&quot;r</th>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">&lt;b&gt;&amp;amp;&quot;q&quot;</td>\n    <td align=\"left\">&nbsp;&nbsp;lead  trail</td>\n    <td align=\"right\">1</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">line1<br />\nline2</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">2</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">p|q \\ # $ % _ { } ~ ^ &gt;</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">3</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">two  spaces  inside</td>\n    <td align=\"left\">back\\slash</td>\n    <td align=\"right\">40</td>\n  </tr>\n</table>\n<p>(5 rows)<br />\n</p>\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 2;
        options.tuples_only = true;
        options.title = "People";
        special_query("special html border 2 tuples only titled", options, "<table border=\"2\">\n  <tr valign=\"top\">\n    <td align=\"left\">&lt;b&gt;&amp;amp;&quot;q&quot;</td>\n    <td align=\"left\">&nbsp;&nbsp;lead  trail</td>\n    <td align=\"right\">1</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">line1<br />\nline2</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">2</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">p|q \\ # $ % _ { } ~ ^ &gt;</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">3</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">two  spaces  inside</td>\n    <td align=\"left\">back\\slash</td>\n    <td align=\"right\">40</td>\n  </tr>\n</table>\n\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 3;
        options.title = "People";
        special_query("special html border 3 titled", options, "<table border=\"3\">\n  <caption>People</caption>\n  <tr>\n    <th align=\"center\">a&amp;b</th>\n    <th align=\"center\">x&lt;y</th>\n    <th align=\"center\">q&quot;r</th>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">&lt;b&gt;&amp;amp;&quot;q&quot;</td>\n    <td align=\"left\">&nbsp;&nbsp;lead  trail</td>\n    <td align=\"right\">1</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">line1<br />\nline2</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">2</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">p|q \\ # $ % _ { } ~ ^ &gt;</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">3</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">two  spaces  inside</td>\n    <td align=\"left\">back\\slash</td>\n    <td align=\"right\">40</td>\n  </tr>\n</table>\n<p>(5 rows)<br />\n</p>\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 3;
        options.tuples_only = true;
        special_query("special html border 3 tuples only", options, "<table border=\"3\">\n  <tr valign=\"top\">\n    <td align=\"left\">&lt;b&gt;&amp;amp;&quot;q&quot;</td>\n    <td align=\"left\">&nbsp;&nbsp;lead  trail</td>\n    <td align=\"right\">1</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">line1<br />\nline2</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">2</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">p|q \\ # $ % _ { } ~ ^ &gt;</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">3</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">two  spaces  inside</td>\n    <td align=\"left\">back\\slash</td>\n    <td align=\"right\">40</td>\n  </tr>\n</table>\n\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 0;
        options.expanded = .ON;
        options.title = "People";
        special_query("special html border 0 expanded titled", options, "<table border=\"0\">\n  <caption>People</caption>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 1</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">&lt;b&gt;&amp;amp;&quot;q&quot;</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">&nbsp;&nbsp;lead  trail</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">1</td>\n  </tr>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 2</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">line1<br />\nline2</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">2</td>\n  </tr>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 3</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 4</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">p|q \\ # $ % _ { } ~ ^ &gt;</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">3</td>\n  </tr>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 5</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">two  spaces  inside</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">back\\slash</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">40</td>\n  </tr>\n</table>\n\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 0;
        options.expanded = .ON;
        options.tuples_only = true;
        options.title = "People";
        special_query("special html border 0 expanded tuples only titled", options, "<table border=\"0\">\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">&lt;b&gt;&amp;amp;&quot;q&quot;</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">&nbsp;&nbsp;lead  trail</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">1</td>\n  </tr>\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">line1<br />\nline2</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">2</td>\n  </tr>\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">p|q \\ # $ % _ { } ~ ^ &gt;</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">3</td>\n  </tr>\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">two  spaces  inside</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">back\\slash</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">40</td>\n  </tr>\n</table>\n\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 1;
        options.expanded = .ON;
        options.title = "People";
        special_query("special html border 1 expanded titled", options, "<table border=\"1\">\n  <caption>People</caption>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 1</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">&lt;b&gt;&amp;amp;&quot;q&quot;</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">&nbsp;&nbsp;lead  trail</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">1</td>\n  </tr>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 2</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">line1<br />\nline2</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">2</td>\n  </tr>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 3</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 4</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">p|q \\ # $ % _ { } ~ ^ &gt;</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">3</td>\n  </tr>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 5</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">two  spaces  inside</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">back\\slash</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">40</td>\n  </tr>\n</table>\n\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 1;
        options.expanded = .ON;
        options.tuples_only = true;
        special_query("special html border 1 expanded tuples only", options, "<table border=\"1\">\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">&lt;b&gt;&amp;amp;&quot;q&quot;</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">&nbsp;&nbsp;lead  trail</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">1</td>\n  </tr>\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">line1<br />\nline2</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">2</td>\n  </tr>\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">p|q \\ # $ % _ { } ~ ^ &gt;</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">3</td>\n  </tr>\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">two  spaces  inside</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">back\\slash</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">40</td>\n  </tr>\n</table>\n\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 2;
        options.expanded = .ON;
        options.title = "People";
        special_query("special html border 2 expanded titled", options, "<table border=\"2\">\n  <caption>People</caption>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 1</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">&lt;b&gt;&amp;amp;&quot;q&quot;</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">&nbsp;&nbsp;lead  trail</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">1</td>\n  </tr>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 2</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">line1<br />\nline2</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">2</td>\n  </tr>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 3</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 4</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">p|q \\ # $ % _ { } ~ ^ &gt;</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">3</td>\n  </tr>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 5</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">two  spaces  inside</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">back\\slash</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">40</td>\n  </tr>\n</table>\n\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 2;
        options.expanded = .ON;
        options.tuples_only = true;
        options.title = "People";
        special_query("special html border 2 expanded tuples only titled", options, "<table border=\"2\">\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">&lt;b&gt;&amp;amp;&quot;q&quot;</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">&nbsp;&nbsp;lead  trail</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">1</td>\n  </tr>\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">line1<br />\nline2</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">2</td>\n  </tr>\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">p|q \\ # $ % _ { } ~ ^ &gt;</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">3</td>\n  </tr>\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">two  spaces  inside</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">back\\slash</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">40</td>\n  </tr>\n</table>\n\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 3;
        options.expanded = .ON;
        options.title = "People";
        special_query("special html border 3 expanded titled", options, "<table border=\"3\">\n  <caption>People</caption>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 1</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">&lt;b&gt;&amp;amp;&quot;q&quot;</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">&nbsp;&nbsp;lead  trail</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">1</td>\n  </tr>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 2</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">line1<br />\nline2</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">2</td>\n  </tr>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 3</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 4</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">p|q \\ # $ % _ { } ~ ^ &gt;</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">3</td>\n  </tr>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 5</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">two  spaces  inside</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">back\\slash</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">40</td>\n  </tr>\n</table>\n\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 3;
        options.expanded = .ON;
        options.tuples_only = true;
        special_query("special html border 3 expanded tuples only", options, "<table border=\"3\">\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">&lt;b&gt;&amp;amp;&quot;q&quot;</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">&nbsp;&nbsp;lead  trail</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">1</td>\n  </tr>\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">line1<br />\nline2</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">2</td>\n  </tr>\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">p|q \\ # $ % _ { } ~ ^ &gt;</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">3</td>\n  </tr>\n\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n  <tr valign=\"top\">\n    <th>a&amp;b</th>\n    <td align=\"left\">two  spaces  inside</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>x&lt;y</th>\n    <td align=\"left\">back\\slash</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>q&quot;r</th>\n    <td align=\"right\">40</td>\n  </tr>\n</table>\n\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 0;
        options.title = "People";
        special_query("special asciidoc border 0 titled", options, "\n.People\n[options=\"header\",cols=\"<l,<l,>l\",frame=\"none\",grid=\"none\"]\n|====\n^l|a&b ^l|x<y ^l|q\"r\n|<b>&amp;\"q\" |  lead  trail |1\n|line1\nline2 |  |2\n|  |  |\n|p\\|q \\ # $ % _ { } ~ ^ > |  |3\n|two  spaces  inside |back\\slash |40\n|====\n\n....\n(5 rows)\n....\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 0;
        options.tuples_only = true;
        options.title = "People";
        special_query("special asciidoc border 0 tuples only titled", options, "\n[cols=\"<l,<l,>l\",frame=\"none\",grid=\"none\"]\n|====\n|<b>&amp;\"q\" |  lead  trail |1\n|line1\nline2 |  |2\n|  |  |\n|p\\|q \\ # $ % _ { } ~ ^ > |  |3\n|two  spaces  inside |back\\slash |40\n|====\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 1;
        options.title = "People";
        special_query("special asciidoc border 1 titled", options, "\n.People\n[options=\"header\",cols=\"<l,<l,>l\",frame=\"none\"]\n|====\n^l|a&b ^l|x<y ^l|q\"r\n|<b>&amp;\"q\" |  lead  trail |1\n|line1\nline2 |  |2\n|  |  |\n|p\\|q \\ # $ % _ { } ~ ^ > |  |3\n|two  spaces  inside |back\\slash |40\n|====\n\n....\n(5 rows)\n....\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 1;
        options.tuples_only = true;
        special_query("special asciidoc border 1 tuples only", options, "\n[cols=\"<l,<l,>l\",frame=\"none\"]\n|====\n|<b>&amp;\"q\" |  lead  trail |1\n|line1\nline2 |  |2\n|  |  |\n|p\\|q \\ # $ % _ { } ~ ^ > |  |3\n|two  spaces  inside |back\\slash |40\n|====\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 2;
        options.title = "People";
        special_query("special asciidoc border 2 titled", options, "\n.People\n[options=\"header\",cols=\"<l,<l,>l\",frame=\"all\",grid=\"all\"]\n|====\n^l|a&b ^l|x<y ^l|q\"r\n|<b>&amp;\"q\" |  lead  trail |1\n|line1\nline2 |  |2\n|  |  |\n|p\\|q \\ # $ % _ { } ~ ^ > |  |3\n|two  spaces  inside |back\\slash |40\n|====\n\n....\n(5 rows)\n....\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 2;
        options.tuples_only = true;
        options.title = "People";
        special_query("special asciidoc border 2 tuples only titled", options, "\n[cols=\"<l,<l,>l\",frame=\"all\",grid=\"all\"]\n|====\n|<b>&amp;\"q\" |  lead  trail |1\n|line1\nline2 |  |2\n|  |  |\n|p\\|q \\ # $ % _ { } ~ ^ > |  |3\n|two  spaces  inside |back\\slash |40\n|====\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 3;
        options.title = "People";
        special_query("special asciidoc border 3 titled", options, "\n.People\n[options=\"header\",cols=\"<l,<l,>l\"]\n|====\n^l|a&b ^l|x<y ^l|q\"r\n|<b>&amp;\"q\" |  lead  trail |1\n|line1\nline2 |  |2\n|  |  |\n|p\\|q \\ # $ % _ { } ~ ^ > |  |3\n|two  spaces  inside |back\\slash |40\n|====\n\n....\n(5 rows)\n....\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 3;
        options.tuples_only = true;
        special_query("special asciidoc border 3 tuples only", options, "\n[cols=\"<l,<l,>l\"]\n|====\n|<b>&amp;\"q\" |  lead  trail |1\n|line1\nline2 |  |2\n|  |  |\n|p\\|q \\ # $ % _ { } ~ ^ > |  |3\n|two  spaces  inside |back\\slash |40\n|====\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 0;
        options.expanded = .ON;
        options.title = "People";
        special_query("special asciidoc border 0 expanded titled", options, "\n.People\n[cols=\"h,l\",frame=\"none\",grid=\"none\"]\n|====\n2+^|Record 1\n<l|a&b <l|<b>&amp;\"q\"\n<l|x<y <l|  lead  trail\n<l|q\"r >l|1\n2+^|Record 2\n<l|a&b <l|line1\nline2\n<l|x<y <l| \n<l|q\"r >l|2\n2+^|Record 3\n<l|a&b <l| \n<l|x<y <l| \n<l|q\"r >l| \n2+^|Record 4\n<l|a&b <l|p\\|q \\ # $ % _ { } ~ ^ >\n<l|x<y <l| \n<l|q\"r >l|3\n2+^|Record 5\n<l|a&b <l|two  spaces  inside\n<l|x<y <l|back\\slash\n<l|q\"r >l|40\n|====\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 0;
        options.expanded = .ON;
        options.tuples_only = true;
        options.title = "People";
        special_query("special asciidoc border 0 expanded tuples only titled", options, "\n[cols=\"h,l\",frame=\"none\",grid=\"none\"]\n|====\n2+|\n<l|a&b <l|<b>&amp;\"q\"\n<l|x<y <l|  lead  trail\n<l|q\"r >l|1\n2+|\n<l|a&b <l|line1\nline2\n<l|x<y <l| \n<l|q\"r >l|2\n2+|\n<l|a&b <l| \n<l|x<y <l| \n<l|q\"r >l| \n2+|\n<l|a&b <l|p\\|q \\ # $ % _ { } ~ ^ >\n<l|x<y <l| \n<l|q\"r >l|3\n2+|\n<l|a&b <l|two  spaces  inside\n<l|x<y <l|back\\slash\n<l|q\"r >l|40\n|====\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 1;
        options.expanded = .ON;
        options.title = "People";
        special_query("special asciidoc border 1 expanded titled", options, "\n.People\n[cols=\"h,l\",frame=\"none\"]\n|====\n2+^|Record 1\n<l|a&b <l|<b>&amp;\"q\"\n<l|x<y <l|  lead  trail\n<l|q\"r >l|1\n2+^|Record 2\n<l|a&b <l|line1\nline2\n<l|x<y <l| \n<l|q\"r >l|2\n2+^|Record 3\n<l|a&b <l| \n<l|x<y <l| \n<l|q\"r >l| \n2+^|Record 4\n<l|a&b <l|p\\|q \\ # $ % _ { } ~ ^ >\n<l|x<y <l| \n<l|q\"r >l|3\n2+^|Record 5\n<l|a&b <l|two  spaces  inside\n<l|x<y <l|back\\slash\n<l|q\"r >l|40\n|====\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 1;
        options.expanded = .ON;
        options.tuples_only = true;
        special_query("special asciidoc border 1 expanded tuples only", options, "\n[cols=\"h,l\",frame=\"none\"]\n|====\n2+|\n<l|a&b <l|<b>&amp;\"q\"\n<l|x<y <l|  lead  trail\n<l|q\"r >l|1\n2+|\n<l|a&b <l|line1\nline2\n<l|x<y <l| \n<l|q\"r >l|2\n2+|\n<l|a&b <l| \n<l|x<y <l| \n<l|q\"r >l| \n2+|\n<l|a&b <l|p\\|q \\ # $ % _ { } ~ ^ >\n<l|x<y <l| \n<l|q\"r >l|3\n2+|\n<l|a&b <l|two  spaces  inside\n<l|x<y <l|back\\slash\n<l|q\"r >l|40\n|====\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 2;
        options.expanded = .ON;
        options.title = "People";
        special_query("special asciidoc border 2 expanded titled", options, "\n.People\n[cols=\"h,l\",frame=\"all\",grid=\"all\"]\n|====\n2+^|Record 1\n<l|a&b <l|<b>&amp;\"q\"\n<l|x<y <l|  lead  trail\n<l|q\"r >l|1\n2+^|Record 2\n<l|a&b <l|line1\nline2\n<l|x<y <l| \n<l|q\"r >l|2\n2+^|Record 3\n<l|a&b <l| \n<l|x<y <l| \n<l|q\"r >l| \n2+^|Record 4\n<l|a&b <l|p\\|q \\ # $ % _ { } ~ ^ >\n<l|x<y <l| \n<l|q\"r >l|3\n2+^|Record 5\n<l|a&b <l|two  spaces  inside\n<l|x<y <l|back\\slash\n<l|q\"r >l|40\n|====\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 2;
        options.expanded = .ON;
        options.tuples_only = true;
        options.title = "People";
        special_query("special asciidoc border 2 expanded tuples only titled", options, "\n[cols=\"h,l\",frame=\"all\",grid=\"all\"]\n|====\n2+|\n<l|a&b <l|<b>&amp;\"q\"\n<l|x<y <l|  lead  trail\n<l|q\"r >l|1\n2+|\n<l|a&b <l|line1\nline2\n<l|x<y <l| \n<l|q\"r >l|2\n2+|\n<l|a&b <l| \n<l|x<y <l| \n<l|q\"r >l| \n2+|\n<l|a&b <l|p\\|q \\ # $ % _ { } ~ ^ >\n<l|x<y <l| \n<l|q\"r >l|3\n2+|\n<l|a&b <l|two  spaces  inside\n<l|x<y <l|back\\slash\n<l|q\"r >l|40\n|====\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 3;
        options.expanded = .ON;
        options.title = "People";
        special_query("special asciidoc border 3 expanded titled", options, "\n.People\n[cols=\"h,l\"]\n|====\n2+^|Record 1\n<l|a&b <l|<b>&amp;\"q\"\n<l|x<y <l|  lead  trail\n<l|q\"r >l|1\n2+^|Record 2\n<l|a&b <l|line1\nline2\n<l|x<y <l| \n<l|q\"r >l|2\n2+^|Record 3\n<l|a&b <l| \n<l|x<y <l| \n<l|q\"r >l| \n2+^|Record 4\n<l|a&b <l|p\\|q \\ # $ % _ { } ~ ^ >\n<l|x<y <l| \n<l|q\"r >l|3\n2+^|Record 5\n<l|a&b <l|two  spaces  inside\n<l|x<y <l|back\\slash\n<l|q\"r >l|40\n|====\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 3;
        options.expanded = .ON;
        options.tuples_only = true;
        special_query("special asciidoc border 3 expanded tuples only", options, "\n[cols=\"h,l\"]\n|====\n2+|\n<l|a&b <l|<b>&amp;\"q\"\n<l|x<y <l|  lead  trail\n<l|q\"r >l|1\n2+|\n<l|a&b <l|line1\nline2\n<l|x<y <l| \n<l|q\"r >l|2\n2+|\n<l|a&b <l| \n<l|x<y <l| \n<l|q\"r >l| \n2+|\n<l|a&b <l|p\\|q \\ # $ % _ { } ~ ^ >\n<l|x<y <l| \n<l|q\"r >l|3\n2+|\n<l|a&b <l|two  spaces  inside\n<l|x<y <l|back\\slash\n<l|q\"r >l|40\n|====\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 0;
        options.title = "People";
        special_query("special latex border 0 titled", options, "\\begin{center}\nPeople\n\\end{center}\n\n\\begin{tabular}{llr}\n\\textit{a\\&b} & \\textit{x\\textless{}y} & \\textit{q\"r} \\\\\n\\hline\n\\textless{}b\\textgreater{}\\&amp;\"q\" &   lead  trail & 1 \\\\\nline1\\\\line2 &   & 2 \\\\\n &  &  \\\\\np\\textbar{}q \\textbackslash{} \\# \\$ \\% \\_ \\{ \\} \\~{} \\^{} \\textgreater{} & \t & 3 \\\\\ntwo  spaces  inside & back\\textbackslash{}slash & 40 \\\\\n\\end{tabular}\n\n\\noindent (5 rows) \\\\\n\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 0;
        options.tuples_only = true;
        options.title = "People";
        special_query("special latex border 0 tuples only titled", options, "\\begin{tabular}{llr}\n\\textless{}b\\textgreater{}\\&amp;\"q\" &   lead  trail & 1 \\\\\nline1\\\\line2 &   & 2 \\\\\n &  &  \\\\\np\\textbar{}q \\textbackslash{} \\# \\$ \\% \\_ \\{ \\} \\~{} \\^{} \\textgreater{} & \t & 3 \\\\\ntwo  spaces  inside & back\\textbackslash{}slash & 40 \\\\\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 1;
        options.title = "People";
        special_query("special latex border 1 titled", options, "\\begin{center}\nPeople\n\\end{center}\n\n\\begin{tabular}{l | l | r}\n\\textit{a\\&b} & \\textit{x\\textless{}y} & \\textit{q\"r} \\\\\n\\hline\n\\textless{}b\\textgreater{}\\&amp;\"q\" &   lead  trail & 1 \\\\\nline1\\\\line2 &   & 2 \\\\\n &  &  \\\\\np\\textbar{}q \\textbackslash{} \\# \\$ \\% \\_ \\{ \\} \\~{} \\^{} \\textgreater{} & \t & 3 \\\\\ntwo  spaces  inside & back\\textbackslash{}slash & 40 \\\\\n\\end{tabular}\n\n\\noindent (5 rows) \\\\\n\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 1;
        options.tuples_only = true;
        special_query("special latex border 1 tuples only", options, "\\begin{tabular}{l | l | r}\n\\textless{}b\\textgreater{}\\&amp;\"q\" &   lead  trail & 1 \\\\\nline1\\\\line2 &   & 2 \\\\\n &  &  \\\\\np\\textbar{}q \\textbackslash{} \\# \\$ \\% \\_ \\{ \\} \\~{} \\^{} \\textgreater{} & \t & 3 \\\\\ntwo  spaces  inside & back\\textbackslash{}slash & 40 \\\\\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 2;
        options.title = "People";
        special_query("special latex border 2 titled", options, "\\begin{center}\nPeople\n\\end{center}\n\n\\begin{tabular}{| l | l | r |}\n\\hline\n\\textit{a\\&b} & \\textit{x\\textless{}y} & \\textit{q\"r} \\\\\n\\hline\n\\textless{}b\\textgreater{}\\&amp;\"q\" &   lead  trail & 1 \\\\\nline1\\\\line2 &   & 2 \\\\\n &  &  \\\\\np\\textbar{}q \\textbackslash{} \\# \\$ \\% \\_ \\{ \\} \\~{} \\^{} \\textgreater{} & \t & 3 \\\\\ntwo  spaces  inside & back\\textbackslash{}slash & 40 \\\\\n\\hline\n\\end{tabular}\n\n\\noindent (5 rows) \\\\\n\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 2;
        options.tuples_only = true;
        options.title = "People";
        special_query("special latex border 2 tuples only titled", options, "\\begin{tabular}{| l | l | r |}\n\\textless{}b\\textgreater{}\\&amp;\"q\" &   lead  trail & 1 \\\\\nline1\\\\line2 &   & 2 \\\\\n &  &  \\\\\np\\textbar{}q \\textbackslash{} \\# \\$ \\% \\_ \\{ \\} \\~{} \\^{} \\textgreater{} & \t & 3 \\\\\ntwo  spaces  inside & back\\textbackslash{}slash & 40 \\\\\n\\hline\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 3;
        options.title = "People";
        special_query("special latex border 3 titled", options, "\\begin{center}\nPeople\n\\end{center}\n\n\\begin{tabular}{| l | l | r |}\n\\hline\n\\textit{a\\&b} & \\textit{x\\textless{}y} & \\textit{q\"r} \\\\\n\\hline\n\\textless{}b\\textgreater{}\\&amp;\"q\" &   lead  trail & 1 \\\\\n\\hline\nline1\\\\line2 &   & 2 \\\\\n\\hline\n &  &  \\\\\n\\hline\np\\textbar{}q \\textbackslash{} \\# \\$ \\% \\_ \\{ \\} \\~{} \\^{} \\textgreater{} & \t & 3 \\\\\n\\hline\ntwo  spaces  inside & back\\textbackslash{}slash & 40 \\\\\n\\hline\n\\end{tabular}\n\n\\noindent (5 rows) \\\\\n\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 3;
        options.tuples_only = true;
        special_query("special latex border 3 tuples only", options, "\\begin{tabular}{| l | l | r |}\n\\textless{}b\\textgreater{}\\&amp;\"q\" &   lead  trail & 1 \\\\\n\\hline\nline1\\\\line2 &   & 2 \\\\\n\\hline\n &  &  \\\\\n\\hline\np\\textbar{}q \\textbackslash{} \\# \\$ \\% \\_ \\{ \\} \\~{} \\^{} \\textgreater{} & \t & 3 \\\\\n\\hline\ntwo  spaces  inside & back\\textbackslash{}slash & 40 \\\\\n\\hline\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 0;
        options.expanded = .ON;
        options.title = "People";
        special_query("special latex border 0 expanded titled", options, "\\begin{center}\nPeople\n\\end{center}\n\n\\begin{tabular}{cl}\n\\multicolumn{2}{c}{\\textit{Record 1}} \\\\\na\\&b & \\textless{}b\\textgreater{}\\&amp;\"q\" \\\\\nx\\textless{}y &   lead  trail \\\\\nq\"r & 1 \\\\\n\\multicolumn{2}{c}{\\textit{Record 2}} \\\\\na\\&b & line1\\\\line2 \\\\\nx\\textless{}y &   \\\\\nq\"r & 2 \\\\\n\\multicolumn{2}{c}{\\textit{Record 3}} \\\\\na\\&b &  \\\\\nx\\textless{}y &  \\\\\nq\"r &  \\\\\n\\multicolumn{2}{c}{\\textit{Record 4}} \\\\\na\\&b & p\\textbar{}q \\textbackslash{} \\# \\$ \\% \\_ \\{ \\} \\~{} \\^{} \\textgreater{} \\\\\nx\\textless{}y & \t \\\\\nq\"r & 3 \\\\\n\\multicolumn{2}{c}{\\textit{Record 5}} \\\\\na\\&b & two  spaces  inside \\\\\nx\\textless{}y & back\\textbackslash{}slash \\\\\nq\"r & 40 \\\\\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 0;
        options.expanded = .ON;
        options.tuples_only = true;
        options.title = "People";
        special_query("special latex border 0 expanded tuples only titled", options, "\\begin{tabular}{cl}\na\\&b & \\textless{}b\\textgreater{}\\&amp;\"q\" \\\\\nx\\textless{}y &   lead  trail \\\\\nq\"r & 1 \\\\\na\\&b & line1\\\\line2 \\\\\nx\\textless{}y &   \\\\\nq\"r & 2 \\\\\na\\&b &  \\\\\nx\\textless{}y &  \\\\\nq\"r &  \\\\\na\\&b & p\\textbar{}q \\textbackslash{} \\# \\$ \\% \\_ \\{ \\} \\~{} \\^{} \\textgreater{} \\\\\nx\\textless{}y & \t \\\\\nq\"r & 3 \\\\\na\\&b & two  spaces  inside \\\\\nx\\textless{}y & back\\textbackslash{}slash \\\\\nq\"r & 40 \\\\\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 1;
        options.expanded = .ON;
        options.title = "People";
        special_query("special latex border 1 expanded titled", options, "\\begin{center}\nPeople\n\\end{center}\n\n\\begin{tabular}{c|l}\n\\multicolumn{2}{c}{\\textit{Record 1}} \\\\\n\\hline\na\\&b & \\textless{}b\\textgreater{}\\&amp;\"q\" \\\\\nx\\textless{}y &   lead  trail \\\\\nq\"r & 1 \\\\\n\\multicolumn{2}{c}{\\textit{Record 2}} \\\\\n\\hline\na\\&b & line1\\\\line2 \\\\\nx\\textless{}y &   \\\\\nq\"r & 2 \\\\\n\\multicolumn{2}{c}{\\textit{Record 3}} \\\\\n\\hline\na\\&b &  \\\\\nx\\textless{}y &  \\\\\nq\"r &  \\\\\n\\multicolumn{2}{c}{\\textit{Record 4}} \\\\\n\\hline\na\\&b & p\\textbar{}q \\textbackslash{} \\# \\$ \\% \\_ \\{ \\} \\~{} \\^{} \\textgreater{} \\\\\nx\\textless{}y & \t \\\\\nq\"r & 3 \\\\\n\\multicolumn{2}{c}{\\textit{Record 5}} \\\\\n\\hline\na\\&b & two  spaces  inside \\\\\nx\\textless{}y & back\\textbackslash{}slash \\\\\nq\"r & 40 \\\\\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 1;
        options.expanded = .ON;
        options.tuples_only = true;
        special_query("special latex border 1 expanded tuples only", options, "\\begin{tabular}{c|l}\n\\hline\na\\&b & \\textless{}b\\textgreater{}\\&amp;\"q\" \\\\\nx\\textless{}y &   lead  trail \\\\\nq\"r & 1 \\\\\n\\hline\na\\&b & line1\\\\line2 \\\\\nx\\textless{}y &   \\\\\nq\"r & 2 \\\\\n\\hline\na\\&b &  \\\\\nx\\textless{}y &  \\\\\nq\"r &  \\\\\n\\hline\na\\&b & p\\textbar{}q \\textbackslash{} \\# \\$ \\% \\_ \\{ \\} \\~{} \\^{} \\textgreater{} \\\\\nx\\textless{}y & \t \\\\\nq\"r & 3 \\\\\n\\hline\na\\&b & two  spaces  inside \\\\\nx\\textless{}y & back\\textbackslash{}slash \\\\\nq\"r & 40 \\\\\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 2;
        options.expanded = .ON;
        options.title = "People";
        special_query("special latex border 2 expanded titled", options, "\\begin{center}\nPeople\n\\end{center}\n\n\\begin{tabular}{|c|l|}\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 1}} \\\\\n\\hline\na\\&b & \\textless{}b\\textgreater{}\\&amp;\"q\" \\\\\nx\\textless{}y &   lead  trail \\\\\nq\"r & 1 \\\\\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 2}} \\\\\n\\hline\na\\&b & line1\\\\line2 \\\\\nx\\textless{}y &   \\\\\nq\"r & 2 \\\\\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 3}} \\\\\n\\hline\na\\&b &  \\\\\nx\\textless{}y &  \\\\\nq\"r &  \\\\\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 4}} \\\\\n\\hline\na\\&b & p\\textbar{}q \\textbackslash{} \\# \\$ \\% \\_ \\{ \\} \\~{} \\^{} \\textgreater{} \\\\\nx\\textless{}y & \t \\\\\nq\"r & 3 \\\\\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 5}} \\\\\n\\hline\na\\&b & two  spaces  inside \\\\\nx\\textless{}y & back\\textbackslash{}slash \\\\\nq\"r & 40 \\\\\n\\hline\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 2;
        options.expanded = .ON;
        options.tuples_only = true;
        options.title = "People";
        special_query("special latex border 2 expanded tuples only titled", options, "\\begin{tabular}{|c|l|}\n\\hline\na\\&b & \\textless{}b\\textgreater{}\\&amp;\"q\" \\\\\nx\\textless{}y &   lead  trail \\\\\nq\"r & 1 \\\\\n\\hline\na\\&b & line1\\\\line2 \\\\\nx\\textless{}y &   \\\\\nq\"r & 2 \\\\\n\\hline\na\\&b &  \\\\\nx\\textless{}y &  \\\\\nq\"r &  \\\\\n\\hline\na\\&b & p\\textbar{}q \\textbackslash{} \\# \\$ \\% \\_ \\{ \\} \\~{} \\^{} \\textgreater{} \\\\\nx\\textless{}y & \t \\\\\nq\"r & 3 \\\\\n\\hline\na\\&b & two  spaces  inside \\\\\nx\\textless{}y & back\\textbackslash{}slash \\\\\nq\"r & 40 \\\\\n\\hline\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 3;
        options.expanded = .ON;
        options.title = "People";
        special_query("special latex border 3 expanded titled", options, "\\begin{center}\nPeople\n\\end{center}\n\n\\begin{tabular}{|c|l|}\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 1}} \\\\\n\\hline\na\\&b & \\textless{}b\\textgreater{}\\&amp;\"q\" \\\\\nx\\textless{}y &   lead  trail \\\\\nq\"r & 1 \\\\\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 2}} \\\\\n\\hline\na\\&b & line1\\\\line2 \\\\\nx\\textless{}y &   \\\\\nq\"r & 2 \\\\\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 3}} \\\\\n\\hline\na\\&b &  \\\\\nx\\textless{}y &  \\\\\nq\"r &  \\\\\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 4}} \\\\\n\\hline\na\\&b & p\\textbar{}q \\textbackslash{} \\# \\$ \\% \\_ \\{ \\} \\~{} \\^{} \\textgreater{} \\\\\nx\\textless{}y & \t \\\\\nq\"r & 3 \\\\\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 5}} \\\\\n\\hline\na\\&b & two  spaces  inside \\\\\nx\\textless{}y & back\\textbackslash{}slash \\\\\nq\"r & 40 \\\\\n\\hline\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 3;
        options.expanded = .ON;
        options.tuples_only = true;
        special_query("special latex border 3 expanded tuples only", options, "\\begin{tabular}{|c|l|}\n\\hline\na\\&b & \\textless{}b\\textgreater{}\\&amp;\"q\" \\\\\nx\\textless{}y &   lead  trail \\\\\nq\"r & 1 \\\\\n\\hline\na\\&b & line1\\\\line2 \\\\\nx\\textless{}y &   \\\\\nq\"r & 2 \\\\\n\\hline\na\\&b &  \\\\\nx\\textless{}y &  \\\\\nq\"r &  \\\\\n\\hline\na\\&b & p\\textbar{}q \\textbackslash{} \\# \\$ \\% \\_ \\{ \\} \\~{} \\^{} \\textgreater{} \\\\\nx\\textless{}y & \t \\\\\nq\"r & 3 \\\\\n\\hline\na\\&b & two  spaces  inside \\\\\nx\\textless{}y & back\\textbackslash{}slash \\\\\nq\"r & 40 \\\\\n\\hline\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 0;
        options.title = "People";
        special_query("special latex-longtable border 0 titled", options, "\\begin{longtable}{llr}\n\\small\\textbf{\\textit{a\\&b}} & \\small\\textbf{\\textit{x\\textless{}y}} & \\small\\textbf{\\textit{q\"r}} \\\\\n\\midrule\n\\endfirsthead\n\\small\\textbf{\\textit{a\\&b}} & \\small\\textbf{\\textit{x\\textless{}y}} & \\small\\textbf{\\textit{q\"r}} \\\\\n\\midrule\n\\endhead\n\\caption[People (Continued)]{People}\n\\endfoot\n\\caption[People]{People}\n\\endlastfoot\n\\raggedright{\\textless{}b\\textgreater{}\\&amp;\"q\"}\n&\n\\raggedright{  lead  trail}\n&\n\\raggedright{1} \\tabularnewline\n\\raggedright{line1\\\\line2}\n&\n\\raggedright{ }\n&\n\\raggedright{2} \\tabularnewline\n\\raggedright{}\n&\n\\raggedright{}\n&\n\\raggedright{} \\tabularnewline\n\\raggedright{p\\textbar{}q \\textbackslash{} \\# \\$ \\% \\_ \\{ \\} \\~{} \\^{} \\textgreater{}}\n&\n\\raggedright{\t}\n&\n\\raggedright{3} \\tabularnewline\n\\raggedright{two  spaces  inside}\n&\n\\raggedright{back\\textbackslash{}slash}\n&\n\\raggedright{40} \\tabularnewline\n\\end{longtable}\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 0;
        options.tuples_only = true;
        options.title = "People";
        special_query("special latex-longtable border 0 tuples only titled", options, "\\begin{longtable}{llr}\n\\raggedright{\\textless{}b\\textgreater{}\\&amp;\"q\"}\n&\n\\raggedright{  lead  trail}\n&\n\\raggedright{1} \\tabularnewline\n\\raggedright{line1\\\\line2}\n&\n\\raggedright{ }\n&\n\\raggedright{2} \\tabularnewline\n\\raggedright{}\n&\n\\raggedright{}\n&\n\\raggedright{} \\tabularnewline\n\\raggedright{p\\textbar{}q \\textbackslash{} \\# \\$ \\% \\_ \\{ \\} \\~{} \\^{} \\textgreater{}}\n&\n\\raggedright{\t}\n&\n\\raggedright{3} \\tabularnewline\n\\raggedright{two  spaces  inside}\n&\n\\raggedright{back\\textbackslash{}slash}\n&\n\\raggedright{40} \\tabularnewline\n\\end{longtable}\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 1;
        options.title = "People";
        special_query("special latex-longtable border 1 titled", options, "\\begin{longtable}{l | l | r}\n\\small\\textbf{\\textit{a\\&b}} & \\small\\textbf{\\textit{x\\textless{}y}} & \\small\\textbf{\\textit{q\"r}} \\\\\n\\midrule\n\\endfirsthead\n\\small\\textbf{\\textit{a\\&b}} & \\small\\textbf{\\textit{x\\textless{}y}} & \\small\\textbf{\\textit{q\"r}} \\\\\n\\midrule\n\\endhead\n\\caption[People (Continued)]{People}\n\\endfoot\n\\caption[People]{People}\n\\endlastfoot\n\\raggedright{\\textless{}b\\textgreater{}\\&amp;\"q\"}\n&\n\\raggedright{  lead  trail}\n&\n\\raggedright{1} \\tabularnewline\n\\raggedright{line1\\\\line2}\n&\n\\raggedright{ }\n&\n\\raggedright{2} \\tabularnewline\n\\raggedright{}\n&\n\\raggedright{}\n&\n\\raggedright{} \\tabularnewline\n\\raggedright{p\\textbar{}q \\textbackslash{} \\# \\$ \\% \\_ \\{ \\} \\~{} \\^{} \\textgreater{}}\n&\n\\raggedright{\t}\n&\n\\raggedright{3} \\tabularnewline\n\\raggedright{two  spaces  inside}\n&\n\\raggedright{back\\textbackslash{}slash}\n&\n\\raggedright{40} \\tabularnewline\n\\end{longtable}\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 1;
        options.tuples_only = true;
        special_query("special latex-longtable border 1 tuples only", options, "\\begin{longtable}{l | l | r}\n\\raggedright{\\textless{}b\\textgreater{}\\&amp;\"q\"}\n&\n\\raggedright{  lead  trail}\n&\n\\raggedright{1} \\tabularnewline\n\\raggedright{line1\\\\line2}\n&\n\\raggedright{ }\n&\n\\raggedright{2} \\tabularnewline\n\\raggedright{}\n&\n\\raggedright{}\n&\n\\raggedright{} \\tabularnewline\n\\raggedright{p\\textbar{}q \\textbackslash{} \\# \\$ \\% \\_ \\{ \\} \\~{} \\^{} \\textgreater{}}\n&\n\\raggedright{\t}\n&\n\\raggedright{3} \\tabularnewline\n\\raggedright{two  spaces  inside}\n&\n\\raggedright{back\\textbackslash{}slash}\n&\n\\raggedright{40} \\tabularnewline\n\\end{longtable}\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 2;
        options.title = "People";
        special_query("special latex-longtable border 2 titled", options, "\\begin{longtable}{| l | l | r |}\n\\toprule\n\\small\\textbf{\\textit{a\\&b}} & \\small\\textbf{\\textit{x\\textless{}y}} & \\small\\textbf{\\textit{q\"r}} \\\\\n\\midrule\n\\endfirsthead\n\\toprule\n\\small\\textbf{\\textit{a\\&b}} & \\small\\textbf{\\textit{x\\textless{}y}} & \\small\\textbf{\\textit{q\"r}} \\\\\n\\midrule\n\\endhead\n\\bottomrule\n\\caption[People (Continued)]{People}\n\\endfoot\n\\bottomrule\n\\caption[People]{People}\n\\endlastfoot\n\\raggedright{\\textless{}b\\textgreater{}\\&amp;\"q\"}\n&\n\\raggedright{  lead  trail}\n&\n\\raggedright{1} \\tabularnewline\n\\raggedright{line1\\\\line2}\n&\n\\raggedright{ }\n&\n\\raggedright{2} \\tabularnewline\n\\raggedright{}\n&\n\\raggedright{}\n&\n\\raggedright{} \\tabularnewline\n\\raggedright{p\\textbar{}q \\textbackslash{} \\# \\$ \\% \\_ \\{ \\} \\~{} \\^{} \\textgreater{}}\n&\n\\raggedright{\t}\n&\n\\raggedright{3} \\tabularnewline\n\\raggedright{two  spaces  inside}\n&\n\\raggedright{back\\textbackslash{}slash}\n&\n\\raggedright{40} \\tabularnewline\n\\end{longtable}\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 2;
        options.tuples_only = true;
        options.title = "People";
        special_query("special latex-longtable border 2 tuples only titled", options, "\\begin{longtable}{| l | l | r |}\n\\raggedright{\\textless{}b\\textgreater{}\\&amp;\"q\"}\n&\n\\raggedright{  lead  trail}\n&\n\\raggedright{1} \\tabularnewline\n\\raggedright{line1\\\\line2}\n&\n\\raggedright{ }\n&\n\\raggedright{2} \\tabularnewline\n\\raggedright{}\n&\n\\raggedright{}\n&\n\\raggedright{} \\tabularnewline\n\\raggedright{p\\textbar{}q \\textbackslash{} \\# \\$ \\% \\_ \\{ \\} \\~{} \\^{} \\textgreater{}}\n&\n\\raggedright{\t}\n&\n\\raggedright{3} \\tabularnewline\n\\raggedright{two  spaces  inside}\n&\n\\raggedright{back\\textbackslash{}slash}\n&\n\\raggedright{40} \\tabularnewline\n\\end{longtable}\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 3;
        options.title = "People";
        special_query("special latex-longtable border 3 titled", options, "\\begin{longtable}{| l | l | r |}\n\\toprule\n\\small\\textbf{\\textit{a\\&b}} & \\small\\textbf{\\textit{x\\textless{}y}} & \\small\\textbf{\\textit{q\"r}} \\\\\n\\midrule\n\\endfirsthead\n\\toprule\n\\small\\textbf{\\textit{a\\&b}} & \\small\\textbf{\\textit{x\\textless{}y}} & \\small\\textbf{\\textit{q\"r}} \\\\\n\\endhead\n\\caption[People (Continued)]{People}\n\\endfoot\n\\caption[People]{People}\n\\endlastfoot\n\\raggedright{\\textless{}b\\textgreater{}\\&amp;\"q\"}\n&\n\\raggedright{  lead  trail}\n&\n\\raggedright{1} \\tabularnewline\n \\hline\n\\raggedright{line1\\\\line2}\n&\n\\raggedright{ }\n&\n\\raggedright{2} \\tabularnewline\n \\hline\n\\raggedright{}\n&\n\\raggedright{}\n&\n\\raggedright{} \\tabularnewline\n \\hline\n\\raggedright{p\\textbar{}q \\textbackslash{} \\# \\$ \\% \\_ \\{ \\} \\~{} \\^{} \\textgreater{}}\n&\n\\raggedright{\t}\n&\n\\raggedright{3} \\tabularnewline\n \\hline\n\\raggedright{two  spaces  inside}\n&\n\\raggedright{back\\textbackslash{}slash}\n&\n\\raggedright{40} \\tabularnewline\n \\hline\n\\end{longtable}\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 3;
        options.tuples_only = true;
        special_query("special latex-longtable border 3 tuples only", options, "\\begin{longtable}{| l | l | r |}\n\\raggedright{\\textless{}b\\textgreater{}\\&amp;\"q\"}\n&\n\\raggedright{  lead  trail}\n&\n\\raggedright{1} \\tabularnewline\n \\hline\n\\raggedright{line1\\\\line2}\n&\n\\raggedright{ }\n&\n\\raggedright{2} \\tabularnewline\n \\hline\n\\raggedright{}\n&\n\\raggedright{}\n&\n\\raggedright{} \\tabularnewline\n \\hline\n\\raggedright{p\\textbar{}q \\textbackslash{} \\# \\$ \\% \\_ \\{ \\} \\~{} \\^{} \\textgreater{}}\n&\n\\raggedright{\t}\n&\n\\raggedright{3} \\tabularnewline\n \\hline\n\\raggedright{two  spaces  inside}\n&\n\\raggedright{back\\textbackslash{}slash}\n&\n\\raggedright{40} \\tabularnewline\n \\hline\n\\end{longtable}\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 0;
        options.expanded = .ON;
        options.title = "People";
        special_query("special latex-longtable border 0 expanded titled", options, "\\begin{center}\nPeople\n\\end{center}\n\n\\begin{tabular}{cl}\n\\multicolumn{2}{c}{\\textit{Record 1}} \\\\\na\\&b & \\textless{}b\\textgreater{}\\&amp;\"q\" \\\\\nx\\textless{}y &   lead  trail \\\\\nq\"r & 1 \\\\\n\\multicolumn{2}{c}{\\textit{Record 2}} \\\\\na\\&b & line1\\\\line2 \\\\\nx\\textless{}y &   \\\\\nq\"r & 2 \\\\\n\\multicolumn{2}{c}{\\textit{Record 3}} \\\\\na\\&b &  \\\\\nx\\textless{}y &  \\\\\nq\"r &  \\\\\n\\multicolumn{2}{c}{\\textit{Record 4}} \\\\\na\\&b & p\\textbar{}q \\textbackslash{} \\# \\$ \\% \\_ \\{ \\} \\~{} \\^{} \\textgreater{} \\\\\nx\\textless{}y & \t \\\\\nq\"r & 3 \\\\\n\\multicolumn{2}{c}{\\textit{Record 5}} \\\\\na\\&b & two  spaces  inside \\\\\nx\\textless{}y & back\\textbackslash{}slash \\\\\nq\"r & 40 \\\\\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 0;
        options.expanded = .ON;
        options.tuples_only = true;
        options.title = "People";
        special_query("special latex-longtable border 0 expanded tuples only titled", options, "\\begin{tabular}{cl}\na\\&b & \\textless{}b\\textgreater{}\\&amp;\"q\" \\\\\nx\\textless{}y &   lead  trail \\\\\nq\"r & 1 \\\\\na\\&b & line1\\\\line2 \\\\\nx\\textless{}y &   \\\\\nq\"r & 2 \\\\\na\\&b &  \\\\\nx\\textless{}y &  \\\\\nq\"r &  \\\\\na\\&b & p\\textbar{}q \\textbackslash{} \\# \\$ \\% \\_ \\{ \\} \\~{} \\^{} \\textgreater{} \\\\\nx\\textless{}y & \t \\\\\nq\"r & 3 \\\\\na\\&b & two  spaces  inside \\\\\nx\\textless{}y & back\\textbackslash{}slash \\\\\nq\"r & 40 \\\\\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 1;
        options.expanded = .ON;
        options.title = "People";
        special_query("special latex-longtable border 1 expanded titled", options, "\\begin{center}\nPeople\n\\end{center}\n\n\\begin{tabular}{c|l}\n\\multicolumn{2}{c}{\\textit{Record 1}} \\\\\n\\hline\na\\&b & \\textless{}b\\textgreater{}\\&amp;\"q\" \\\\\nx\\textless{}y &   lead  trail \\\\\nq\"r & 1 \\\\\n\\multicolumn{2}{c}{\\textit{Record 2}} \\\\\n\\hline\na\\&b & line1\\\\line2 \\\\\nx\\textless{}y &   \\\\\nq\"r & 2 \\\\\n\\multicolumn{2}{c}{\\textit{Record 3}} \\\\\n\\hline\na\\&b &  \\\\\nx\\textless{}y &  \\\\\nq\"r &  \\\\\n\\multicolumn{2}{c}{\\textit{Record 4}} \\\\\n\\hline\na\\&b & p\\textbar{}q \\textbackslash{} \\# \\$ \\% \\_ \\{ \\} \\~{} \\^{} \\textgreater{} \\\\\nx\\textless{}y & \t \\\\\nq\"r & 3 \\\\\n\\multicolumn{2}{c}{\\textit{Record 5}} \\\\\n\\hline\na\\&b & two  spaces  inside \\\\\nx\\textless{}y & back\\textbackslash{}slash \\\\\nq\"r & 40 \\\\\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 1;
        options.expanded = .ON;
        options.tuples_only = true;
        special_query("special latex-longtable border 1 expanded tuples only", options, "\\begin{tabular}{c|l}\n\\hline\na\\&b & \\textless{}b\\textgreater{}\\&amp;\"q\" \\\\\nx\\textless{}y &   lead  trail \\\\\nq\"r & 1 \\\\\n\\hline\na\\&b & line1\\\\line2 \\\\\nx\\textless{}y &   \\\\\nq\"r & 2 \\\\\n\\hline\na\\&b &  \\\\\nx\\textless{}y &  \\\\\nq\"r &  \\\\\n\\hline\na\\&b & p\\textbar{}q \\textbackslash{} \\# \\$ \\% \\_ \\{ \\} \\~{} \\^{} \\textgreater{} \\\\\nx\\textless{}y & \t \\\\\nq\"r & 3 \\\\\n\\hline\na\\&b & two  spaces  inside \\\\\nx\\textless{}y & back\\textbackslash{}slash \\\\\nq\"r & 40 \\\\\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 2;
        options.expanded = .ON;
        options.title = "People";
        special_query("special latex-longtable border 2 expanded titled", options, "\\begin{center}\nPeople\n\\end{center}\n\n\\begin{tabular}{|c|l|}\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 1}} \\\\\n\\hline\na\\&b & \\textless{}b\\textgreater{}\\&amp;\"q\" \\\\\nx\\textless{}y &   lead  trail \\\\\nq\"r & 1 \\\\\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 2}} \\\\\n\\hline\na\\&b & line1\\\\line2 \\\\\nx\\textless{}y &   \\\\\nq\"r & 2 \\\\\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 3}} \\\\\n\\hline\na\\&b &  \\\\\nx\\textless{}y &  \\\\\nq\"r &  \\\\\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 4}} \\\\\n\\hline\na\\&b & p\\textbar{}q \\textbackslash{} \\# \\$ \\% \\_ \\{ \\} \\~{} \\^{} \\textgreater{} \\\\\nx\\textless{}y & \t \\\\\nq\"r & 3 \\\\\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 5}} \\\\\n\\hline\na\\&b & two  spaces  inside \\\\\nx\\textless{}y & back\\textbackslash{}slash \\\\\nq\"r & 40 \\\\\n\\hline\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 2;
        options.expanded = .ON;
        options.tuples_only = true;
        options.title = "People";
        special_query("special latex-longtable border 2 expanded tuples only titled", options, "\\begin{tabular}{|c|l|}\n\\hline\na\\&b & \\textless{}b\\textgreater{}\\&amp;\"q\" \\\\\nx\\textless{}y &   lead  trail \\\\\nq\"r & 1 \\\\\n\\hline\na\\&b & line1\\\\line2 \\\\\nx\\textless{}y &   \\\\\nq\"r & 2 \\\\\n\\hline\na\\&b &  \\\\\nx\\textless{}y &  \\\\\nq\"r &  \\\\\n\\hline\na\\&b & p\\textbar{}q \\textbackslash{} \\# \\$ \\% \\_ \\{ \\} \\~{} \\^{} \\textgreater{} \\\\\nx\\textless{}y & \t \\\\\nq\"r & 3 \\\\\n\\hline\na\\&b & two  spaces  inside \\\\\nx\\textless{}y & back\\textbackslash{}slash \\\\\nq\"r & 40 \\\\\n\\hline\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 3;
        options.expanded = .ON;
        options.title = "People";
        special_query("special latex-longtable border 3 expanded titled", options, "\\begin{center}\nPeople\n\\end{center}\n\n\\begin{tabular}{|c|l|}\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 1}} \\\\\n\\hline\na\\&b & \\textless{}b\\textgreater{}\\&amp;\"q\" \\\\\nx\\textless{}y &   lead  trail \\\\\nq\"r & 1 \\\\\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 2}} \\\\\n\\hline\na\\&b & line1\\\\line2 \\\\\nx\\textless{}y &   \\\\\nq\"r & 2 \\\\\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 3}} \\\\\n\\hline\na\\&b &  \\\\\nx\\textless{}y &  \\\\\nq\"r &  \\\\\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 4}} \\\\\n\\hline\na\\&b & p\\textbar{}q \\textbackslash{} \\# \\$ \\% \\_ \\{ \\} \\~{} \\^{} \\textgreater{} \\\\\nx\\textless{}y & \t \\\\\nq\"r & 3 \\\\\n\\hline\n\\multicolumn{2}{|c|}{\\textit{Record 5}} \\\\\n\\hline\na\\&b & two  spaces  inside \\\\\nx\\textless{}y & back\\textbackslash{}slash \\\\\nq\"r & 40 \\\\\n\\hline\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 3;
        options.expanded = .ON;
        options.tuples_only = true;
        special_query("special latex-longtable border 3 expanded tuples only", options, "\\begin{tabular}{|c|l|}\n\\hline\na\\&b & \\textless{}b\\textgreater{}\\&amp;\"q\" \\\\\nx\\textless{}y &   lead  trail \\\\\nq\"r & 1 \\\\\n\\hline\na\\&b & line1\\\\line2 \\\\\nx\\textless{}y &   \\\\\nq\"r & 2 \\\\\n\\hline\na\\&b &  \\\\\nx\\textless{}y &  \\\\\nq\"r &  \\\\\n\\hline\na\\&b & p\\textbar{}q \\textbackslash{} \\# \\$ \\% \\_ \\{ \\} \\~{} \\^{} \\textgreater{} \\\\\nx\\textless{}y & \t \\\\\nq\"r & 3 \\\\\n\\hline\na\\&b & two  spaces  inside \\\\\nx\\textless{}y & back\\textbackslash{}slash \\\\\nq\"r & 40 \\\\\n\\hline\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 0;
        options.title = "People";
        special_query("special troff-ms border 0 titled", options, ".LP\n.DS C\nPeople\n.DE\n.LP\n.TS\ncenter;\nllr.\n\\fIa&b\\fP\t\\fIx<y\\fP\t\\fIq\"r\\fP\n_\n<b>&amp;\"q\"\t  lead  trail\t1\nline1\nline2\t \t2\n\t\t\np|q \\(rs # $ % _ { } ~ ^ >\t\t\t3\ntwo  spaces  inside\tback\\(rsslash\t40\n.TE\n.DS L\n(5 rows)\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 0;
        options.tuples_only = true;
        options.title = "People";
        special_query("special troff-ms border 0 tuples only titled", options, ".LP\n.TS\ncenter;\nllr.\n<b>&amp;\"q\"\t  lead  trail\t1\nline1\nline2\t \t2\n\t\t\np|q \\(rs # $ % _ { } ~ ^ >\t\t\t3\ntwo  spaces  inside\tback\\(rsslash\t40\n.TE\n.DS L\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 1;
        options.title = "People";
        special_query("special troff-ms border 1 titled", options, ".LP\n.DS C\nPeople\n.DE\n.LP\n.TS\ncenter;\nl | l | r.\n\\fIa&b\\fP\t\\fIx<y\\fP\t\\fIq\"r\\fP\n_\n<b>&amp;\"q\"\t  lead  trail\t1\nline1\nline2\t \t2\n\t\t\np|q \\(rs # $ % _ { } ~ ^ >\t\t\t3\ntwo  spaces  inside\tback\\(rsslash\t40\n.TE\n.DS L\n(5 rows)\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 1;
        options.tuples_only = true;
        special_query("special troff-ms border 1 tuples only", options, ".LP\n.TS\ncenter;\nl | l | r.\n<b>&amp;\"q\"\t  lead  trail\t1\nline1\nline2\t \t2\n\t\t\np|q \\(rs # $ % _ { } ~ ^ >\t\t\t3\ntwo  spaces  inside\tback\\(rsslash\t40\n.TE\n.DS L\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 2;
        options.title = "People";
        special_query("special troff-ms border 2 titled", options, ".LP\n.DS C\nPeople\n.DE\n.LP\n.TS\ncenter box;\nl | l | r.\n\\fIa&b\\fP\t\\fIx<y\\fP\t\\fIq\"r\\fP\n_\n<b>&amp;\"q\"\t  lead  trail\t1\nline1\nline2\t \t2\n\t\t\np|q \\(rs # $ % _ { } ~ ^ >\t\t\t3\ntwo  spaces  inside\tback\\(rsslash\t40\n.TE\n.DS L\n(5 rows)\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 2;
        options.tuples_only = true;
        options.title = "People";
        special_query("special troff-ms border 2 tuples only titled", options, ".LP\n.TS\ncenter box;\nl | l | r.\n<b>&amp;\"q\"\t  lead  trail\t1\nline1\nline2\t \t2\n\t\t\np|q \\(rs # $ % _ { } ~ ^ >\t\t\t3\ntwo  spaces  inside\tback\\(rsslash\t40\n.TE\n.DS L\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 3;
        options.title = "People";
        special_query("special troff-ms border 3 titled", options, ".LP\n.DS C\nPeople\n.DE\n.LP\n.TS\ncenter box;\nl | l | r.\n\\fIa&b\\fP\t\\fIx<y\\fP\t\\fIq\"r\\fP\n_\n<b>&amp;\"q\"\t  lead  trail\t1\nline1\nline2\t \t2\n\t\t\np|q \\(rs # $ % _ { } ~ ^ >\t\t\t3\ntwo  spaces  inside\tback\\(rsslash\t40\n.TE\n.DS L\n(5 rows)\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 3;
        options.tuples_only = true;
        special_query("special troff-ms border 3 tuples only", options, ".LP\n.TS\ncenter box;\nl | l | r.\n<b>&amp;\"q\"\t  lead  trail\t1\nline1\nline2\t \t2\n\t\t\np|q \\(rs # $ % _ { } ~ ^ >\t\t\t3\ntwo  spaces  inside\tback\\(rsslash\t40\n.TE\n.DS L\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 0;
        options.expanded = .ON;
        options.title = "People";
        special_query("special troff-ms border 0 expanded titled", options, ".LP\n.DS C\nPeople\n.DE\n.LP\n.TS\ncenter;\nc s.\n\\fIRecord 1\\fP\n.T&\nc l.\na&b\t<b>&amp;\"q\"\nx<y\t  lead  trail\nq\"r\t1\n.T&\nc s.\n\\fIRecord 2\\fP\n.T&\nc l.\na&b\tline1\nline2\nx<y\t \nq\"r\t2\n.T&\nc s.\n\\fIRecord 3\\fP\n.T&\nc l.\na&b\t\nx<y\t\nq\"r\t\n.T&\nc s.\n\\fIRecord 4\\fP\n.T&\nc l.\na&b\tp|q \\(rs # $ % _ { } ~ ^ >\nx<y\t\t\nq\"r\t3\n.T&\nc s.\n\\fIRecord 5\\fP\n.T&\nc l.\na&b\ttwo  spaces  inside\nx<y\tback\\(rsslash\nq\"r\t40\n.TE\n.DS L\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 0;
        options.expanded = .ON;
        options.tuples_only = true;
        options.title = "People";
        special_query("special troff-ms border 0 expanded tuples only titled", options, ".LP\n.TS\ncenter;\nc l;\na&b\t<b>&amp;\"q\"\nx<y\t  lead  trail\nq\"r\t1\na&b\tline1\nline2\nx<y\t \nq\"r\t2\na&b\t\nx<y\t\nq\"r\t\na&b\tp|q \\(rs # $ % _ { } ~ ^ >\nx<y\t\t\nq\"r\t3\na&b\ttwo  spaces  inside\nx<y\tback\\(rsslash\nq\"r\t40\n.TE\n.DS L\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 1;
        options.expanded = .ON;
        options.title = "People";
        special_query("special troff-ms border 1 expanded titled", options, ".LP\n.DS C\nPeople\n.DE\n.LP\n.TS\ncenter;\nc s.\n\\fIRecord 1\\fP\n_\n.T&\nc | l.\na&b\t<b>&amp;\"q\"\nx<y\t  lead  trail\nq\"r\t1\n.T&\nc s.\n\\fIRecord 2\\fP\n_\n.T&\nc | l.\na&b\tline1\nline2\nx<y\t \nq\"r\t2\n.T&\nc s.\n\\fIRecord 3\\fP\n_\n.T&\nc | l.\na&b\t\nx<y\t\nq\"r\t\n.T&\nc s.\n\\fIRecord 4\\fP\n_\n.T&\nc | l.\na&b\tp|q \\(rs # $ % _ { } ~ ^ >\nx<y\t\t\nq\"r\t3\n.T&\nc s.\n\\fIRecord 5\\fP\n_\n.T&\nc | l.\na&b\ttwo  spaces  inside\nx<y\tback\\(rsslash\nq\"r\t40\n.TE\n.DS L\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 1;
        options.expanded = .ON;
        options.tuples_only = true;
        special_query("special troff-ms border 1 expanded tuples only", options, ".LP\n.TS\ncenter;\nc l;\n_\na&b\t<b>&amp;\"q\"\nx<y\t  lead  trail\nq\"r\t1\n_\na&b\tline1\nline2\nx<y\t \nq\"r\t2\n_\na&b\t\nx<y\t\nq\"r\t\n_\na&b\tp|q \\(rs # $ % _ { } ~ ^ >\nx<y\t\t\nq\"r\t3\n_\na&b\ttwo  spaces  inside\nx<y\tback\\(rsslash\nq\"r\t40\n.TE\n.DS L\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 2;
        options.expanded = .ON;
        options.title = "People";
        special_query("special troff-ms border 2 expanded titled", options, ".LP\n.DS C\nPeople\n.DE\n.LP\n.TS\ncenter box;\nc s.\n\\fIRecord 1\\fP\n_\n.T&\nc l.\na&b\t<b>&amp;\"q\"\nx<y\t  lead  trail\nq\"r\t1\n_\n.T&\nc s.\n\\fIRecord 2\\fP\n_\n.T&\nc l.\na&b\tline1\nline2\nx<y\t \nq\"r\t2\n_\n.T&\nc s.\n\\fIRecord 3\\fP\n_\n.T&\nc l.\na&b\t\nx<y\t\nq\"r\t\n_\n.T&\nc s.\n\\fIRecord 4\\fP\n_\n.T&\nc l.\na&b\tp|q \\(rs # $ % _ { } ~ ^ >\nx<y\t\t\nq\"r\t3\n_\n.T&\nc s.\n\\fIRecord 5\\fP\n_\n.T&\nc l.\na&b\ttwo  spaces  inside\nx<y\tback\\(rsslash\nq\"r\t40\n.TE\n.DS L\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 2;
        options.expanded = .ON;
        options.tuples_only = true;
        options.title = "People";
        special_query("special troff-ms border 2 expanded tuples only titled", options, ".LP\n.TS\ncenter box;\nc l;\n_\na&b\t<b>&amp;\"q\"\nx<y\t  lead  trail\nq\"r\t1\n_\na&b\tline1\nline2\nx<y\t \nq\"r\t2\n_\na&b\t\nx<y\t\nq\"r\t\n_\na&b\tp|q \\(rs # $ % _ { } ~ ^ >\nx<y\t\t\nq\"r\t3\n_\na&b\ttwo  spaces  inside\nx<y\tback\\(rsslash\nq\"r\t40\n.TE\n.DS L\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 3;
        options.expanded = .ON;
        options.title = "People";
        special_query("special troff-ms border 3 expanded titled", options, ".LP\n.DS C\nPeople\n.DE\n.LP\n.TS\ncenter box;\nc s.\n\\fIRecord 1\\fP\n_\n.T&\nc l.\na&b\t<b>&amp;\"q\"\nx<y\t  lead  trail\nq\"r\t1\n_\n.T&\nc s.\n\\fIRecord 2\\fP\n_\n.T&\nc l.\na&b\tline1\nline2\nx<y\t \nq\"r\t2\n_\n.T&\nc s.\n\\fIRecord 3\\fP\n_\n.T&\nc l.\na&b\t\nx<y\t\nq\"r\t\n_\n.T&\nc s.\n\\fIRecord 4\\fP\n_\n.T&\nc l.\na&b\tp|q \\(rs # $ % _ { } ~ ^ >\nx<y\t\t\nq\"r\t3\n_\n.T&\nc s.\n\\fIRecord 5\\fP\n_\n.T&\nc l.\na&b\ttwo  spaces  inside\nx<y\tback\\(rsslash\nq\"r\t40\n.TE\n.DS L\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 3;
        options.expanded = .ON;
        options.tuples_only = true;
        special_query("special troff-ms border 3 expanded tuples only", options, ".LP\n.TS\ncenter box;\nc l;\n_\na&b\t<b>&amp;\"q\"\nx<y\t  lead  trail\nq\"r\t1\n_\na&b\tline1\nline2\nx<y\t \nq\"r\t2\n_\na&b\t\nx<y\t\nq\"r\t\n_\na&b\tp|q \\(rs # $ % _ { } ~ ^ >\nx<y\t\t\nq\"r\t3\n_\na&b\ttwo  spaces  inside\nx<y\tback\\(rsslash\nq\"r\t40\n.TE\n.DS L\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 1;
        empty_query("empty html border 1", options, "<table border=\"1\">\n  <tr>\n    <th align=\"center\">id</th>\n    <th align=\"center\">name</th>\n  </tr>\n</table>\n<p>(0 rows)<br />\n</p>\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 1;
        options.tuples_only = true;
        empty_query("empty html border 1 tuples only", options, "<table border=\"1\">\n</table>\n\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 2;
        empty_query("empty html border 2", options, "<table border=\"2\">\n  <tr>\n    <th align=\"center\">id</th>\n    <th align=\"center\">name</th>\n  </tr>\n</table>\n<p>(0 rows)<br />\n</p>\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 2;
        options.tuples_only = true;
        empty_query("empty html border 2 tuples only", options, "<table border=\"2\">\n</table>\n\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 1;
        options.expanded = .ON;
        empty_query("empty html border 1 expanded", options, "<table border=\"1\">\n</table>\n\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 2;
        options.expanded = .ON;
        empty_query("empty html border 2 expanded", options, "<table border=\"2\">\n</table>\n\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 1;
        empty_query("empty asciidoc border 1", options, "\n[options=\"header\",cols=\">l,<l\",frame=\"none\"]\n|====\n^l|id ^l|name\n|====\n\n....\n(0 rows)\n....\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 1;
        options.tuples_only = true;
        empty_query("empty asciidoc border 1 tuples only", options, "\n[cols=\">l,<l\",frame=\"none\"]\n|====\n|====\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 2;
        empty_query("empty asciidoc border 2", options, "\n[options=\"header\",cols=\">l,<l\",frame=\"all\",grid=\"all\"]\n|====\n^l|id ^l|name\n|====\n\n....\n(0 rows)\n....\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 2;
        options.tuples_only = true;
        empty_query("empty asciidoc border 2 tuples only", options, "\n[cols=\">l,<l\",frame=\"all\",grid=\"all\"]\n|====\n|====\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 1;
        options.expanded = .ON;
        empty_query("empty asciidoc border 1 expanded", options, "\n[cols=\"h,l\",frame=\"none\"]\n|====\n|====\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 2;
        options.expanded = .ON;
        empty_query("empty asciidoc border 2 expanded", options, "\n[cols=\"h,l\",frame=\"all\",grid=\"all\"]\n|====\n|====\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 1;
        empty_query("empty latex border 1", options, "\\begin{tabular}{r | l}\n\\textit{id} & \\textit{name} \\\\\n\\hline\n\\end{tabular}\n\n\\noindent (0 rows) \\\\\n\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 1;
        options.tuples_only = true;
        empty_query("empty latex border 1 tuples only", options, "\\begin{tabular}{r | l}\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 2;
        empty_query("empty latex border 2", options, "\\begin{tabular}{| r | l |}\n\\hline\n\\textit{id} & \\textit{name} \\\\\n\\hline\n\\hline\n\\end{tabular}\n\n\\noindent (0 rows) \\\\\n\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 2;
        options.tuples_only = true;
        empty_query("empty latex border 2 tuples only", options, "\\begin{tabular}{| r | l |}\n\\hline\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 1;
        options.expanded = .ON;
        empty_query("empty latex border 1 expanded", options, "\\begin{tabular}{c|l}\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 2;
        options.expanded = .ON;
        empty_query("empty latex border 2 expanded", options, "\\begin{tabular}{|c|l|}\n\\hline\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 1;
        empty_query("empty latex-longtable border 1", options, "\\begin{longtable}{r | l}\n\\small\\textbf{\\textit{id}} & \\small\\textbf{\\textit{name}} \\\\\n\\midrule\n\\endfirsthead\n\\small\\textbf{\\textit{id}} & \\small\\textbf{\\textit{name}} \\\\\n\\midrule\n\\endhead\n\\end{longtable}\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 1;
        options.tuples_only = true;
        empty_query("empty latex-longtable border 1 tuples only", options, "\\begin{longtable}{r | l}\n\\end{longtable}\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 2;
        empty_query("empty latex-longtable border 2", options, "\\begin{longtable}{| r | l |}\n\\toprule\n\\small\\textbf{\\textit{id}} & \\small\\textbf{\\textit{name}} \\\\\n\\midrule\n\\endfirsthead\n\\toprule\n\\small\\textbf{\\textit{id}} & \\small\\textbf{\\textit{name}} \\\\\n\\midrule\n\\endhead\n\\bottomrule\n\\endfoot\n\\bottomrule\n\\endlastfoot\n\\end{longtable}\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 2;
        options.tuples_only = true;
        empty_query("empty latex-longtable border 2 tuples only", options, "\\begin{longtable}{| r | l |}\n\\end{longtable}\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 1;
        options.expanded = .ON;
        empty_query("empty latex-longtable border 1 expanded", options, "\\begin{tabular}{c|l}\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 2;
        options.expanded = .ON;
        empty_query("empty latex-longtable border 2 expanded", options, "\\begin{tabular}{|c|l|}\n\\hline\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 1;
        empty_query("empty troff-ms border 1", options, ".LP\n.TS\ncenter;\nr | l.\n\\fIid\\fP\t\\fIname\\fP\n_\n.TE\n.DS L\n(0 rows)\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 1;
        options.tuples_only = true;
        empty_query("empty troff-ms border 1 tuples only", options, ".LP\n.TS\ncenter;\nr | l.\n.TE\n.DS L\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 2;
        empty_query("empty troff-ms border 2", options, ".LP\n.TS\ncenter box;\nr | l.\n\\fIid\\fP\t\\fIname\\fP\n_\n.TE\n.DS L\n(0 rows)\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 2;
        options.tuples_only = true;
        empty_query("empty troff-ms border 2 tuples only", options, ".LP\n.TS\ncenter box;\nr | l.\n.TE\n.DS L\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 1;
        options.expanded = .ON;
        empty_query("empty troff-ms border 1 expanded", options, ".LP\n.TS\ncenter;\n.TE\n.DS L\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 2;
        options.expanded = .ON;
        empty_query("empty troff-ms border 2 expanded", options, ".LP\n.TS\ncenter box;\n.TE\n.DS L\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 1;
        one_query("one html border 1", options, "<table border=\"1\">\n  <tr>\n    <th align=\"center\">only</th>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">x</td>\n  </tr>\n</table>\n<p>(1 row)<br />\n</p>\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 1;
        options.tuples_only = true;
        one_query("one html border 1 tuples only", options, "<table border=\"1\">\n  <tr valign=\"top\">\n    <td align=\"left\">x</td>\n  </tr>\n</table>\n\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 1;
        options.expanded = .ON;
        one_query("one html border 1 expanded", options, "<table border=\"1\">\n\n  <tr><td colspan=\"2\" align=\"center\">Record 1</td></tr>\n  <tr valign=\"top\">\n    <th>only</th>\n    <td align=\"left\">x</td>\n  </tr>\n</table>\n\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 1;
        one_query("one asciidoc border 1", options, "\n[options=\"header\",cols=\"<l\",frame=\"none\"]\n|====\n^l|only\n|x\n|====\n\n....\n(1 row)\n....\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 1;
        options.tuples_only = true;
        one_query("one asciidoc border 1 tuples only", options, "\n[cols=\"<l\",frame=\"none\"]\n|====\n|x\n|====\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 1;
        options.expanded = .ON;
        one_query("one asciidoc border 1 expanded", options, "\n[cols=\"h,l\",frame=\"none\"]\n|====\n2+^|Record 1\n<l|only <l|x\n|====\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 1;
        one_query("one latex border 1", options, "\\begin{tabular}{l}\n\\textit{only} \\\\\n\\hline\nx \\\\\n\\end{tabular}\n\n\\noindent (1 row) \\\\\n\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 1;
        options.tuples_only = true;
        one_query("one latex border 1 tuples only", options, "\\begin{tabular}{l}\nx \\\\\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 1;
        options.expanded = .ON;
        one_query("one latex border 1 expanded", options, "\\begin{tabular}{c|l}\n\\multicolumn{2}{c}{\\textit{Record 1}} \\\\\n\\hline\nonly & x \\\\\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 1;
        one_query("one latex-longtable border 1", options, "\\begin{longtable}{l}\n\\small\\textbf{\\textit{only}} \\\\\n\\midrule\n\\endfirsthead\n\\small\\textbf{\\textit{only}} \\\\\n\\midrule\n\\endhead\n\\raggedright{x} \\tabularnewline\n\\end{longtable}\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 1;
        options.tuples_only = true;
        one_query("one latex-longtable border 1 tuples only", options, "\\begin{longtable}{l}\n\\raggedright{x} \\tabularnewline\n\\end{longtable}\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 1;
        options.expanded = .ON;
        one_query("one latex-longtable border 1 expanded", options, "\\begin{tabular}{c|l}\n\\multicolumn{2}{c}{\\textit{Record 1}} \\\\\n\\hline\nonly & x \\\\\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 1;
        one_query("one troff-ms border 1", options, ".LP\n.TS\ncenter;\nl.\n\\fIonly\\fP\n_\nx\n.TE\n.DS L\n(1 row)\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 1;
        options.tuples_only = true;
        one_query("one troff-ms border 1 tuples only", options, ".LP\n.TS\ncenter;\nl.\nx\n.TE\n.DS L\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 1;
        options.expanded = .ON;
        one_query("one troff-ms border 1 expanded", options, ".LP\n.TS\ncenter;\nc s.\n\\fIRecord 1\\fP\n_\n.T&\nc | l.\nonly\tx\n.TE\n.DS L\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 0;
        namespace_table("describe pg_namespace html border 0", options, "<table border=\"0\">\n  <caption>Table &quot;pg_catalog.pg_namespace&quot;</caption>\n  <tr>\n    <th align=\"center\">Column</th>\n    <th align=\"center\">Type</th>\n    <th align=\"center\">Collation</th>\n    <th align=\"center\">Nullable</th>\n    <th align=\"center\">Default</th>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">oid</td>\n    <td align=\"left\">oid</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"left\">not null</td>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">nspname</td>\n    <td align=\"left\">name</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"left\">not null</td>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">nspowner</td>\n    <td align=\"left\">oid</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"left\">not null</td>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">nspacl</td>\n    <td align=\"left\">aclitem[]</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n</table>\n<p>Indexes:<br />\n&nbsp;&nbsp;&nbsp;&nbsp;&quot;pg_namespace_oid_index&quot; PRIMARY KEY, btree (oid)<br />\n&nbsp;&nbsp;&nbsp;&nbsp;&quot;pg_namespace_nspname_index&quot; UNIQUE CONSTRAINT, btree (nspname)<br />\n</p>\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 1;
        namespace_table("describe pg_namespace html border 1", options, "<table border=\"1\">\n  <caption>Table &quot;pg_catalog.pg_namespace&quot;</caption>\n  <tr>\n    <th align=\"center\">Column</th>\n    <th align=\"center\">Type</th>\n    <th align=\"center\">Collation</th>\n    <th align=\"center\">Nullable</th>\n    <th align=\"center\">Default</th>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">oid</td>\n    <td align=\"left\">oid</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"left\">not null</td>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">nspname</td>\n    <td align=\"left\">name</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"left\">not null</td>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">nspowner</td>\n    <td align=\"left\">oid</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"left\">not null</td>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">nspacl</td>\n    <td align=\"left\">aclitem[]</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n</table>\n<p>Indexes:<br />\n&nbsp;&nbsp;&nbsp;&nbsp;&quot;pg_namespace_oid_index&quot; PRIMARY KEY, btree (oid)<br />\n&nbsp;&nbsp;&nbsp;&nbsp;&quot;pg_namespace_nspname_index&quot; UNIQUE CONSTRAINT, btree (nspname)<br />\n</p>\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 1;
        options.tuples_only = true;
        namespace_table("describe pg_namespace html border 1 tuples only", options, "<table border=\"1\">\n  <tr valign=\"top\">\n    <td align=\"left\">oid</td>\n    <td align=\"left\">oid</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"left\">not null</td>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">nspname</td>\n    <td align=\"left\">name</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"left\">not null</td>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">nspowner</td>\n    <td align=\"left\">oid</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"left\">not null</td>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">nspacl</td>\n    <td align=\"left\">aclitem[]</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n</table>\n\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 2;
        namespace_table("describe pg_namespace html border 2", options, "<table border=\"2\">\n  <caption>Table &quot;pg_catalog.pg_namespace&quot;</caption>\n  <tr>\n    <th align=\"center\">Column</th>\n    <th align=\"center\">Type</th>\n    <th align=\"center\">Collation</th>\n    <th align=\"center\">Nullable</th>\n    <th align=\"center\">Default</th>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">oid</td>\n    <td align=\"left\">oid</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"left\">not null</td>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">nspname</td>\n    <td align=\"left\">name</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"left\">not null</td>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">nspowner</td>\n    <td align=\"left\">oid</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"left\">not null</td>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"left\">nspacl</td>\n    <td align=\"left\">aclitem[]</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n</table>\n<p>Indexes:<br />\n&nbsp;&nbsp;&nbsp;&nbsp;&quot;pg_namespace_oid_index&quot; PRIMARY KEY, btree (oid)<br />\n&nbsp;&nbsp;&nbsp;&nbsp;&quot;pg_namespace_nspname_index&quot; UNIQUE CONSTRAINT, btree (nspname)<br />\n</p>\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 0;
        namespace_table("describe pg_namespace asciidoc border 0", options, "\n.Table \"pg_catalog.pg_namespace\"\n[options=\"header\",cols=\"<l,<l,<l,<l,<l\",frame=\"none\",grid=\"none\"]\n|====\n^l|Column ^l|Type ^l|Collation ^l|Nullable ^l|Default\n|oid |oid |  |not null |\n|nspname |name |  |not null |\n|nspowner |oid |  |not null |\n|nspacl |aclitem[] |  |  |\n|====\n\n....\nIndexes:\n    \"pg_namespace_oid_index\" PRIMARY KEY, btree (oid)\n    \"pg_namespace_nspname_index\" UNIQUE CONSTRAINT, btree (nspname)\n....\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 1;
        namespace_table("describe pg_namespace asciidoc border 1", options, "\n.Table \"pg_catalog.pg_namespace\"\n[options=\"header\",cols=\"<l,<l,<l,<l,<l\",frame=\"none\"]\n|====\n^l|Column ^l|Type ^l|Collation ^l|Nullable ^l|Default\n|oid |oid |  |not null |\n|nspname |name |  |not null |\n|nspowner |oid |  |not null |\n|nspacl |aclitem[] |  |  |\n|====\n\n....\nIndexes:\n    \"pg_namespace_oid_index\" PRIMARY KEY, btree (oid)\n    \"pg_namespace_nspname_index\" UNIQUE CONSTRAINT, btree (nspname)\n....\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 1;
        options.tuples_only = true;
        namespace_table("describe pg_namespace asciidoc border 1 tuples only", options, "\n[cols=\"<l,<l,<l,<l,<l\",frame=\"none\"]\n|====\n|oid |oid |  |not null |\n|nspname |name |  |not null |\n|nspowner |oid |  |not null |\n|nspacl |aclitem[] |  |  |\n|====\n");
    }

    {
        options: Print_Options;
        options.format = .ASCIIDOC;
        options.border = 2;
        namespace_table("describe pg_namespace asciidoc border 2", options, "\n.Table \"pg_catalog.pg_namespace\"\n[options=\"header\",cols=\"<l,<l,<l,<l,<l\",frame=\"all\",grid=\"all\"]\n|====\n^l|Column ^l|Type ^l|Collation ^l|Nullable ^l|Default\n|oid |oid |  |not null |\n|nspname |name |  |not null |\n|nspowner |oid |  |not null |\n|nspacl |aclitem[] |  |  |\n|====\n\n....\nIndexes:\n    \"pg_namespace_oid_index\" PRIMARY KEY, btree (oid)\n    \"pg_namespace_nspname_index\" UNIQUE CONSTRAINT, btree (nspname)\n....\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 0;
        namespace_table("describe pg_namespace latex border 0", options, "\\begin{center}\nTable \"pg\\_catalog.pg\\_namespace\"\n\\end{center}\n\n\\begin{tabular}{lllll}\n\\textit{Column} & \\textit{Type} & \\textit{Collation} & \\textit{Nullable} & \\textit{Default} \\\\\n\\hline\noid & oid &  & not null &  \\\\\nnspname & name &  & not null &  \\\\\nnspowner & oid &  & not null &  \\\\\nnspacl & aclitem[] &  &  &  \\\\\n\\end{tabular}\n\n\\noindent Indexes: \\\\\n    \"pg\\_namespace\\_oid\\_index\" PRIMARY KEY, btree (oid) \\\\\n    \"pg\\_namespace\\_nspname\\_index\" UNIQUE CONSTRAINT, btree (nspname) \\\\\n\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 1;
        namespace_table("describe pg_namespace latex border 1", options, "\\begin{center}\nTable \"pg\\_catalog.pg\\_namespace\"\n\\end{center}\n\n\\begin{tabular}{l | l | l | l | l}\n\\textit{Column} & \\textit{Type} & \\textit{Collation} & \\textit{Nullable} & \\textit{Default} \\\\\n\\hline\noid & oid &  & not null &  \\\\\nnspname & name &  & not null &  \\\\\nnspowner & oid &  & not null &  \\\\\nnspacl & aclitem[] &  &  &  \\\\\n\\end{tabular}\n\n\\noindent Indexes: \\\\\n    \"pg\\_namespace\\_oid\\_index\" PRIMARY KEY, btree (oid) \\\\\n    \"pg\\_namespace\\_nspname\\_index\" UNIQUE CONSTRAINT, btree (nspname) \\\\\n\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 1;
        options.tuples_only = true;
        namespace_table("describe pg_namespace latex border 1 tuples only", options, "\\begin{tabular}{l | l | l | l | l}\noid & oid &  & not null &  \\\\\nnspname & name &  & not null &  \\\\\nnspowner & oid &  & not null &  \\\\\nnspacl & aclitem[] &  &  &  \\\\\n\\end{tabular}\n\n\\noindent \n");
    }

    {
        options: Print_Options;
        options.format = .LATEX;
        options.border = 2;
        namespace_table("describe pg_namespace latex border 2", options, "\\begin{center}\nTable \"pg\\_catalog.pg\\_namespace\"\n\\end{center}\n\n\\begin{tabular}{| l | l | l | l | l |}\n\\hline\n\\textit{Column} & \\textit{Type} & \\textit{Collation} & \\textit{Nullable} & \\textit{Default} \\\\\n\\hline\noid & oid &  & not null &  \\\\\nnspname & name &  & not null &  \\\\\nnspowner & oid &  & not null &  \\\\\nnspacl & aclitem[] &  &  &  \\\\\n\\hline\n\\end{tabular}\n\n\\noindent Indexes: \\\\\n    \"pg\\_namespace\\_oid\\_index\" PRIMARY KEY, btree (oid) \\\\\n    \"pg\\_namespace\\_nspname\\_index\" UNIQUE CONSTRAINT, btree (nspname) \\\\\n\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 0;
        namespace_table("describe pg_namespace latex-longtable border 0", options, "\\begin{longtable}{lllll}\n\\small\\textbf{\\textit{Column}} & \\small\\textbf{\\textit{Type}} & \\small\\textbf{\\textit{Collation}} & \\small\\textbf{\\textit{Nullable}} & \\small\\textbf{\\textit{Default}} \\\\\n\\midrule\n\\endfirsthead\n\\small\\textbf{\\textit{Column}} & \\small\\textbf{\\textit{Type}} & \\small\\textbf{\\textit{Collation}} & \\small\\textbf{\\textit{Nullable}} & \\small\\textbf{\\textit{Default}} \\\\\n\\midrule\n\\endhead\n\\caption[Table \"pg\\_catalog.pg\\_namespace\" (Continued)]{Table \"pg\\_catalog.pg\\_namespace\"}\n\\endfoot\n\\caption[Table \"pg\\_catalog.pg\\_namespace\"]{Table \"pg\\_catalog.pg\\_namespace\"}\n\\endlastfoot\n\\raggedright{oid}\n&\n\\raggedright{oid}\n&\n\\raggedright{}\n&\n\\raggedright{not null}\n&\n\\raggedright{} \\tabularnewline\n\\raggedright{nspname}\n&\n\\raggedright{name}\n&\n\\raggedright{}\n&\n\\raggedright{not null}\n&\n\\raggedright{} \\tabularnewline\n\\raggedright{nspowner}\n&\n\\raggedright{oid}\n&\n\\raggedright{}\n&\n\\raggedright{not null}\n&\n\\raggedright{} \\tabularnewline\n\\raggedright{nspacl}\n&\n\\raggedright{aclitem[]}\n&\n\\raggedright{}\n&\n\\raggedright{}\n&\n\\raggedright{} \\tabularnewline\n\\end{longtable}\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 1;
        namespace_table("describe pg_namespace latex-longtable border 1", options, "\\begin{longtable}{l | l | l | l | l}\n\\small\\textbf{\\textit{Column}} & \\small\\textbf{\\textit{Type}} & \\small\\textbf{\\textit{Collation}} & \\small\\textbf{\\textit{Nullable}} & \\small\\textbf{\\textit{Default}} \\\\\n\\midrule\n\\endfirsthead\n\\small\\textbf{\\textit{Column}} & \\small\\textbf{\\textit{Type}} & \\small\\textbf{\\textit{Collation}} & \\small\\textbf{\\textit{Nullable}} & \\small\\textbf{\\textit{Default}} \\\\\n\\midrule\n\\endhead\n\\caption[Table \"pg\\_catalog.pg\\_namespace\" (Continued)]{Table \"pg\\_catalog.pg\\_namespace\"}\n\\endfoot\n\\caption[Table \"pg\\_catalog.pg\\_namespace\"]{Table \"pg\\_catalog.pg\\_namespace\"}\n\\endlastfoot\n\\raggedright{oid}\n&\n\\raggedright{oid}\n&\n\\raggedright{}\n&\n\\raggedright{not null}\n&\n\\raggedright{} \\tabularnewline\n\\raggedright{nspname}\n&\n\\raggedright{name}\n&\n\\raggedright{}\n&\n\\raggedright{not null}\n&\n\\raggedright{} \\tabularnewline\n\\raggedright{nspowner}\n&\n\\raggedright{oid}\n&\n\\raggedright{}\n&\n\\raggedright{not null}\n&\n\\raggedright{} \\tabularnewline\n\\raggedright{nspacl}\n&\n\\raggedright{aclitem[]}\n&\n\\raggedright{}\n&\n\\raggedright{}\n&\n\\raggedright{} \\tabularnewline\n\\end{longtable}\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 1;
        options.tuples_only = true;
        namespace_table("describe pg_namespace latex-longtable border 1 tuples only", options, "\\begin{longtable}{l | l | l | l | l}\n\\raggedright{oid}\n&\n\\raggedright{oid}\n&\n\\raggedright{}\n&\n\\raggedright{not null}\n&\n\\raggedright{} \\tabularnewline\n\\raggedright{nspname}\n&\n\\raggedright{name}\n&\n\\raggedright{}\n&\n\\raggedright{not null}\n&\n\\raggedright{} \\tabularnewline\n\\raggedright{nspowner}\n&\n\\raggedright{oid}\n&\n\\raggedright{}\n&\n\\raggedright{not null}\n&\n\\raggedright{} \\tabularnewline\n\\raggedright{nspacl}\n&\n\\raggedright{aclitem[]}\n&\n\\raggedright{}\n&\n\\raggedright{}\n&\n\\raggedright{} \\tabularnewline\n\\end{longtable}\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 2;
        namespace_table("describe pg_namespace latex-longtable border 2", options, "\\begin{longtable}{| l | l | l | l | l |}\n\\toprule\n\\small\\textbf{\\textit{Column}} & \\small\\textbf{\\textit{Type}} & \\small\\textbf{\\textit{Collation}} & \\small\\textbf{\\textit{Nullable}} & \\small\\textbf{\\textit{Default}} \\\\\n\\midrule\n\\endfirsthead\n\\toprule\n\\small\\textbf{\\textit{Column}} & \\small\\textbf{\\textit{Type}} & \\small\\textbf{\\textit{Collation}} & \\small\\textbf{\\textit{Nullable}} & \\small\\textbf{\\textit{Default}} \\\\\n\\midrule\n\\endhead\n\\bottomrule\n\\caption[Table \"pg\\_catalog.pg\\_namespace\" (Continued)]{Table \"pg\\_catalog.pg\\_namespace\"}\n\\endfoot\n\\bottomrule\n\\caption[Table \"pg\\_catalog.pg\\_namespace\"]{Table \"pg\\_catalog.pg\\_namespace\"}\n\\endlastfoot\n\\raggedright{oid}\n&\n\\raggedright{oid}\n&\n\\raggedright{}\n&\n\\raggedright{not null}\n&\n\\raggedright{} \\tabularnewline\n\\raggedright{nspname}\n&\n\\raggedright{name}\n&\n\\raggedright{}\n&\n\\raggedright{not null}\n&\n\\raggedright{} \\tabularnewline\n\\raggedright{nspowner}\n&\n\\raggedright{oid}\n&\n\\raggedright{}\n&\n\\raggedright{not null}\n&\n\\raggedright{} \\tabularnewline\n\\raggedright{nspacl}\n&\n\\raggedright{aclitem[]}\n&\n\\raggedright{}\n&\n\\raggedright{}\n&\n\\raggedright{} \\tabularnewline\n\\end{longtable}\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 0;
        namespace_table("describe pg_namespace troff-ms border 0", options, ".LP\n.DS C\nTable \"pg_catalog.pg_namespace\"\n.DE\n.LP\n.TS\ncenter;\nlllll.\n\\fIColumn\\fP\t\\fIType\\fP\t\\fICollation\\fP\t\\fINullable\\fP\t\\fIDefault\\fP\n_\noid\toid\t\tnot null\t\nnspname\tname\t\tnot null\t\nnspowner\toid\t\tnot null\t\nnspacl\taclitem[]\t\t\t\n.TE\n.DS L\nIndexes:\n    \"pg_namespace_oid_index\" PRIMARY KEY, btree (oid)\n    \"pg_namespace_nspname_index\" UNIQUE CONSTRAINT, btree (nspname)\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 1;
        namespace_table("describe pg_namespace troff-ms border 1", options, ".LP\n.DS C\nTable \"pg_catalog.pg_namespace\"\n.DE\n.LP\n.TS\ncenter;\nl | l | l | l | l.\n\\fIColumn\\fP\t\\fIType\\fP\t\\fICollation\\fP\t\\fINullable\\fP\t\\fIDefault\\fP\n_\noid\toid\t\tnot null\t\nnspname\tname\t\tnot null\t\nnspowner\toid\t\tnot null\t\nnspacl\taclitem[]\t\t\t\n.TE\n.DS L\nIndexes:\n    \"pg_namespace_oid_index\" PRIMARY KEY, btree (oid)\n    \"pg_namespace_nspname_index\" UNIQUE CONSTRAINT, btree (nspname)\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 1;
        options.tuples_only = true;
        namespace_table("describe pg_namespace troff-ms border 1 tuples only", options, ".LP\n.TS\ncenter;\nl | l | l | l | l.\noid\toid\t\tnot null\t\nnspname\tname\t\tnot null\t\nnspowner\toid\t\tnot null\t\nnspacl\taclitem[]\t\t\t\n.TE\n.DS L\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .TROFF_MS;
        options.border = 2;
        namespace_table("describe pg_namespace troff-ms border 2", options, ".LP\n.DS C\nTable \"pg_catalog.pg_namespace\"\n.DE\n.LP\n.TS\ncenter box;\nl | l | l | l | l.\n\\fIColumn\\fP\t\\fIType\\fP\t\\fICollation\\fP\t\\fINullable\\fP\t\\fIDefault\\fP\n_\noid\toid\t\tnot null\t\nnspname\tname\t\tnot null\t\nnspowner\toid\t\tnot null\t\nnspacl\taclitem[]\t\t\t\n.TE\n.DS L\nIndexes:\n    \"pg_namespace_oid_index\" PRIMARY KEY, btree (oid)\n    \"pg_namespace_nspname_index\" UNIQUE CONSTRAINT, btree (nspname)\n.DE\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 1;
        options.table_attr = "class=\"x\" id=7";
        people_query("people html border 1 \\pset tableattr 'class=\"x\" id=7'", options, "<table border=\"1\" class=\"x\" id=7>\n  <tr>\n    <th align=\"center\">id</th>\n    <th align=\"center\">name</th>\n    <th align=\"center\">score</th>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">1</td>\n    <td align=\"left\">alice</td>\n    <td align=\"right\">10.5</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">22</td>\n    <td align=\"left\">bob</td>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">333</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">-7</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">4</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">1000.25</td>\n  </tr>\n</table>\n<p>(4 rows)<br />\n</p>\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 2;
        options.table_attr = "class=\"x\" id=7";
        people_query("people html border 2 \\pset tableattr 'class=\"x\" id=7'", options, "<table border=\"2\" class=\"x\" id=7>\n  <tr>\n    <th align=\"center\">id</th>\n    <th align=\"center\">name</th>\n    <th align=\"center\">score</th>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">1</td>\n    <td align=\"left\">alice</td>\n    <td align=\"right\">10.5</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">22</td>\n    <td align=\"left\">bob</td>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">333</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">-7</td>\n  </tr>\n  <tr valign=\"top\">\n    <td align=\"right\">4</td>\n    <td align=\"left\">&nbsp; </td>\n    <td align=\"right\">1000.25</td>\n  </tr>\n</table>\n<p>(4 rows)<br />\n</p>\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 1;
        options.expanded = .ON;
        options.table_attr = "class=\"x\" id=7";
        people_query("people html border 1 expanded \\pset tableattr 'class=\"x\" id=7'", options, "<table border=\"1\" class=\"x\" id=7>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 1</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">1</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">alice</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">10.5</td>\n  </tr>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 2</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">22</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">bob</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 3</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">333</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">-7</td>\n  </tr>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 4</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">4</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">1000.25</td>\n  </tr>\n</table>\n\n");
    }

    {
        options: Print_Options;
        options.format = .HTML;
        options.border = 2;
        options.expanded = .ON;
        options.table_attr = "class=\"x\" id=7";
        people_query("people html border 2 expanded \\pset tableattr 'class=\"x\" id=7'", options, "<table border=\"2\" class=\"x\" id=7>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 1</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">1</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">alice</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">10.5</td>\n  </tr>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 2</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">22</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">bob</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">&nbsp; </td>\n  </tr>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 3</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">333</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">-7</td>\n  </tr>\n\n  <tr><td colspan=\"2\" align=\"center\">Record 4</td></tr>\n  <tr valign=\"top\">\n    <th>id</th>\n    <td align=\"right\">4</td>\n  </tr>\n  <tr valign=\"top\">\n    <th>name</th>\n    <td align=\"left\">&nbsp; </td>\n  </tr>\n  <tr valign=\"top\">\n    <th>score</th>\n    <td align=\"right\">1000.25</td>\n  </tr>\n</table>\n\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 0;
        options.title = "People";
        options.table_attr = "2cm 5cm";
        people_query("people latex-longtable border 0 titled \\pset tableattr '2cm 5cm'", options, "\\begin{longtable}{rp{2cm\\textwidth}r}\n\\small\\textbf{\\textit{id}} & \\small\\textbf{\\textit{name}} & \\small\\textbf{\\textit{score}} \\\\\n\\midrule\n\\endfirsthead\n\\small\\textbf{\\textit{id}} & \\small\\textbf{\\textit{name}} & \\small\\textbf{\\textit{score}} \\\\\n\\midrule\n\\endhead\n\\caption[People (Continued)]{People}\n\\endfoot\n\\caption[People]{People}\n\\endlastfoot\n\\raggedright{1}\n&\n\\raggedright{alice}\n&\n\\raggedright{10.5} \\tabularnewline\n\\raggedright{22}\n&\n\\raggedright{bob}\n&\n\\raggedright{} \\tabularnewline\n\\raggedright{333}\n&\n\\raggedright{}\n&\n\\raggedright{-7} \\tabularnewline\n\\raggedright{4}\n&\n\\raggedright{}\n&\n\\raggedright{1000.25} \\tabularnewline\n\\end{longtable}\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 2;
        options.title = "People";
        options.table_attr = "2cm 5cm";
        people_query("people latex-longtable border 2 titled \\pset tableattr '2cm 5cm'", options, "\\begin{longtable}{| r | p{2cm\\textwidth} | r |}\n\\toprule\n\\small\\textbf{\\textit{id}} & \\small\\textbf{\\textit{name}} & \\small\\textbf{\\textit{score}} \\\\\n\\midrule\n\\endfirsthead\n\\toprule\n\\small\\textbf{\\textit{id}} & \\small\\textbf{\\textit{name}} & \\small\\textbf{\\textit{score}} \\\\\n\\midrule\n\\endhead\n\\bottomrule\n\\caption[People (Continued)]{People}\n\\endfoot\n\\bottomrule\n\\caption[People]{People}\n\\endlastfoot\n\\raggedright{1}\n&\n\\raggedright{alice}\n&\n\\raggedright{10.5} \\tabularnewline\n\\raggedright{22}\n&\n\\raggedright{bob}\n&\n\\raggedright{} \\tabularnewline\n\\raggedright{333}\n&\n\\raggedright{}\n&\n\\raggedright{-7} \\tabularnewline\n\\raggedright{4}\n&\n\\raggedright{}\n&\n\\raggedright{1000.25} \\tabularnewline\n\\end{longtable}\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 0;
        options.title = "People";
        options.table_attr = "4cm";
        people_query("people latex-longtable border 0 titled \\pset tableattr '4cm'", options, "\\begin{longtable}{rp{4cm\\textwidth}r}\n\\small\\textbf{\\textit{id}} & \\small\\textbf{\\textit{name}} & \\small\\textbf{\\textit{score}} \\\\\n\\midrule\n\\endfirsthead\n\\small\\textbf{\\textit{id}} & \\small\\textbf{\\textit{name}} & \\small\\textbf{\\textit{score}} \\\\\n\\midrule\n\\endhead\n\\caption[People (Continued)]{People}\n\\endfoot\n\\caption[People]{People}\n\\endlastfoot\n\\raggedright{1}\n&\n\\raggedright{alice}\n&\n\\raggedright{10.5} \\tabularnewline\n\\raggedright{22}\n&\n\\raggedright{bob}\n&\n\\raggedright{} \\tabularnewline\n\\raggedright{333}\n&\n\\raggedright{}\n&\n\\raggedright{-7} \\tabularnewline\n\\raggedright{4}\n&\n\\raggedright{}\n&\n\\raggedright{1000.25} \\tabularnewline\n\\end{longtable}\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 2;
        options.title = "People";
        options.table_attr = "4cm";
        people_query("people latex-longtable border 2 titled \\pset tableattr '4cm'", options, "\\begin{longtable}{| r | p{4cm\\textwidth} | r |}\n\\toprule\n\\small\\textbf{\\textit{id}} & \\small\\textbf{\\textit{name}} & \\small\\textbf{\\textit{score}} \\\\\n\\midrule\n\\endfirsthead\n\\toprule\n\\small\\textbf{\\textit{id}} & \\small\\textbf{\\textit{name}} & \\small\\textbf{\\textit{score}} \\\\\n\\midrule\n\\endhead\n\\bottomrule\n\\caption[People (Continued)]{People}\n\\endfoot\n\\bottomrule\n\\caption[People]{People}\n\\endlastfoot\n\\raggedright{1}\n&\n\\raggedright{alice}\n&\n\\raggedright{10.5} \\tabularnewline\n\\raggedright{22}\n&\n\\raggedright{bob}\n&\n\\raggedright{} \\tabularnewline\n\\raggedright{333}\n&\n\\raggedright{}\n&\n\\raggedright{-7} \\tabularnewline\n\\raggedright{4}\n&\n\\raggedright{}\n&\n\\raggedright{1000.25} \\tabularnewline\n\\end{longtable}\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 0;
        options.title = "People";
        options.table_attr = "  3cm\t";
        people_query("people latex-longtable border 0 titled \\pset tableattr '  3cm\t'", options, "\\begin{longtable}{rp{3cm\\textwidth}r}\n\\small\\textbf{\\textit{id}} & \\small\\textbf{\\textit{name}} & \\small\\textbf{\\textit{score}} \\\\\n\\midrule\n\\endfirsthead\n\\small\\textbf{\\textit{id}} & \\small\\textbf{\\textit{name}} & \\small\\textbf{\\textit{score}} \\\\\n\\midrule\n\\endhead\n\\caption[People (Continued)]{People}\n\\endfoot\n\\caption[People]{People}\n\\endlastfoot\n\\raggedright{1}\n&\n\\raggedright{alice}\n&\n\\raggedright{10.5} \\tabularnewline\n\\raggedright{22}\n&\n\\raggedright{bob}\n&\n\\raggedright{} \\tabularnewline\n\\raggedright{333}\n&\n\\raggedright{}\n&\n\\raggedright{-7} \\tabularnewline\n\\raggedright{4}\n&\n\\raggedright{}\n&\n\\raggedright{1000.25} \\tabularnewline\n\\end{longtable}\n");
    }

    {
        options: Print_Options;
        options.format = .LATEX_LONGTABLE;
        options.border = 2;
        options.title = "People";
        options.table_attr = "  3cm\t";
        people_query("people latex-longtable border 2 titled \\pset tableattr '  3cm\t'", options, "\\begin{longtable}{| r | p{3cm\\textwidth} | r |}\n\\toprule\n\\small\\textbf{\\textit{id}} & \\small\\textbf{\\textit{name}} & \\small\\textbf{\\textit{score}} \\\\\n\\midrule\n\\endfirsthead\n\\toprule\n\\small\\textbf{\\textit{id}} & \\small\\textbf{\\textit{name}} & \\small\\textbf{\\textit{score}} \\\\\n\\midrule\n\\endhead\n\\bottomrule\n\\caption[People (Continued)]{People}\n\\endfoot\n\\bottomrule\n\\caption[People]{People}\n\\endlastfoot\n\\raggedright{1}\n&\n\\raggedright{alice}\n&\n\\raggedright{10.5} \\tabularnewline\n\\raggedright{22}\n&\n\\raggedright{bob}\n&\n\\raggedright{} \\tabularnewline\n\\raggedright{333}\n&\n\\raggedright{}\n&\n\\raggedright{-7} \\tabularnewline\n\\raggedright{4}\n&\n\\raggedright{}\n&\n\\raggedright{1000.25} \\tabularnewline\n\\end{longtable}\n");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        options.field_separator_zero = true;
        people_query("people unaligned fieldsep_zero", options, "id\x00name\x00score\n1\x00alice\x0010.5\n22\x00bob\x00\n333\x00\x00-7\n4\x00\x001000.25\n(4 rows)\n");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        options.field_separator_zero = true;
        options.tuples_only = true;
        people_query("people unaligned fieldsep_zero tuples only", options, "1\x00alice\x0010.5\n22\x00bob\x00\n333\x00\x00-7\n4\x00\x001000.25\n");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        options.record_separator_zero = true;
        people_query("people unaligned recordsep_zero", options, "id|name|score\x001|alice|10.5\x0022|bob|\x00333||-7\x004||1000.25\x00(4 rows)\x00");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        options.record_separator_zero = true;
        options.tuples_only = true;
        people_query("people unaligned recordsep_zero tuples only", options, "1|alice|10.5\x0022|bob|\x00333||-7\x004||1000.25\x00");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        options.field_separator_zero = true;
        options.record_separator_zero = true;
        people_query("people unaligned fieldsep_zero recordsep_zero", options, "id\x00name\x00score\x001\x00alice\x0010.5\x0022\x00bob\x00\x00333\x00\x00-7\x004\x00\x001000.25\x00(4 rows)\x00");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        options.field_separator_zero = true;
        options.record_separator_zero = true;
        options.tuples_only = true;
        people_query("people unaligned fieldsep_zero recordsep_zero tuples only", options, "1\x00alice\x0010.5\x0022\x00bob\x00\x00333\x00\x00-7\x004\x00\x001000.25\x00");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        options.expanded = .ON;
        options.field_separator_zero = true;
        people_query("people unaligned expanded fieldsep_zero", options, "id\x001\nname\x00alice\nscore\x0010.5\n\nid\x0022\nname\x00bob\nscore\x00\n\nid\x00333\nname\x00\nscore\x00-7\n\nid\x004\nname\x00\nscore\x001000.25\n");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        options.expanded = .ON;
        options.field_separator_zero = true;
        options.tuples_only = true;
        people_query("people unaligned expanded fieldsep_zero tuples only", options, "id\x001\nname\x00alice\nscore\x0010.5\n\nid\x0022\nname\x00bob\nscore\x00\n\nid\x00333\nname\x00\nscore\x00-7\n\nid\x004\nname\x00\nscore\x001000.25\n");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        options.expanded = .ON;
        options.record_separator_zero = true;
        people_query("people unaligned expanded recordsep_zero", options, "id|1\x00name|alice\x00score|10.5\x00\x00id|22\x00name|bob\x00score|\x00\x00id|333\x00name|\x00score|-7\x00\x00id|4\x00name|\x00score|1000.25\x00");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        options.expanded = .ON;
        options.record_separator_zero = true;
        options.tuples_only = true;
        people_query("people unaligned expanded recordsep_zero tuples only", options, "id|1\x00name|alice\x00score|10.5\x00\x00id|22\x00name|bob\x00score|\x00\x00id|333\x00name|\x00score|-7\x00\x00id|4\x00name|\x00score|1000.25\x00");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        options.expanded = .ON;
        options.field_separator_zero = true;
        options.record_separator_zero = true;
        people_query("people unaligned expanded fieldsep_zero recordsep_zero", options, "id\x001\x00name\x00alice\x00score\x0010.5\x00\x00id\x0022\x00name\x00bob\x00score\x00\x00\x00id\x00333\x00name\x00\x00score\x00-7\x00\x00id\x004\x00name\x00\x00score\x001000.25\x00");
    }

    {
        options: Print_Options;
        options.format = .UNALIGNED;
        options.expanded = .ON;
        options.field_separator_zero = true;
        options.record_separator_zero = true;
        options.tuples_only = true;
        people_query("people unaligned expanded fieldsep_zero recordsep_zero tuples only", options, "id\x001\x00name\x00alice\x00score\x0010.5\x00\x00id\x0022\x00name\x00bob\x00score\x00\x00\x00id\x00333\x00name\x00\x00score\x00-7\x00\x00id\x004\x00name\x00\x00score\x001000.25\x00");
    }
}
