main :: () {
    value := 3;
    assert(value == 4, "value was %", value);
    print("not reached\n");
}
