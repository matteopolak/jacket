// Volatile accesses read and write in the program's byte order, as plain loads and stores do.
#import "Basic";
#import "Extensions/Embedded";

main :: () {
    word: u32 = 0x0102_0304;

    print("%\n", volatile_load(*word));
    volatile_store(*word, 0x0a0b_0c0d);
    bytes := cast(*u8) *word;

    print("% % %\n", word, bytes[0], volatile_load(cast(*u16) *word));
}
