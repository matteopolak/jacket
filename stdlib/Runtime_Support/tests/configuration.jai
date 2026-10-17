#import "Basic";
C :: #import "Compiler";

#if !#exists(REQUESTED_SIZE) REQUESTED_SIZE :: 65536;

#run {
    options := C.get_build_options();
    options.temporary_storage_size = REQUESTED_SIZE;
    C.set_build_options(options);
}

main :: () {
    storage := cast(*Temporary_Storage_Paged_Alloc) context.temporary_storage;

    assert(storage != null);
    assert(storage.size == REQUESTED_SIZE && storage.original_size == REQUESTED_SIZE);
    assert(storage.data == storage.original_data);
    assert((storage.data == null) == (REQUESTED_SIZE == 0));

    if REQUESTED_SIZE > 0 {
        storage.data[REQUESTED_SIZE - 1] = 73;
        assert(storage.data[REQUESTED_SIZE - 1] == 73);
    }

    print("ok\n");
}
