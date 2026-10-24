"""Compiles programs that must not compile and checks where jaic reports each error.

    python3 bad.py <jaic> <work directory>

Needs DATABASE_URL and the schema of schema.sql, like run.sh. Each case is a program, the text
the error must point at (`at`, found in the program), and fragments of the message.
"""
import os
import subprocess
import sys

PRELUDE = '''#import "Basic";
Pg :: #import "Extensions/Postgres";
User :: struct { id: s64; email: string; display_name: string; }
Orm_User :: struct @pg_table(public.users) @pg_has_many(posts:Orm_Post:author_id:id) { id: s64; email: string; }
Orm_Post :: struct @pg_table(public.posts) @pg_belongs_to(author:author_id:Orm_User:id) { id: s64; author_id: s64; title: string; }
main :: (db: *Pg.Connection) {
'''

CASES = [
    ("unknown column", 'rows, error := Pg.query(db, "select id, emal from users");', "emal", "42703", "Perhaps you meant"),
    ("unknown table", 'rows, error := Pg.query(db, "select * from nope");', "nope", "42P01", ""),
    ("syntax error", 'rows, error := Pg.query(db, "select id frm users");', 'users");', "42601", ""),
    (
        "error on a later line",
        'rows, error := Pg.query(db, #string SQL\n    select id\n    from users\n    where emial = 1\nSQL);',
        "emial", "42703", "",
    ),
    (
        "argument of the wrong type",
        'name := "x"; rows, error := Pg.query(db, "select id from users where id = %", name);',
        "name);", "argument 1 is string, but parameter $1 is bigint", "",
    ),
    (
        "s64 does not narrow",
        'big: s64 = 1; rows, error := Pg.query(db, "select id from users where score = %", big);',
        "big);", "argument 1 is s64, but parameter $1 is integer", "",
    ),
    (
        "second argument",
        'id: s64 = 1; rows, error := Pg.query(db, "select id from users where id = % and email = %", id, 5);',
        "5);", "argument 2 is", "",
    ),
    (
        "too few arguments",
        'rows, error := Pg.query(db, "select id from users where id = %");',
        '"select id from', "has 1 placeholder but 0 arguments were given", "",
    ),
    ("postgres placeholder", 'rows, error := Pg.query(db, "select id from users where id = $1", 1);', "$1", "own $n", ""),
    ("no rows", 'rows, error := Pg.query(db, "delete from posts where id = 0");', '"delete', "returns no rows", ""),
    (
        "query_as missing column",
        'rows, error := Pg.query_as(User, db, "select id, email from users");',
        '"select id', "no column of that name", "",
    ),
    (
        "query_as nullable column",
        'rows, error := Pg.query_as(User, db, "select id, email, display_name from users");',
        '"select id', "can be NULL", "",
    ),
    ("duplicate column", 'rows, error := Pg.query(db, "select id, id from users");', '"select id', "two columns are named", ""),
    ("orm unknown field", 'rows, error := Pg.select(db, Orm_User, where = #code it.emial == "x");', "it.emial", 'has no field "emial"', ""),
    (
        "orm call",
        'twice :: (n: s64) -> s64 { return n * 2; } rows, error := Pg.select(db, Orm_User, where = #code it.id == twice(it.id));',
        "twice(it.id)", "cannot call twice", "",
    ),
    ("orm no table", 'rows, error := Pg.select(db, User, where = #code it.id == 1);', "it.id == 1", "no @pg_table note", ""),
    ("orm set", 'n, error := Pg.update(db, Orm_User, set = #code it.nope = 1, where = #code it.id == 1);', "it.nope", "set assigns to a field", ""),
    ("orm json operator", 'rows, error := Pg.select(db, Orm_User, where = #code Pg.has_key(it.email, "a"));', "it.email", "is not a json column", ""),
    ("orm sql literal", 'clause := "true"; rows, error := Pg.select(db, Orm_User, where = #code Pg.sql(clause));', "clause));", "takes a string literal", ""),
    ("orm related field", 'rows, error := Pg.select(db, Orm_Post, where = #code it.author.nope == 1);', "it.author.nope", 'has no field "nope"', ""),
    ("orm has-many filter", 'rows, error := Pg.select(db, Orm_User, where = #code it.posts.title == "x");', "it.posts.title", "has-many relation", ""),
    ("orm load kind", 'users: [] Orm_User; loaded, error := Pg.load_one(db, users, "posts");', "Pg.load_one", "load it with Pg.load_many", ""),
    ("orm sql field", 'rows, error := Pg.select(db, Orm_User, where = #code Pg.sql("%(it.nope) = 1"));', 'Pg.sql("%(it.nope) = 1")', 'has no field "nope"', ""),
]


def main():
    jaic, work = sys.argv[1], sys.argv[2]
    failed = 0

    for index, (name, statement, at, message, hint) in enumerate(CASES):
        source = PRELUDE + "    " + statement + "\n}\n#run main(null);\n"
        path = os.path.join(work, "bad%d.jai" % index)

        with open(path, "w") as f:
            f.write(source)

        done = subprocess.run([jaic, os.path.basename(path), "-exe", "bad"], capture_output=True, text=True, cwd=work)
        output = done.stdout + done.stderr

        offset = source.index(at)
        line = source.count("\n", 0, offset) + 1
        column = offset - (source.rfind("\n", 0, offset) + 1) + 1
        expected = "%s:%d:%d: error" % (os.path.basename(path), line, column)

        problems = []
        if done.returncode == 0:
            problems.append("it compiled")
        if expected not in output:
            problems.append("no error at %d:%d" % (line, column))
        for fragment in (message, hint):
            if fragment and fragment not in output:
                problems.append("no %r in the output" % fragment)

        if problems:
            failed += 1
            print("FAILED %s: %s\n%s" % (name, "; ".join(problems), output[:1500]))
        else:
            print("ok: %s" % name)

    sys.exit(1 if failed else 0)


main()
