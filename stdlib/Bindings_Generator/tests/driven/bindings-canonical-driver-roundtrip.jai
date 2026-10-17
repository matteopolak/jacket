#import "Basic";
G :: #import "developer-tools-driver-generated";

main :: () {
    record := G.Record.{ 12, 30 };

    assert(record.first + record.second == 42);
    assert(G.Alias == G.Record && G.DRIVE_CODE == 7);
    assert(cast(s64) G.FAMILY.ONE == 4 && cast(s64) G.FAMILY.TWO == 5);
    assert(cast(s32) G.DriveStatus.BAD == -3 && cast(s32) G.DriveStatus.GOOD == 5);
    assert(size_of(G.Record) == 8 && align_of(G.Record) == 4);
    print("ok\n");
}
