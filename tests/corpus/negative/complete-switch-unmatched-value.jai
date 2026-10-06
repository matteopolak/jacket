// A `#complete` switch without a default stops the program when its value is none of the enum's
// members, instead of running no case.
Color :: enum { RED; GREEN; }
work :: () {
    c := cast(Color) 7;
    if #complete c == {
        case .RED;
        case .GREEN;
    }
}
#run work();
