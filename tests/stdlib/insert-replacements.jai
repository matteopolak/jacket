// `#insert (break=..., remove=...) body` in a for_expansion: the loop body's `break` and
// `remove` become the macro's replacements (here: leave both nested loops; count removals).
// rules: code.12
#import "Basic";
Grid :: struct {
    rows: [3][3] int;
    removed: int;
}
for_expansion :: (g: *Grid, body: Code, flags: For_Flags) #expand {
    `it_index := 0;
    for row, r: g.rows {
        for cell, c: row {
            `it := cell;
            #insert (break = break row, remove = {
                g.removed += 1;
            }) body;
            it_index += 1;
        }
    }
}
main :: () {
    g: Grid;
    for r: 0..2 for c : 0..2 g.rows[r][c] = r * 3 + c;
    seen := 0;
    for g {
        if it == 4 break;
        if it % 2 == 1 remove it;
        seen += 1;
    }
    assert(seen == 4 && g.removed == 2);
}
