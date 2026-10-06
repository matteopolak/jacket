// `#poke_name Module Name` makes the module see the importer's declaration itself,
// so both sides agree on the type (modules/Poke_Target).
#import "Basic";
Poke_Target :: #import "Poke_Target";
Mode :: enum {
    a;
    b;
}
#poke_name Poke_Target Mode;

main :: () {
    m: Mode = .a;
    Poke_Target.set_mode(m);
    assert(Poke_Target.current == .a);
}
