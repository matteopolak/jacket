#import "Basic";
W :: #import "Windows";
F :: #import "File";

main :: () {
    args := get_command_line_arguments();

    assert(args.count >= 2);

    if args[1] == "emit" {
        assert(args.count == 3);
        write_string(args[2]);
        write_string("stderr\n", true);
    } else if args[1] == "echo" {
        input := F.File.{ W.GetStdHandle(-10) };
        buffer: [256] u8;

        while true {
            ok, count := F.file_read(input, buffer.data, buffer.count);
            assert(ok, "stdin read failed");

            if count == 0 break;

            write_string(string.{ count, buffer.data });
        }
    } else if args[1] == "zero" {
        written: u32;
        assert(W.WriteFile(W.GetStdHandle(-11), null, 0, *written, null) != 0);
        W.Sleep(10);
        write_string("after-zero");
    } else if args[1] == "sleep" W.Sleep(10000);
    else if args[1] == "exit-high" W.ExitProcess(0xffffffff);
    else if args[1] == "bulk" {
        block: [1024] u8;
        for *byte: block byte.* = 120;

        for 0..255 {
            write_string(cast(string) block);
            write_string(cast(string) block, true);
        }
    } else if args[1] == "marker" {
        assert(args.count == 3);
        W.Sleep(500);
        assert(F.write_entire_file(args[2], "survived"));
    } else assert(false, "unknown child operation");
}
