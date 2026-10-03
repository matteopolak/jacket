identity :: (value: Type) -> Type #c_call { return value; }
main :: () -> int {
    value: Type = identity(u8);
    callback := identity;
    returned := callback(value);
    if returned != u8 return 1;
    if (cast(*Type_Info)returned).runtime_size != 1 return 2;
    return 42;
}
