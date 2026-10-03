inner :: () #expand {
    `inner_value := `private_value + 2;
}

outer :: () #expand {
    private_value := 40;
    inner();
    `result := inner_value;
}

main :: () -> int {
    private_value := 100;
    outer();
    return result + private_value - 100;
}
