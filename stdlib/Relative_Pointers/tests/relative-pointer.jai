#import "Basic";
R :: #import "Relative_Pointers";

Bundle :: struct(Word: Type) {
    pointer: R.Relative_Pointer(Word, u8);
    target: [8] u8;
}

Text_Blob :: struct(Word: Type) {
    text: R.Relative_String(Word);
    bytes: [6] u8;
}

exercise :: ($Word: Type) {
    if type_info(R.Relative_Pointer(Word, u8)).name != "Relative_Pointer" exit(2);
    if type_info(R.Relative_String(Word)).name != "Relative_String" exit(3);

    #assert size_of(R.Relative_Pointer(Word, u8)) == size_of(Word);
    #assert size_of(R.Relative_String(Word)) == 2 * size_of(Word);
    storage: Word;
    bytes: [256] u8;
    base := bytes.data + 128;

    for offset: s64.[-127, -1, 0, 1, 127] {
        target := base + offset;
        R.write_relative_pointer(Word, *storage, base, target);
        decoded, reported := R.decode_relative_pointer(cast(s32) size_of(Word), *storage, base);
        if decoded != target || reported != offset exit(4);
    }

    R.write_relative_pointer(Word, *storage, base, null);
    decoded, reported := R.decode_relative_pointer(cast(s32) size_of(Word), *storage, base);
    if storage != 0 || decoded != null || reported != 0 exit(5);

    R.write_relative_pointer(Word, *storage, *storage);
    decoded, reported = R.decode_relative_pointer(cast(s32) size_of(Word), *storage);
    if storage == 0 || decoded != *storage || reported != 0 exit(6);
    if R.decode_relative_pointer_static(Word, *storage) != *storage exit(7);

    R.write_relative_pointer(Word, *storage, null);
    if R.decode_relative_pointer_static(Word, *storage) != null exit(8);

    bundle: Bundle(Word);
    bundle.target[7] = 93;
    R.encode(*bundle.pointer, bundle.target.data + 7);
    if R.decode(*bundle.pointer) != bundle.target.data + 7 || R.decode(*bundle.pointer).* != 93 exit(9);

    relocated := bundle;
    if R.decode(*relocated.pointer) != relocated.target.data + 7 || R.decode(*relocated.pointer).* != 93 exit(10);
    if R.decode(*relocated.pointer) == R.decode(*bundle.pointer) exit(11);

    R.encode(*bundle.pointer, null);
    if R.decode(*bundle.pointer) != null || bundle.pointer.value != 0 exit(12);

    blob: Text_Blob(Word);
    for byte, index: "abcdef" blob.bytes[index] = byte;

    blob.text.count = 3;
    R.write_relative_pointer(Word, *blob.text.data, blob.bytes.data + 1);
    view := R.decode(*blob.text);
    if view != "bcd" || view.data != blob.bytes.data + 1 exit(13);

    moved := blob;

    view = R.decode(*moved.text);
    if view != "bcd" || view.data != moved.bytes.data + 1 exit(14);

    moved.bytes[2] = 90;
    if view != "bZd" || R.decode(*blob.text) != "bcd" exit(15);
}

main :: () {
    #assert !#exists(R.signed_mask);
    #assert !#exists(R.Impl);
    exercise(s8);
    exercise(s16);
    exercise(s32);
    exercise(s64);
    write_string("ok\n");
}
