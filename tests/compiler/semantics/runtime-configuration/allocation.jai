#import "Basic";
C :: #import "Compiler";

#run {
    options := C.get_build_options();
    options.temporary_storage_size = 65536;
    C.set_build_options(options);
}

main :: () {
    storage := cast(*Temporary_Storage_Paged_Alloc) context.temporary_storage;

    assert(storage != null);
    assert(storage.size == 65536 && storage.original_size == 65536);
    assert(storage.data != null && storage.original_data == storage.data);
    storage.data[65535] = 73;
    assert(storage.data[65535] == 73);
    print("ok\n");
}
