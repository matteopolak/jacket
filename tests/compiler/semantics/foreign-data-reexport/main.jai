// No read/write/address request: merely importing a public storage alias.
Facade :: #import "Facade";

#assert type_of(Facade.state) == s32;
main :: () {}
