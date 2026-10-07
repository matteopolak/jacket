// Foreign calls passing and returning C structs by value (interpreter and native).
#import "Basic";

structs :: #library "libstructs";

#load "types.jai";

main :: () {
    print("% % % % % % %\n", add_v2(.{ 1, 2 }, .{ 10, 20 }), scale_v3(.{ 1, 2, 3 }, 2), make_big(5), sum_big(.{ 1, 2, 3, 4 }),
        make_small(-7, 9), swap_mixed(.{ 2.5, 99 }), add_large(.{ 1, 2, 3 }, .{ 10, 20, 30 }));
    print("%\n", many(1, 2, 3, 4, 5, 6, 7, .{ 100, 200, 300 }, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 2));
}
