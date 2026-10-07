Columns :: #import "Soa";

Point :: struct {
    x: s32 = 5;
    y: float64;
    LABEL :: "point";
}

main :: () -> s64 {
    rows: Columns.Soa(Point, 3);
    if rows.x.count != 3 || rows.y.count != 3 return 1;
    if rows.x[0] != 0 return 2;
    rows.x[1] = 9;
    rows.y[1] = 1.25;
    observed := rows[1];
    if observed.x != 9 || observed.y != 1.25 return 3;
    rows[2] = Point.{ x = 12, y = 2.5 };
    if rows.x[2] != 12 || rows.y[2] != 2.5 return 4;
    for *row: rows {
        row.x += 1;
    }
    if rows.x[0] != 1 || rows.x[1] != 10 || rows.x[2] != 13 return 5;
    if (cast(*u8) *rows.x[2] - cast(*u8) *rows.x[0]) != 2 * size_of(s32) return 6;
    empty: Columns.Soa(Point, 0);
    if empty.x.count != 0 || empty.y.count != 0 return 7;
    return 0;
}
