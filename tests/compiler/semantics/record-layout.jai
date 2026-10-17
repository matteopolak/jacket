// Original general record-layout regression for strict semantic lint coverage.
#import "Basic";

Table :: struct {
    items: [] Padded;
    Padded :: struct {
        using info: Payload;
        INFO_SIZE :: size_of(type_of(info));
        PADDING_SIZE :: #run align_forward(INFO_SIZE, 64) - INFO_SIZE;
        padding: [PADDING_SIZE] u8;
    }
    Payload :: struct {
        owner: *Padded;
    }
}

main :: () {
    assert(Table.Padded.INFO_SIZE == size_of(Table.Payload));
    assert(Table.Padded.PADDING_SIZE == 64 - size_of(Table.Payload));
    assert(size_of(Table.Padded) == 64);
    print("ok\n");
}
