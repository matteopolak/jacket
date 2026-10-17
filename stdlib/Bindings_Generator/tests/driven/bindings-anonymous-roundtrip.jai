#import "Basic";
G :: #import "developer-tools-anonymous-generated";
D :: #import "developer-tools-anonymous-default";

main :: () {
    record := G.Renamed_Record.{ value = 10, weight = 12.5 };
    value: G.Anonymous_Union;

    assert(G.original_anonymous(*record, *value, .ORIGINAL_MODE_NEGATIVE) == 7);
    assert(record.value == 7 && value.number == 12.5);
    alias: G.Record_Alias = record;
    mode: G.Mode_Alias = .ORIGINAL_MODE_POSITIVE;
    forward := G.Forward_Record.{ count = 1 };

    assert(G.original_alias(*alias, mode, *forward) == 15);
    assert(forward.count == 15);
    default_record := D.Anonymous_Record.{ value = 3, weight = 4.5 };
    default_union: D.Anonymous_Union;

    assert(D.original_anonymous(*default_record, *default_union, .ORIGINAL_MODE_POSITIVE) == 10);
    assert(default_union.number == 4.5);
    print("ok\n");
}
