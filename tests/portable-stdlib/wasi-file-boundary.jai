// Actual caller-file control: module, data and receipt paths must be distinct.
#import "Basic";
F :: #import "File";

main :: () {
    args := get_command_line_arguments();

    assert(args.count == 3);
    assert(args[0] != args[1]);
    assert(args[2] == "boundary-token");
    print("argc=%\n", args.count);
    for arg, index: args print("argv[%]=%\n", index, arg);

    assert(F.write_entire_file(args[1], "boundary-file-payload\n"));
    data, ok := F.read_entire_file(args[1]);

    assert(ok && data == "boundary-file-payload\n");
    free(data);
    print("ok\n");
}
