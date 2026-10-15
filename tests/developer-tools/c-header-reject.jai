#import "Basic";
#import "Compiler";
#import "File";
H :: #import "Generate_C_Header";

main :: () {
    path := tprint("%../../target/developer-tools-reject.h", #filepath);

    assert(write_entire_file(path, "preserve existing output"));
    declaration: Code_Declaration;
    declaration.name = "unsafe_abi";
    declaration.type = type_info(#type (value: s64) -> s64);
    info: H.Header_Info;

    array_add(*info.exported_procedures, *declaration);
    assert(!H.generate_header(*info, path));
    data, ok := read_entire_file(path);

    defer free(data.data);
    assert(ok && data == "preserve existing output");
    print("ok\n");
}
