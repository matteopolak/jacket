#import "Basic";

depth :: () -> s64 {
    count: s64;
    node := context.stack_trace;

    while node {
        count += 1;
        node = node.next;
    }

    return count;
}

main :: () {
    arguments := get_command_line_arguments();

    print("trace %\n", depth());
    print("arguments %\n", arguments.count - 1);
    for index: 1..arguments.count - 1 print("%\n", arguments[index]);
}
