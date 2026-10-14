#import "Basic";
M :: #import "md5";
H :: #import "Hash";

main :: () {
    empty := M.md5("");

    defer free(empty);
    abc := M.md5("abc");

    defer free(abc);
    message := M.md5("message digest");

    defer free(message);
    assert(empty == "d41d8cd98f00b204e9800998ecf8427e");
    assert(abc == "900150983cd24fb0d6963f7d28e17f72");
    assert(message == "f96b697d7cb7938d525a2f31aaf161d0");
    assert(H.fnv1a_hash("hello".data, 5) == 0xa430d84680aabd0b);
    assert(H.sdbm_hash(null, 0) == 5381);
    assert(H.djb2_hash("hello") == 261238937);
    print("ok\n");
}
