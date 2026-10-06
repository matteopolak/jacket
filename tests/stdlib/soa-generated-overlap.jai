Columns :: #import "Soa";
Overlaid :: union {
    first: u32;
    second: float;
}
main :: () -> s64 {
    rows: Columns.Soa(Overlaid, 2);
    rows.first[0] = 42;
    value := rows[0];
    if value.first != 42 return 1;
    rows[1] = Overlaid.{ first = 7 };
    if rows.first[1] != 7 return 2;
    return 0;
}
