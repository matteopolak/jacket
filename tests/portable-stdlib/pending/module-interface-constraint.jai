// Must reject even though touch does not invoke the constrained interface.
Broken :: struct {}

M :: #import "interface-parameter-module"(HOOKS = Broken);

main :: () {
    result := M.touch();
}
