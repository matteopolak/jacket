// compiler_add_library_search_directory: the library sits in ./lib, which no default search
// path covers; the metaprogram adds it before the first foreign call.
#import "Basic";
#import "Compiler";

#run compiler_add_library_search_directory(tprint("%/lib", #filepath));

probe :: #library "libsearchprobe";
search_probe_value :: () -> s32 #foreign probe;

FROM_COMPILE_TIME :: #run search_probe_value();

main :: () {
    print("% %\n", FROM_COMPILE_TIME, search_probe_value());
}
