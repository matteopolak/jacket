// The `data` of a global fixed array is the address of its storage, so a global initializer may hold it.
#import "Basic";

SIZE :: 4;

buffer: [SIZE] s64 = ---;
view: [] s64 = .{ 0, buffer.data };

main :: () {
    view.count = 2;
    view[1] = 5;

    assert(buffer[1] == 5 && view.data == buffer.data);

    print("ok\n");
}
