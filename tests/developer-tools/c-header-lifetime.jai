#import "Basic";
#import "Hash_Table";
H :: #import "Generate_C_Header";

Lifetime_Point :: struct {
    x: s32;
}

main :: () {
    info: H.Header_Info;
    info.jai_type_prefix = "Jai";
    table_add(*info.exported_types, type_info(Lifetime_Point), "Lifetime");
    path := sprint("%../../target/developer-tools-lifetime.h", #filepath);

    defer free(path);
    assert(H.generate_header(*info, path));
    reset_temporary_storage();
    scratch := talloc(16384);

    memset(scratch, #char "X", 16384);
    found, stored := table_find(*info.name_types, "JaiLifetime");

    assert(found && stored == type_info(Lifetime_Point));
    assert(H.generate_header(*info, path));
    for entry: info.name_types.entries if entry.hash >= FIRST_VALID_HASH free(entry.key);

    deinit(*info.name_types);
    deinit(*info.exported_types);
    print("ok\n");
}
