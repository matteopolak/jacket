#import "Basic";

main :: () {
    args := get_command_line_arguments();

    assert(args.count == 4);
    assert(args[1] == "雪😀");
    assert(args[2] == "");
    assert(args[3] == "quote\" and slash\\");
    print("ok\n");
}
