#module_parameters(with_candidates := false, with_unwraps := false);
copy :: Impl.move_bytes;

#if with_candidates {
    copy_0 :: (destination: *void, source: *void, count: s64) {
        Impl.move_bytes(destination, source, count);
    }

    copy_10_sse_128 :: (destination: *void, source: *void, count: s64) {
        Impl.move_bytes(destination, source, count);
    }

    copy_11_sse_256 :: (destination: *void, source: *void, count: s64) {
        Impl.move_bytes(destination, source, count);
    }

    copy_12_sse_512 :: (destination: *void, source: *void, count: s64) {
        Impl.move_bytes(destination, source, count);
    }

    copy_13_sse_1024 :: (destination: *void, source: *void, count: s64) {
        Impl.move_bytes(destination, source, count);
    }

    copy_1_u64 :: (destination: *void, source: *void, count: s64) {
        Impl.move_bytes(destination, source, count);
    }

    copy_20_avx_256 :: (destination: *void, source: *void, count: s64) {
        Impl.move_bytes(destination, source, count);
    }

    copy_2_u64_128 :: (destination: *void, source: *void, count: s64) {
        Impl.move_bytes(destination, source, count);
    }

    copy_3_u64_256 :: (destination: *void, source: *void, count: s64) {
        Impl.move_bytes(destination, source, count);
    }

    copy_4_u64_256_asm :: (destination: *void, source: *void, count: s64) {
        Impl.move_bytes(destination, source, count);
    }

    copy_5_asm_rep :: (destination: *void, source: *void, count: s64) {
        Impl.move_bytes(destination, source, count);
    }

    copy_6_asm_rep_bytes :: (destination: *void, source: *void, count: s64) {
        Impl.move_bytes(destination, source, count);
    }

    has_avx: enum {
        UNDETERMINED :: -1;
        NO;
        YES;
    } = .UNDETERMINED;
}

#if with_unwraps {
    #if CPU == .X64 {
        avx_copy_256 :: inline (destination: *void, source: *void) {
            Impl.move_bytes(destination, source, 32);
        }
    }

    copy_128 :: inline (destination: *void, source: *void) {
        Impl.move_bytes(destination, source, 16);
    }

    copy_256 :: inline (destination: *void, source: *void) {
        Impl.move_bytes(destination, source, 32);
    }

    copy_256_expand_array :: (destination: *void, source: *void) #expand {
        Impl.move_bytes(destination, source, 32);
    }

    copy_256_expand_assign :: (destination: *void, source: *void) #expand {
        Impl.move_bytes(destination, source, 32);
    }

    copy_256_expand_index :: (destination: *void, source: *void) #expand {
        Impl.move_bytes(destination, source, 32);
    }

    copy_256_inline_array :: inline (destination: *void, source: *void) {
        Impl.move_bytes(destination, source, 32);
    }

    copy_256_inline_assign :: inline (destination: *void, source: *void) {
        Impl.move_bytes(destination, source, 32);
    }

    copy_256_inline_index :: inline (destination: *void, source: *void) {
        Impl.move_bytes(destination, source, 32);
    }

    copy_u64_unwrap :: (destination: *void, source: *void, count: s64, $call: type_of(copy_256)) {
        offset: s64;
        while offset + 32 <= count {
            call(cast(*u8) destination + offset, cast(*u8) source + offset);
            offset += 32;
        }

        Impl.move_bytes(cast(*u8) destination + offset, cast(*u8) source + offset, count - offset);
    }

    copy_u64_unwrap_nocall_array :: (destination: *void, source: *void, count: s64) {
        Impl.move_bytes(destination, source, count);
    }

    copy_u64_unwrap_nocall_asm :: (destination: *void, source: *void, count: s64) {
        Impl.move_bytes(destination, source, count);
    }

    copy_u64_unwrap_nocall_assign :: (destination: *void, source: *void, count: s64) {
        Impl.move_bytes(destination, source, count);
    }

    copy_u64_unwrap_nocall_index :: (destination: *void, source: *void, count: s64) {
        Impl.move_bytes(destination, source, count);
    }

    sse_copy_1024 :: inline (destination: *void, source: *void) {
        Impl.move_bytes(destination, source, 128);
    }

    sse_copy_128 :: inline (destination: *void, source: *void) {
        Impl.move_bytes(destination, source, 16);
    }

    sse_copy_256 :: inline (destination: *void, source: *void) {
        Impl.move_bytes(destination, source, 32);
    }

    sse_copy_512 :: inline (destination: *void, source: *void) {
        Impl.move_bytes(destination, source, 64);
    }
}

#scope_file
Impl :: #import "internal/memory";
