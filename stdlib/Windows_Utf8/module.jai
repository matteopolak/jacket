#if OS == .WINDOWS {
    utf8_to_wide :: Impl.utf8_to_wide;
    wide_to_utf8 :: Impl.wide_to_utf8;
}

#scope_file
#if OS == .WINDOWS Impl :: #import "internal/text/windows-utf8";
