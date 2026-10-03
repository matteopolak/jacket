Vector :: struct { x, y, z: float; }
Rotation :: struct { x, y, z, w: float; }

operator + :: (a: Vector, b: Vector) -> Vector {
    return Vector.{a.x+b.x, a.y+b.y, a.z+b.z};
}
operator * :: (value: Vector, scale: float) -> Vector #symmetric {
    return Vector.{value.x*scale, value.y*scale, value.z*scale};
}
operator * :: (value: Rotation, scale: float) -> Rotation {
    return Rotation.{value.x*scale, value.y*scale, value.z*scale, value.w*scale};
}

main :: () -> int {
    vector := 2.0*Vector.{1,2,3}+Vector.{4,5,6};
    rotation := Rotation.{1,2,3,4}*3.0;
    return cast(int)(vector.z+rotation.w);
}
