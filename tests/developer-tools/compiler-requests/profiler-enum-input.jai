Tag :: enum {
    ONE;
}

take :: (tag: Tag) -> s64 {
    return cast(s64) tag;
}

enum_call :: () -> s64 {
    return take(.ONE);
}

main :: () {
    enum_call();
}
