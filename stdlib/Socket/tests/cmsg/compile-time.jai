S :: #import "Socket";

#assert OS == .MACOS || OS == .LINUX || OS == .ANDROID;
#assert #run verify();

verify :: () -> bool {
    storage: [32] u64;
    message: S.msghdr;
    message.msg_control = storage.data;

    for size: 0..255 {
        message.msg_controllen = cast(type_of(message.msg_controllen)) size;
        pointer := S.CMSG_FIRSTHDR(*message);

        if size < size_of(S.cmsghdr) {
            if pointer != null return false;
        } else {
            if pointer != cast(*S.cmsghdr) storage.data return false;
            bytes: u64 = cast(u64) S.CMSG_DATA(pointer);
            base: u64 = cast(u64) pointer;
            if bytes - base != size_of(S.cmsghdr) return false;
        }

        #if OS == .MACOS expected := ((size + 3) / 4) * 4;
        else expected := ((size + 7) / 8) * 8;
        if S.CMSG_ALIGN(size) != expected || S.CMSG_LEN(size) != size_of(S.cmsghdr) + size return false;
    }

    return true;
}

main :: () {}
