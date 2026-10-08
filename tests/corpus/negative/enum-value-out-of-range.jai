// An enum value must fit the enum's declared integer type.
Small :: enum u8 {
    TOO_BIG :: 256;
}

main :: () {
    s := Small.TOO_BIG;
}
