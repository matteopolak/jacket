#import "Basic";
E :: #import "executable_formats";

main :: () {
    assert(size_of(E.relocation_info) == 8 && size_of(E.scattered_relocation_info) == 8);
    assert(size_of(E.dylib_reference) == 4 && size_of(E.twolevel_hint) == 4);
    r: E.relocation_info;
    r.r_address = -19;
    r.__bitfield = 0xffffffff;
    E.set_r_symbolnum(*r, 0x12345678);
    assert(E.get_r_symbolnum(r) == 0x345678 && r.__bitfield == 0xff345678);
    E.set_r_pcrel(*r, false);
    E.set_r_length(*r, 1);
    E.set_r_extern(*r, false);
    E.set_r_type(*r, 2);
    assert(r.r_address == -19 && E.get_r_length(r) == 1 && E.get_r_type(r) == 2);
    assert(!E.get_r_pcrel(r) && !E.get_r_extern(r));
    s: E.scattered_relocation_info;
    s.r_value = -7;
    E.set_r_address(*s, 0xffffffff);
    E.set_r_type(*s, 3);
    E.set_r_length(*s, 2);
    E.set_r_pcrel(*s, true);
    E.set_r_scattered(*s, true);
    assert(E.get_r_address(s) == 0xffffff && E.get_r_type(s) == 3 && E.get_r_length(s) == 2);
    assert(E.get_r_pcrel(s) && E.get_r_scattered(s) && s.r_value == -7);
    d: E.dylib_reference;

    E.set_isym(*d, 0xffffffff);
    E.set_flags(*d, 0x123);
    assert(E.get_isym(d) == 0xffffff && E.get_flags(d) == 0x23);
    h: E.twolevel_hint;

    E.set_isub_image(*h, 0x123);
    E.set_itoc(*h, 0xffffffff);
    assert(E.get_isub_image(h) == 0x23 && E.get_itoc(h) == 0xffffff);
    print("ok\n");
}
