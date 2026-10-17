// Original Focus lookup reduction. Compile-time and runtime check the public
// procedure's type without invoking an OS working-directory change.
#run,stallable verify();
Expected :: (string) -> bool;

verify :: () -> bool {
    procedure: Expected = set_working_directory;

    return type_of(procedure) == Expected
        && size_of(Version_Info) == 3 * size_of(s32)
        && size_of(File) > 0 && size_of(Plugin_To_Create) > 0
        && size_of(Process_Result) > 0 && trim(" sample ") == "sample";
}

#assert verify();

main :: () -> int {
    if !verify() return 1;
    return 0;
}

#import "Compiler";
#import "Basic";
#import "File";
#import "Metaprogram_Plugins";
#import "Process";
#import "String";
