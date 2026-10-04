Basic :: #import "Basic";
main :: () -> s64 {
    text: string = Basic.make_leak_report();
    if text != "Memory debugging disabled\n" return 1;
    Basic.log_leak_report();
    return 0;
}
