#if OS == .WINDOWS {
    Native :: #system_library "kernel32";
    acquire_file :: (path:*u16, access:u32, sharing:u32, security:*void, disposition:u32, flags:u32, template:*void)->*void #foreign Native "CreateFileW";
    close_file :: (handle:*void)->s32 #foreign Native "CloseHandle";
    native_open :: (path:*u8, mode:*u8)->*void {
        return acquire_file(cast(*u16)path,0,0,null,3,0,null);
    }
} else {
    Native :: #system_library "c";
    native_open :: (path:*u8, mode:*u8)->*void #foreign Native "fopen";
    close_file :: (handle:*void)->s32 #foreign Native "fclose";
}
External :: #library "unknown_callback";
callback :: () #foreign External;
consume :: (handle:*void) #foreign External;
wrap :: (path:*u8, mode:*u8)->*void { return native_open(path,mode); }
pair :: (path:*u8, mode:*u8)->bool,*void { return true,native_open(path,mode); }
both :: (path:*u8, mode:*u8)->*void,*void { first:=native_open(path,mode); second:=native_open(path,mode); return first,second; } @Opens
callback_before :: (path:*u8, mode:*u8)->*void { callback(); return native_open(path,mode); }
callback_after :: (path:*u8, mode:*u8)->*void { h:=native_open(path,mode); callback(); return h; }
closed_return :: (path:*u8, mode:*u8)->*void { h:=native_open(path,mode); close_file(h); return h; }
fopen :: (path:*u8, mode:*u8)->*void { return path; }

leaked_first :: (path:*u8, mode:*u8) { h:=wrap(path,mode); if h!=null {} }
leaked_second :: (path:*u8, mode:*u8) { ok,h:=pair(path,mode); if ok && h!=null {} }
leaked_before_callback :: (path:*u8, mode:*u8) { h:=callback_before(path,mode); if h!=null {} }
leaked_named :: (path:*u8, mode:*u8) { h:=wrap(mode=mode,path=path); if h!=null {} }
closed :: (path:*u8, mode:*u8) { h:=wrap(path,mode); close_file(h); }
transferred :: (path:*u8, mode:*u8) { h:=wrap(path,mode); consume(h); }
uncertain_callback :: (path:*u8, mode:*u8) { h:=callback_after(path,mode); if h!=null {} }
stale_return :: (path:*u8, mode:*u8) { h:=closed_return(path,mode); if h!=null {} }
name_alone :: (path:*u8, mode:*u8) { h:=fopen(path,mode); if h!=null {} }

leaked_annotated_both :: (path:*u8, mode:*u8) { first,second:=both(path,mode); if first!=null && second!=null {} }

closed_one :: (path:*u8, mode:*u8) { first,second:=both(path,mode); close_file(first); if second!=null {} }

leaked_returns_other :: (path:*u8,mode:*u8)->*void { h:=native_open(path,mode); other:=native_open(path,mode); return other; }
