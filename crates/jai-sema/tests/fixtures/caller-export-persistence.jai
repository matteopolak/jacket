expose :: () #expand {
    `hash := 40;
    `index := 2;
}

main :: () -> int {
    expose();
    return hash + index;
}
