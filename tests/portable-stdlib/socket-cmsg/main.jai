// Reviewed Socket CMSG signatures and actual native SDK macro output.
#import "Basic";
S :: #import "Socket";
F :: #import "File";
P :: #import "String";

main :: () {
    filename := sprint("%/../../../target/portable-socket-cmsg/vectors.bin", P.path_strip_filename(#file));

    defer free(filename);
    data, valid := F.read_entire_file(filename);

    assert(valid && data.count == 16 + 4097 * 32);
    defer free(data);
    header := cast(*u32) data.data;

    assert(header[0] == 0x434d5347 && header[1] == 4097);
    assert(header[2] == size_of(S.cmsghdr) && header[3] == size_of(S.msghdr));
    values := cast(*u64) (data.data + 16);
    storage: [1024] u64;
    message: S.msghdr;
    message.msg_control = storage.data;

    for size: 0..4096 {
        assert(cast(u64) S.CMSG_ALIGN(size) == values[size * 4]);
        assert(cast(u64) S.CMSG_LEN(size) == values[size * 4 + 1]);
        message.msg_controllen = cast(type_of(message.msg_controllen)) size;
        first := S.CMSG_FIRSTHDR(*message);

        assert((first == cast(*S.cmsghdr) storage.data) == (values[size * 4 + 2] != 0));
        if !first continue;

        actual_data: u64 = cast(u64) S.CMSG_DATA(first);
        actual_header: u64 = cast(u64) first;

        assert(actual_data - actual_header == values[size * 4 + 3]);
    }

    message.msg_control = null;
    message.msg_controllen = size_of(S.cmsghdr);
    assert(S.CMSG_FIRSTHDR(*message) == null);
    print("ok\n");
}
