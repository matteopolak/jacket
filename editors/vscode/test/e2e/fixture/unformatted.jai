main :: () {
    x := 1;
    bump(*x);
}

bump :: (x: *int) {
    x.* += 1;
}


