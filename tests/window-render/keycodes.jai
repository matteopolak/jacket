#import "Basic";
I :: #import "Input";

main :: () {
    #if OS == .LINUX {
        assert(I.get_key_code(0xff08) == .BACKSPACE);
        assert(I.get_key_code(0xff52) == .ARROW_UP);
        assert(I.get_key_code(0xffe2) == .SHIFT);
        assert(I.get_key_code(0xffec) == .CMD);
        assert(I.get_key_code(0xffd5) == .F24);
        assert(I.get_key_code(0x1008ff12) == .UNKNOWN);
        assert(cast(u32) I.get_key_code(#char "a") == #char "A");
        for i: 0..23 assert(cast(u32) I.get_key_code(0xffbe + cast(u64) i) == cast(u32) I.Key_Code.F1 + cast(u32) i);
    } else #if OS == .WINDOWS {
        for i: 0..23 assert(cast(u32) I.get_key_code(0x70 + cast(u64) i) == cast(u32) I.Key_Code.F1 + cast(u32) i);
        assert(I.get_key_code(0x26) == .ARROW_UP);
        assert(I.get_vk(.ARROW_UP) == 0x26);
        assert(I.get_vk(.F24) == 0x87);
    }

    print("ok\n");
}
