// rules: compiler.22
// compiler_get_struct_location gives the file and line where a struct is declared
// (n00bmind/reflector reports field-note errors there).
#import "Basic";
#import "Compiler";
#import "String";

Point :: struct {
    x, y: float;
}

LOCATION :: #run -> Source_Code_Location {
    return compiler_get_struct_location(get_current_workspace(), type_info(Point));
};

main :: () {
    assert(ends_with(LOCATION.fully_pathed_filename, "compiler-struct-location.jai"), "file: %", LOCATION.fully_pathed_filename);
    assert(LOCATION.line_number == 8, "line: %", LOCATION.line_number);
    print("ok\n");
}
