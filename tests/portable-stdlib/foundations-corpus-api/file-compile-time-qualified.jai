#import "Basic";
#import "Compiler";
F :: #import "File";

#run {
    options := get_build_options();
    args := options.compile_time_command_line;
    label := ifx args.count then args[0] else "";

    if label == "test" {
        path, ok := F.get_absolute_path(".");
        assert(ok);
        free(path);
        print("selected\n");
    }
}

main :: () {
    print("ok\n");
}
