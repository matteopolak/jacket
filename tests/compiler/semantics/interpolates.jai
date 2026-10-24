// jaic extension: `@Interpolates("mode")` turns `%name` holes in a string literal into
// positional arguments before type checking.
#import "Basic";

Row :: struct {
    format: string;
    values: [4] s64;
    count: s64;
}

record :: (format: string, args: ..Any) -> Row {
    row: Row;
    row.format = format;
    row.count = args.count;
    for args row.values[it_index] = <<cast(*s64) it.value_pointer;

    return row;
} @Interpolates("plain")

sql :: ($text: string, args: ..Any) -> Row #expand {
    return record(text, ..args);
} @Interpolates("sql")

Account :: struct {
    id: s64;
    limit: s64;
}

main :: () {
    account := Account.{ 7, 100 };
    ids := s64.[11, 22, 33];
    i := 2;

    a := record("id %account.id of %(account.limit * 2) at %ids[i]");

    assert(a.format == "id % of % at %");
    assert(a.count == 3 && a.values[0] == 7 && a.values[1] == 200 && a.values[2] == 33);

    // `%` that starts no hole, and `%%`, are left for the callee.
    b := record("100% sure, 10 % 3, %%");

    assert(b.count == 0 && b.format == "100% sure, 10 % 3, %%");

    // SQL mode leaves quoted text, comments and dollar quotes alone.
    c := sql("select '%account', \"%account\", $$%account$$ -- %account\n where limit > %account.limit and id % 2 = %account.id /* %account */");

    assert(c.count == 2 && c.values[0] == 100 && c.values[1] == 7);
    assert(c.format == "select '%account', \"%account\", $$%account$$ -- %account\n where limit > % and id % 2 = % /* %account */");

    // A string without holes keeps positional arguments.
    d := record("a % b %", 1, 2);

    assert(d.count == 2 && d.values[1] == 2);

    // The same holes in a here-string.
    e := sql(#string END
select %account.id
  from t where n = %(ids[0] + 1)
END);

    assert(e.count == 2 && e.values[0] == 7 && e.values[1] == 12);
}

#run main();
