#assert OS == .MACOS || OS == .LINUX || OS == .ANDROID;

CMSG_FIRSTHDR :: (mhdr: *msghdr) -> *cmsghdr {
    if mhdr.msg_controllen < size_of(cmsghdr) return null;
    return cast(*cmsghdr) mhdr.msg_control;
}

CMSG_DATA :: (cmsg: *cmsghdr) -> *u8 {
    return cast(*u8) cmsg + CMSG_ALIGN(size_of(cmsghdr));
}

CMSG_ALIGN :: (size: int) -> int {
    B.assert(size >= 0 && size <= 0x7fffffffffffffff - (BOUNDARY - 1), "ancillary size cannot be aligned");
    return (size + BOUNDARY - 1) & ~(BOUNDARY - 1);
}

CMSG_LEN :: (size: int) -> u32 {
    header := CMSG_ALIGN(size_of(cmsghdr));
    B.assert(size >= 0 && size <= 0xffffffff - header, "ancillary length exceeds u32");
    return cast(u32) (header + size);
}

CMSG_SIZE :: (size: int) -> u32 {
    header := size_of(cmsghdr);
    B.assert(size >= 0 && size <= 0xffffffff - header - (BOUNDARY - 1), "ancillary space exceeds u32");
    return cast(u32) (header + CMSG_ALIGN(size));
}

#scope_file
B :: #import "Basic";

#if OS == .MACOS BOUNDARY :: 4;
else BOUNDARY :: size_of(u64);
