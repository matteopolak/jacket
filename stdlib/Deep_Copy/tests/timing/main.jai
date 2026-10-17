// Reviewed Deep_Copy graph/callback API; independent timer side-effect regression.
#import "Basic";
D :: #import "Deep_Copy";
P :: #import "POSIX";

map_string :: (input: string, data: *void) -> string {
    delay := P.timespec.{ 0, 20000000 };
    status := P.nanosleep(*delay, null);
    if status != 0 exit(2);
    return input;
}

main :: () {
    before := D.time_total;
    text := "timer callback";
    copy := D.Deep_Copy(*text, .{ string_mapper_proc = map_string });

    if copy == null exit(3);
    if copy.* != text exit(4);

    free(copy);
    if D.time_total - before < 0.015 exit(5);

    print("ok\n");
}
