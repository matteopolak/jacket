consume :: (value: int) #expand {}
consume :: (value: string) #expand {}

make :: () -> int {
    return 7;
}

main :: () {
    consume(ifx true then make() else 0);
}
