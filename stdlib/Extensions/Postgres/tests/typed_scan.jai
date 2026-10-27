// test: skip_on(.WASM)
// The SQL scanner behind typed queries: placeholders, skipped text and position maps. Needs no
// server.
#import "Basic";
#import "String";
Pg :: #import "Extensions/Postgres";

main :: () {
    test_rewrite();
    test_skipped_text();
    test_errors();
    test_positions();
    test_ids();
    print("ok\n");
}

rewritten :: (source: string) -> string {
    r := Pg.rewrite_placeholders(source);
    assert(r.error == "", "unexpected error: %", r.error);
    return r.sql;
}

test_rewrite :: () {
    assert(rewritten("select 1") == "select 1");
    assert(rewritten("select % + %") == "select $1 + $2");
    assert(rewritten("select 5 %% 3") == "select 5 % 3");
    assert(rewritten("a %, b %, c %, d %, e %, f %, g %, h %, i %, j %, k %") == "a $1, b $2, c $3, d $4, e $5, f $6, g $7, h $8, i $9, j $10, k $11");

    r := Pg.rewrite_placeholders("where a = % and b %% 2 = % ");

    assert(r.holes == 2 && r.edits.count == 3);
}

test_skipped_text :: () {
    assert(rewritten("select 'a % b', %") == "select 'a % b', $1");
    assert(rewritten("select 'it''s %', %") == "select 'it''s %', $1");
    assert(rewritten("select E'it\\'s %', %") == "select E'it\\'s %', $1");
    assert(rewritten("select \"a%b\", %") == "select \"a%b\", $1");
    assert(rewritten("select \"a\"\"%\", %") == "select \"a\"\"%\", $1");
    assert(rewritten("select $$a % $1 b$$, %") == "select $$a % $1 b$$, $1");
    assert(rewritten("select $tag$ % 'x $tag$, %") == "select $tag$ % 'x $tag$, $1");
    assert(rewritten("select 1 -- 100% sure\n, %") == "select 1 -- 100% sure\n, $1");
    assert(rewritten("select /* 100% /* nested % */ still */ %") == "select /* 100% /* nested % */ still */ $1");
    assert(rewritten("select a$1, %") == "select a$1, $1");
    assert(rewritten("select 'unterminated %") == "select 'unterminated %");
    assert(rewritten("select x from t where name like '%@example.com' and id = %") == "select x from t where name like '%@example.com' and id = $1");
}

test_errors :: () {
    r := Pg.rewrite_placeholders("select $1");
    assert(r.error != "" && r.error_offset == 7);

    r = Pg.rewrite_placeholders("select % where a = $2");
    assert(r.error != "" && r.error_offset == 19);
}

test_positions :: () {
    // select %, %%, % from t
    source := "select %, %% , % from t";
    r := Pg.rewrite_placeholders(source);

    assert(r.sql == "select $1, % , $2 from t");

    // "from" is at 18 in the rewritten text and at 19 in the source.
    from := find_index_from_left(r.sql, "from");

    assert(Pg.source_offset_of(*r, from) == find_index_from_left(source, "from"));
    // the "$1" maps to its "%".
    assert(Pg.source_offset_of(*r, 7) == 7 && Pg.source_offset_of(*r, 8) == 7);
    // the escaped "%"
    assert(Pg.source_offset_of(*r, 11) == 10);
    assert(Pg.source_offset_of(*r, 0) == 0);

    assert(Pg.byte_of_character("héllo wörld", 3) == 3);
    assert(Pg.byte_of_character("héllo wörld", 9) == 10);
    assert(Pg.byte_of_character("abc", 99) == 3);
}

test_ids :: () {
    a := Pg.statement_id("select 1");
    b := Pg.statement_id("select 2");

    assert(a != b && a == Pg.statement_id("select 1"));
    assert(Pg.id_hex(0x0123456789abcdef) == "0123456789abcdef");
    assert(Pg.statement_digest_hex("") == "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
}
