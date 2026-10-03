Box :: struct {
    value: int;
}

apply :: (amount: int) #expand {
    cursor := 1;
    `destination.value = `items[cursor] + amount;
}

main :: () -> int {
    items: [2]int = .[100, 40];
    cursor := 0;
    destination: Box;
    apply(2);
    return destination.value + cursor;
}
