Resource :: struct { token: s64; }
open_resource :: (target: *Resource, other: *Resource) {} @Opens

reordered_open :: () {
    a: Resource;
    b: Resource;
    open_resource(other = *a, target = *b);
}

passed_on_as_other :: () -> Resource {
    a: Resource;
    b: Resource;
    open_resource(*a, *b);
    open_resource(target = *b, other = *a);
    return b;
}
main :: () {}
