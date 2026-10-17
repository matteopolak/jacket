// The original macOS SDK driver counts actual SIGTRAP deliveries.
#import "Basic";
D :: #import "Debug";

main :: () {
    first := Source_Code_Location.{ "original-once.jai", 17, 3 };
    second := Source_Code_Location.{ "original-once.jai", 17, 4 };
    attached := D.is_debugger_present();

    assert(D.attach_to_debugger() == attached);
    assert(!D.os_attach_to_debugger());

    assert(D.break_once(first) == attached);
    assert(!D.break_once(first));
    assert(!D.break_once(first));
    assert(D.break_once(second) == attached);
    assert(!D.break_once(second));
    // The first location must retain its own filename bytes.
    bytes: [5] u8 = .[111, 119, 110, 101, 100];
    owned := Source_Code_Location.{ string.{ bytes.count, bytes.data }, 9, 2 };

    assert(D.break_once(owned) == attached);
    bytes[0] = 88;
    assert(!D.break_once(Source_Code_Location.{ "owned", 9, 2 }));
    print("ok\n");
}
