#import "Basic";
#import "Compiler";
#import "String"; // jailint: allow(unused_import) the import is part of the rejected program
#load "file-private-import.jai";

#run {
    options := get_build_options();
    args := options.compile_time_command_line;
    label := ifx args.count then args[0] else "";

    if label == "test" {
        path, ok := get_absolute_path(".");
        assert(ok);
        free(path);
    }
}

main :: () {}
