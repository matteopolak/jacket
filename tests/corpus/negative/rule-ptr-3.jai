// rules: ptr.3
// `remove` outside a for loop over an array is rejected.
main :: () {
    count := 3;
    while count > 0 {
        remove count;
        count -= 1;
    }
}
