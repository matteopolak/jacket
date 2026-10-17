// The overwrite allocator poisons fresh bytes and is torn down after its clients.
#import "Basic";
Overwriting_Allocator :: #import "Overwriting_Allocator";

main :: () {
    tracked := Overwriting_Allocator.get_overwriting_allocator(overwrite_byte = 0xa5);

    {
        push_allocator(tracked);
        bytes := alloc(5);

        assert(bytes != null);
        data := cast(*u8) bytes;
        for i: 0..4 assert(data[i] == 0xa5);

        data[2] = 0x39;
        resized := realloc(bytes, 8, 5);

        assert(resized != null);
        grown := cast(*u8) resized;

        assert(grown[0] == 0xa5 && grown[1] == 0xa5 && grown[2] == 0x39);
        assert(grown[3] == 0xa5 && grown[4] == 0xa5);
        for i: 5..7 assert(grown[i] == 0xa5);

        free(resized);
    }

    Overwriting_Allocator.deinit_overwriting_allocator(tracked);
    print("ok\n");
}
