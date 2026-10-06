// rules: code.14
// A plain `#insert c` checks the Code where it was written, so the macro's `it` is not
// visible to it; `#insert,scope()` would be needed.
repeat_twice :: (c: Code) #expand {
    for 0..1 {
        #insert c;
    }
}

main :: () {
    sum := 0;
    repeat_twice(#code sum += it);
}
