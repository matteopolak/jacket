#import "Basic";

main :: () {
    info: Type_Info_Float;
    info.type = .FLOAT;
    info.runtime_size = 16;
    storage: [16] u8;
    value := Any.{ *info, *storage };

    print("%\n", value);
    print("unexpected continuation\n");
}
