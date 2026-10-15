#import "Basic";
U :: #import "Windows_Utf8";

#assert !#exists(U.Native);

#if OS == .WINDOWS {
    main :: () {
        input := "A\xc3\xa9\xf0\x9f\x99\x82";
        wide, success, bytes := U.utf8_to_wide(input);

        assert(success && wide != null && bytes == 8);
        assert(wide[0] == 65 && wide[1] == 0xe9 && wide[2] == 0xd83d && wide[3] == 0xde42 && wide[4] == 0);
        roundtrip, valid := U.wide_to_utf8(wide);

        assert(valid && roundtrip == input && roundtrip.data[roundtrip.count] == 0);
        free(roundtrip);
        explicit, explicit_valid := U.wide_to_utf8(wide, 4);

        assert(explicit_valid && explicit == input && explicit.data[explicit.count] == 0);
        free(explicit);
        free(wide);
        empty, empty_valid, empty_bytes := U.utf8_to_wide("");

        assert(empty_valid && empty == null && empty_bytes == 0);
        embedded := "x\0y";
        embedded_wide, embedded_valid, embedded_bytes := U.utf8_to_wide(embedded);

        assert(embedded_valid && embedded_bytes == 6 && embedded_wide[1] == 0 && embedded_wide[2] == 121 && embedded_wide[3] == 0);
        embedded_utf8, embedded_utf8_valid := U.wide_to_utf8(embedded_wide, 3);

        assert(embedded_utf8_valid && embedded_utf8 == embedded && embedded_utf8.data[3] == 0);
        free(embedded_utf8);
        free(embedded_wide);
        bad, bad_valid := U.wide_to_utf8(null);

        assert(!bad_valid && bad.count == 0 && bad.data == null);
        unit: u16 = 0;
        empty_wide, empty_wide_valid := U.wide_to_utf8(*unit);

        assert(empty_wide_valid && empty_wide.count == 0 && empty_wide.data != null && empty_wide.data[0] == 0);
        free(empty_wide);
        bad, bad_valid = U.wide_to_utf8(*unit, 0);
        assert(!bad_valid);

        {
            push_allocator(.{ fail_allocation, null });
            failed, failed_valid, failed_bytes := U.utf8_to_wide("x");

            assert(!failed_valid && failed == null && failed_bytes == 0);
            failed_utf8, failed_utf8_valid := U.wide_to_utf8(*unit);

            assert(!failed_utf8_valid && failed_utf8.data == null && failed_utf8.count == 0);
        }

        print("ok\n");
    }
} else {
    main :: () {
        assert(false, "Windows_Utf8 requires actual Windows execution");
    }
}

#scope_file
fail_allocation :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    assert(mode == .ALLOCATE);
    return null;
}
