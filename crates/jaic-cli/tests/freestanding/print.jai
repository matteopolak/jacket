main :: () {
    print("hello % %\n", 42, "world");
    text := tprint("%-%", 1, 2);

    print("%\n", text);
    reset_temporary_storage();
    print("%\n", 3.5);
    print("%\n", cast(u64) 0xdeadbeefcafe);
}
