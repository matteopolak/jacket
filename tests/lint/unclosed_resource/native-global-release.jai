Native :: #system_library "c";
acquire_stream :: (path:*u8,mode:*u8)->*void #foreign Native "fopen";
close_streams :: ()->s32 #foreign Native "fcloseall";
acquire_fd :: (path:*u8,flags:s32)->s32 #foreign Native "open";
close_fd :: (fd:s32)->s32 #foreign Native "close";
External :: #library "unknown_callback";
callback :: () #foreign External;

all_streams_closed :: (path:*u8,mode:*u8) {
    h:=acquire_stream(path,mode);
    close_streams();
    if h!=null {}
}
constant_may_close :: (path:*u8) {
    h:=acquire_fd(path,0);
    close_fd(3);
    if h>=0 {}
}
callback_may_close :: (path:*u8,mode:*u8) {
    h:=acquire_stream(path,mode);
    callback();
    if h!=null {}
}

deferred_streams_closed :: (path:*u8,mode:*u8) {
    defer close_streams();
    h:=acquire_stream(path,mode);
    if h!=null {}
}
repeated_site_unknown :: (path:*u8,mode:*u8,count:s64) {
    for 0..count {
        h:=acquire_stream(path,mode);
        if h!=null {}
    }
}

duplicate :: (path:*u8,mode:*u8)->*void,*void {
    h:=acquire_stream(path,mode);
    return h,h;
}
alias_returned :: (path:*u8,mode:*u8)->*void {
    first,second:=duplicate(path,mode);
    if second!=null {}
    return first;
}
