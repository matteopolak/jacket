#import "Basic";
G :: #import "developer-tools-multi-generated";
C :: #import "developer-tools-multi-custom";
D :: #import "developer-tools-multi-cpp-generated";

main :: () {
    record := G.MultiRecord.{ value = 11 };
    custom := C.MultiRecord.{ value = 11 };

    assert(G.MULTI_SCALE == 3 && G.MULTI_OFFSET == 7);
    assert(G.multi_base(*record) == 33);
    assert(G.multi_sum(*record, 5) == 45);
    assert(C.MULTI_SCALE == 3 && C.MULTI_OFFSET == 7);
    assert(C.custom_sum(*custom, 5) == 45);
    cpp := D.MultiRecord.{ value = 11 };

    assert(D.multi_base(*cpp) == 33 && D.multi_sum(*cpp, 5) == 45);
    print("ok\n");
}
