// Code `bad.jai` calls from another file.
grow :: () -> *int {
    return New(int);
}
