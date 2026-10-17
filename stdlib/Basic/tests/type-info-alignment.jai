#import "Basic";

Record :: struct {
    value: u64;
}

main :: () {
    info := cast(*Type_Info_Struct) type_info(Record);
    assert(info.alignment == align_of(Record));
    print("ok\n");
}
