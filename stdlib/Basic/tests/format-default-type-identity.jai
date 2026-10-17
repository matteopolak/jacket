#import "Basic";

Container :: struct {
    descriptor: Type;
    count: int;
}

Explicit :: struct {
    descriptor: Type = void;
}

main :: () -> s64 {
    scalar: Type;
    record: Container;
    array: [2] Type;
    explicit: Type = void;
    explicit_record: Explicit;

    // Print only boolean values: a bad default cannot reach the type formatter.
    print("% % % % % %\n", scalar == void, record.descriptor == void,
          array[0] == void, array[1] == void, explicit == void,
          explicit_record.descriptor == void);
    if explicit != void || explicit_record.descriptor != void return 2;
    if scalar != void || record.descriptor != void return 1;
    if array[0] != void || array[1] != void return 1;

    return 0;
}
