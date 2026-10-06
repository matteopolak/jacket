// A procedure declared inside another body is checked though nothing calls it.
main :: () {
    nested :: () {
        missing_in_nested_proc();
    }
}
