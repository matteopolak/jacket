#import "Basic";
N :: #import "macOS";

main :: () {
    info := cast(*Type_Info_Struct) type_info(N.FSEventStreamContext);

    assert(info.members.count == 5 && info.runtime_size == 40);
    assert(info.members[0].name == "version" && info.members[0].offset_in_bytes == 0);
    assert(info.members[1].name == "info" && info.members[1].offset_in_bytes == 8);
    assert(info.members[2].name == "retain" && info.members[2].offset_in_bytes == 16);
    assert(info.members[3].name == "release" && info.members[3].offset_in_bytes == 24);
    assert(info.members[4].name == "copyDescription" && info.members[4].offset_in_bytes == 32);
    print("ok\n");
}
