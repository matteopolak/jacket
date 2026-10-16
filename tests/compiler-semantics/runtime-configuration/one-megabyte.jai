#import "Basic";
C :: #import "Compiler";

#run {
    options := C.get_build_options();
    options.temporary_storage_size = 1048576;
    C.set_build_options(options);
}

main :: () {
    storage := cast(*Temporary_Storage_Paged_Alloc) context.temporary_storage;

    assert(storage != null);
    assert(storage.size == 1048576 && storage.original_size == 1048576);
    assert(storage.data != null && storage.original_data == storage.data);
    storage.data[1048575] = 73;
    assert(storage.data[1048575] == 73);
    print("ok\n");
}
