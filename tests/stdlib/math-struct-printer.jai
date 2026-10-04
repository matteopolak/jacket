// Math's struct printer prints vectors as (x, y, z) and matrices row by row once it is installed in the print style.
#import "Basic";
#import "Math";

main :: () -> s64 {
    v3 := Vector3.{1, 2, 3};
    m2 := Matrix2.{_11=1, _12=2, _21=3, _22=4};
    if tprint("%", v3) == "(1, 2, 3)" return 1;

    context.print_style.struct_printer = math_struct_printer;
    if tprint("%", v3) != "(1, 2, 3)" return 2;
    if tprint("%", Vector2.{0.5, -2}) != "(0.5, -2)" return 3;
    if tprint("%", m2) != "(1, 2), (3, 4)" return 4;
    if tprint("%", Matrix3_Identity) != "(1, 0, 0), (0, 1, 0), (0, 0, 1)" return 5;
    if tprint("%", Matrix4x3_Identity) != "(1, 0, 0, 0), (0, 1, 0, 0), (0, 0, 1, 0)" return 6;

    if m2.floats.count != 4 || m2.floats[2] != 3 return 7;
    #assert Matrix4.Row_Type == Vector4 && Matrix4x3.Column_Type == Vector3;
    #assert Matrix2.IsMatrixFromMathModule;
    c := Complex.{0, 1};
    squared := c * c;
    if squared.x != -1 || squared.y != 0 return 8;
    #assert size_of(type_of(PI)) == 4 && size_of(type_of(PI64)) == 8;
    return 0;
}
