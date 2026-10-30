// Fills a buffer from the operating system's secure random generator.
random_bytes :: (out: [] u8) -> bool {
    #if OS == .WINDOWS {
        // Largest request kept well inside a 32-bit count.
        offset := 0;
        while offset < out.count {
            step := out.count - offset;
            if step > 0x10000000 step = 0x10000000;
            if bcrypt_fill(null, out.data + offset, cast(u32) step, USE_SYSTEM_PREFERRED_RNG) != 0 return false;
            offset += step;
        }

        return true;
    } else {
        offset := 0;
        while offset < out.count {
            step := out.count - offset;
            if step > 256 step = 256;
            if getentropy(out.data + offset, cast(u64) step) != 0 return false;
            offset += step;
        }

        return true;
    }
}

#scope_file

#if OS == .WINDOWS {
    bcrypt :: #system_library "bcrypt"; @native(external)
    bcrypt_fill :: (algorithm: *void, buffer: *u8, count: u32, flags: u32) -> s32 #foreign bcrypt "BCryptGenRandom";
    USE_SYSTEM_PREFERRED_RNG: u32 : 0x2;
} else {
    libc :: #library,system "c"; @native(external)
    getentropy :: (buffer: *void, length: u64) -> s32 #foreign libc;
}
