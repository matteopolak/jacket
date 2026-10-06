// rules: typeval.8
// A variable holding a Type cannot be used as a declaration type.
main :: () {
    t := int;
    v: t;
}
