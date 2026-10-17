#import "Basic";
#import "File";
D :: #import "Debug_Info";
#load "codeview-test-support.jai";
#load "../../../../target/debug-info-symbol-sdk.jai";
#load "../../../../target/debug-info-symbol-mutations.jai";
#load "../../../../target/debug-info-symbol-synthetic.jai";

main :: () {
    push_context {
        context.allocator = .{ count_allocation, null };
        context.logger = capture_error;
        check_all();
    }

    assert(live_allocations == 0 && diagnostics > 0);
    print("ok\n");
}

check_all :: () {
    check_synthetic_symbol_records();
    check_sdk_symbol_workflows();
    bytes, read_ok := read_entire_file(tprint("%../../../../target/codeview-inline-original.bin", #filepath));

    assert(read_ok);
    defer free(bytes);
    info: D.Codeview_Debug_Info;

    assert(D.parse_codeview_symbols(*info, cast([] u8) bytes));
    defer release_debug_symbols(*info);
    assert(info.symbols.count > 0 && info.line_info.count > 0 && info.inlinee_line_info.count == 2);
    procedure: *D.Codeview_Symbol_Procedure;
    for symbol: info.symbols if D.is_procedure(symbol) procedure = cast(*D.Codeview_Symbol_Procedure) symbol;

    assert(procedure != null && procedure.name == "original_inline_entry" && procedure.inlinesites.count > 0);

    for site: procedure.inlinesites {
        ranges, ok := D.decode_inline_site_line_info(info, procedure.*, site.*);
        assert(ok && ranges.count > 0);
        release_sequences(ranges);
    }

    overlaid, sequences := D.overlay_inlinesites(info.line_info, procedure.*, procedure.inlinesites, info);

    assert(overlaid && sequences.count > 0);
    release_sequences(sequences);
    before := live_allocations;
    saved_symbols := info.symbols.data;

    assert(!D.parse_codeview_symbols(*info, cast([] u8) bytes));
    assert(info.symbols.data == saved_symbols && live_allocations == before);
    original_parent: [4] u8;

    for index: 0..3 {
        original_parent[index] = bytes[Native_Site_Parent_Offset + index];
        bytes[Native_Site_Parent_Offset + index] = 0xff;
    }

    malformed: D.Codeview_Debug_Info;

    assert(!D.parse_codeview_symbols(*malformed, cast([] u8) bytes));
    assert(malformed.symbols.count == 0 && live_allocations == before);
    for index: 0..3 bytes[Native_Site_Parent_Offset + index] = original_parent[index];

    bytes[Native_Procedure_Next_Offset] = 1;
    assert(!D.parse_codeview_symbols(*malformed, cast([] u8) bytes));
    assert(malformed.symbols.count == 0 && live_allocations == before);
    bytes[Native_Procedure_Next_Offset] = 0;

    for size: s64.[size_of(D.Codeview_Symbol_Procedure) + Native_Procedure_Payload_Size, 16] {
        partial: D.Codeview_Debug_Info;

        deny_size = size;
        denied = false;
        assert(!D.parse_codeview_symbols(*partial, cast([] u8) bytes));
        assert(denied && partial.symbols.count == 0 && live_allocations == before);
    }

    for prefix: 0..bytes.count - 1 {
        partial: D.Codeview_Debug_Info;

        before = live_allocations;
        partial_ok := D.parse_codeview_symbols(*partial, .{ prefix, bytes.data });
        if partial_ok release_debug_symbols(*partial);
        else assert(!partial.symbols.data && !partial.line_info.data && !partial.string_table.data);

        assert(live_allocations == before);
    }
}
