#import "Basic";
#import "Compiler";
#import "Hash_Table";
H :: #import "Generate_C_Header";

Point :: struct {
    x: s32;
    y: float64;
    samples: [3] u16;
    next: *Point;
    callback: (value: s32) -> s64 #c_call;
}

main :: () {
    info: H.Header_Info;

    table_add(*info.exported_types, type_info(Point), "Point");
    array_add(*info.ordered_types, type_info(Point));
    procedure: Code_Declaration;
    procedure.name = "point_sum";
    procedure.type = type_info(#type (point: *Point) -> s64 #c_call);
    array_add(*info.exported_procedures, *procedure);
    assert(H.generate_header(*info, sprint("%../../target/developer-tools-point.h", #filepath)));
    print("ok\n");
}
