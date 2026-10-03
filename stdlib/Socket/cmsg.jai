#if OS == .MACOS || OS == .LINUX || OS == .ANDROID {
    CMSG_FIRSTHDR :: Cmsg.CMSG_FIRSTHDR;
    CMSG_DATA :: Cmsg.CMSG_DATA;
    CMSG_ALIGN :: Cmsg.CMSG_ALIGN;
    CMSG_LEN :: Cmsg.CMSG_LEN;
    CMSG_SIZE :: Cmsg.CMSG_SIZE;
}

#scope_file
#if OS == .MACOS || OS == .LINUX || OS == .ANDROID Cmsg :: #import "internal/net/socket-cmsg";
