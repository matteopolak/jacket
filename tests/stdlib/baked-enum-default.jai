// A baked parameter's default takes the parameter's declared type (`.X` needs it);
// System.get_number_of_processors relies on it.
#import "Basic";
#import "System";

Mode :: enum {
    slow;
    fast;
}
speed :: ($mode: Mode = .fast) -> int {
    return ifx mode == .fast then 2 else 1;
}

main :: () {
    assert(speed() == 2 && speed(.slow) == 1);
    n := get_number_of_processors();
    assert(type_of(n) == s32 && n > 0);
}

Info :: struct {
    background := false;
    translucent := false;
}
flags :: ($info := Info.{}) -> int {
    return cast(int) info.background + 2 * cast(int) info.translucent;
}
#run assert(flags() == 0 && flags(info = .{ translucent = true }) == 2 && flags(.{ background = true }) == 1);
