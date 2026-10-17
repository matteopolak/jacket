#import "Basic";
#import "Compiler";
#load "anonymous-file.jai";

#run {
    options := get_build_options();
    args := options.compile_time_command_line;
    label := ifx args.count then args[0] else "";

    if label == "test" {
        path, ok := get_absolute_path(".");
        assert(ok && path.count > 0);
        free(path);
        print("selected\n");
    }
}

main :: () {
    print("ok\n");
}
