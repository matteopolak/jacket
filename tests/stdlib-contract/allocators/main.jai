// The overwrite allocator poisons fresh bytes and is torn down after its clients.
#import "Basic";
Overwriting_Allocator :: #import "Overwriting_Allocator";

main :: () {
    tracked := Overwriting_Allocator.get_overwriting_allocator(overwrite_byte=0xa5);
    {
        push_allocator(tracked);
        bytes := alloc(5);
        assert(bytes != null);
        data := cast(*u8) bytes;
        for i: 0..4 assert(data[i] == 0xa5);
        data[2] = 0x39;
        free(bytes);
    }
    Overwriting_Allocator.deinit_overwriting_allocator(tracked);
    print("ok\n");
}
