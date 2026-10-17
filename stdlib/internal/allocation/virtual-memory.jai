// Private address-space operations shared by bulk allocators.
#if OS == .WINDOWS {
    page_size :: () -> s64 {
        info: W.SYSTEM_INFO;
        W.GetSystemInfo(*info);
        return cast(s64) info.dwPageSize;
    }

    reserve :: (bytes: u64) -> *u8 {
        return cast(*u8) W.VirtualAlloc(null, bytes, W.MEM_RESERVE, W.PAGE_NOACCESS);
    }

    commit :: (start: *u8, bytes: u64) -> bool {
        return W.VirtualAlloc(start, bytes, W.MEM_COMMIT, W.PAGE_READWRITE) == start;
    }

    read_only :: (start: *u8, bytes: u64) -> bool {
        previous: u32;
        return W.VirtualProtect(start, bytes, W.PAGE_READONLY, *previous) != 0;
    }

    release :: (start: *u8, bytes: u64) -> bool {
        return W.VirtualFree(start, 0, W.MEM_RELEASE) != 0;
    } @Releases
} else #if OS == .MACOS || OS == .LINUX || OS == .ANDROID {
    page_size :: () -> s64 {
        return P.sysconf(P._SC_PAGESIZE);
    }

    reserve :: (bytes: u64) -> *u8 {
        memory := P.mmap(null, bytes, P.PROT_NONE, P.MAP_PRIVATE | P.MAP_ANONYMOUS, -1, 0);
        if memory == P.MAP_FAILED return null;
        return cast(*u8) memory;
    }

    commit :: (start: *u8, bytes: u64) -> bool {
        return P.mprotect(start, bytes, P.PROT_READ | P.PROT_WRITE) == 0;
    }

    read_only :: (start: *u8, bytes: u64) -> bool {
        return P.mprotect(start, bytes, P.PROT_READ) == 0;
    }

    release :: (start: *u8, bytes: u64) -> bool {
        return P.munmap(start, bytes) == 0;
    } @Releases
}

#scope_file
#if OS == .WINDOWS W :: #import "Windows";
#if OS == .MACOS || OS == .LINUX || OS == .ANDROID P :: #import "POSIX";
