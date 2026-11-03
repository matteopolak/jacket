#import "Basic";
D :: #import "Deep_Copy";

main :: () {
    text := "owned";
    copy := D.Deep_Copy(*text, .{});

    assert(copy.* == text && copy.data != text.data);
    free(copy.*);
    free(copy);

    copy = D.Deep_Copy(*text, .{});
    free(copy.*);
    free(copy);
}
