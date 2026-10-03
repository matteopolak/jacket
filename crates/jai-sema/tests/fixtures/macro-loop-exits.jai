trace: int;
stop :: (selected: bool) #expand {
    defer trace += 1;
    if selected break;
}
skip :: () #expand { continue; }
main :: () -> int {
    for row: 0..2 {
        defer trace += 10;
        stop(row == 1);
        skip();
    }
    return trace + 20;
}
