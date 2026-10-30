#assert OS == .MACOS || OS == .LINUX || OS == .ANDROID;

// Returns the first control message of `message`, or null when its control buffer is smaller than one header.
//
// `message` must be a live `msghdr` whose `msg_control` and `msg_controllen` describe the buffer. The result points
// into that buffer and is not a separate allocation. Control messages carry ancillary data such as descriptors
// (`SCM_RIGHTS`) with `sendmsg` and `recvmsg`, and exist on Unix only.
//
// ```jai,only(linux, macos)
// message: msghdr;
// assert(CMSG_FIRSTHDR(*message) == null);
//
// buffer: [64] u8;
// message.msg_control = buffer.data;
// message.msg_controllen = xx buffer.count;
// assert(cast(*u8) CMSG_FIRSTHDR(*message) == buffer.data);
// ```
CMSG_FIRSTHDR :: (mhdr: *msghdr) -> *cmsghdr {
    if mhdr.msg_controllen < size_of(cmsghdr) return null;
    return cast(*cmsghdr) mhdr.msg_control;
}

// Returns the payload of control message `cmsg`: the first byte after its header, rounded up to the platform's alignment.
//
// The payload is `cmsg_len` minus the header's size bytes long. See `CMSG_FIRSTHDR` for where `cmsg` comes from.
CMSG_DATA :: (cmsg: *cmsghdr) -> *u8 {
    return cast(*u8) cmsg + CMSG_ALIGN(size_of(cmsghdr));
}

// Rounds `size` up to the alignment of control message data: 4 bytes on macOS, 8 elsewhere. A negative size, or one too
// large to round, fails an assertion.
//
// ```jai,only(linux, macos)
// assert(CMSG_ALIGN(0) == 0);
// assert(CMSG_ALIGN(1) % 4 == 0);
// assert(CMSG_ALIGN(CMSG_ALIGN(5)) == CMSG_ALIGN(5));
// ```
CMSG_ALIGN :: (size: int) -> int {
    B.assert(size >= 0 && size <= 0x7fffffffffffffff - (BOUNDARY - 1), "ancillary size cannot be aligned");
    return (size + BOUNDARY - 1) & ~(BOUNDARY - 1);
}

// The value for a control message's `cmsg_len` when its payload is `size` bytes: the aligned header plus `size`, without
// padding after the payload. Fails an assertion for a negative size or one that does not fit in a `u32`.
//
// ```jai,only(linux, macos)
// // A descriptor-passing message carries one s32.
// assert(CMSG_LEN(size_of(s32)) == cast(u32) (CMSG_ALIGN(size_of(cmsghdr)) + size_of(s32)));
// ```
CMSG_LEN :: (size: int) -> u32 {
    header := CMSG_ALIGN(size_of(cmsghdr));
    B.assert(size >= 0 && size <= 0xffffffff - header, "ancillary length exceeds u32");
    return cast(u32) (header + size);
}

// The buffer space one control message with a `size`-byte payload needs: the aligned header plus the aligned payload. Use
// it for `msg_controllen` and for sizing the buffer. Fails an assertion for a negative size or one that does not fit.
//
// ```jai,only(linux, macos)
// assert(CMSG_SIZE(0) >= cast(u32) size_of(cmsghdr));
// assert(CMSG_SIZE(size_of(s32)) >= CMSG_LEN(size_of(s32)));
// ```
CMSG_SIZE :: (size: int) -> u32 {
    header := size_of(cmsghdr);
    B.assert(size >= 0 && size <= 0xffffffff - header - (BOUNDARY - 1), "ancillary space exceeds u32");
    return cast(u32) (header + CMSG_ALIGN(size));
}

#scope_file
B :: #import "Basic";

#if OS == .MACOS BOUNDARY :: 4;
else BOUNDARY :: size_of(u64);
