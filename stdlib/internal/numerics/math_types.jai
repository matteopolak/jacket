Vector2 :: struct {
    x, y: float;
    #overlay(x) component: [2] float;
}

Vector3 :: struct {
    x, y, z: float;
    #overlay(x) component: [3] float;
    #overlay(x) xy: Vector2;
    #overlay(y) yz: Vector2;
}

Vector4 :: struct {
    x, y, z, w: float;
    #overlay(x) component: [4] float;
    #overlay(x) xy: Vector2;
    #overlay(y) yz: Vector2;
    #overlay(z) zw: Vector2;
    #overlay(x) xyz: Vector3;
    #overlay(y) yzw: Vector3;
}

Quaternion :: struct {
    x, y, z: float;
    w := 1.0;
    #overlay(x) xyz: Vector3;
    #overlay(x) component: [4] float;
}

Complex :: #type,isa Vector2;

Plane3 :: struct {
    a, b, c, d: float;
    #overlay(a) #as vector4: Vector4;
    #overlay(a) normal: Vector3;
}

AnyVector2 :: struct {
    x, y: float;
}

Matrix2 :: struct {
    _11, _12, _21, _22: float;
    #overlay(_11) v: [2] Vector2;
    #overlay(_11) coef: [2][2] float;
    #overlay(_11) floats: [4] float;
    Row_Type :: Vector2;
    Column_Type :: Vector2;
    IsMatrixFromMathModule :: true;
}

Matrix2_Identity :: Matrix2.{ _11 = 1, _22 = 1 };

AnyMatrix2 :: struct {
    _11, _12, _21, _22: float;
}

Matrix3 :: struct {
    _11, _12, _13, _21, _22, _23, _31, _32, _33: float;
    #overlay(_11) v: [3] Vector3;
    #overlay(_11) coef: [3][3] float;
    #overlay(_11) floats: [9] float;
    Row_Type :: Vector3;
    Column_Type :: Vector3;
    IsMatrixFromMathModule :: true;
}

Matrix3_Identity :: Matrix3.{ _11 = 1, _22 = 1, _33 = 1 };

AnyMatrix3 :: struct {
    _11, _12, _13, _21, _22, _23, _31, _32, _33: float;
}

Matrix4x3 :: struct {
    _11, _12, _13, _14, _21, _22, _23, _24, _31, _32, _33, _34: float;
    #overlay(_11) v: [3] Vector4;
    #overlay(_11) coef: [3][4] float;
    #overlay(_11) floats: [12] float;
    Row_Type :: Vector4;
    Column_Type :: Vector3;
    IsMatrixFromMathModule :: true;
}

Matrix4x3_Identity :: Matrix4x3.{ _11 = 1, _22 = 1, _33 = 1 };

AnyMatrix4x3 :: struct {
    _11, _12, _13, _14, _21, _22, _23, _24, _31, _32, _33, _34: float;
}

Matrix4 :: struct {
    _11, _12, _13, _14, _21, _22, _23, _24, _31, _32, _33, _34, _41, _42, _43, _44: float;
    #overlay(_11) v: [4] Vector4;
    #overlay(_11) coef: [4][4] float;
    #overlay(_11) floats: [16] float;
    #overlay(_11) mat4x3: Matrix4x3;
    Row_Type :: Vector4;
    Column_Type :: Vector4;
    IsMatrixFromMathModule :: true;
}

Matrix4_Identity :: Matrix4.{ _11 = 1, _22 = 1, _33 = 1, _44 = 1 };

AnyMatrix :: struct {
    IsMatrixFromMathModule :: true;
}
