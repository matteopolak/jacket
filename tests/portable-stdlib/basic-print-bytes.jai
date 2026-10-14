#import "Basic";

main :: () {
    bytes: [4] u8 = .[0, 1, 127, 255];

    print_bytes(bytes.data, bytes.count);
    print("\n");
    print_bytes(bytes.data, 2, format_string = "[%]");
    print_bytes(null, 0);
    print("\n");
}
