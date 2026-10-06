// context.stack_trace lists the active procedures, innermost first. Each node holds the
// line of the call it is currently making.

depth_seen: u32;
names: [8] string;
lines: [8] u32;
count: int;

record :: () {
    node := context.stack_trace;
    while node {
        if node.info && count < 8 {
            names[count] = node.info.name;
            lines[count] = node.line_number;
            count += 1;
            depth_seen = node.call_depth;
        }
        node = node.next;
    }
}

inner :: (x: int) {
    if x == 0 {
        record();
    } else {
        inner(x - 1);
    }
}

main :: () -> s64 {
    inner(1);
    // record's own node comes first, then inner (x == 0), inner (x == 1), main.
    if count != 4 return 1;
    if names[0] != "record" return 2;
    if names[1] != "inner" || lines[1] != 24 return 3;
    if names[2] != "inner" || lines[2] != 26 return 4;
    if names[3] != "main" || lines[3] != 31 return 5;
    if depth_seen != 1 return 6;
    return 0;
}
