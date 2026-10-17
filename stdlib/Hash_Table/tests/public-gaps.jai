using B :: #import "Basic";
H :: #import "Hash_Table";
using M :: #import "Math";

main :: () {
    pointer := B.New(int);

    B.free(pointer);
    text := B.copy_string("owned");

    B.free(text);
    table: H.Table(int, int);

    H.table_add(*table, 7, 9);
    found := H.table_find_pointer(*table, 7);

    B.assert(found != null && found.* == 9);
    H.deinit(*table);
    vector := M.Vector2.{ 2, 4 };
    scaled := vector * 2.0;

    B.assert(scaled.x == 4 && scaled.y == 8);
    mirrored := 2.0 * vector;

    B.assert(mirrored.x == scaled.x && mirrored.y == scaled.y);
    divided := 8.0 / vector;

    B.assert(divided.x == 4 && divided.y == 2);
    quotient := vector / 2.0;

    B.assert(quotient.x == 1 && quotient.y == 2);
    product := vector * M.Vector2.{ 3, 5 };

    B.assert(product.x == 6 && product.y == 20);
    ratios := M.Vector2.{ 6, 20 } / vector;

    B.assert(ratios.x == 3 && ratios.y == 5);
    zeroes := M.Vector2.{ -0.0, 0.0 };
    infinity := 1.0 / zeroes;

    B.assert((cast,force(u32) infinity.x) == 0xff800000);
    B.assert((cast,force(u32) infinity.y) == 0x7f800000);
    B.assert((cast(*Type_Info_Struct) type_info(M.Vector2)).name == "Vector2");
    sum := B.S128.{ 0xffffffffffffffff, 1 } + B.S128.{ 1, 2 };

    B.assert(sum.low == 0 && sum.high == 4);
    context.print_style.default_format_int.base = 16;
    output := B.sprint("%", 255);

    B.assert(output == "ff");
    B.free(output);
    builder: B.String_Builder;
    handled := M.math_struct_printer(*builder, vector, null);

    B.assert(handled);
    expected := B.builder_to_string(*builder);

    context.print_style.struct_printer = M.math_struct_printer;
    output = B.sprint("%", vector);
    B.assert(output == expected);
    B.free(output);
    B.free(expected);
    context.print_style.struct_printer = null;

    B.print("ok\n");
}
