// Reuse the NEW independent platform-io strict UTF-8/UTF-16 implementation.
// Native argv additionally permits an empty terminated string (paths do not).
argument :: (text: string) -> *u16 {
    if text.count == 0 {
        result := cast(*u16) Basic.alloc(2);
        if !result return null;
        result[0] = 0;
        return result;
    }

    return utf16_path(text);
}

#load "../../platform-io/windows-text.jai";

#scope_file
Basic :: #import "Basic";
