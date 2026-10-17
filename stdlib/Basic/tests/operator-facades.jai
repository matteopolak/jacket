#import "Basic";
#import "Math";
#import "Bit_Array";

B :: #import "Basic";
B_Again :: #import "Basic";
M :: #import "Math";
M_Again :: #import "Math";
Bits :: #import "Bit_Array";

main :: () {
    assert((cast(*Type_Info_Struct) type_info(S128)).name == "S128");
    assert((cast(*Type_Info_Struct) type_info(Vector2)).name == "Vector2");
    assert((cast(*Type_Info_Struct) type_info(Bit_Array)).name == "Bit_Array");

    a := seconds_to_apollo(9);
    b := seconds_to_apollo(4);
    sum := a + b;
    difference := a - b;

    assert(type_of(sum) == Apollo_Time && type_of(difference) == Apollo_Time);
    assert(sum.low == seconds_to_apollo(13).low && sum.high == seconds_to_apollo(13).high);
    assert(difference.low == seconds_to_apollo(5).low && difference.high == seconds_to_apollo(5).high);

    add128: (S128, S128) -> S128 = B.operator +;
    signed_subtract: (S128, S128) -> S128 = B.operator -;
    unsigned_subtract: (U128, U128) -> U128 = B.operator -;
    add128_again: (S128, S128) -> S128 = B_Again.operator +;

    assert(add128 == add128_again);
    named_sum := B.operator +(b = S128.{2, 0}, a = S128.{3, 0});
    carried := add128(.{0xffffffffffffffff, 0}, .{1, 0});
    borrowed := signed_subtract(.{0, 1}, .{1, 0});
    unsigned_borrowed := unsigned_subtract(.{0, 1}, .{1, 0});

    assert(carried.low == 0 && carried.high == 1);
    assert(borrowed.low == 0xffffffffffffffff && borrowed.high == 0);
    assert(unsigned_borrowed.low == 0xffffffffffffffff && unsigned_borrowed.high == 0);
    assert(named_sum.low == 5 && named_sum.high == 0);

    add_vector: (Vector2, Vector2) -> Vector2 = M.operator +;
    negate_vector: (Vector2) -> Vector2 = M.operator -;
    scale_vector: (Vector2, float) -> Vector2 = M.operator *;
    divide_vector: (float, Vector2) -> Vector2 = M.operator /;
    add_vector_again: (Vector2, Vector2) -> Vector2 = M_Again.operator +;

    assert(add_vector == add_vector_again);
    vector := add_vector(.{1, 2}, .{3, 4});
    negated := negate_vector(vector);
    scaled := scale_vector(vector, 2);
    divided := divide_vector(24, vector);

    assert(vector.x == 4 && vector.y == 6);
    assert(negated.x == -4 && negated.y == -6);
    assert(scaled.x == 8 && scaled.y == 12);
    assert(divided.x == 6 && divided.y == 4);
    symmetric := 2 * vector;
    component_product := M.operator *(a = vector, b = Vector2.{2, 3});
    component_quotient := M.operator /(a = vector, b = Vector2.{2, 3});
    scalar_quotient := M.operator /(a = vector, k = 2);
    subtracted := M.operator -(a = vector, b = Vector2.{1, 2});

    assert(symmetric.x == 8 && symmetric.y == 12);
    assert(component_product.x == 8 && component_product.y == 18);
    assert(component_quotient.x == 2 && component_quotient.y == 2);
    assert(scalar_quotient.x == 2 && scalar_quotient.y == 3);
    assert(subtracted.x == 3 && subtracted.y == 4);

    bits: Bit_Array;

    init_bit_array(*bits, 65);
    defer deinit(*bits);
    bits[64] = true;
    get_bit: (Bit_Array, int) -> bool = Bits.operator [];
    put_bit: (*Bit_Array, int, bool) = Bits.operator []=;

    assert(get_bit(bits, 64) && !get_bit(bits, 0));
    put_bit(*bits, 0, true);
    assert(bits[0] && bits[64]);
    print("operator facades ok\n");
}
