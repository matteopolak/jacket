// Reviewed public ancillary signatures; independent layout and extent safety.
#import "Basic";
S :: #import "Socket";

main :: () {
    #if OS == .MACOS || OS == .LINUX || OS == .ANDROID {
        bytes: [8] u64;
        header := cast(*S.cmsghdr) bytes.data;
        length := size_of(S.cmsghdr);

        assert(S.CMSG_DATA(header) == cast(*u8) header + length);
        assert(S.CMSG_LEN(0) == length);
        assert(S.CMSG_LEN(0xffffffff - length) == 0xffffffff);
        #if OS == .MACOS boundary := 4;
        else boundary := 8;
        maximum := 0x7fffffffffffffff - (boundary - 1);

        assert(S.CMSG_ALIGN(maximum) == maximum);

        for size: 0..1024 {
            aligned := S.CMSG_ALIGN(size);
            assert(aligned >= size && aligned - size < boundary && aligned % boundary == 0);
            assert(S.CMSG_SIZE(size) == length + aligned);
        }

        maximum_payload := ((0xffffffff - length) / boundary) * boundary;

        assert(S.CMSG_SIZE(maximum_payload) == length + maximum_payload);

        message: S.msghdr;
        message.msg_control = header;
        message.msg_controllen = cast(type_of(message.msg_controllen)) (length - 1);
        assert(S.CMSG_FIRSTHDR(*message) == null);
        message.msg_controllen += 1;
        assert(S.CMSG_FIRSTHDR(*message) == header);
    } else {
        #assert !#exists(S.CMSG_DATA) && !#exists(S.CMSG_LEN);
    }

    print("ok\n");
}
