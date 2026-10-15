int original_line_query(int argument) {
    volatile int result = argument + 3;
    if (argument < 0) {
        result -= 2;
    } else {
        result += 5;
    }
    return result;
}
