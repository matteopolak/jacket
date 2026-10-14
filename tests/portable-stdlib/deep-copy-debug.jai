#import "Basic";
D :: #import "Deep_Copy"(DC_DEBUG = true, DEBUG_PRINT = false);
Plain :: #import "Deep_Copy";

main :: () {
    #assert #exists(D.indent);
    #assert !#exists(Plain.indent);
    D.helper_count = 17;
    value: s32 = 9;
    copy := D.Deep_Copy(*value, .{});

    assert(copy.* == 9 && D.helper_count > 17 && D.indent == 0);
    free(copy);
    print("ok\n");
}
