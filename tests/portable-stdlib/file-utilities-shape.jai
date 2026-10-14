#import "Basic";
U :: #import "File_Utilities";

main :: () {
    assert(type_info(U.File_Visit_Info).name == "File_Visit_Info");
    info: U.File_Visit_Info;

    assert(info.short_name.count == 0 && info.full_name.count == 0);
    assert(info.size == 0 && !info.had_error && !info.is_symlink && !info.is_directory);
    assert(!info.descend_into_directory);
    print("ok\n");
}
