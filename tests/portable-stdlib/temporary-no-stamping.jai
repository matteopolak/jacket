// Without TEMP_ALLOCATOR_OVERWRITE_MEMORY a reset leaves released storage untouched.
#import "Basic";

main :: () {
    initial: [256] u8;
    plain: Temporary_Storage_Paged_Alloc;
    plain.data = initial.data;
    plain.size = initial.count;

    push_context {
        context.temporary_storage = *plain;
        memory := cast(*u8) talloc(16);
        memory[0] = 7;
        reset_temporary_storage();
        assert(memory[0] == 7);
    }

    print("ok\n");
}
