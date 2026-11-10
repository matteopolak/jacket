// test: skip_on(.WASM, reason = "needs Extensions/Io fibers, which wasm cannot run (no stack switching)") compile_fail("the statement has 3 placeholders but 2 arguments were given")
// `..values` of a fixed `[N] Any` stands for N arguments, so a typed query with another number of
// placeholders is refused at compile time. Needs no server: the count is checked before the
// statement is described.
#import "Basic";
Pg :: #import "Extensions/Postgres";

main :: () {
    db: *Pg.Connection;
    values := Any.[1, "Alice"];

    _, _ = Pg.query_one(db, "select count(*) as n from users where id = % and display_name = % and email = %", ..values);
}
