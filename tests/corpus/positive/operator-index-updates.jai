Box :: struct { value: int; }
storage: Box;
trace: int;

base :: () -> *Box { trace = trace*10+1; return *storage; }
key :: () -> int { trace = trace*10+2; return 0; }
operand :: () -> int { trace = trace*10+4; return 3; }

operator [] :: (box: *Box, index: int) -> int {
    trace = trace*10+3;
    return box.value+index+1;
}
operator []= :: (box: *Box, index: int, value: int) {
    trace = trace*10+5;
    box.value = value+index+2;
}
operator *[] :: (box: *Box, index: int) -> *int {
    trace = 99;
    return *box.value;
}

main :: () -> int {
    storage.value = 5;
    base()[key()] += operand();
    if trace != 12345 return 0;
    return storage.value;
}
