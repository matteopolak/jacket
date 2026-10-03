#module_parameters(ASSERT_ON_OVERFLOW := false);
Float16 :: Impl.Float16;
Vector2_Float16 :: Impl.Vector2_Float16;
Vector3_Float16 :: Impl.Vector3_Float16;
Vector4_Float16 :: Impl.Vector4_Float16;
Matrix3_Float16 :: Impl.Matrix3_Float16;
half_to_float_fast4 :: Impl.half_to_float_fast4;
float_to_half_fast3 :: Impl.float_to_half_fast3;
float16_to_float32 :: Impl.float16_to_float32;
float32_to_float16 :: Impl.float32_to_float16;
matrix3_float16 :: Impl.matrix3_float16;
matrix3_float32 :: Impl.matrix3_float32;
vector2_float16 :: Impl.vector2_float16;
vector2_float32 :: Impl.vector2_float32;
vector3_float16 :: Impl.vector3_float16;
vector3_float32 :: Impl.vector3_float32;
vector4_float16 :: Impl.vector4_float16;

#scope_file
Impl :: #import "internal/numerics/half"(ASSERT_ON_OVERFLOW = ASSERT_ON_OVERFLOW);
