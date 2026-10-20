// An integer argument does not convert to a float parameter without `xx` or a cast.
#import "Basic";
set_opacity :: (level: float) {}
main :: () {
    val: int = 75;
    set_opacity(val);
}
